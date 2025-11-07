pub mod batch_manager;
pub mod proof_orchestrator;
pub mod gas_optimizer;
pub mod health_monitor;
pub mod retry_engine;
pub mod queue_manager;
pub mod types;
pub mod error;
pub mod http_server;
pub mod database;
pub mod solana_client;
pub mod metrics;

pub use batch_manager::BatchManager;
pub use proof_orchestrator::ProofOrchestrator;
pub use gas_optimizer::GasOptimizer;
pub use health_monitor::HealthMonitor;
pub use retry_engine::RetryEngine;
pub use queue_manager::QueueManager;
pub use types::{OrchestratorConfig, Deposit, SystemHealth, QueueStats, Batch};
pub use error::{OrchestratorError, Result};
pub use database::DatabaseService;
pub use solana_client::SolanaClient;
pub use metrics::BridgeMetrics;

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use std::time::Instant;
use prometheus::Registry;

pub struct SubmissionManager {
    batch_manager: BatchManager,
    proof_orchestrator: ProofOrchestrator,
    gas_optimizer: GasOptimizer,
    health_monitor: HealthMonitor,
    retry_engine: RetryEngine,
    queue_manager: QueueManager,
    database: DatabaseService,
    solana_client: SolanaClient, 
    config: OrchestratorConfig,
    metrics: BridgeMetrics,
    registry: Registry,
    is_running: bool,
}

impl SubmissionManager {
    pub async fn new(config: OrchestratorConfig) -> Result<Self> {
        // Initialize database
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:submission_manager.db".to_string());
        let database = DatabaseService::new(&db_url).await?;

        // Initialize Solana client - USE CONFIG, NOT ENV VARS
        let solana_client = SolanaClient::new(
            &config.solana_rpc_url,
            &config.solana_program_id,
            &config.solana_bridge_account,
            config.verification_key.as_ref().map(String::as_str),
        )?;

        // Initialize metrics
        let registry = Registry::new();
        let metrics = BridgeMetrics::new(&registry)?;

        Ok(Self {
            batch_manager: BatchManager::new(config.batch_size),
            proof_orchestrator: ProofOrchestrator::new(config.validators.clone(), config.validator_count),
            gas_optimizer: GasOptimizer::new(config.gas_update_interval),
            health_monitor: HealthMonitor::new(config.health_check_interval),
            retry_engine: RetryEngine::new(config.max_retries as usize),
            queue_manager: QueueManager::new(),
            database,
            solana_client,
            metrics,
            registry,
            config,
            is_running: false,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        self.is_running = true;
        log::info!("🚀 Starting Rust Submission Manager...");

        // Start health monitoring
        self.start_health_monitoring().await;

        // Start batch processing
        self.start_batch_processing().await;

        log::info!("✅ Rust Submission Manager started successfully");
        Ok(())
    }

    pub async fn stop(&mut self) {
        self.is_running = false;
        log::info!("🛑 Rust Submission Manager stopped");
    }

    pub async fn add_deposit(&mut self, deposit: Deposit) -> Result<()> {
        // Track metrics
        self.metrics.deposits_received.inc();
        
        // Store deposit in database first
        let deposit_record = database::DepositRecord {
            deposit_id: deposit.deposit_id.clone(),
            ton_tx_hash: deposit.ton_tx_hash.clone(),
            sender_address: deposit.sender_address.clone(),
            recipient_solana: deposit.recipient_solana.clone(),
            amount: deposit.amount.clone(),
            status: "pending".to_string(),
            error_message: None,
            created_at: 0,
            updated_at: 0,
        };
        
        self.database.store_deposit(deposit_record).await?;

        // Generate proof for this individual deposit
        let proof_start = Instant::now();
        let proof = match self.proof_orchestrator.generate_proof(&deposit).await {
            Ok(proof) => {
                self.metrics.proof_generation_time.observe(proof_start.elapsed().as_secs_f64());
                self.metrics.proofs_generated.inc();
                proof
            }
            Err(e) => {
                log::error!("Failed to generate proof for deposit {}: {}", deposit.deposit_id, e);
                self.database.update_deposit_status(&deposit.deposit_id, "failed", Some(&e.to_string())).await?;
                return Ok(()); // Don't add to batch if proof generation fails
            }
        };

        // Add to batch (deposit + proof)
        if let Some(batch) = self.batch_manager.add_to_batch(deposit, proof).await? {
            log::info!("🎯 Batch completed with {} deposits, adding to queue", batch.deposits.len());
            self.metrics.current_batch_size.set(batch.deposits.len() as f64);
            self.queue_manager.enqueue_batch(batch).await;
        }

        Ok(())
    }

    async fn start_health_monitoring(&self) {
        let health_monitor = self.health_monitor.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                match health_monitor.get_system_health().await {
                    Ok(health) => {
                        let health_status = if health_monitor.is_system_healthy(&health) {
                            "✅ Healthy"
                        } else {
                            "❌ Unhealthy"
                        };

                        log::info!("📊 System Health: {}", health_status);
                    }
                    Err(e) => log::error!("Health check failed: {}", e),
                }
            }
        });
    }

    async fn start_batch_processing(&self) {
        log::info!("🔄 Starting batch processing engine...");
        
        let mut manager = self.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10)); // Process every 10 seconds
            
            loop {
                interval.tick().await;
                
                // Process queued batches
                if let Err(e) = manager.process_queued_batches().await {
                    log::error!("Error processing batches: {}", e);
                }

                // Finalize any partial batch that's been waiting too long
                if let Err(e) = manager.finalize_stale_batch().await {
                    log::error!("Error finalizing stale batch: {}", e);
                }
            }
        });
    }

    async fn process_queued_batches(&mut self) -> Result<()> {
        // Get the next batch from queue (FIFO)
        if let Some(batch) = self.queue_manager.dequeue_batch().await {
            log::info!("📦 Processing batch with {} deposits", batch.deposits.len());
            
            // METRIC: Batch processing started
            self.metrics.batches_processing.inc();
            let batch_start_time = Instant::now();

            // Submit batch to Solana
            let tx_start = Instant::now();
            match self.solana_client.submit_batch(&batch).await {
                Ok(tx_signature) => {
                    // METRICS: Success
                    self.metrics.solana_tx_time.observe(tx_start.elapsed().as_secs_f64());
                    self.metrics.batches_submitted.inc();
                    self.metrics.deposits_completed.inc_by(batch.deposits.len() as f64);
                    self.metrics.last_successful_batch_time.set(chrono::Utc::now().timestamp() as f64);
                    
                    log::info!("✅ Batch successfully submitted to Solana: {}", tx_signature);
                    
                    // Update all deposits in this batch to "completed"
                    for deposit in &batch.deposits {
                        self.database.update_deposit_status(&deposit.deposit_id, "completed", None).await?;
                    }
                    
                    // Log batch completion
                    log::info!("🎉 Batch completed: {} deposits bridged to Solana", batch.deposits.len());
                }
                Err(e) => {
                    // METRICS: Submission failure
                    self.metrics.batch_submission_failures.inc();
                    self.metrics.last_failure_time.set(chrono::Utc::now().timestamp() as f64);
                    
                    // METRIC: Failure by type
                    match &e {
                        OrchestratorError::NetworkError(_) => self.metrics.network_failures.inc(),
                        _ => self.metrics.other_failures.inc(),
                    }
                    
                    log::error!("❌ Failed to submit batch to Solana: {}", e);
                    
                    // Handle retry logic
                    self.handle_batch_submission_failure(batch, e).await?;
                }
            }
            
            // METRIC: Batch processing completed
            self.metrics.batch_processing_time.observe(batch_start_time.elapsed().as_secs_f64());
            self.metrics.batches_processing.dec();
        } else {
            // METRIC: No batches to process (queue empty)
            self.metrics.empty_queue_checks.inc();
        }
        
        Ok(())
    }

    async fn handle_batch_submission_failure(&mut self, batch: Batch, error: OrchestratorError) -> Result<()> {
        log::warn!("Handling batch submission failure, will retry...");
        
        // Check if we should retry
        if self.retry_engine.should_retry(batch.retry_count) {
            // METRIC: Batch retry
             self.metrics.batch_retries.inc();
        
            // FIX: Store retry_count before moving batch
            let retry_count = batch.retry_count + 1;
            
            // Re-queue the batch for retry
            let mut retry_batch = batch;
            retry_batch.retry_count = retry_count;
            
            self.queue_manager.enqueue_batch(retry_batch).await;
        log::info!("🔄 Batch re-queued for retry (attempt {})", retry_count);  // Use stored value
        } else {
            // METRIC: Max retries exceeded
            self.metrics.max_retries_exceeded.inc();
            
            // Max retries exceeded - mark all deposits as failed
            log::error!("❌ Max retries exceeded for batch, marking deposits as failed");
            
            for deposit in &batch.deposits {
                self.database.update_deposit_status(
                    &deposit.deposit_id, 
                    "failed", 
                    Some(&format!("Max retries exceeded: {}", error))
                ).await?;
            }
        }
        
        Ok(())
    }

    async fn finalize_stale_batch(&mut self) -> Result<()> {
        // Check if current batch is getting stale (e.g., waiting more than 2 minutes)
        if let Some(batch) = self.batch_manager.finalize_if_stale(Duration::from_secs(120)).await? {
            log::info!("⏰ Finalizing stale batch with {} deposits", batch.deposits.len());
            self.queue_manager.enqueue_batch(batch).await;
        }
        
        Ok(())
    }

    pub async fn get_queue_stats(&self) -> QueueStats {
        let stats = self.queue_manager.get_queue_stats().await;
        // Update metrics with current queue size
        self.metrics.queue_size.set(stats.pending as f64);
        stats
    }

    pub async fn finalize_current_batch(&mut self) -> Result<()> {
        if let Some(batch) = self.batch_manager.finalize_batch().await? {
            log::info!("👤 Manually finalizing batch with {} deposits", batch.deposits.len());
            self.queue_manager.enqueue_batch(batch).await;
        } else {
            log::info!("No current batch to finalize");
        }
        Ok(())
    }

    pub async fn start_http_server(self) -> Result<()> {
        let manager = Arc::new(Mutex::new(self));
        
        // Start the manager first
        {
            let mut mgr = manager.lock().await;
            mgr.start().await?;
        }
        
        // Then start HTTP server (this will block)
        http_server::start_http_server(manager).await;
        
        Ok(())
    }
}

impl Clone for SubmissionManager {
    fn clone(&self) -> Self {
        Self {
            batch_manager: BatchManager::new(self.config.batch_size),
            proof_orchestrator: self.proof_orchestrator.clone(),
            gas_optimizer: self.gas_optimizer.clone(),
            health_monitor: self.health_monitor.clone(),
            retry_engine: self.retry_engine.clone(),
            queue_manager: self.queue_manager.clone(),
            database: self.database.clone(),
            solana_client: self.solana_client.clone(),
            metrics: self.metrics.clone(),
            registry: Registry::new(), // New registry for clone
            config: self.config.clone(),
            is_running: self.is_running,
        }
    }
}
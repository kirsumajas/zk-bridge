use prometheus::{Counter, Gauge, Histogram, Registry};

pub struct BridgeMetrics {
    // Counters
    pub deposits_received: Counter,
    pub deposits_completed: Counter,
    pub batches_submitted: Counter,
    pub proofs_generated: Counter,
    
    // Gauges
    pub queue_size: Gauge,
    pub current_batch_size: Gauge,
    pub solana_rpc_latency: Gauge,
    
    // Histograms
    pub proof_generation_time: Histogram,
    pub solana_tx_time: Histogram,

    // Batch processing metrics
    pub batches_processing: Gauge,
    pub batch_submission_failures: Counter,
    pub batch_validation_failures: Counter,
    pub batch_timeouts: Counter,
    pub batch_processing_time: Histogram,
    
    // Error classification
    pub network_failures: Counter,
    pub insufficient_funds_failures: Counter,
    pub db_update_failures: Counter,
    pub other_failures: Counter,
    
    // System health
    pub last_successful_batch_time: Gauge,
    pub last_failure_time: Gauge,
    pub empty_queue_checks: Counter,
    
    // Retry metrics
    pub batch_retries: Counter,
    pub max_retries_exceeded: Counter,
}

impl BridgeMetrics {
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let metrics = BridgeMetrics {
            // Existing metrics
            deposits_received: Counter::new("deposits_received_total", "Total deposits received")?,
            deposits_completed: Counter::new("deposits_completed_total", "Total deposits completed")?,
            batches_submitted: Counter::new("batches_submitted_total", "Total batches submitted")?,
            proofs_generated: Counter::new("proofs_generated_total", "Total proofs generated")?,
            
            queue_size: Gauge::new("queue_size", "Current queue size")?,
            current_batch_size: Gauge::new("current_batch_size", "Current batch size")?,
            solana_rpc_latency: Gauge::new("solana_rpc_latency_ms", "Solana RPC latency in ms")?,
            
            proof_generation_time: Histogram::with_opts(
                HistogramOpts::new("proof_generation_time", "Time taken to generate ZK proofs")
            )?,
            solana_tx_time: Histogram::new(
                "solana_tx_time_seconds", 
                "Solana transaction time in seconds"
            )?,

            // New metrics
            batches_processing: Gauge::new("batches_processing", "Number of batches currently processing")?,
            batch_submission_failures: Counter::new("batch_submission_failures_total", "Total batch submission failures")?,
            batch_validation_failures: Counter::new("batch_validation_failures_total", "Total batch validation failures")?,
            batch_timeouts: Counter::new("batch_timeouts_total", "Total batch timeouts")?,
            batch_processing_time: Histogram::new(
                "batch_processing_time_seconds",
                "Total batch processing time in seconds"
            )?,
            
            network_failures: Counter::new("network_failures_total", "Total network failures")?,
            insufficient_funds_failures: Counter::new("insufficient_funds_failures_total", "Total insufficient funds failures")?,
            db_update_failures: Counter::new("db_update_failures_total", "Total database update failures")?,
            other_failures: Counter::new("other_failures_total", "Total other failures")?,
            
            last_successful_batch_time: Gauge::new("last_successful_batch_time", "Timestamp of last successful batch")?,
            last_failure_time: Gauge::new("last_failure_time", "Timestamp of last failure")?,
            empty_queue_checks: Counter::new("empty_queue_checks_total", "Total empty queue checks")?,
            
            batch_retries: Counter::new("batch_retries_total", "Total batch retries")?,
            max_retries_exceeded: Counter::new("max_retries_exceeded_total", "Total max retries exceeded")?,
        };

        // Register ALL metrics
        registry.register(Box::new(metrics.deposits_received.clone()))?;
        registry.register(Box::new(metrics.deposits_completed.clone()))?;
        registry.register(Box::new(metrics.batches_submitted.clone()))?;
        registry.register(Box::new(metrics.proofs_generated.clone()))?;
        
        registry.register(Box::new(metrics.queue_size.clone()))?;
        registry.register(Box::new(metrics.current_batch_size.clone()))?;
        registry.register(Box::new(metrics.solana_rpc_latency.clone()))?;
        
        registry.register(Box::new(metrics.proof_generation_time.clone()))?;
        registry.register(Box::new(metrics.solana_tx_time.clone()))?;

        // Register new metrics
        registry.register(Box::new(metrics.batches_processing.clone()))?;
        registry.register(Box::new(metrics.batch_submission_failures.clone()))?;
        registry.register(Box::new(metrics.batch_validation_failures.clone()))?;
        registry.register(Box::new(metrics.batch_timeouts.clone()))?;
        registry.register(Box::new(metrics.batch_processing_time.clone()))?;
        
        registry.register(Box::new(metrics.network_failures.clone()))?;
        registry.register(Box::new(metrics.insufficient_funds_failures.clone()))?;
        registry.register(Box::new(metrics.db_update_failures.clone()))?;
        registry.register(Box::new(metrics.other_failures.clone()))?;
        
        registry.register(Box::new(metrics.last_successful_batch_time.clone()))?;
        registry.register(Box::new(metrics.last_failure_time.clone()))?;
        registry.register(Box::new(metrics.empty_queue_checks.clone()))?;
        
        registry.register(Box::new(metrics.batch_retries.clone()))?;
        registry.register(Box::new(metrics.max_retries_exceeded.clone()))?;

        Ok(metrics)
    }
}

impl Clone for BridgeMetrics {
    fn clone(&self) -> Self {
        // Prometheus types don't implement Clone, so we create new instances
        let registry = Registry::new();
        BridgeMetrics::new(&registry).expect("Failed to clone metrics")
    }
}
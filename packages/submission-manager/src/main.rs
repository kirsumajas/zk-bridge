use submission_manager::{SubmissionManager, OrchestratorConfig};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    
    println!("🚀 Starting Rust Submission Manager with Solana ZK Program...");
    
    // Create configuration from environment variables
    let config = OrchestratorConfig {
        batch_size: std::env::var("BATCH_SIZE")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .unwrap_or(5),
        max_retries: std::env::var("MAX_RETRIES")
            .unwrap_or_else(|_| "3".to_string())
            .parse()
            .unwrap_or(3),
        health_check_interval: std::env::var("HEALTH_CHECK_INTERVAL_MS")
            .unwrap_or_else(|_| "30000".to_string())
            .parse()
            .unwrap_or(30000),
        gas_update_interval: std::env::var("GAS_UPDATE_INTERVAL_MS")
            .unwrap_or_else(|_| "60000".to_string())
            .parse()
            .unwrap_or(60000),
        validator_count: 2,
        validators: vec![
            "http://circuit-service:8080".to_string(),
        ],
        // ADD SOLANA CONFIG
        solana_rpc_url: std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string()),
        solana_program_id: std::env::var("SOLANA_PROGRAM_ID")
            .unwrap_or_else(|_| "BridgeProgram11111111111111111111111111111".to_string()),
        solana_bridge_account: std::env::var("SOLANA_BRIDGE_ACCOUNT")
            .unwrap_or_else(|_| "BridgeAccount1111111111111111111111111111".to_string()),
        verification_key: std::env::var("VERIFICATION_KEY")
            .unwrap_or_else(|_| "path/to/verification_key.json".to_string()),
    };
    
    // Create and start submission manager
    let manager = SubmissionManager::new(config).await?;
    
    println!("✅ Submission Manager with Solana client initialized, starting HTTP server...");
    
    // Start HTTP server (this will block)
    manager.start_http_server().await?;
    
    Ok(())
}
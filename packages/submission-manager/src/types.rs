use serde::{Deserialize, Serialize};
use chrono;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    pub pending: usize,
    pub processing: usize,
    pub completed: usize,
    pub total: usize,
}

#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    pub batch_size: usize,
    pub max_retries: u32,  // Keep as u32
    pub health_check_interval: u64,
    pub gas_update_interval: u64,
    pub validator_count: usize,
    pub validators: Vec<String>,

    pub solana_rpc_url: String,
    pub solana_program_id: String,
    pub solana_bridge_account: String,
    pub verification_key: String, // For ZK verification
}

#[derive(Debug, Clone)]
pub struct Deposit {
    pub deposit_id: String,
    pub ton_tx_hash: String,
    pub sender_address: String,
    pub recipient_solana: String,
    pub amount: String,
    pub fee_est: String,
    pub nonce: String,
    pub created_at: u64,
}

#[derive(Debug, Clone)]
pub struct SystemHealth {
    pub ton_rpc: bool,
    pub solana_rpc: bool,
    pub database: bool,
    pub validators: bool,
    pub queue_size: bool,
    pub last_batch_time: bool,
}

#[derive(Debug, Clone)]
pub struct Batch {
    pub deposits: Vec<Deposit>,
    pub proofs: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
     pub retry_count: usize,
}
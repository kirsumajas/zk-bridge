use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrchestratorError {
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Solana client error: {0}")]
    SolanaError(#[from] solana_client::client_error::ClientError),
    
    #[error("Metrics error: {0}")]
    MetricsError(#[from] prometheus::Error),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Insufficient validator signatures: {current}/{required}")]
    InsufficientSignatures { current: usize, required: usize },
    
    #[error("Operation failed after {attempts} attempts: {message}")]
    MaxRetriesExceeded { attempts: usize, message: String },
    
    #[error("System unhealthy: {reason}")]
    SystemUnhealthy { reason: String },
    
    #[error("Batch processing failed: {reason}")]
    BatchProcessingFailed { reason: String },
}

pub type Result<T> = std::result::Result<T, OrchestratorError>;
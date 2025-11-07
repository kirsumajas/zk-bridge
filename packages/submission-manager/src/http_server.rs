use warp::Filter;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use crate::SubmissionManager;
use crate::types::Deposit;
use prometheus::{TextEncoder, Encoder};

#[derive(Debug, Serialize, Deserialize)]
pub struct DepositRequest {
    pub deposit_id: String,
    pub ton_tx_hash: String,
    pub sender_address: String,
    pub recipient_solana: String,
    pub amount: String,
    pub fee_est: String,
    pub nonce: String,
    pub created_at: u64,
}

#[derive(Debug, Serialize)]
pub struct QueueStatsResponse {
    pub pending: usize,
    pub total: usize,
    pub completed: usize,
}

pub async fn start_http_server(manager: Arc<Mutex<SubmissionManager>>) {
    // Health check endpoint
    let health = warp::path!("health")
        .map(|| warp::reply::json(&serde_json::json!({"status": "healthy"})));

    // Add deposit endpoint
    let add_deposit = {
        let manager = manager.clone();
        warp::path!("api" / "deposits")
            .and(warp::post())
            .and(warp::body::json())
            .map(move |deposit: DepositRequest| {
                let manager = manager.clone();
                
                // Use tokio::spawn to handle async operations
                tokio::spawn(async move {
                    let mut mgr = manager.lock().await;
                    
                    let internal_deposit = Deposit {
                        deposit_id: deposit.deposit_id.clone(),
                        ton_tx_hash: deposit.ton_tx_hash,
                        sender_address: deposit.sender_address,
                        recipient_solana: deposit.recipient_solana,
                        amount: deposit.amount,
                        fee_est: deposit.fee_est,
                        nonce: deposit.nonce,
                        created_at: deposit.created_at,
                    };

                    match mgr.add_deposit(internal_deposit).await {
                        Ok(()) => {
                            log::info!("✅ Deposit {} queued successfully", deposit.deposit_id);
                            warp::reply::json(&serde_json::json!({"status": "queued"}))
                        }
                        Err(e) => {
                            log::error!("❌ Failed to queue deposit {}: {}", deposit.deposit_id, e);
                            warp::reply::json(&serde_json::json!({"error": e.to_string()}))
                        }
                    }
                });
                
                // Return immediate response - processing happens in background
                warp::reply::json(&serde_json::json!({"status": "processing"}))
            })
    };

    // Get queue stats endpoint
    let queue_stats = {
        let manager = manager.clone();
        warp::path!("api" / "queue-stats")
            .and(warp::get())
            .and_then(move || {
                let manager = manager.clone();
                async move {
                    let mgr = manager.lock().await;
                    let stats = mgr.get_queue_stats().await;
                    let response = QueueStatsResponse {
                        pending: stats.pending,
                        total: stats.total,
                        completed: stats.completed,
                    };
                    Ok::<_, Infallible>(warp::reply::json(&response))
                }
            })
    };

    // Metrics endpoint
    let metrics_endpoint = {
        let manager = manager.clone();
        warp::path!("metrics")
            .and(warp::get())
            .map(move || {
                let manager = manager.lock();
                // Note: This is a simple implementation. In production, you might want
                // to handle the async lock properly or use a different approach
                let encoder = TextEncoder::new();
                let metric_families = prometheus::gather();
                let mut buffer = vec![];
                encoder.encode(&metric_families, &mut buffer).unwrap();
                warp::reply::with_header(
                    String::from_utf8(buffer).unwrap(),
                    "content-type",
                    "text/plain; version=0.0.4",
                )
            })
    };

    let routes = health
        .or(add_deposit)
        .or(queue_stats)
        .or(metrics_endpoint)
        .with(warp::cors().allow_any_origin());

    log::info!("🌐 Starting HTTP server on :3000");
    warp::serve(routes).run(([0, 0, 0, 0], 3000)).await;
}
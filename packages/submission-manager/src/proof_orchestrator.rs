use crate::{OrchestratorError, Result};
use serde_json::json;
use std::time::Duration;


#[derive(Clone)]  
pub struct ProofOrchestrator {
    circuit_service_url: String,
    client: reqwest::Client,
}

impl ProofOrchestrator {
    pub fn new(validators: Vec<String>, _validator_count: usize) -> Self {
        let circuit_service_url = validators.first()
            .cloned()
            .unwrap_or_else(|| "http://localhost:8080".to_string());

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        Self {
            circuit_service_url,
            client,
        }
    }

    pub async fn generate_proof(&self, deposit: &crate::Deposit) -> Result<String> {
        log::info!("Generating proof for deposit: {}", deposit.deposit_id);

        let proof_request = json!({
            "publicInputs": [
                deposit.deposit_id,
                deposit.ton_tx_hash,
                deposit.sender_address,
                deposit.recipient_solana,
                deposit.amount
            ]
        });

        let response = self.client
            .post(&format!("{}/generate-proof", self.circuit_service_url))
            .json(&proof_request)
            .send()
            .await
            .map_err(OrchestratorError::NetworkError)?;

        if !response.status().is_success() {
            return Err(OrchestratorError::NetworkError(
                reqwest::Error::from(response.error_for_status().unwrap_err())
            ));
        }

        let proof_data: serde_json::Value = response.json()
            .await
            .map_err(OrchestratorError::NetworkError)?;

        Ok(proof_data["proof"]
            .as_str()
            .unwrap_or("mock_proof")
            .to_string())
    }
}
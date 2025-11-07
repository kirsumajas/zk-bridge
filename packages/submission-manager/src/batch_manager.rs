use crate::{Deposit, Result};
use crate::types::Batch;
use std::time::Duration;
use chrono::Utc;

pub struct BatchManager {
    batch_size: usize,
    current_batch: Option<Batch>,
}

impl BatchManager {
    pub fn new(batch_size: usize) -> Self {
        Self {
            batch_size,
            current_batch: None,
        }
    }

    pub async fn add_to_batch(&mut self, deposit: Deposit, proof: String) -> Result<Option<Batch>> {
        if self.current_batch.is_none() {
            self.current_batch = Some(Batch {
                deposits: Vec::new(),
                proofs: Vec::new(),
                created_at: Utc::now(),
                retry_count: 0, // Initialize retry_count
            });
        }

        if let Some(batch) = &mut self.current_batch {
            batch.deposits.push(deposit);
            batch.proofs.push(proof);

            if batch.deposits.len() >= self.batch_size {
                let completed_batch = self.current_batch.take();
                return Ok(completed_batch);
            }
        }

        Ok(None)
    }

    pub async fn finalize_batch(&mut self) -> Result<Option<Batch>> {
        Ok(self.current_batch.take())
    }

    // ADD THIS METHOD
    pub async fn finalize_if_stale(&mut self, max_age: Duration) -> Result<Option<Batch>> {
        if let Some(batch) = &self.current_batch {
            let batch_age = Utc::now() - batch.created_at;
            if batch_age > chrono::Duration::from_std(max_age).unwrap() {
                log::info!("Batch is stale (age: {}s), finalizing", batch_age.num_seconds());
                return Ok(self.current_batch.take());
            }
        }
        Ok(None)
    }
}
use crate::types::Batch;

#[derive(Debug, Clone)]
pub struct QueueManager {
    batches: Vec<Batch>,
}

impl QueueManager {
    pub fn new() -> Self {
        Self {
            batches: Vec::new(),
        }
    }

    pub async fn enqueue_batch(&mut self, batch: Batch) {
        self.batches.push(batch);
        log::info!("Enqueuing batch with {} deposits", self.batches.last().unwrap().deposits.len());
    }

    pub async fn dequeue_batch(&mut self) -> Option<Batch> {
        if self.batches.is_empty() {
            None
        } else {
            Some(self.batches.remove(0))
        }
    }

    pub async fn get_queue_stats(&self) -> crate::types::QueueStats {
        let total_batches = self.batches.len();
        let total_deposits: usize = self.batches.iter().map(|b| b.deposits.len()).sum();
        
        crate::types::QueueStats {
            pending: total_deposits,
            processing: 0,
            completed: 0,
            total: total_deposits,
        }
    }
}
pub struct RetryEngine {
    max_retries: usize,
}

impl RetryEngine {
    pub fn new(max_retries: usize) -> Self {
        Self { max_retries }
    }

    // ADD THIS METHOD
    pub fn should_retry(&self, current_retries: usize) -> bool {
        current_retries < self.max_retries
    }
}

impl Clone for RetryEngine {
    fn clone(&self) -> Self {
        Self {
            max_retries: self.max_retries,
        }
    }
}
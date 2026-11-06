#[derive(Debug, Clone)]
pub struct PidController {
    target: f32,
    min_pool_size: usize,
    max_pool_size: usize,
    current_pool: usize,
}

impl PidController {
    pub fn new(target: f32, min_pool: usize, max_pool: usize, initial: Option<usize>) -> Self {
        Self {
            target,
            min_pool_size: min_pool,
            max_pool_size: max_pool,
            current_pool: initial.unwrap_or(min_pool),
        }
    }

    pub fn current_pool_size(&self) -> Option<usize> {
        Some(self.current_pool)
    }

    pub fn update(&mut self, _dt: std::time::Duration, elapsed_ms: f32) -> usize {
        if elapsed_ms > self.target {
            self.current_pool = self.current_pool.saturating_sub(5).max(self.min_pool_size);
        } else {
            self.current_pool = (self.current_pool + 5).min(self.max_pool_size);
        }
        self.current_pool
    }
}

// FILE-CONTEXT
// ZWECK: Homeostat Wrapper in contextra-db für Rerank PID Candidate Pool Regulation und Deadline Management.
// INVARIANTEN: Re-exportiert `RerankPidController` und `RerankDeadline` für Latenzregelung.

//! Homeostat-Modul für Rerank-Kandidatenpool-Regulierung und Deadline-Steuerung.

use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct RerankDeadline {
    deadline: Instant,
}

impl RerankDeadline {
    pub fn new(timeout: Duration) -> Self {
        Self {
            deadline: Instant::now() + timeout,
        }
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.deadline
    }

    pub fn remaining(&self) -> Duration {
        self.deadline.saturating_duration_since(Instant::now())
    }
}

#[derive(Debug, Clone)]
#[deprecated(note = "use PidLatencyController instead")]
pub struct RerankPidController {
    target_ms: f32,
    pool_size: usize,
}

#[allow(deprecated)]
impl RerankPidController {
    pub fn new(target_ms: f32, initial_pool: usize) -> Self {
        Self {
            target_ms,
            pool_size: initial_pool,
        }
    }

    pub fn pool_size(&self) -> usize {
        self.pool_size
    }

    pub fn update(&mut self, elapsed_ms: f32) -> usize {
        if elapsed_ms > self.target_ms {
            self.pool_size = self.pool_size.saturating_sub(5).max(50);
        } else {
            self.pool_size = (self.pool_size + 5).min(200);
        }
        self.pool_size
    }
}

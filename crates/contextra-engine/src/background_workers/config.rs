// FILE-CONTEXT
// ZWECK: Konfiguration und Trait für Orphan-Cleanup-Backoff und HNSW-Index-Rebuild.
// INVARIANTEN: Berechnet exponentielle Cooldown-Zeiten für Rebuild-Versuche.

use std::time::Duration;

/// Configuration parameters for HNSW index rebuild backoff and failure escalation.
#[derive(Debug, Clone, Copy)]
pub struct OrphanCleanupBackoffConfig {
    /// Initial cooldown duration after the first failed rebuild attempt.
    pub base_delay: Duration,
    /// Maximum cooldown cap between rebuild attempts.
    pub max_delay: Duration,
    /// Number of consecutive rebuild failures triggering a structural problem alert log.
    pub alert_threshold: u32,
}

impl Default for OrphanCleanupBackoffConfig {
    fn default() -> Self {
        Self {
            base_delay: Duration::from_secs(5),
            max_delay: Duration::from_secs(300),
            alert_threshold: 3,
        }
    }
}

/// Helper calculating exponential backoff cooldown given consecutive failures.
pub fn calculate_rebuild_cooldown(
    consecutive_failures: u32,
    base_delay: Duration,
    max_delay: Duration,
) -> Duration {
    if consecutive_failures == 0 {
        Duration::ZERO
    } else {
        let shift = consecutive_failures.saturating_sub(1).min(30);
        let multiplier = 1u64.checked_shl(shift).unwrap_or(u64::MAX);
        let calculated = base_delay.saturating_mul(multiplier as u32);
        calculated.min(max_delay)
    }
}

/// Abstraction trait over vector indexes capable of connectivity check and rebuild.
pub trait OrphanCleanupIndex: Send + Sync {
    fn check_connectivity(&self) -> contextra_types::Result<()>;
    fn rebuild(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = contextra_types::Result<()>> + Send + '_>>;
}

impl OrphanCleanupIndex for contextra_vector::hnsw::HnswIndex {
    fn check_connectivity(&self) -> contextra_types::Result<()> {
        self.check_connectivity()
    }

    fn rebuild(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = contextra_types::Result<()>> + Send + '_>>
    {
        Box::pin(async move { self.rebuild().await })
    }
}

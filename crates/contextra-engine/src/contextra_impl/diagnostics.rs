use crate::*;
use contextra_core::Result;
use std::sync::Arc;

impl Contextra {
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn stats(&self) -> Result<ContextraStats> {
        let default_col = self.default_col().await?;
        let active_memory_count = default_col.len().await;
        let index_stats = default_col.stats().await?;
        let storage_stats = self.storage.stats().await?;

        let drift_status = self
            .router
            .read()
            .as_ref()
            .and_then(|w| w.upgrade())
            .map(|r| r.overall_drift_status())
            .unwrap_or_else(|| "nicht verfügbar".to_string());

        let (calibration_ece, last_calibration_at) = self
            .calibrator
            .read()
            .as_ref()
            .and_then(|w| w.upgrade())
            .map(|c| {
                let guard = c.lock();
                (guard.cached_ece(), guard.last_calibration_at())
            })
            .unwrap_or((None, None));

        let pid_pool_size = self
            .pid_controller
            .read()
            .as_ref()
            .and_then(|w| w.upgrade())
            .map(|p| p.lock().current_pool_size())
            .unwrap_or(None);

        Ok(ContextraStats {
            drift_status,
            calibration_ece,
            last_calibration_at,
            active_memory_count,
            pid_pool_size,
            index_stats,
            storage_stats,
        })
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn flush(&self) -> Result<()> {
        self.storage.flush().await?;
        Ok(())
    }

    pub fn orphan_registry(&self) -> &Arc<contextra_checkpoint::InstanceOrphanRegistry> {
        &self.orphan_registry
    }
}

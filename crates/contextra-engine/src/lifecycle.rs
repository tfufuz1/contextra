// FILE-CONTEXT
// STAND: 2026-10-14
// ZWECK: EngineLifecycle und ErasureSupervisor (F-21, F-22, M3.7, Spezifikation §7.7).
// INVARIANTEN: Geordnetes Shutdown via CancellationToken, Drop-Garantie, Zero-Panic.

#![forbid(unsafe_code)]

use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use contextra_types::Result;

/// ErasureSupervisor überwacht und steuert Hintergrund-Löschaufträge.
#[derive(Debug, Clone, Default)]
pub struct ErasureSupervisor {
    active_jobs: Arc<std::sync::atomic::AtomicU64>,
}

impl ErasureSupervisor {
    /// Erstellt eine neue Instanz des ErasureSupervisors.
    pub fn new() -> Self {
        Self {
            active_jobs: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Gibt die Anzahl aktiver Löschaufträge zurück.
    pub fn active_job_count(&self) -> u64 {
        self.active_jobs.load(Ordering::SeqCst)
    }
}

/// EngineLifecycle bündelt Hintergrundarbeitsdienste (Reaper, ErasureSupervisor, KV-Eviction)
/// und stellt geordneten Shutdown via Shutdown-Signal, `.close()` und Drop bereit.
pub struct EngineLifecycle {
    supervisor: ErasureSupervisor,
    cancel_token: CancellationToken,
    tasks: Vec<JoinHandle<()>>,
    is_closed: AtomicBool,
}

impl EngineLifecycle {
    /// Erstellt eine neue `EngineLifecycle`-Instanz mit integriertem `ErasureSupervisor`.
    pub fn new() -> Self {
        Self {
            supervisor: ErasureSupervisor::new(),
            cancel_token: CancellationToken::new(),
            tasks: Vec::new(),
            is_closed: AtomicBool::new(false),
        }
    }

    /// Gibt eine Referenz auf den `ErasureSupervisor` zurück.
    pub fn supervisor(&self) -> &ErasureSupervisor {
        &self.supervisor
    }

    /// Liefert ein `CancellationToken` für das geordnete Shutdown-Signal.
    pub fn shutdown_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    /// Startet den Expiry Reaper Task.
    pub fn start_expiry_reaper<F, Fut>(&mut self, task_fn: F)
    where
        F: FnOnce(CancellationToken) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let token = self.cancel_token.clone();
        let handle = tokio::spawn(task_fn(token));
        self.tasks.push(handle);
    }

    /// Startet den Orphan Reaper Task.
    pub fn start_orphan_reaper<F, Fut>(&mut self, task_fn: F)
    where
        F: FnOnce(CancellationToken) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let token = self.cancel_token.clone();
        let handle = tokio::spawn(task_fn(token));
        self.tasks.push(handle);
    }

    /// Startet den Thermostat Reaper Task.
    pub fn start_thermostat_reaper<F, Fut>(&mut self, task_fn: F)
    where
        F: FnOnce(CancellationToken) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let token = self.cancel_token.clone();
        let handle = tokio::spawn(task_fn(token));
        self.tasks.push(handle);
    }

    /// Setzt und startet den KV Eviction Worker Task.
    pub fn set_kv_eviction_worker<F, Fut>(&mut self, task_fn: F)
    where
        F: FnOnce(CancellationToken) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let token = self.cancel_token.clone();
        let handle = tokio::spawn(task_fn(token));
        self.tasks.push(handle);
    }

    /// Beendet alle verknüpften Hintergrundarbeiter geordnet.
    pub async fn close(&mut self) -> Result<()> {
        if self.is_closed.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        self.cancel_token.cancel();

        for handle in self.tasks.drain(..) {
            let _ = handle.await;
        }

        Ok(())
    }
}

impl Default for EngineLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for EngineLifecycle {
    fn drop(&mut self) {
        if !self.is_closed.load(Ordering::SeqCst) {
            self.cancel_token.cancel();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_engine_lifecycle_start_and_close() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut lifecycle = EngineLifecycle::new();

        let c1 = counter.clone();
        lifecycle.start_expiry_reaper(move |cancel| async move {
            while !cancel.is_cancelled() {
                c1.fetch_add(1, Ordering::SeqCst);
                tokio::select! {
                    _ = cancel.cancelled() => break,
                    _ = sleep(Duration::from_millis(10)) => {},
                }
            }
        });

        let c2 = counter.clone();
        lifecycle.start_orphan_reaper(move |cancel| async move {
            while !cancel.is_cancelled() {
                c2.fetch_add(1, Ordering::SeqCst);
                tokio::select! {
                    _ = cancel.cancelled() => break,
                    _ = sleep(Duration::from_millis(10)) => {},
                }
            }
        });

        sleep(Duration::from_millis(30)).await;
        assert!(counter.load(Ordering::SeqCst) > 0);

        lifecycle.close().await.unwrap();
        let val_at_close = counter.load(Ordering::SeqCst);

        sleep(Duration::from_millis(30)).await;
        assert_eq!(counter.load(Ordering::SeqCst), val_at_close);
    }

    #[test]
    fn test_erasure_supervisor_active_jobs() {
        let supervisor = ErasureSupervisor::new();
        assert_eq!(supervisor.active_job_count(), 0);
    }
}

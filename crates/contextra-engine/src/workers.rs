//! Hintergrundtasks (SPEC §5.4, §14.3).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::task::JoinHandle;

/// Repräsentiert einen kooperativ stoppbaren Hintergrund-Worker.
pub struct BackgroundWorker {
    name: &'static str,
    stop_flag: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl BackgroundWorker {
    /// Startet einen neuen Hintergrund-Worker.
    pub fn spawn<F, Fut>(name: &'static str, runtime: &tokio::runtime::Handle, task_fn: F) -> Self
    where
        F: FnOnce(Arc<AtomicBool>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let flag = stop_flag.clone();
        let handle = runtime.spawn(task_fn(flag));
        Self {
            name,
            stop_flag,
            handle: Some(handle),
        }
    }

    /// Liefert den Namen des Workers.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Signaliert dem Worker zu stoppen.
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }

    /// Wartet auf die Beendigung des Workers.
    pub async fn join(&mut self) {
        self.stop();
        if let Some(h) = self.handle.take() {
            let _ = h.await;
        }
    }
}

impl Drop for BackgroundWorker {
    fn drop(&mut self) {
        // Drop blockiert nie (SPEC §5.4, §14.3), setzt nur das Flag.
        self.stop();
    }
}

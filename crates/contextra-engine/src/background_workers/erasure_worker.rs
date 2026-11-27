//! Event-driven erasure worker (Spezifikation §8).
//!
//! Unlike the polling workers this one sleeps on `Notify` / a computed deadline / cancel.
//! With no open jobs there is **no** timer at all (idle CPU 0).

use crate::erasure::ErasureRuntimeInner;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

/// Worker body (spawned by `ErasureRuntime::start` on the engine's task tracker).
pub async fn erasure_worker_loop(rt: Arc<ErasureRuntimeInner>, cancel: CancellationToken) {
    loop {
        if cancel.is_cancelled() {
            break;
        }
        let sleep_for = rt.run_pass().await;
        let wait = async {
            match sleep_for {
                Some(d) => tokio::time::sleep(d).await,
                None => std::future::pending::<()>().await,
            }
        };
        tokio::select! {
            _ = rt.notify.notified() => {}
            _ = wait => {}
            _ = cancel.cancelled() => break,
        }
    }
    tracing::debug!("erasure worker stopped");
}

/// Spawns the worker as a free-standing task (tests, embedders without a task tracker).
pub fn start_erasure_worker(rt: Arc<ErasureRuntimeInner>, cancel: CancellationToken) -> JoinHandle<()> {
    tokio::spawn(erasure_worker_loop(rt, cancel))
}

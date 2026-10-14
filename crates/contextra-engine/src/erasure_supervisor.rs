//! Erasure supervisor task ensuring roll-forward completion after commit (§5.3).

#![forbid(unsafe_code)]

use contextra_types::Result;
use tokio::sync::oneshot;

/// Task supervisor ensuring that post-commit deletion operations run to completion
/// regardless of caller future cancellation (CRASH-10 / Roll-Forward).
#[derive(Debug, Clone, Default)]
pub struct ErasureSupervisor {
    _private: (),
}

impl ErasureSupervisor {
    /// Creates a new `ErasureSupervisor`.
    #[must_use]
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Spawns a non-cancellable roll-forward task.
    ///
    /// Even if the outer caller future is dropped after the Commit-Punkt, the spawned async task
    /// runs to completion in the background, completing tombstone commit and RAM index cleanup.
    pub fn spawn_roll_forward<F, T>(&self, f: F) -> oneshot::Receiver<Result<T>>
    where
        F: std::future::Future<Output = Result<T>> + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        tokio::spawn(async move {
            let res = f.await;
            let _ = tx.send(res);
        });
        rx
    }
}

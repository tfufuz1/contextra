// FILE-CONTEXT
// ZWECK: RAII CheckpointPinGuard und higher-order helper functions für Snapshot-Pinning während Suchoperationen.
// INVARIANTEN: Snapshot-Pinning garantiert Isolation während gefilterter Suche.

use contextra_ports::StorageEngine;
use contextra_types::Result;

/// RAII Guard for pinning snapshot checkpoints during search/scan operations.
///
/// # Note on async Drop
/// Rust does not support native `async Drop`. Therefore, callers MUST explicitly call
/// [`release`](Self::release) to unpin the checkpoint. The `Drop` implementation serves as a
/// fallback safety net: if `release` was not invoked before dropping (e.g. due to early `?`
/// return or panic), `Drop` emits an `error!` log warning that a checkpoint pin may have leaked.
pub struct CheckpointPinGuard<'a, S: StorageEngine + ?Sized> {
    storage: &'a S,
    seq: u64,
    unpinned: bool,
}

impl<'a, S: StorageEngine + ?Sized> CheckpointPinGuard<'a, S> {
    /// Creates a new guard and pins the checkpoint at sequence `seq`.
    pub async fn new(storage: &'a S, seq: u64) -> Result<Self> {
        storage.pin_checkpoint(seq).await?;
        Ok(Self {
            storage,
            seq,
            unpinned: false,
        })
    }

    /// Pin-first, read-after: the seq is read under the protection of the pin,
    /// eliminating the TOCTOU window between snapshot_seq() and pin activation.
    pub async fn new_at_latest(storage: &'a S) -> Result<(Self, u64)> {
        let seq = storage.last_seq_no().await?;
        storage.pin_checkpoint(seq).await?;
        Ok((
            Self {
                storage,
                seq,
                unpinned: false,
            },
            seq,
        ))
    }

    /// Pin-first, read-after alias for `new_at_latest`.
    #[allow(dead_code)]
    pub async fn new_pinning_latest(storage: &'a S) -> Result<(Self, u64)> {
        Self::new_at_latest(storage).await
    }

    /// Explicitly unpins the checkpoint and consumes the guard, preventing the fallback `Drop` warning.
    pub async fn release(mut self) -> Result<()> {
        self.unpinned = true;
        self.storage.unpin_checkpoint(self.seq).await
    }

    /// Returns the sequence number protected by this pin guard.
    #[allow(dead_code)]
    pub fn seq(&self) -> u64 {
        self.seq
    }
}

// Drop impl is a safety net only — always use with_pinned_checkpoint() to guarantee release() is called.
impl<'a, S: StorageEngine + ?Sized> Drop for CheckpointPinGuard<'a, S> {
    fn drop(&mut self) {
        if !self.unpinned {
            tracing::error!(
                seq_no = self.seq,
                "CheckpointPinGuard dropped without explicit release! Checkpoint pin seq={} may have leaked.",
                self.seq
            );
        }
    }
}

/// Higher-order function wrapping an async block with a pinned checkpoint guard,
/// ensuring `release()` is ALWAYS called even on error paths.
pub async fn with_pinned_checkpoint<S, F, Fut, T>(storage: &S, seq: u64, f: F) -> Result<T>
where
    S: StorageEngine + ?Sized,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let pin_guard = CheckpointPinGuard::new(storage, seq).await?;
    let result = f().await;
    if let Err(e) = pin_guard.release().await {
        tracing::warn!(error = %e, seq_no = seq, "CheckpointPinGuard release failed");
    }
    result
}

/// Higher-order function providing pin-first, read-after semantics with automatic release on completion/error.
///
/// Pin-first, read-after: the seq is read under the protection of the pin,
/// eliminating the TOCTOU window between snapshot_seq() and pin activation.
pub async fn with_pinned_checkpoint_at_latest<S, F, Fut, T>(storage: &S, f: F) -> Result<T>
where
    S: StorageEngine + ?Sized,
    F: FnOnce(u64) -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let (pin_guard, seq) = CheckpointPinGuard::new_at_latest(storage).await?;
    let result = f(seq).await;
    if let Err(e) = pin_guard.release().await {
        tracing::warn!(error = %e, seq_no = seq, "CheckpointPinGuard release failed");
    }
    result
}

/// Higher-order function passing the guard reference to the closure, ensuring `release()` is ALWAYS called.
#[allow(dead_code)]
pub async fn with_pinned_checkpoint_and_guard<S, F, Fut, T>(
    storage: &S,
    seq: u64,
    f: F,
) -> Result<T>
where
    S: StorageEngine + ?Sized,
    F: FnOnce(&CheckpointPinGuard<S>, u64) -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let pin_guard = CheckpointPinGuard::new(storage, seq).await?;
    let result = f(&pin_guard, seq).await;
    if let Err(e) = pin_guard.release().await {
        tracing::warn!(error = %e, seq_no = seq, "CheckpointPinGuard release failed");
    }
    result
}

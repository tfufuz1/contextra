// FILE-CONTEXT
// ZWECK: Worker-Tasks zur Bereinigung verwaister Transaktionen (Orphan Cleanup) und HNSW-Index-Rebuilds.
// INVARIANTEN: Geordnete Abschaltung via CancellationToken; Beschränkung der pro Tick verarbeiteten Waisen.

use super::config::{calculate_rebuild_cooldown, OrphanCleanupBackoffConfig, OrphanCleanupIndex};
use contextra_mvcc::tx_buffer::TxBuffer;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Maximum number of orphan transactions processed in a single worker tick
/// to avoid starving foreground operations.
pub const MAX_ORPHANS_PER_TICK: usize = 100;

/// Starts a background task to periodically clean up orphan transactions and manage HNSW rebuilds with exponential backoff.
pub fn start_orphan_cleanup_worker_with_config<
    T: Clone + Send + Sync + 'static,
    I: OrphanCleanupIndex + 'static,
>(
    buffer: Arc<TxBuffer<T>>,
    hnsw_index: Arc<I>,
    interval: Duration,
    backoff_config: OrphanCleanupBackoffConfig,
    cancel_token: tokio_util::sync::CancellationToken,
) -> (tokio::task::JoinHandle<()>, Arc<AtomicU32>) {
    let consecutive_failed_rebuilds = Arc::new(AtomicU32::new(0));
    let failure_counter = consecutive_failed_rebuilds.clone();

    let handle = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let mut last_rebuild_attempt: Option<std::time::Instant> = None;

        tracing::info!(
            "Orphan cleanup worker started (timeout: {:?}, interval: {:?}, base_backoff: {:?})",
            buffer.tx_timeout(),
            interval,
            backoff_config.base_delay
        );
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    let buf = buffer.clone();
                    let expired = tokio::task::spawn_blocking(move || {
                        buf.reap_orphans_bounded(MAX_ORPHANS_PER_TICK)
                    })
                    .await
                    .unwrap_or_default();

                    if !expired.is_empty() {
                        tracing::warn!(
                            "Orphan cleanup worker cleaned up {} expired transactions",
                            expired.len()
                        );
                    }

                    if let Err(err) = hnsw_index.check_connectivity() {
                        let failures = consecutive_failed_rebuilds.load(Ordering::Relaxed);
                        let cooldown = calculate_rebuild_cooldown(
                            failures,
                            backoff_config.base_delay,
                            backoff_config.max_delay,
                        );

                        let is_on_cooldown = if let Some(last_attempt) = last_rebuild_attempt {
                            last_attempt.elapsed() < cooldown
                        } else {
                            false
                        };

                        if is_on_cooldown {
                            if let Some(last_attempt) = last_rebuild_attempt {
                                let remaining = cooldown.saturating_sub(last_attempt.elapsed());
                                tracing::debug!(
                                    failures = failures,
                                    cooldown_remaining_ms = remaining.as_millis(),
                                    "HNSW index degraded but rebuild is on cooldown, skipping this tick"
                                );
                            }
                        } else {
                            tracing::warn!(
                                error = %err,
                                failures = failures,
                                "HNSW index degraded — triggering automatic rebuild"
                            );

                            last_rebuild_attempt = Some(std::time::Instant::now());

                            let rebuild_res = tokio::time::timeout(
                                Duration::from_secs(120),
                                hnsw_index.rebuild(),
                            )
                            .await;

                            let rebuild_ok = match rebuild_res {
                                Ok(Ok(())) => true,
                                Ok(Err(rebuild_err)) => {
                                    tracing::error!(error = %rebuild_err, "HNSW rebuild failed");
                                    false
                                }
                                Err(_) => {
                                    tracing::warn!("HNSW rebuild timed out after 120s; skipping this tick");
                                    false
                                }
                            };

                            let connectivity_restored = rebuild_ok && hnsw_index.check_connectivity().is_ok();

                            if connectivity_restored {
                                consecutive_failed_rebuilds.store(0, Ordering::Relaxed);
                                last_rebuild_attempt = None;
                                tracing::info!("HNSW rebuild succeeded and index connectivity restored");
                            } else {
                                let new_failures = consecutive_failed_rebuilds.fetch_add(1, Ordering::Relaxed) + 1;
                                tracing::error!(
                                    consecutive_failures = new_failures,
                                    "HNSW rebuild did not restore index connectivity"
                                );

                                if new_failures >= backoff_config.alert_threshold {
                                    tracing::error!(
                                        consecutive_failures = new_failures,
                                        "ALERT[STRUCTURAL_PROBLEM]: HNSW index rebuild failed {} consecutive times to restore connectivity. Deletion rate may exceed rebuild capacity or index is corrupted.",
                                        new_failures
                                    );
                                }
                            }
                        }
                    } else if consecutive_failed_rebuilds.load(Ordering::Relaxed) > 0 {
                        consecutive_failed_rebuilds.store(0, Ordering::Relaxed);
                        last_rebuild_attempt = None;
                        tracing::info!("HNSW index connectivity healthy, reset failed rebuild counter");
                    }
                }
                _ = cancel_token.cancelled() => {
                    tracing::info!("Orphan cleanup worker shutting down via token");
                    break;
                }
            }
        }
    });

    (handle, failure_counter)
}

/// Starts a background task to periodically clean up orphan transactions.
///
/// This worker handles the cleanup of transactions that have exceeded their
/// configured timeout without being committed or rolled back.
pub fn start_orphan_cleanup_worker<T: Clone + Send + Sync + 'static>(
    buffer: Arc<TxBuffer<T>>,
    hnsw_index: Arc<contextra_vector::hnsw::HnswIndex>,
    interval: Duration,
    cancel_token: tokio_util::sync::CancellationToken,
) -> tokio::task::JoinHandle<()> {
    let (handle, _) = start_orphan_cleanup_worker_with_config(
        buffer,
        hnsw_index,
        interval,
        OrphanCleanupBackoffConfig::default(),
        cancel_token,
    );
    handle
}

/// Deprecated legacy alias for `start_orphan_cleanup_worker`.
#[deprecated(note = "use start_orphan_cleanup_worker instead")]
pub fn start_orphan_reaper<T: Clone + Send + Sync + 'static>(
    buffer: Arc<TxBuffer<T>>,
    hnsw_index: Arc<contextra_vector::hnsw::HnswIndex>,
    interval: Duration,
    cancel_token: tokio_util::sync::CancellationToken,
) -> tokio::task::JoinHandle<()> {
    start_orphan_cleanup_worker(buffer, hnsw_index, interval, cancel_token)
}

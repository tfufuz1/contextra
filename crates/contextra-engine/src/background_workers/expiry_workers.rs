// FILE-CONTEXT
// ZWECK: Worker-Tasks zur TTL-Löschung abgelaufener Dokumente und Entropie-Pruning/Decay-Eviction.
// INVARIANTEN: Intervallsteuerung und geordnete Task-Beendigung über CancellationToken.

use crate::collection::Collection;
#[cfg(feature = "background-maintenance")]
use contextra_adapt::{AdaptiveDecayController, DecayControllerConfig};
use contextra_ports::StorageEngine;
#[cfg(feature = "background-maintenance")]
use contextra_ports::VectorIndex;
use std::sync::Arc;
use std::time::Duration;

/// Maximum number of expired documents processed in a single expiry cleanup tick.
pub const MAX_EXPIRED_PER_TICK: usize = 100;

/// Starts a background task to periodically clean up expired documents with TTL.
pub fn start_expiry_cleanup_worker<S: StorageEngine>(
    collection: Arc<Collection<S>>,
    interval: Duration,
    cancel_token: tokio_util::sync::CancellationToken,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        tracing::info!(
            collection = %collection.name(),
            interval = ?interval,
            "Expiry cleanup worker task started"
        );
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    match collection.reap_expired_documents(MAX_EXPIRED_PER_TICK).await {
                        Ok(reaped) if reaped > 0 => {
                            tracing::info!(
                                collection = %collection.name(),
                                reaped = reaped,
                                "Expiry cleanup worker cleaned up expired documents"
                            );
                        }
                        Ok(_) => {}
                        Err(err) => {
                            tracing::error!(
                                collection = %collection.name(),
                                error = %err,
                                "Error during expiry cleanup worker execution"
                            );
                        }
                    }
                }
                _ = cancel_token.cancelled() => {
                    tracing::info!(
                        collection = %collection.name(),
                        "Expiry cleanup worker task shutting down via token"
                    );
                    break;
                }
            }
        }
    })
}

/// Deprecated legacy alias for `start_expiry_cleanup_worker`.
#[deprecated(note = "use start_expiry_cleanup_worker instead")]
pub fn start_expiry_reaper<S: StorageEngine>(
    collection: Arc<Collection<S>>,
    interval: Duration,
    cancel_token: tokio_util::sync::CancellationToken,
) -> tokio::task::JoinHandle<()> {
    start_expiry_cleanup_worker(collection, interval, cancel_token)
}

/// Starts a background task for decay-controller-driven importance-score eviction.
/// Nur aktiv wenn `background-maintenance` Feature-Flag gesetzt.
#[cfg(feature = "background-maintenance")]
pub fn start_decay_cleanup_worker<S: StorageEngine, V: VectorIndex>(
    collection: Arc<Collection<S, V>>,
    decay_config: DecayControllerConfig,
    interval: Duration,
    cancel_token: tokio_util::sync::CancellationToken,
) -> tokio::task::JoinHandle<()> {
    let decay_controller = AdaptiveDecayController::new(decay_config);
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    match collection.evict_decayed_chunks(&decay_controller, 100).await {
                        Ok(n) if n > 0 => tracing::info!(evicted = n, "Decay controller worker evicted chunks"),
                        Ok(_) => {},
                        Err(e) => tracing::error!(error = %e, "Decay controller worker error"),
                    }
                }
                _ = cancel_token.cancelled() => break,
            }
        }
    })
}

/// Deprecated legacy alias for `start_decay_cleanup_worker`.
#[cfg(feature = "background-maintenance")]
#[deprecated(note = "use start_decay_cleanup_worker instead")]
pub fn start_thermostat_reaper<S: StorageEngine, V: VectorIndex>(
    collection: Arc<Collection<S, V>>,
    decay_config: DecayControllerConfig,
    interval: Duration,
    cancel_token: tokio_util::sync::CancellationToken,
) -> tokio::task::JoinHandle<()> {
    start_decay_cleanup_worker(collection, decay_config, interval, cancel_token)
}

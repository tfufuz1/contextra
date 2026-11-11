// FILE-CONTEXT
// ZWECK: Öffentliche Einstiegspunkte für HNSW-Rebuild, Status, Wartefunktionen und Konnektivitätsprüfung.
// INVARIANTEN: Rebuild atomar per Swap; Maintain-Trait Implementierung; INV-VEC-CONNECTIVITY-1.
// HOTSPOTS: rebuild_sync_with_clock, verify_full_connectivity, do_delete
// STAND: TS:2026-11-11T02:00:00Z

use std::sync::atomic::Ordering;

use crate::error::{Result, VectorError};
use crate::hnsw::types::{HnswIndex, HnswIndexCore, RebuildGuard, RebuildStatus};
use crate::types::DocId;

use super::backoff::{get_last_rebuild_nanos, set_last_rebuild_nanos};
use super::seq_replay::replay_delta_and_swap;

impl HnswIndexCore {
    // ANCHOR[contextra-vector:REBUILD-STATUS]
    pub fn rebuild_status(&self) -> RebuildStatus {
        if self.hot.rebuilding.load(Ordering::SeqCst) {
            RebuildStatus::Running
        } else {
            RebuildStatus::Idle
        }
    }

    // ANCHOR[contextra-vector:REBUILD-WAIT-BLOCKING]
    pub fn wait_for_rebuild_blocking(
        &self,
        timeout: std::time::Duration,
        _clock: &dyn crate::types::Clock,
    ) -> bool {
        if !self.hot.rebuilding.load(Ordering::Acquire) {
            return true;
        }
        let mut guard = self.hot.rebuild_mutex.lock();
        while self.hot.rebuilding.load(Ordering::Acquire) {
            let res = self.hot.rebuild_condvar.wait_for(&mut guard, timeout);
            if res.timed_out() {
                return !self.hot.rebuilding.load(Ordering::Acquire);
            }
        }
        true
    }

    pub fn wait_for_rebuild(&self) -> bool {
        let clock = self.cold.clock.read().clone();
        self.wait_for_rebuild_with_clock(clock.as_ref())
    }

    pub fn wait_for_rebuild_with_clock(&self, clock: &dyn crate::types::Clock) -> bool {
        self.wait_for_rebuild_blocking(std::time::Duration::from_secs(60), clock)
    }

    pub fn wait_for_rebuild_with_timeout(&self, timeout: std::time::Duration) -> bool {
        let clock = self.cold.clock.read().clone();
        self.wait_for_rebuild_blocking(timeout, clock.as_ref())
    }

    pub fn wait_for_rebuild_with_timeout_and_clock(
        &self,
        timeout: std::time::Duration,
        clock: &dyn crate::types::Clock,
    ) -> bool {
        self.wait_for_rebuild_blocking(timeout, clock)
    }

    pub(crate) fn do_delete(&self, id: DocId) -> Result<()> {
        let node_idx = self.hot.doc_to_node.write().remove(&id.inner());
        if let Some(idx) = node_idx {
            self.cold.deleted_nodes.write().insert(idx as u64);
            self.hot.deleted_count.fetch_add(1, Ordering::SeqCst);

            let ep_val = self.hot.get_entry_point();
            let ram_ep_val = self.hot.get_ram_entry_point();

            if ep_val == Some(idx) || ram_ep_val == Some(idx) {
                let nodes = self.hot.nodes.read();
                let mmap_guard = self.cold.mmap_index.read();
                let mmap_node_count = mmap_guard
                    .as_ref()
                    .map(|m| m.header.node_count() as usize)
                    .unwrap_or(0);
                let deleted = self.cold.deleted_nodes.read();

                let mut best_node = None;
                let mut best_ram_node = None;
                let mut max_layer = 0;
                let mut max_ram_layer = 0;

                if let Some(mmap) = mmap_guard.as_ref() {
                    for i in 0..mmap_node_count {
                        if i != idx && !deleted.contains(i as u64) {
                            let record = match mmap.get_node_record(i) {
                                Ok(r) => r,
                                Err(_) => continue,
                            };
                            let rec_layer = record.max_layer as usize;
                            if rec_layer > max_layer
                                || (rec_layer == max_layer && best_node.is_none_or(|b| i < b))
                            {
                                max_layer = rec_layer;
                                best_node = Some(i);
                            }
                        }
                    }
                }

                for (i, node) in nodes.iter().enumerate() {
                    let global_idx = mmap_node_count + i;
                    if global_idx != idx && !deleted.contains(global_idx as u64) {
                        if node.max_layer > max_layer
                            || (node.max_layer == max_layer
                                && best_node.is_none_or(|b| global_idx < b))
                        {
                            max_layer = node.max_layer;
                            best_node = Some(global_idx);
                        }
                        if node.max_layer > max_ram_layer
                            || (node.max_layer == max_ram_layer
                                && best_ram_node.is_none_or(|b| global_idx < b))
                        {
                            max_ram_layer = node.max_layer;
                            best_ram_node = Some(global_idx);
                        }
                    }
                }

                if ep_val == Some(idx) {
                    self.hot.set_entry_point(best_node);
                    if let Some(new_idx) = best_node {
                        let node_max_layer = if let Some(mmap) = mmap_guard.as_ref() {
                            if new_idx < mmap_node_count {
                                mmap.get_node_record(new_idx)
                                    .map(|r| r.max_layer as usize)
                                    .unwrap_or(0)
                            } else {
                                nodes[new_idx - mmap_node_count].max_layer
                            }
                        } else {
                            nodes[new_idx].max_layer
                        };
                        self.hot
                            .max_layer
                            .store(node_max_layer as u64, Ordering::SeqCst);
                    } else {
                        self.hot.max_layer.store(0, Ordering::SeqCst);
                    }
                }

                if ram_ep_val == Some(idx) {
                    self.hot.set_ram_entry_point(best_ram_node);
                }
            }
        }
        Ok(())
    }

    pub fn deleted_ratio(&self) -> f64 {
        1.0 - self.connectivity_score()
    }

    pub fn rebuild_count(&self) -> u64 {
        self.cold.rebuild_count.load(Ordering::SeqCst)
    }

    pub fn visited_dead_nodes(&self) -> u64 {
        self.cold.visited_dead_nodes.load(Ordering::SeqCst)
    }

    pub fn is_rebuild_required(&self) -> bool {
        if self.connectivity_score() < self.cold.config.rebuild_threshold {
            return true;
        }
        if self.cold.config.quantize {
            if let Some(q) = self.cold.quantizer.read().as_ref() {
                if q.is_rebuild_required(self.cold.config.quantizer_drift_threshold) {
                    return true;
                }
            }
        }
        false
    }

    pub fn rebuild(&self) -> Result<()> {
        let clock = self.cold.clock.read().clone();
        self.rebuild_with_clock(clock.as_ref())
    }

    pub fn rebuild_with_clock(&self, clock: &dyn crate::types::Clock) -> Result<()> {
        self.rebuild_sync_with_clock(clock)
    }

    pub fn rebuild_sync(&self) -> Result<()> {
        let clock = self.cold.clock.read().clone();
        self.rebuild_sync_with_clock(clock.as_ref())
    }

    // ANCHOR[contextra-vector:REBUILD-SYNC-WITH-CLOCK]
    pub fn rebuild_sync_with_clock(&self, clock: &dyn crate::types::Clock) -> Result<()> {
        let now_nanos = clock.monotonic_nanos();
        {
            if let Some(last_nanos) = get_last_rebuild_nanos(self) {
                let elapsed_nanos = now_nanos.saturating_sub(last_nanos);
                let elapsed = std::time::Duration::from_nanos(elapsed_nanos);
                let consecutive = self.cold.consecutive_rapid_rebuilds.load(Ordering::SeqCst);

                let cooldown = if consecutive == 0 {
                    self.cold.config.backoff_base_delay
                } else {
                    let shift = consecutive.saturating_sub(1).min(30);
                    let multiplier = 1u64.checked_shl(shift as u32).unwrap_or(u64::MAX);
                    let calc = self
                        .cold
                        .config
                        .backoff_base_delay
                        .saturating_mul(multiplier as u32);
                    calc.min(self.cold.config.backoff_max_delay)
                };

                if elapsed < cooldown {
                    let new_consecutive = self
                        .cold
                        .consecutive_rapid_rebuilds
                        .fetch_add(1, Ordering::SeqCst)
                        + 1;
                    let sink = self.cold.metrics_sink.read().clone();
                    sink.record_counter(
                        "vector_rebuild_backoff_suppressed_total",
                        1,
                        &[("reason", "rapid_rebuild")],
                    );

                    if new_consecutive >= self.cold.config.backoff_alert_threshold {
                        sink.record_counter(
                            "vector_rebuild_alarm_total",
                            1,
                            &[("reason", "excessive_rapid_rebuilds")],
                        );
                        tracing::warn!(
                            consecutive_rebuilds = new_consecutive,
                            cooldown_seconds = cooldown.as_secs_f64(),
                            "Excessive HNSW rebuilds triggered in rapid succession; suppressing rebuild due to backoff"
                        );
                    } else {
                        tracing::warn!(
                            cooldown_seconds = cooldown.as_secs_f64(),
                            "HNSW rebuild suppressed by backoff cooldown"
                        );
                    }

                    return Err(VectorError::Index(format!(
                        "Rebuild rate limited by backoff cooldown ({:?})",
                        cooldown
                    )));
                } else if elapsed >= self.cold.config.backoff_max_delay.saturating_mul(2) {
                    self.cold
                        .consecutive_rapid_rebuilds
                        .store(0, Ordering::SeqCst);
                }
            }
        }

        if self.hot.rebuilding.swap(true, Ordering::SeqCst) {
            tracing::debug!("HNSW rebuild already in progress, skipping");
            return Ok(());
        }
        let _guard = RebuildGuard(&self.hot.rebuilding, &self.hot.rebuild_condvar);

        tracing::info!("Starting HNSW index rebuild (Phase 1)");
        let start_nanos = clock.monotonic_nanos();

        let rebuild_res =
            self.rebuild_phase1_snapshot_and_build()
                .and_then(|(new_index, snapshot_tx)| {
                    tracing::info!(
                        "HNSW index rebuild Phase 1 completed, starting Phase 2 merge & swap"
                    );
                    replay_delta_and_swap(self, new_index, snapshot_tx)
                });

        let elapsed_secs =
            (clock.monotonic_nanos().saturating_sub(start_nanos) as f64) / 1_000_000_000.0;
        let sink = self.cold.metrics_sink.read().clone();

        if rebuild_res.is_ok() {
            self.cold.rebuild_count.fetch_add(1, Ordering::SeqCst);
            set_last_rebuild_nanos(self, clock.monotonic_nanos());
            self.cold
                .consecutive_rapid_rebuilds
                .store(0, Ordering::SeqCst);
            sink.record_histogram("vector_rebuild_duration_seconds", elapsed_secs, &[]);
            sink.record_counter("vector_rebuild_total", 1, &[("status", "success")]);
            tracing::info!("HNSW rebuild completed in {:.6}s", elapsed_secs);
        } else {
            sink.record_counter("vector_rebuild_total", 1, &[("status", "error")]);
        }

        rebuild_res
    }
}

impl HnswIndex {
    /// Führt Graphtraversierung durch zur Überprüfung von `INV-VEC-CONNECTIVITY-1`.
    pub fn verify_full_connectivity(&self) -> bool {
        let mmap_guard = self.inner.cold.mmap_index.read();
        let mmap_node_count = mmap_guard
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);
        let nodes = self.inner.hot.nodes.read();
        let deleted = self.inner.cold.deleted_nodes.read();
        let total = mmap_node_count + nodes.len();

        let mut alive_nodes = ahash::AHashSet::new();
        for i in 0..total {
            if !deleted.contains(i as u64) {
                alive_nodes.insert(i as u32);
            }
        }

        if alive_nodes.is_empty() {
            return true;
        }

        let ep = match self.inner.hot.get_entry_point() {
            Some(e) => e as u32,
            None => return false,
        };

        if !alive_nodes.contains(&ep) {
            return false;
        }

        let m = self.inner.cold.config.m;
        let mut visited = ahash::AHashSet::new();
        let mut queue = std::collections::VecDeque::new();

        visited.insert(ep);
        queue.push_back(ep);

        while let Some(curr) = queue.pop_front() {
            let curr_idx = curr as usize;
            let max_layer = if curr_idx < mmap_node_count {
                if let Some(mmap) = mmap_guard.as_ref() {
                    mmap.get_node_record(curr_idx)
                        .map(|r| r.max_layer as usize)
                        .unwrap_or(0)
                } else {
                    0
                }
            } else {
                let ram_idx = curr_idx - mmap_node_count;
                nodes.get(ram_idx).map(|n| n.max_layer).unwrap_or(0)
            };

            for layer in 0..=max_layer {
                let conns = if curr_idx < mmap_node_count {
                    if let Some(mmap) = mmap_guard.as_ref() {
                        if let Ok(rec) = mmap.get_node_record(curr_idx) {
                            mmap.get_connections(&rec, layer).unwrap_or_default()
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    let ram_idx = curr_idx - mmap_node_count;
                    self.inner
                        .hot
                        .arena
                        .get_ram_node_connections(ram_idx, layer, m)
                };

                for &neighbor in &conns {
                    if alive_nodes.contains(&neighbor) && visited.insert(neighbor) {
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        alive_nodes.is_subset(&visited)
    }
}

impl crate::types::Maintain for HnswIndex {
    fn compact(&self) -> crate::error::Result<()> {
        self.inner
            .rebuild_sync()
            .map_err(|e| crate::error::VectorError::Index(e.to_string()))?;
        if !self.verify_full_connectivity() {
            return Err(crate::error::VectorError::ConnectivityDegraded {
                deleted_ratio: self.deleted_ratio(),
            });
        }
        Ok(())
    }

    fn is_rebuild_required(&self) -> bool {
        self.is_rebuild_required()
    }
}

impl HnswIndex {
    /// Führt den synchronen Rebuild mit der übergebenen Uhr-Instanz aus.
    pub fn rebuild_sync_with_clock(&self, clock: &dyn crate::types::Clock) -> Result<()> {
        self.inner.rebuild_sync_with_clock(clock)
    }

    /// Führt den Rebuild mit der übergebenen Uhr-Instanz aus.
    pub fn rebuild_with_clock(&self, clock: &dyn crate::types::Clock) -> Result<()> {
        self.inner.rebuild_with_clock(clock)
    }

    /// Wartet auf den Abschluss eines laufenden Rebuilds mit Timeout und Uhr.
    pub fn wait_for_rebuild_with_timeout_and_clock(
        &self,
        timeout: std::time::Duration,
        clock: &dyn crate::types::Clock,
    ) -> bool {
        self.inner
            .wait_for_rebuild_with_timeout_and_clock(timeout, clock)
    }
}

impl HnswIndex {
    /// Liefert den aktuellen Backoff-Tracker Zustand für Diagnose- und Testzwecke.
    pub fn create_backoff_tracker(&self) -> super::backoff::BackoffTracker {
        super::backoff::BackoffTracker::new(
            self.inner.cold.config.backoff_base_delay,
            self.inner.cold.config.backoff_max_delay,
            self.inner.cold.config.backoff_alert_threshold as u64,
        )
    }
}

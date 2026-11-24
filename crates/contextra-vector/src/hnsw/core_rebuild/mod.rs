// FILE-CONTEXT
// ZWECK: Index-Reorganisation (Rebuild Phase 1 & 2), Backoff-Cooldown und globale Konnektivitäts-Verifikation.
// INVARIANTEN: Zero-Panic; deterministische Rekonstruktion (INV-INDEX-APPLY-1); Swap unter write_mutex (SSI-Determinismus).
// HOTSPOTS: full.rs, snapshot.rs, seq_replay.rs, backoff.rs, region.rs
// STAND: TS:2026-11-11T02:00:00Z

//! HNSW-Rebuild-Verzeichnismodul.
//!
//! Kapselt Phase 1 (Snapshot & Re-Indexierung), Phase 2 (Sequenzlog-Wiedergabe & atomarer Swap),
//! Backoff-Ratenbegrenzung sowie lokale Teil-Rebuilds von Knotenregionen.

pub mod backoff;
pub mod full;
pub mod region;
pub mod seq_replay;
pub mod snapshot;

pub use backoff::BackoffTracker;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hnsw::batch::SearchContext;
    use crate::hnsw::config::HnswConfig;
    use crate::hnsw::types::HnswIndex;
    use crate::types::{Clock, DocId, SeededRng};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    struct TestClock {
        nanos: AtomicU64,
    }

    impl TestClock {
        fn new(start_nanos: u64) -> Self {
            Self {
                nanos: AtomicU64::new(start_nanos),
            }
        }

        fn advance_nanos(&self, delta: u64) {
            self.nanos.fetch_add(delta, Ordering::SeqCst);
        }
    }

    impl Clock for TestClock {
        fn now_unix_nanos(&self) -> u64 {
            self.nanos.load(Ordering::SeqCst)
        }

        fn monotonic_nanos(&self) -> u64 {
            self.nanos.load(Ordering::SeqCst)
        }
    }

    #[test]
    fn test_core_rebuild_no_direct_time_and_deterministic_with_same_seed() {
        let clock = TestClock::new(1_000_000_000);

        let build_and_rebuild_index = |seed: u64| -> crate::error::Result<(f32, Vec<u32>, usize)> {
            let mut config = HnswConfig::default();
            config.dimension = 4;
            config.m = 8;
            config.ef_construction = 16;
            let rng: Arc<dyn crate::types::Rng> = Arc::new(SeededRng::new(seed));
            let index = HnswIndex::try_new_with_rng(config, rng)?;

            for i in 1..=20 {
                let doc_id = DocId::new(i as u64);
                let vec = vec![i as f32, (i * 2) as f32, (i * 3) as f32, (i * 4) as f32];
                index.inner.do_insert(doc_id, &vec, i as u64)?;
            }

            for i in 1..=5 {
                index.inner.do_delete(DocId::new(i as u64))?;
            }

            clock.advance_nanos(10_000_000_000);
            assert!(clock.now_unix_nanos() > 0);
            index.inner.rebuild_sync_with_clock(&clock)?;

            let ep = index.inner.hot.get_entry_point().unwrap_or(0);
            let conns = index.inner.hot.get_ram_node_connections(0, 0, 8);

            let query = vec![10.0, 20.0, 30.0, 40.0];
            let nodes_guard = index.inner.hot.nodes.read();
            let search_ctx = SearchContext {
                mmap: None,
                nodes: nodes_guard.as_slice(),
                mmap_node_count: 0,
                prior_prepared: &[],
                quantizer: None,
                arena: &index.inner.hot.arena,
                backlink_map: None,
            };
            let dist = index.inner.resolve_dist(0, &query, None, &search_ctx)?;

            Ok((dist, conns, ep))
        };

        let run1 = build_and_rebuild_index(42).expect("run 1 succeeds");
        let run2 = build_and_rebuild_index(42).expect("run 2 succeeds");

        assert_eq!(run1.2, run2.2, "Entry points must be bit-identical");
        assert_eq!(run1.1, run2.1, "RAM connections must be bit-identical");
        assert_eq!(run1.0, run2.0, "Distances must be bit-identical");
    }

    #[test]
    fn test_core_rebuild_last_rebuild_nanos_field_backoff() {
        let clock = Arc::new(TestClock::new(10_000_000_000));
        let mut config = HnswConfig::default();
        config.dimension = 4;
        config.backoff_base_delay = std::time::Duration::from_millis(500);
        let rng: Arc<dyn crate::types::Rng> = Arc::new(SeededRng::new(42));
        let index = HnswIndex::try_new_with_rng_and_clock(config, rng, clock.clone())
            .expect("index creation succeeds");

        assert_eq!(
            index.inner.cold.last_rebuild_nanos.load(Ordering::Acquire),
            0
        );
        index.inner.rebuild_sync().expect("rebuild succeeds");
        let first_rebuild_nanos = index.inner.cold.last_rebuild_nanos.load(Ordering::Acquire);
        assert_eq!(first_rebuild_nanos, 10_000_000_000);

        let err = index.inner.rebuild_sync().unwrap_err();
        assert!(format!("{err:?}").contains("Rebuild rate limited by backoff"));

        clock.advance_nanos(600_000_000);
        index
            .inner
            .rebuild_sync()
            .expect("second rebuild succeeds after clock advance");
        let second_rebuild_nanos = index.inner.cold.last_rebuild_nanos.load(Ordering::Acquire);
        assert_eq!(second_rebuild_nanos, 10_600_000_000);
    }
}

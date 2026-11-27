//! Small state model for the catalog publication order used by flush, compaction and rollover.

#[cfg(loom)]
mod model {
    use loom::sync::{Arc, Mutex};
    use loom::thread;
    use std::collections::BTreeSet;

    #[derive(Clone)]
    struct Catalog {
        manifest: BTreeSet<u8>,
        ram: BTreeSet<u8>,
        rollover_snapshot: Option<BTreeSet<u8>>,
    }

    #[test]
    fn loom_catalog_publish_serializes_flush_compaction_and_rollover() {
        let mut builder = loom::model::Builder::new();
        builder.preemption_bound = Some(2);
        builder.check(|| {
            let catalog = Arc::new(Mutex::new(Catalog {
                manifest: BTreeSet::from([1]),
                ram: BTreeSet::from([1]),
                rollover_snapshot: None,
            }));

            let flush_catalog = Arc::clone(&catalog);
            let flush = thread::spawn(move || {
                let mut state = flush_catalog.lock().unwrap();
                state.manifest.insert(2);
                state.ram.insert(2);
            });

            let compact_catalog = Arc::clone(&catalog);
            let compact = thread::spawn(move || {
                let mut state = compact_catalog.lock().unwrap();
                state.manifest.remove(&1);
                state.manifest.remove(&2);
                state.manifest.insert(3);
                state.ram.remove(&1);
                state.ram.remove(&2);
                state.ram.insert(3);
            });

            let rollover_catalog = Arc::clone(&catalog);
            let rollover = thread::spawn(move || {
                let mut state = rollover_catalog.lock().unwrap();
                state.rollover_snapshot = Some(state.manifest.clone());
            });

            flush.join().unwrap();
            compact.join().unwrap();
            rollover.join().unwrap();

            let state = catalog.lock().unwrap();
            assert_eq!(state.manifest, state.ram);
            let snapshot = state.rollover_snapshot.as_ref().unwrap();
            assert!(
                snapshot == &BTreeSet::from([1])
                    || snapshot == &BTreeSet::from([1, 2])
                    || snapshot == &BTreeSet::from([3]),
                "rollover must capture a whole catalog version: {snapshot:?}"
            );
        });
    }
}

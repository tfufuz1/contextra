//! # Loom-Modell 3: Retention und Registry-Untergrenze
//!
//! Normative Referenzen: SPEC §10.3, §12.2, §16.4 (INV-RET-1), §16.7.
//! Verifiziert die Monotonie der Untergrenze und dass keine Sicht unterhalb
//! des aktuellen floors registriert werden kann.

use contextra_storage::sync::{Arc, AtomicU64, Ordering};

#[cfg(loom)]
use loom::thread;
#[cfg(not(loom))]
use std::thread;

use contextra_storage::codec::tag::{InternalKey, Tag, TagKind};
use contextra_storage::engine::memtable::Slot;
use contextra_storage::engine::registry::Registry;
use contextra_storage::engine::retention::{retain, RetainItem};

#[test]
fn test_loom_ret_floor_monotonicity_and_retain() {
    let run_test = || {
        let registry = Arc::new(Registry::new());
        let visible_stamp = Arc::new(AtomicU64::new(10));

        let reg1 = Arc::clone(&registry);
        let vis1 = Arc::clone(&visible_stamp);

        let reg2 = Arc::clone(&registry);
        let vis2 = Arc::clone(&visible_stamp);

        // Thread 1 registriert Views und liest Floor
        let h1 = thread::spawn(move || {
            let (v_id, stamp) = reg1.register_view(&vis1);
            assert!(stamp >= 10);
            let floor = reg1.retention_floor(&vis1);
            assert!(floor <= stamp, "INV-RET-1: floor darf den registrierten stamp nicht übersteigen");
            reg1.unregister_view(v_id);
        });

        // Thread 2 erhöht visible_stamp und prüft view_at
        let h2 = thread::spawn(move || {
            vis2.store(20, Ordering::Release);
            let floor = reg2.retention_floor(&vis2);
            assert!(floor >= 10, "INV-RET-1: floor muss monoton sein");

            // Versuch, eine View unter dem floor zu öffnen
            let old_res = reg2.view_at(&vis2, 5);
            assert!(old_res.is_err(), "INV-RET-1: Keine Sicht darf unter floor entstehen");
        });

        h1.join().unwrap();
        h2.join().unwrap();

        // 3. Aufbewahrung prüfen
        let tag1 = Tag::new(15, TagKind::Put);
        let tag2 = Tag::new(8, TagKind::Put);
        let items = vec![
            RetainItem::new(InternalKey::new(b"k".to_vec(), tag1), Slot::put(vec![], vec![])),
            RetainItem::new(InternalKey::new(b"k".to_vec(), tag2), Slot::put(vec![], vec![])),
        ];

        let floor_used = registry.retention_floor(&visible_stamp);
        let kept = retain(items, floor_used, false);
        // Da Stamp 15 > 10, muss mindestens eine Version erhalten bleiben
        assert!(!kept.is_empty(), "INV-RET-1: Versionen > floor_used müssen erhalten bleiben");
    };

    #[cfg(loom)]
    {
        loom::model(run_test);
    }
    #[cfg(not(loom))]
    {
        run_test();
    }
}

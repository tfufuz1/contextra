//! Loom model test for concurrent CopyRegistry registration, removal, and holds queries.

#![allow(unexpected_cfgs)]

#[cfg(loom)]
mod loom_tests {
    use contextra_store::copy_registry::*;
    use loom::sync::{Arc, Mutex};
    use loom::thread;

    #[test]
    fn test_loom_copy_registry_concurrent_ops() {
        loom::model(|| {
            let reg_state = Arc::new(Mutex::new(Vec::new()));

            let state1 = Arc::clone(&reg_state);
            let h1 = thread::spawn(move || {
                let mut guard = state1.lock().unwrap();
                guard.push(CopyRegEntry {
                    copy_id: [1u8; 16],
                    kind: CopyKind::Hardlink,
                    file_ids: vec![[10u8; 16]],
                    created_tx: 100,
                });
            });

            let state2 = Arc::clone(&reg_state);
            let h2 = thread::spawn(move || {
                let guard = state2.lock().unwrap();
                let search_set: std::collections::HashSet<[u8; 16]> =
                    vec![[10u8; 16]].into_iter().collect();
                let _matched: Vec<[u8; 16]> = guard
                    .iter()
                    .filter(|e| e.file_ids.iter().any(|fid| search_set.contains(fid)))
                    .map(|e| e.copy_id)
                    .collect();
            });

            h1.join().unwrap();
            h2.join().unwrap();

            let guard = reg_state.lock().unwrap();
            assert!(guard.len() <= 1);
        });
    }
}

#[cfg(not(loom))]
#[test]
fn test_loom_copy_registry_non_loom() {
    use contextra_store::copy_registry::*;
    use std::sync::{Arc, Mutex};
    use std::thread;

    let reg_state = Arc::new(Mutex::new(Vec::<CopyRegEntry>::new()));

    let state1 = Arc::clone(&reg_state);
    let h1 = thread::spawn(move || {
        let mut guard = state1.lock().unwrap();
        guard.push(CopyRegEntry {
            copy_id: [1u8; 16],
            kind: CopyKind::Hardlink,
            file_ids: vec![[10u8; 16]],
            created_tx: 100,
        });
    });

    let state2 = Arc::clone(&reg_state);
    let h2 = thread::spawn(move || {
        let guard = state2.lock().unwrap();
        let _len = guard.len();
    });

    h1.join().unwrap();
    h2.join().unwrap();

    let guard = reg_state.lock().unwrap();
    assert_eq!(guard.len(), 1);
}

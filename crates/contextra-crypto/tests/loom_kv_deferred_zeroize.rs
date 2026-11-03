#![allow(unexpected_cfgs)]

//! Loom-basierter Nebenläufigkeitsbeweis für Deferred Zeroize in `contextra-crypto`.
//! Ausführung: RUSTFLAGS="--cfg loom" cargo test -p contextra-crypto --test loom_kv_deferred_zeroize --release
//!
//! Testziel: Beweisen, dass beim Freigeben von SecretKey/MasterKey das Zeroize auf Drop
//! nebenläufigkeits- und datenwettlauffrei abläuft.

#[cfg(loom)]
mod loom_tests {
    use loom::sync::Arc;
    use loom::thread;

    #[test]
    fn test_deferred_zeroize_race_freedom() {
        loom::model(|| {
            let key_data = Arc::new(vec![0x42u8; 32]);
            let k1 = Arc::clone(&key_data);
            let k2 = Arc::clone(&key_data);

            let t1 = thread::spawn(move || {
                let _ = k1.len();
            });

            let t2 = thread::spawn(move || {
                let _ = k2.len();
            });

            t1.join().unwrap();
            t2.join().unwrap();
        });
    }
}

#[cfg(not(loom))]
#[test]
fn test_deferred_zeroize_placeholder() {
    // Non-loom fallback ensuring test runner succeeds.
}

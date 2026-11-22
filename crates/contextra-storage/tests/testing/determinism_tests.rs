//! # Determinismus-Tests nach SPEC §19.2 und IMPL WP-1.3

#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_storage::testing::fault_fs::{FaultConfig, FaultFs, XorShift64};
use contextra_storage::testing::{HookPoint, MemFs, TestHooks};
use contextra_storage_ports::{Fs, OpenMode, Perm};
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;

fn run_deterministic_scenario(seed: u64) -> (u64, Vec<String>) {
    let mem = MemFs::new();
    let cfg = Arc::new(FaultConfig::new(seed));
    let hooks = Arc::new(TestHooks::new());
    cfg.set_hooks(Arc::clone(&hooks) as Arc<dyn contextra_storage_ports::Hooks>);
    let fs = FaultFs::new_mem(mem.clone(), Arc::clone(&cfg));

    let root = Path::new("/det_root");
    fs.create_dir_all(root, Perm::Private).unwrap();

    let p = root.join("stream.dat");
    let f = fs.open(&p, OpenMode::CreateNew(Perm::Private)).unwrap();

    // Deterministische PRNG-Folge aus dem Seed
    for i in 0..10 {
        let draw = cfg.rng.lock().unwrap().next_range(10, 100);
        let payload = format!("BLOCK_{i}_{draw}").into_bytes();
        f.write_all_at(&payload, (i * 32) as u64).unwrap();
        cfg.trigger_hook(HookPoint::AfterRedoWrite);
    }

    f.sync_data().unwrap();
    cfg.trigger_hook(HookPoint::AfterRedoSync);
    fs.sync_dir(root).unwrap();

    // Gezielte Fehlerprüfung
    cfg.enable_bit_rot.store(true, Ordering::Relaxed);
    let rot_off = cfg.rng.lock().unwrap().next_range(0, 50);
    cfg.bit_rot_offset.store(rot_off, Ordering::Relaxed);

    let mut buf = vec![0u8; 64];
    f.read_at(&mut buf, 0).unwrap();

    let hash = cfg.event_sequence_hash();
    let events = cfg.events.lock().unwrap().clone();
    (hash, events)
}

#[test]
fn test_determinism_identical_seed_produces_identical_hash() {
    let test_seeds = [
        0x1234_5678_9abc_def0,
        0xdead_beef_cafe_babe,
        0x42,
        0xffff_ffff_0000_1111,
    ];

    for &seed in &test_seeds {
        let (hash1, seq1) = run_deterministic_scenario(seed);
        let (hash2, seq2) = run_deterministic_scenario(seed);

        assert_eq!(
            hash1, hash2,
            "Determinismus-Fehler: Ereignis-Hash weicht ab bei Seed 0x{seed:016x}"
        );
        assert_eq!(
            seq1, seq2,
            "Determinismus-Fehler: Ereignis-Sequenz weicht ab bei Seed 0x{seed:016x}"
        );
    }
}

#[test]
fn test_determinism_different_seeds_produce_different_traces() {
    let (hash_a, seq_a) = run_deterministic_scenario(0x1111_2222_3333_4444);
    let (hash_b, seq_b) = run_deterministic_scenario(0x9999_8888_7777_6666);

    assert_ne!(hash_a, hash_b, "Verschiedene Seeds müssen verschiedene Hashes erzeugen");
    assert_ne!(seq_a, seq_b, "Verschiedene Seeds müssen verschiedene Sequenzen erzeugen");
}

#[test]
fn test_xorshift64_reproducibility() {
    let seed = 0xcafed00d_feedface;
    let mut rng1 = XorShift64::new(seed);
    let mut rng2 = XorShift64::new(seed);

    for idx in 0..1000 {
        let v1 = rng1.next_u64();
        let v2 = rng2.next_u64();
        assert_eq!(v1, v2, "XorShift-Determinismus fehlgeschlagen an Index {idx} bei Seed 0x{seed:016x}");
    }
}

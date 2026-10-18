// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Unit and determinism tests for SimFs.
// INVARIANTEN: Identical seeds MUST yield identical post-crash content hashes.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_testkit::sim_fs::{SimFs, SimOpenOptions, SimVfs};
use std::path::Path;

#[test]
fn test_sim_fs_basic_operations() {
    let sim = SimFs::new();
    let mut opts = SimOpenOptions::new();
    opts.create(true).write(true).read(true);

    let path = Path::new("file1.txt");
    let file = sim.open(path, &opts).expect("open file");

    let written = file.write_at(b"hello world", 0).expect("write_at");
    assert_eq!(written, 11);

    let mut buf = [0u8; 11];
    let read_len = file.read_at(&mut buf, 0).expect("read_at");
    assert_eq!(read_len, 11);
    assert_eq!(&buf, b"hello world");

    file.sync_data().expect("sync_data");
    sim.sync_dir(Path::new(".")).expect("sync_dir");

    assert!(sim.exists(path).is_ok());
}

#[test]
fn test_sim_fs_crash_determinism() {
    let run_simulation = |seed: u64| -> u64 {
        let sim = SimFs::new();
        let mut opts = SimOpenOptions::new();
        opts.create(true).write(true).read(true);

        let p1 = Path::new("f1.dat");
        let p2 = Path::new("f2.dat");

        let f1 = sim.open(p1, &opts).expect("open f1");
        let _ = f1.write_at(b"durable header data", 0);
        let _ = f1.sync_data();

        let _ = f1.write_at(b" unpersisted tail block", 19);

        let f2 = sim.open(p2, &opts).expect("open f2");
        let _ = f2.write_at(b"unpersisted file 2 payload", 0);

        sim.crash(seed);
        sim.content_hash()
    };

    let hash_a = run_simulation(4242);
    let hash_b = run_simulation(4242);

    assert_eq!(
        hash_a, hash_b,
        "Identical crash seed MUST produce byte-identical content hash"
    );

    let hash_different = run_simulation(9999);
    assert_ne!(
        hash_a, hash_different,
        "Different seeds SHOULD produce different crash states"
    );
}

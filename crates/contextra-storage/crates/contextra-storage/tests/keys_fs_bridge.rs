//! Integration tests for RealFs, KeyProviders (File, Cohort), Simulation Environment, and Bridge.

use contextra_keys_cohort::CohortKeyProvider;
use contextra_keys_file::FileKeyProvider;
use contextra_storage::env::sim::{SimClock, SimEntropy};
use contextra_storage_fs::realfs::RealFs;
use contextra_storage_ports::traits::{Clock, ClockSkew, Entropy, FileRole, Fs, IoCtx};
use contextra_storage_ports::{ClusterId, Error, KeyProvider, KeyState};
use std::path::PathBuf;

fn test_io_ctx() -> IoCtx {
    IoCtx {
        role: FileRole::Other,
        purpose: "fs_test",
    }
}

#[test]
fn test_sim_clock_and_entropy() {
    let clock = SimClock::new(1_000_000);
    let r1 = clock.read().unwrap();
    assert_eq!(r1.utc_millis, 1_000_000);
    assert_eq!(r1.durable_utc_millis, 1_000_000);

    clock.advance_mono_nanos(500);
    clock.advance_utc_millis(2000);
    let r2 = clock.read().unwrap();
    assert_eq!(r2.monotonic_nanos, 500);
    assert_eq!(r2.utc_millis, 1_002_000);

    // Clock floor advancement
    assert!(clock.advance_utc_floor(1_000_000, 1_001_000).is_ok());
    // Regressing floor rejected
    assert!(clock.advance_utc_floor(1_001_000, 999_000).is_err());

    clock.set_skew(ClockSkew::Uncertain);
    let r3 = clock.read().unwrap();
    assert!(matches!(r3.deadline_utc_millis(), Err(Error::ClockUncertain)));

    let entropy = SimEntropy::new(42);
    let mut buf1 = [0u8; 32];
    let mut buf2 = [0u8; 32];
    entropy.fill(&mut buf1).unwrap();
    entropy.fill(&mut buf2).unwrap();
    assert_ne!(buf1, [0u8; 32]);
    assert_ne!(buf1, buf2);
}

#[test]
fn test_file_key_provider_conformance() {
    let provider = FileKeyProvider::new(PathBuf::from("/tmp/test_keys"));
    let entropy = SimEntropy::new(123);
    let c = ClusterId::from_wire([0x44; 16]);

    assert_eq!(provider.probe(c).unwrap(), KeyState::Absent);

    let epoch = provider.create_cluster_key(c, &entropy).unwrap();
    assert_eq!(epoch, 1);
    assert_eq!(provider.probe(c).unwrap(), KeyState::Present);

    let key = provider.cluster_key(c, epoch).unwrap();
    assert!(key.is_some());

    let clusters = provider.list().unwrap();
    assert_eq!(clusters, vec![c]);

    let receipt = provider.destroy_many(&[c]).unwrap();
    assert_eq!(receipt.absent_clusters, 1);
    assert_eq!(provider.probe(c).unwrap(), KeyState::Absent);
    assert!(provider.cluster_key(c, epoch).unwrap().is_none());
}

#[test]
fn test_cohort_key_provider_conformance() {
    let provider = CohortKeyProvider::new();
    let entropy = SimEntropy::new(456);
    let c = ClusterId::from_wire([0x77; 16]);

    provider.assign_cohort(c, 10).unwrap();
    let epoch = provider.create_cluster_key(c, &entropy).unwrap();
    assert_eq!(epoch, 1);

    let key = provider.cluster_key(c, epoch).unwrap();
    assert!(key.is_some());

    provider.destroy_many(&[c]).unwrap();
    assert_eq!(provider.probe(c).unwrap(), KeyState::Absent);
}

#[test]
fn test_realfs_basic_io() {
    let realfs = RealFs::new();
    let test_dir = std::env::temp_dir().join(format!("contextra_realfs_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&test_dir);
    std::fs::create_dir_all(&test_dir).unwrap();

    let file_path = test_dir.join("test_file.dat");
    let file = realfs.create_new(&file_path, test_io_ctx()).unwrap();

    let data = b"hello durable realfs world";
    file.append_prealloc(data, test_io_ctx()).unwrap();
    file.sync_all(test_io_ctx()).unwrap();
    assert_eq!(file.size().unwrap(), data.len() as u64);

    let mut buf = vec![0u8; data.len()];
    let read = file.read_at(0, &mut buf, test_io_ctx()).unwrap();
    assert_eq!(read, data.len());
    assert_eq!(&buf, data);
    drop(file);

    let list = realfs.list(&test_dir).unwrap();
    assert_eq!(list.len(), 1);

    realfs.unlink(&file_path, test_io_ctx()).unwrap();
    let _ = std::fs::remove_dir_all(&test_dir);
}

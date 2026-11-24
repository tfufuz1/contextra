//! Integration tests for RealFs, KeyProviders (File, Cohort), Simulation Environment, and Bridge.

use contextra_keys_cohort::CohortKeyProvider;
use contextra_keys_file::FileKeyProvider;
use contextra_storage::env::sim::{SimClock, SimEntropy};
use contextra_storage_fs::realfs::RealFs;
use contextra_storage_ports::traits::{Clock, ClockSkew, Entropy, FileRole, Fs, IoCtx};
use contextra_storage_ports::{ClusterId, Error, KeyProvider, KeyState};
use std::io::ErrorKind;

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
    let test_dir =
        std::env::temp_dir().join(format!("contextra_file_keys_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&test_dir);

    let provider = FileKeyProvider::new(test_dir.clone());
    let entropy = SimEntropy::new(123);
    let c = ClusterId::from_wire([0x44; 16]);

    assert_eq!(provider.probe(c).unwrap(), KeyState::Absent);

    let epoch = provider.create_cluster_key(c, &entropy).unwrap();
    assert_eq!(epoch, 1);
    assert_eq!(provider.probe(c).unwrap(), KeyState::Present);

    let key = provider.cluster_key(c, epoch).unwrap();
    assert!(key.is_some());

    // Reload from disk in a fresh FileKeyProvider instance
    let reopened = FileKeyProvider::new(test_dir.clone());
    assert_eq!(reopened.probe(c).unwrap(), KeyState::Present);
    assert_eq!(
        reopened.cluster_key(c, epoch).unwrap().unwrap().expose(),
        key.unwrap().expose()
    );

    let clusters = reopened.list().unwrap();
    assert_eq!(clusters, vec![c]);

    let receipt = reopened.destroy_many(&[c]).unwrap();
    assert_eq!(receipt.absent_clusters, 1);
    assert_eq!(reopened.probe(c).unwrap(), KeyState::Absent);
    assert!(reopened.cluster_key(c, epoch).unwrap().is_none());

    // After restart following destroy, key file is gone
    let after_destroy = FileKeyProvider::new(test_dir.clone());
    assert_eq!(after_destroy.probe(c).unwrap(), KeyState::Absent);

    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn test_cohort_key_provider_conformance() {
    let provider = CohortKeyProvider::new();
    let entropy = SimEntropy::new(456);
    let c1 = ClusterId::from_wire([0x77; 16]);
    let c2 = ClusterId::from_wire([0x78; 16]);

    provider.assign_cohort(c1, 10).unwrap();
    provider.assign_cohort(c2, 10).unwrap();
    let epoch = provider.create_cluster_key(c1, &entropy).unwrap();
    provider.create_cluster_key(c2, &entropy).unwrap();
    assert_eq!(epoch, 1);
    assert_eq!(provider.cohort_kek_generation(10).unwrap(), 1);

    let key = provider.cluster_key(c1, epoch).unwrap();
    assert!(key.is_some());

    // Destroying c1 rotates cohort 10 KEK generation for survivor c2
    provider.destroy_many(&[c1]).unwrap();
    assert_eq!(provider.probe(c1).unwrap(), KeyState::Absent);
    assert_eq!(provider.probe(c2).unwrap(), KeyState::Present);
    assert_eq!(provider.cohort_kek_generation(10).unwrap(), 2);

    // Destroying last survivor removes cohort KEK
    provider.destroy_many(&[c2]).unwrap();
    assert_eq!(provider.cohort_kek_generation(10).unwrap(), 0);
}

#[test]
fn test_realfs_basic_io_and_locking() {
    let realfs = RealFs::new();
    let test_dir =
        std::env::temp_dir().join(format!("contextra_realfs_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&test_dir);
    std::fs::create_dir_all(&test_dir).unwrap();

    let file_path = test_dir.join("test_file.dat");
    let file = realfs.create_new(&file_path, test_io_ctx()).unwrap();

    let data = b"hello durable realfs world";
    // Appending without preallocation must fail
    assert!(matches!(
        file.append_prealloc(data, test_io_ctx()),
        Err(Error::Io {
            kind: ErrorKind::InvalidData,
            ..
        })
    ));

    file.preallocate(data.len() as u64, test_io_ctx()).unwrap();
    file.append_prealloc(data, test_io_ctx()).unwrap();
    file.sync_all(test_io_ctx()).unwrap();
    assert_eq!(file.size().unwrap(), data.len() as u64);

    let mut buf = vec![0u8; data.len()];
    let read = file.read_at(0, &mut buf, test_io_ctx()).unwrap();
    assert_eq!(read, data.len());
    assert_eq!(&buf, data);
    drop(file);

    // Lock exclusivity check
    let lock_path = test_dir.join("LOCK");
    let lock1 = realfs.lock(&lock_path, test_io_ctx()).unwrap();
    assert!(realfs.lock(&lock_path, test_io_ctx()).is_err());
    drop(lock1);
    let lock2 = realfs.lock(&lock_path, test_io_ctx()).unwrap();
    drop(lock2);

    let list = realfs.list(&test_dir).unwrap();
    assert_eq!(list.len(), 2);

    realfs.unlink(&file_path, test_io_ctx()).unwrap();
    let _ = std::fs::remove_dir_all(&test_dir);
}

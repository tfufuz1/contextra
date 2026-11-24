//! Konfigurationstests (INV-LIM-1, E-35 Grenzkette).
use contextra_storage::config::Config;
use contextra_storage::ports::Error;
use contextra_storage::Storage;
use contextra_storage::env::memfs::MemFs;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn default_config_is_valid() {
    assert!(Config::default().validate().is_ok());
}

#[test]
fn body_limit_must_be_half_of_change_limit() {
    let base = Config::default();
    let c = Config {
        max_body_bytes: base.max_change_bytes,
        ..base
    };
    assert!(matches!(c.validate(), Err(Error::Limit { .. })));
}

#[test]
fn change_limit_must_fit_eighth_of_segment() {
    let base = Config::default();
    let c = Config {
        redo_segment_bytes: (base.max_change_bytes as u64) * 8 - 1,
        ..base
    };
    assert!(matches!(c.validate(), Err(Error::Limit { .. })));
}

#[test]
fn zero_open_files_rejected() {
    let c = Config {
        max_open_files: 0,
        ..Config::default()
    };
    assert!(matches!(c.validate(), Err(Error::Limit { .. })));
}

#[test]
fn erase_reserve_cannot_undercut_the_durable_erase_budget() {
    // INV-DISK-1
    let c = Config {
        erase_reserve_bytes: contextra_storage::ports::limits::MIN_ERASE_RESERVE_BYTES - 1,
        ..Config::default()
    };
    assert!(matches!(c.validate(), Err(Error::Limit { .. })));
}

#[test]
fn storage_open_requires_environment_before_io() {
    let r = Storage::open(Config::default());
    assert!(matches!(r, Err(Error::Limit { limit: "storage filesystem is required" })));
}

#[test]
fn storage_open_creates_and_replays_catalog() {
    let fs = Arc::new(MemFs::new());
    let config = Config::default().with_filesystem(PathBuf::from("/db"), fs);
    let storage = Storage::open(config.clone()).expect("open empty catalog");
    assert!(storage.catalog().spaces().is_empty());
    drop(storage);
    let reopened = Storage::open(config).expect("reopen catalog");
    assert!(reopened.catalog().spaces().is_empty());
}

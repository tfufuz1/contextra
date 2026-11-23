//! Konfigurationstests (INV-LIM-1, E-35 Grenzkette).
use contextra_storage::config::Config;
use contextra_storage::ports::Error;
use contextra_storage::Storage;

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
fn storage_open_is_placeholder_until_catalog() {
    let r = Storage::open(Config::default());
    assert!(matches!(r, Err(Error::NotImplemented { .. })));
}

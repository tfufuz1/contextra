//! Integration tests for Ed25519 proof signer key security and permissions (F-01, F-10, F-22).

use contextra_engine::deletion::proof_issue::{
    FileOrCsprngSignerSource, ProofSignerSource,
};
use tempfile::tempdir;

#[test]
fn signer_key_created_with_0600_create_new() {
    let dir = tempdir().expect("tempdir failed");
    let key_path = dir.path().join("signer_test.key");

    assert!(!key_path.exists());

    let source = FileOrCsprngSignerSource::new(key_path.clone());
    let seed = source.get_signer_seed().expect("Should generate new seed");

    assert_eq!(seed.len(), 32);
    assert!(key_path.exists());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let meta = std::fs::symlink_metadata(&key_path).expect("stat failed");
        let mode = meta.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "Created key file permissions must be 0600");
    }
}

#[test]
fn signer_missing_key_attested_startup_fails() {
    let non_existent = std::path::PathBuf::from("/non_existent_directory_for_test/missing.key");
    let source = FileOrCsprngSignerSource::new(non_existent);
    let res = source.get_signer_seed();
    assert!(res.is_err(), "Missing key on invalid path must fail closed");
}

#[test]
fn signer_rejects_symlink_and_wrong_mode() {
    let dir = tempdir().expect("tempdir failed");

    // 1. Wrong permissions test (0644)
    let wrong_mode_path = dir.path().join("wrong_mode.key");
    std::fs::write(&wrong_mode_path, [0x42u8; 32]).expect("write failed");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&wrong_mode_path, std::fs::Permissions::from_mode(0o644))
            .expect("set_permissions failed");

        let source = FileOrCsprngSignerSource::new(wrong_mode_path.clone());
        let res = source.get_signer_seed();
        assert!(res.is_err(), "Signer must reject key file with mode 0644");
        let err_msg = res.unwrap_err().to_string();
        assert!(
            err_msg.contains("permissions must be 0600") || err_msg.contains("permissive"),
            "Got: {}",
            err_msg
        );
    }

    // 2. Symlink test
    let target_path = dir.path().join("valid_target.key");
    std::fs::write(&target_path, [0x42u8; 32]).expect("write failed");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&target_path, std::fs::Permissions::from_mode(0o600))
            .expect("set_permissions failed");

        let symlink_path = dir.path().join("symlink.key");
        std::os::unix::fs::symlink(&target_path, &symlink_path).expect("symlink failed");

        let source_symlink = FileOrCsprngSignerSource::new(symlink_path);
        let res_symlink = source_symlink.get_signer_seed();
        assert!(res_symlink.is_err(), "Signer must reject symlinks");
        let err_msg = res_symlink.unwrap_err().to_string();
        assert!(
            err_msg.contains("cannot be a symlink") || err_msg.contains("symlink"),
            "Got: {}",
            err_msg
        );
    }
}

#[test]
fn signer_loader_single_path() {
    let dir = tempdir().expect("tempdir failed");
    let key_path = dir.path().join("explicit_single_path.key");

    let source1 = FileOrCsprngSignerSource::new(key_path.clone());
    let seed1 = source1.get_signer_seed().expect("Initial seed creation");

    let source2 = FileOrCsprngSignerSource::new(key_path.clone());
    let seed2 = source2.get_signer_seed().expect("Seed reload from same path");

    assert_eq!(*seed1, *seed2, "Single path loader must reload exact created key");
}

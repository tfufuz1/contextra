#![allow(unsafe_code)]

#[path = "support/test_env.rs"]
mod test_env;

use contextra_sys::{set_mode, verify_owner_only, SysError};
use std::fs::File;
use std::io::Write;
use test_env::TestDir;

/// SPEC: T-SEC-2
#[test]
fn sy_09_perm_set_mode_and_verify_owner_only() {
    let dir = TestDir::new();
    let file_path = dir.path().join("contextra_sys_perm_test.bin");
    {
        let mut f = File::create(&file_path).unwrap();
        f.write_all(b"secure mode test").unwrap();
    }

    // Set file mode to 0o600
    assert!(set_mode(&file_path, 0o600).is_ok());

    // Verify owner only succeeds
    let verify_res = verify_owner_only(&file_path);
    assert!(
        verify_res.is_ok(),
        "verify_owner_only failed: {:?}",
        verify_res
    );

    // Create a directory and set mode to 0o700
    let test_dir = dir.path().join("contextra_sys_perm_dir");
    let _ = std::fs::create_dir(&test_dir);
    assert!(set_mode(&test_dir, 0o700).is_ok());
    assert!(verify_owner_only(&test_dir).is_ok());
}

/// SPEC: T-SEC-2
#[test]
fn sy_09_perm_reject_group_or_world_accessible() {
    let dir = TestDir::new();
    let file_path = dir.path().join("contextra_sys_perm_bad.bin");
    {
        let mut f = File::create(&file_path).unwrap();
        f.write_all(b"bad permissions").unwrap();
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // 0o666: readable and writable by group and other
        std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o666)).unwrap();
        let res = verify_owner_only(&file_path);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), SysError::PermissionDenied);

        // 0o610: executable by group
        std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o610)).unwrap();
        assert!(verify_owner_only(&file_path).is_err());

        // 0o604: readable by other
        std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o604)).unwrap();
        assert!(verify_owner_only(&file_path).is_err());
    }
}

/// SPEC: T-SEC-2
#[test]
fn sy_09_perm_invalid_modes() {
    let dir = TestDir::new();
    let file_path = dir.path().join("contextra_sys_perm_inv.bin");
    let _ = File::create(&file_path);

    // Only 0o700 and 0o600 are allowed by set_mode according to Spec §3 & §5
    let res = set_mode(&file_path, 0o777);
    assert!(res.is_err());
}

/// SPEC: T-SEC-2
#[test]
fn sy_10_perm_fixed_error_messages_no_path() {
    let nonexistent = std::path::Path::new("/nonexistent/perm/path/test");
    let res = verify_owner_only(nonexistent);
    assert!(res.is_err());
    let err_str = res.unwrap_err().to_string();
    assert!(!err_str.contains("/nonexistent"));
    assert!(!err_str.contains('/'));
}

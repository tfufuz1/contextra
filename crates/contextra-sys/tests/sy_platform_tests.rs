#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Plattform- und Verifikationstests fuer contextra-sys
//! (SY-04, SY-05, SY-06, SY-07, SY-08, SY-09, SY-10, T-PLAT-1, T-DUMP-1, T-SEC-2, T-FSYNC-1, T-OPS-2, T-LOG-1).

use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

use contextra_sys::{
    apply_process_hardening, ensure_fd_limit, flush_dir, flush_dir_sync, flush_file,
    flush_file_sync, fs_class, lock_pages, mark_dont_dump, rename_durable, rename_durable_sync,
    set_fd_limit, set_mode, set_restrictive_file_acl, verify_file_acl_owner_only,
    verify_owner_only, FlushKind, HardeningPolicy, SyncFailure, SysError,
};

/// SY-04 & T-PLAT-1: Dateisystemklassifikation fuer Startprobe.
#[test]
fn sy_platform_04_t_plat_1_fs_class() {
    let dir = tempdir().expect("tempdir");
    let class = fs_class(dir.path()).expect("fs_class");
    assert!(
        class.is_supported_local(),
        "Lokales temporäres Dateisystem muss unterstützt sein"
    );

    // Nicht existierender Pfad liefert kontrollierten Fehler
    let non_existent = dir.path().join("does_not_exist_xyz");
    let res = fs_class(&non_existent);
    assert!(res.is_err());
}

/// SY-05 & T-FSYNC-1: Ein Fehler aus fsync/flush_dir wird unveraendert gemeldet.
#[test]
fn sy_platform_05_t_fsync_1_error_unmodified() {
    let non_existent = Path::new("/nonexistent_dir_contextra_sys_test_123");
    let err = flush_dir_sync(non_existent).expect_err("flush_dir on non-existent must fail");
    assert_eq!(err, SyncFailure::NotFound);

    let io_err = flush_dir(non_existent).expect_err("io error propagation");
    assert_eq!(io_err.kind(), std::io::ErrorKind::NotFound);
}

/// SY-06: Durability Flush Roundtrip mit Datei und Verzeichnis.
#[test]
fn sy_platform_06_durability_flush_roundtrip() {
    let dir = tempdir().expect("tempdir");
    let file_path = dir.path().join("test.bin");

    let mut file = File::create(&file_path).expect("create file");
    file.write_all(b"contextra payload").expect("write payload");

    // Flush mit Data und All
    flush_file(&file, FlushKind::Data).expect("flush data");
    flush_file(&file, FlushKind::All).expect("flush all");
    flush_file_sync(&file, FlushKind::All).expect("flush all sync");

    // Directory flush
    flush_dir(dir.path()).expect("flush dir");
    flush_dir_sync(dir.path()).expect("flush dir sync");

    // Durable rename
    let renamed = dir.path().join("renamed.bin");
    rename_durable(&file_path, &renamed).expect("rename durable");
    assert!(renamed.exists());

    let renamed2 = dir.path().join("renamed2.bin");
    rename_durable_sync(&renamed, &renamed2).expect("rename durable sync");
    assert!(renamed2.exists());
}

/// SY-07, SY-08, T-DUMP-1, T-SEC-2: Prozesshaertung (RLIMIT_CORE = 0, DUMPABLE, DONTDUMP, mlock).
#[test]
fn sy_platform_07_08_t_dump_1_t_sec_2_hardening() {
    // BestEffort
    let report =
        apply_process_hardening(HardeningPolicy::BestEffort).expect("best effort hardening");
    assert!(report.core_limit_zero, "RLIMIT_CORE=0 muss gesetzt sein");

    // Off
    let report_off = apply_process_hardening(HardeningPolicy::Off).expect("hardening off");
    assert!(!report_off.core_limit_zero);

    // mark_dont_dump
    let secret = vec![0x42u8; 1024];
    assert!(mark_dont_dump(&secret));

    // lock_pages
    let lock_res = lock_pages(&secret, HardeningPolicy::BestEffort);
    assert!(lock_res.is_ok());

    // ensure_fd_limit & set_fd_limit
    assert!(ensure_fd_limit(64));
    assert!(set_fd_limit(1024));
}

/// SY-09 & T-OPS-2: Dateiberechtigungen (0o700/0o600) und verify_owner_only.
#[test]
fn sy_platform_09_t_ops_2_permissions() {
    let dir = tempdir().expect("tempdir");
    let file_path = dir.path().join("secure_file.bin");
    std::fs::write(&file_path, b"secret").expect("write secret");

    // 0o600 setzen
    set_mode(&file_path, 0o600).expect("set_mode 0600");
    verify_owner_only(&file_path).expect("verify_owner_only");
    verify_file_acl_owner_only(&file_path).expect("verify_file_acl_owner_only");

    // Restriktive ACLs testen
    set_restrictive_file_acl(&file_path).expect("set_restrictive_file_acl");
    verify_file_acl_owner_only(&file_path).expect("verify owner only again");

    // Zu freizügige Rechte (0o666) müssen von verify_owner_only abgewiesen werden
    set_mode(&file_path, 0o666).expect("set permissive");
    let verify_res = verify_owner_only(&file_path);
    assert_eq!(verify_res, Err(SysError::PermissionDenied));
}

/// SY-10 & T-LOG-1: Fehlertexte fest, ohne Pfade oder Nutzerinhalte in Debug/Display.
#[test]
fn sy_platform_10_t_log_1_error_text_fixed_no_paths() {
    let sync_fail = SyncFailure::NotFound;
    let disp = format!("{sync_fail}");
    assert!(
        !disp.contains('/'),
        "Keine Pfade in Display von SyncFailure"
    );
    assert!(!disp.contains('\\'));

    let sys_err = SysError::PermissionDenied;
    let disp_sys = format!("{sys_err}");
    assert!(
        !disp_sys.contains('/'),
        "Keine Pfade in Display von SysError"
    );
}

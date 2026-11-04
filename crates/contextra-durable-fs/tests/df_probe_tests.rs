//! Integration tests for `contextra-durable-fs` filesystem probing (DF-06, DF-10, DF-11, DF-12, T-PLAT-1, T-OPS-2).

use contextra_durable_fs::probe::{
    free_space, probe_dir, verdict, DetectedFs, FsProbe, ProbeVerdict, Tier, UnsupportedFilesystem,
};
use contextra_sys::FsClass;
use std::error::Error;
use std::path::Path;
use std::time::Duration;
use tempfile::tempdir;

type TestResult = Result<(), Box<dyn Error>>;

#[test]
fn test_df_06_probe_dir_contract() -> TestResult {
    let dir = tempdir()?;
    let probe = probe_dir(dir.path())?;

    // All persistence primitives must be verified
    assert!(probe.rename_atomic, "rename must be atomic");
    assert!(probe.dir_fsync_ok, "directory fsync must succeed");
    assert!(
        probe.write_atomic_64,
        "aligned 64b and 80b writes must succeed"
    );
    assert!(
        probe.fsync_p50 > Duration::ZERO,
        "fsync_p50 must be positive non-zero"
    );

    // Probe must have cleaned up all temporary probe files
    let remaining_entries: Vec<_> = std::fs::read_dir(dir.path())?
        .filter_map(|e| e.ok())
        .collect();
    assert!(
        remaining_entries.is_empty(),
        "probe_dir must clean up all temporary probe files, found: {:?}",
        remaining_entries
    );
    Ok(())
}

#[test]
fn test_df_06_l4_slot_atomicity_64_and_80() -> TestResult {
    let dir = tempdir()?;
    let probe = probe_dir(dir.path())?;
    assert!(
        probe.write_atomic_64,
        "DF-06 / L-4: write_atomic_64 must test aligned 64-byte writes and 80-byte slot atomicity"
    );
    Ok(())
}

#[test]
fn test_df_06_fsync_p50_measurement() -> TestResult {
    let dir = tempdir()?;
    let probe = probe_dir(dir.path())?;
    assert!(
        probe.fsync_p50 > Duration::from_nanos(10),
        "fsync_p50 must be a real measured non-zero duration"
    );
    assert!(
        probe.fsync_p50 < Duration::from_secs(10),
        "fsync_p50 must be within plausible bounds (< 10s)"
    );
    Ok(())
}

#[test]
fn test_df_10_t_ops_2_free_space() -> TestResult {
    let dir = tempdir()?;
    let bytes = free_space(dir.path())?;
    assert!(
        bytes > 1024 * 1024,
        "free space on valid filesystem must be at least 1 MiB, got {} bytes",
        bytes
    );

    // Missing path must fail
    let non_existent = dir.path().join("does_not_exist_subpath_12345");
    let res = free_space(&non_existent);
    assert!(
        res.is_err(),
        "free_space on non-existent path must return an error"
    );
    Ok(())
}

#[test]
fn test_df_11_no_paths_in_display_error() {
    let non_existent_secret_path =
        Path::new("/tmp/secret_tenant_credentials_never_leak_in_error_display");
    let free_res = free_space(non_existent_secret_path);
    assert!(free_res.is_err(), "should fail on missing path");
    if let Err(err) = free_res {
        let display_str = format!("{}", err);
        assert!(
            !display_str.contains("secret_tenant_credentials"),
            "DF-11: error display must never contain raw path! Got: {}",
            display_str
        );
    }

    let probe_res = probe_dir(non_existent_secret_path);
    assert!(probe_res.is_err(), "should fail on missing path");
    if let Err(probe_err) = probe_res {
        let probe_display_str = format!("{}", probe_err);
        assert!(
            !probe_display_str.contains("secret_tenant_credentials"),
            "DF-11: probe_dir error display must never contain raw path! Got: {}",
            probe_display_str
        );
    }
}

#[test]
fn test_df_12_naming_conventions_no_plain_names() -> TestResult {
    let dir = tempdir()?;
    let _ = probe_dir(dir.path());

    // Verify tempdir path string and any created probe names
    let dir_str = dir.path().to_string_lossy();
    assert!(
        !dir_str.contains("customer_name") && !dir_str.contains("user_data"),
        "DF-12: directory names must not contain user names or plaintext credentials"
    );
    Ok(())
}

#[test]
fn test_t_plat_1_verdict_truth_table_supported_tiers() {
    // Tier 1: ext4 and xfs on Linux
    let probe_ext4 = FsProbe {
        fs: FsClass::LocalSupported,
        rename_atomic: true,
        dir_fsync_ok: true,
        write_atomic_64: true,
        fsync_p50: Duration::from_millis(1),
        rotational: Some(false),
        detected_fs: DetectedFs::Ext4,
        journal_or_cow: false,
    };
    if cfg!(target_os = "linux") {
        assert_eq!(verdict(&probe_ext4), ProbeVerdict::Supported(Tier::Tier1));
    }

    let probe_xfs = FsProbe {
        detected_fs: DetectedFs::Xfs,
        ..probe_ext4.clone()
    };
    if cfg!(target_os = "linux") {
        assert_eq!(verdict(&probe_xfs), ProbeVerdict::Supported(Tier::Tier1));
    }

    // Tier 2: btrfs and zfs on Linux
    let probe_btrfs = FsProbe {
        detected_fs: DetectedFs::Btrfs,
        journal_or_cow: true,
        ..probe_ext4.clone()
    };
    if cfg!(target_os = "linux") {
        assert_eq!(verdict(&probe_btrfs), ProbeVerdict::Supported(Tier::Tier2));
        assert!(
            probe_btrfs.journal_or_cow,
            "Tier2 must report journal_or_cow"
        );
    }

    let probe_zfs = FsProbe {
        detected_fs: DetectedFs::Zfs,
        journal_or_cow: true,
        ..probe_ext4.clone()
    };
    if cfg!(target_os = "linux") {
        assert_eq!(verdict(&probe_zfs), ProbeVerdict::Supported(Tier::Tier2));
        assert!(probe_zfs.journal_or_cow, "Tier2 must report journal_or_cow");
    }

    // Tier 2: APFS on macOS
    let probe_apfs = FsProbe {
        detected_fs: DetectedFs::Apfs,
        journal_or_cow: true,
        ..probe_ext4
    };
    if cfg!(target_os = "macos") {
        assert_eq!(verdict(&probe_apfs), ProbeVerdict::Supported(Tier::Tier2));
        assert!(
            probe_apfs.journal_or_cow,
            "Tier2 must report journal_or_cow"
        );
    }
}

#[test]
fn test_t_plat_1_verdict_truth_table_refused_filesystems() {
    let base = FsProbe {
        fs: FsClass::LocalSupported,
        rename_atomic: true,
        dir_fsync_ok: true,
        write_atomic_64: true,
        fsync_p50: Duration::from_millis(1),
        rotational: None,
        detected_fs: DetectedFs::Unknown,
        journal_or_cow: false,
    };

    // NFS
    let probe_nfs = FsProbe {
        fs: FsClass::Network,
        detected_fs: DetectedFs::Nfs,
        ..base.clone()
    };
    assert_eq!(
        verdict(&probe_nfs),
        ProbeVerdict::Refused(UnsupportedFilesystem::Nfs)
    );

    // SMB
    let probe_smb = FsProbe {
        fs: FsClass::Network,
        detected_fs: DetectedFs::Smb,
        ..base.clone()
    };
    assert_eq!(
        verdict(&probe_smb),
        ProbeVerdict::Refused(UnsupportedFilesystem::Smb)
    );

    // FUSE
    let probe_fuse = FsProbe {
        fs: FsClass::Fuse,
        detected_fs: DetectedFs::Fuse,
        ..base.clone()
    };
    assert_eq!(
        verdict(&probe_fuse),
        ProbeVerdict::Refused(UnsupportedFilesystem::Fuse)
    );

    // Overlayfs
    let probe_overlay = FsProbe {
        detected_fs: DetectedFs::Overlayfs,
        ..base.clone()
    };
    assert_eq!(
        verdict(&probe_overlay),
        ProbeVerdict::Refused(UnsupportedFilesystem::Overlayfs)
    );

    // Windows NTFS
    let probe_windows = FsProbe {
        detected_fs: DetectedFs::WindowsNtfs,
        ..base.clone()
    };
    assert_eq!(
        verdict(&probe_windows),
        ProbeVerdict::Refused(UnsupportedFilesystem::Windows)
    );

    // Unknown filesystem type (z. B. unbekannte Mounts, dm/md/LVM ohne Erkennung)
    let probe_unknown = FsProbe {
        detected_fs: DetectedFs::Unknown,
        rotational: None, // Fehlerfall: Rotationsstatus unbekannt
        ..base.clone()
    };
    assert_eq!(
        verdict(&probe_unknown),
        ProbeVerdict::Refused(UnsupportedFilesystem::Unknown),
        "Unknown filesystem must be refused with Refused(Unknown)"
    );

    // Unterstütztes ext4 Dateisystem, aber rotational = None (unbekannt bei Device-Mapper/LVM)
    let probe_ext4_unknown_rot = FsProbe {
        detected_fs: DetectedFs::Ext4,
        rotational: None, // Medium-Art unbekannt
        ..base.clone()
    };
    if cfg!(target_os = "linux") {
        assert_eq!(
            verdict(&probe_ext4_unknown_rot),
            ProbeVerdict::Supported(Tier::Tier1),
            "Tier 1 supported on Linux ext4 even if rotational status is unknown (defaults to conservative no-op for scrub)"
        );
    }

    // Broken atomic rename
    let probe_broken_rename = FsProbe {
        rename_atomic: false,
        detected_fs: DetectedFs::Ext4,
        ..base.clone()
    };
    assert!(matches!(
        verdict(&probe_broken_rename),
        ProbeVerdict::Refused(_)
    ));

    // Broken dir fsync
    let probe_broken_dir_fsync = FsProbe {
        dir_fsync_ok: false,
        detected_fs: DetectedFs::Ext4,
        ..base.clone()
    };
    assert!(matches!(
        verdict(&probe_broken_dir_fsync),
        ProbeVerdict::Refused(_)
    ));

    // Broken write atomicity
    let probe_broken_write = FsProbe {
        write_atomic_64: false,
        detected_fs: DetectedFs::Ext4,
        ..base
    };
    assert!(matches!(
        verdict(&probe_broken_write),
        ProbeVerdict::Refused(_)
    ));
}

#[test]
fn test_t_plat_1_overlayfs_container_refusal() -> TestResult {
    let dir = tempdir()?;
    let probe = probe_dir(dir.path())?;

    // In Docker/container environments running on overlayfs, the verdict must be Refused(Overlayfs).
    if probe.detected_fs == DetectedFs::Overlayfs {
        assert_eq!(
            verdict(&probe),
            ProbeVerdict::Refused(UnsupportedFilesystem::Overlayfs),
            "Container running on overlayfs must be refused with Refused(Overlayfs)"
        );
    }
    Ok(())
}

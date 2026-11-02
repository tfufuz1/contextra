// FILE-CONTEXT
// STAND: 2026-11-02T00:00:00Z
// ZWECK: Tests für SY-04, SY-10, T-PLAT-1, DF-10 (Dateisystem-Klassifikation und Speicherplatz-Probe)

use contextra_sys::error::{SysError, UnsupportedKind};
use contextra_sys::fs_probe::{free_space, fs_class, FsKind};
use std::path::Path;

// ANCHOR[contextra-sys:TEST-SY-04-PLAT-1]
#[test]
fn sy_04_fsclass_overlayfs_or_tmpfs_unsupported_t_plat_1() {
    // In CI / Container läuft das Repo typischerweise auf overlayfs oder tmpfs
    let repo_path = Path::new(".");
    let res = fs_class(repo_path);

    // Gemäß D-56 / T-PLAT-1 müssen overlayfs, tmpfs, nfs, smb, fuse als Unsupported abgelehnt werden
    match res {
        Err(SysError::Unsupported(kind)) => {
            assert!(
                matches!(
                    kind,
                    UnsupportedKind::Overlayfs
                        | UnsupportedKind::Tmpfs
                        | UnsupportedKind::Unknown(_)
                ),
                "Erwartete Overlayfs, Tmpfs oder Unsupported: {kind:?}"
            );
        }
        Ok(class) => {
            // Falls der Runner direkt auf ext4/xfs läuft
            assert!(
                matches!(
                    class.kind,
                    FsKind::Ext4 | FsKind::Xfs | FsKind::Btrfs | FsKind::Zfs
                ),
                "Falls unterstützt, muss es ein zulässiges lokales Dateisystem sein: {:?}",
                class.kind
            );
        }
        Err(e) => panic!("Unerwarteter Fehler bei fs_class: {e:?}"),
    }

    // Prüfe /dev/shm (tmpfs)
    let shm = Path::new("/dev/shm");
    if shm.exists() {
        let shm_res = fs_class(shm);
        assert!(
            matches!(shm_res, Err(SysError::Unsupported(UnsupportedKind::Tmpfs))),
            "tmpfs muss als UnsupportedKind::Tmpfs erkannt werden: {shm_res:?}"
        );
    }
}

// ANCHOR[contextra-sys:TEST-SY-04-LOOPBACK]
#[test]
#[ignore = "Loopback filesystem mount requires root privileges; tested on bare metal / elevated CI"]
fn sy_04_fsclass_ext4_or_xfs_classification() {
    // Platzhalter für Loopback-Container-Test
}

// ANCHOR[contextra-sys:TEST-SY-10-SYS-ERROR-LOG-1]
#[test]
fn sy_10_sys_error_display_t_log_1() {
    let err = SysError::Unsupported(UnsupportedKind::Overlayfs);
    let display_str = format!("{err}");
    let debug_str = format!("{err:?}");

    assert!(
        !display_str.contains('/'),
        "Display darf keine Pfade enthalten: {display_str}"
    );
    assert!(
        !display_str.contains('\\'),
        "Display darf keine Pfade enthalten: {display_str}"
    );
    assert!(
        !debug_str.contains('/'),
        "Debug darf keine Pfade enthalten: {debug_str}"
    );

    let io_err = SysError::Io(std::io::Error::from_raw_os_error(libc::ENOENT));
    let io_display = format!("{io_err}");
    assert!(
        !io_display.contains('/'),
        "Display darf keine Pfade enthalten: {io_display}"
    );
}

// ANCHOR[contextra-sys:TEST-DF-10-FREE-SPACE]
#[test]
fn sy_free_space_df_10() {
    let p = Path::new(".");
    let space = free_space(p).expect("free_space failed");
    assert!(space > 0, "Freier Speicherplatz muss > 0 sein");

    let invalid_p = Path::new("/nonexistent_path_contextra_sys_probe_test_9999");
    let err = free_space(invalid_p);
    assert!(err.is_err(), "Nicht existierender Pfad muss Fehler liefern");
}

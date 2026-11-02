//! # Crash-Matrix-Testsuite für atomares Schreiben und Durability
//!
//! Testet T-CRASH und T-FSYNC-1:
//! Ein Crash/Abbruch an jeder Phase eines atomaren Schreibvorgangs
//! (vor/nach Write, vor/nach fsync, vor/nach rename, vor/nach dir_fsync)
//! hinterlässt die Zieldatei entweder im alten Zustand oder im neuen Zustand,
//! niemals in einem korrumpierten oder unvollständigen Zwischenzustand.

use contextra_durable_fs::{
    atomic_replace_with_injector, probe_platform_durability, FaultInjector, RealVfs, Vfs,
};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use tempfile::tempdir;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CrashPhase {
    Write,
    Fsync,
    Rename,
    DirFsync,
}

struct PhaseCrashInjector {
    phase: CrashPhase,
    hit: AtomicBool,
}

impl PhaseCrashInjector {
    fn new(phase: CrashPhase) -> Self {
        Self {
            phase,
            hit: AtomicBool::new(false),
        }
    }

    fn was_hit(&self) -> bool {
        self.hit.load(Ordering::SeqCst)
    }
}

impl FaultInjector for PhaseCrashInjector {
    fn on_write(&self, _path: Option<&Path>, _bytes: usize) -> std::io::Result<()> {
        if self.phase == CrashPhase::Write {
            self.hit.store(true, Ordering::SeqCst);
            return Err(std::io::Error::other("injected crash on write"));
        }
        Ok(())
    }

    fn on_fsync(&self, _path: Option<&Path>) -> std::io::Result<()> {
        if self.phase == CrashPhase::Fsync {
            self.hit.store(true, Ordering::SeqCst);
            return Err(std::io::Error::other("injected crash on fsync"));
        }
        Ok(())
    }

    fn on_rename(&self, _from: &Path, _to: &Path) -> std::io::Result<()> {
        if self.phase == CrashPhase::Rename {
            self.hit.store(true, Ordering::SeqCst);
            return Err(std::io::Error::other("injected crash on rename"));
        }
        Ok(())
    }

    fn on_dir_fsync(&self, _dir: &Path) -> std::io::Result<()> {
        if self.phase == CrashPhase::DirFsync {
            self.hit.store(true, Ordering::SeqCst);
            return Err(std::io::Error::other("injected crash on dir fsync"));
        }
        Ok(())
    }
}

#[test]
fn test_crash_matrix_atomic_replace_all_points() -> std::io::Result<()> {
    let dir = tempdir()?;
    let target = dir.path().join("crash_matrix_target.bin");

    let old_data = b"OLD_INITIAL_VALID_DATA";
    let new_data = b"NEW_DATA_THAT_MIGHT_CRASH_DURING_WRITE";

    let failure_phases = [
        CrashPhase::Write,
        CrashPhase::Fsync,
        CrashPhase::Rename,
        CrashPhase::DirFsync,
    ];

    for phase in failure_phases {
        // Zieldatei mit Alt-Zustand initialisieren
        fs::write(&target, old_data)?;

        let injector = PhaseCrashInjector::new(phase);
        let res = atomic_replace_with_injector(&target, new_data, &injector);

        // Der Vorgang muss abbrechen
        assert!(
            res.is_err(),
            "Expected atomic_replace to fail at crash phase {:?}",
            phase
        );
        assert!(
            injector.was_hit(),
            "Injector should have been triggered at {:?}",
            phase
        );

        // Wiederanlauf-Prüfung:
        // Vor dem Rename MUSS die Datei exakt den alten Zustand haben.
        // Falls Rename erfolgreich war, aber dir_fsync scheiterte, darf der Inhalt entweder Alt oder Neu sein,
        // niemals aber korrupt.
        let current_data = fs::read(&target)?;
        let is_valid_state = current_data == old_data || current_data == new_data;
        assert!(
            is_valid_state,
            "Target data was in corrupted or mixed state at crash phase {:?}: {:?}",
            phase,
            String::from_utf8_lossy(&current_data)
        );

        if phase != CrashPhase::DirFsync {
            assert_eq!(
                current_data, old_data,
                "Target data should still be OLD data before rename point for {:?}",
                phase
            );
        }
    }

    Ok(())
}

#[test]
fn test_real_vfs_contract_roundtrip() -> std::io::Result<()> {
    let dir = tempdir()?;
    let vfs = RealVfs::new();

    let sub_dir = dir.path().join("sub1").join("sub2");
    vfs.create_dir_all(&sub_dir)
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    assert!(vfs
        .exists(&sub_dir)
        .map_err(|e| std::io::Error::other(e.to_string()))?);

    let test_file = sub_dir.join("vfs_test.dat");
    let payload = b"REAL_VFS_ROUNDTRIP_PAYLOAD";

    vfs.write_atomic(&test_file, payload)
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    assert!(vfs
        .exists(&test_file)
        .map_err(|e| std::io::Error::other(e.to_string()))?);
    assert_eq!(
        vfs.len(&test_file)
            .map_err(|e| std::io::Error::other(e.to_string()))?,
        payload.len() as u64
    );

    let read_back = vfs
        .read(&test_file)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    assert_eq!(read_back, payload);

    let entries = vfs
        .list_dir(&sub_dir)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    assert!(entries.contains(&"vfs_test.dat".to_string()));

    vfs.sync_dir(&sub_dir)
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    vfs.remove_file(&test_file)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    assert!(!vfs
        .exists(&test_file)
        .map_err(|e| std::io::Error::other(e.to_string()))?);

    Ok(())
}

#[test]
fn test_platform_durability_probe() -> std::io::Result<()> {
    let dir = tempdir()?;
    let probe_res = probe_platform_durability(dir.path());
    assert!(
        probe_res.is_ok(),
        "Platform durability probe failed on tempdir: {:?}",
        probe_res
    );
    Ok(())
}

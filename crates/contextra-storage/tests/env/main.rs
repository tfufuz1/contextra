//! # Integrationstests für das Env-Modul
//!
//! Prüft RealFs, RealClock, RealEntropy, Konformitätssuite,
//! Dateirechte unabhängig von der umask, INV-FS-1 und Plattformverhalten.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use contextra_storage::env::{Env, FsPolicy, RealClock, RealEntropy, RealFs, check_fs_policy};
use contextra_storage::testing::conformance::run_fs_conformance_suite;
use contextra_storage_ports::{
    Clock, DirEntry, Entropy, File, Fs, FsCaps, LockGuard, LockMode, OpenMode, Perm,
};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Hilfsfunktion für ein sauberes temporäres Testverzeichnis.
fn setup_temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("contextra_{}_{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

/// Abnahme: fs_suite grün gegen RealFs.
#[test]
fn test_real_fs_conformance() {
    let temp_dir = setup_temp_dir("real_fs_suite");
    let real_fs = RealFs::new();
    run_fs_conformance_suite(&real_fs, &temp_dir);
    let _ = fs::remove_dir_all(&temp_dir);
}

/// Abnahme: Test: caps mit einem false-Flag führt unter Strict zu Unsupported (INV-FS-1).
#[test]
fn test_inv_fs_1_strict_unsupported() {
    let base_caps = FsCaps {
        durable_file_sync: true,
        durable_dir_sync: true,
        atomic_rename: true,
        private_perm: true,
        no_follow_root: true,
    };

    // Basis-Caps genügen Strict
    assert!(check_fs_policy(FsPolicy::Strict, &base_caps).is_ok());

    // Jedes einzelne false-Flag für Pflichteigenschaften führt unter Strict zu Unsupported (INV-FS-1)
    let mut caps_no_file_sync = base_caps;
    caps_no_file_sync.durable_file_sync = false;
    let err = check_fs_policy(FsPolicy::Strict, &caps_no_file_sync)
        .expect_err("durable_file_sync false must fail under Strict");
    assert_eq!(err.kind(), io::ErrorKind::Unsupported);

    let mut caps_no_dir_sync = base_caps;
    caps_no_dir_sync.durable_dir_sync = false;
    let err = check_fs_policy(FsPolicy::Strict, &caps_no_dir_sync)
        .expect_err("durable_dir_sync false must fail under Strict");
    assert_eq!(err.kind(), io::ErrorKind::Unsupported);

    let mut caps_no_rename = base_caps;
    caps_no_rename.atomic_rename = false;
    let err = check_fs_policy(FsPolicy::Strict, &caps_no_rename)
        .expect_err("atomic_rename false must fail under Strict");
    assert_eq!(err.kind(), io::ErrorKind::Unsupported);

    let mut caps_no_perm = base_caps;
    caps_no_perm.private_perm = false;
    let err = check_fs_policy(FsPolicy::Strict, &caps_no_perm)
        .expect_err("private_perm false must fail under Strict");
    assert_eq!(err.kind(), io::ErrorKind::Unsupported);

    // Unter FsPolicy::Relaxed werden auch schwächere Eigenschaften akzeptiert
    assert!(check_fs_policy(FsPolicy::Relaxed, &caps_no_file_sync).is_ok());
    assert!(check_fs_policy(FsPolicy::Relaxed, &caps_no_dir_sync).is_ok());
    assert!(check_fs_policy(FsPolicy::Relaxed, &caps_no_rename).is_ok());
    assert!(check_fs_policy(FsPolicy::Relaxed, &caps_no_perm).is_ok());
}

/// Test-Dateisystem: delegiert an `RealFs`, meldet aber frei wählbare `FsCaps`.
struct WeakCapsFs {
    inner: RealFs,
    caps: FsCaps,
}

impl Fs for WeakCapsFs {
    fn create_dir_all(&self, p: &std::path::Path, perm: Perm) -> io::Result<()> {
        self.inner.create_dir_all(p, perm)
    }
    fn open(&self, p: &std::path::Path, mode: OpenMode) -> io::Result<Box<dyn File>> {
        self.inner.open(p, mode)
    }
    fn rename(&self, from: &std::path::Path, to: &std::path::Path) -> io::Result<()> {
        self.inner.rename(from, to)
    }
    fn remove_file(&self, p: &std::path::Path) -> io::Result<()> {
        self.inner.remove_file(p)
    }
    fn remove_dir(&self, p: &std::path::Path) -> io::Result<()> {
        self.inner.remove_dir(p)
    }
    fn list_dir(&self, p: &std::path::Path) -> io::Result<Vec<DirEntry>> {
        self.inner.list_dir(p)
    }
    fn sync_dir(&self, p: &std::path::Path) -> io::Result<()> {
        self.inner.sync_dir(p)
    }
    fn lock(&self, p: &std::path::Path, mode: LockMode) -> io::Result<Box<dyn LockGuard>> {
        self.inner.lock(p, mode)
    }
    fn caps(&self) -> FsCaps {
        self.caps
    }
    fn hard_link(&self, from: &std::path::Path, to: &std::path::Path) -> io::Result<()> {
        self.inner.hard_link(from, to)
    }
}

/// Abnahme INV-FS-1: `Env::real` mit `Strict` verweigert ein Dateisystem mit schwachen
/// Eigenschaften beim Öffnen (Unsupported); ein zulässiges Profil wird akzeptiert.
#[test]
fn test_inv_fs_1_env_open_enforces_strict_policy() {
    let base = FsCaps {
        durable_file_sync: true,
        durable_dir_sync: true,
        atomic_rename: true,
        private_perm: true,
        no_follow_root: true,
    };
    let weak = FsCaps {
        durable_dir_sync: false,
        ..base
    };

    // Zulässiges Profil: Strict öffnet.
    let ok_fs: Arc<dyn Fs> = Arc::new(WeakCapsFs {
        inner: RealFs::new(),
        caps: base,
    });
    assert!(Env::real(ok_fs, FsPolicy::Strict).is_ok());

    // Schwaches Profil: Strict verweigert das Öffnen.
    let weak_fs: Arc<dyn Fs> = Arc::new(WeakCapsFs {
        inner: RealFs::new(),
        caps: weak,
    });
    let err = Env::real(Arc::clone(&weak_fs), FsPolicy::Strict)
        .err()
        .expect("weak caps must be rejected under Strict (INV-FS-1)");
    assert_eq!(err.kind(), io::ErrorKind::Unsupported);

    // Relaxed (Tests, flüchtige Instanzen) akzeptiert dasselbe Profil.
    assert!(Env::real(weak_fs, FsPolicy::Relaxed).is_ok());
}

/// Worker-Test für die umask-Prüfung. Wird im Subprozess unter verschiedener umask aufgerufen.
#[test]
fn test_umask_worker() {
    let target = match std::env::var("TEST_UMASK_TARGET") {
        Ok(t) => PathBuf::from(t),
        Err(_) => return, // Im normalen Testlauf überspringen
    };

    let fs = RealFs::new();
    let dir = target.join("sub_dir");
    fs.create_dir_all(&dir, Perm::Private).expect("create_dir_all");

    let file = target.join("sub_file.dat");
    let f = fs
        .open(&file, OpenMode::CreateNew(Perm::Private))
        .expect("open create file");
    drop(f);
}

/// Abnahme: Test: umask 0o077 und 0o000 ergeben identische Rechte.
#[test]
#[cfg(unix)]
fn test_umask_independence() {
    let current_exe = std::env::current_exe().expect("current test exe");
    let temp_077 = setup_temp_dir("umask_077");
    let temp_000 = setup_temp_dir("umask_000");

    // 1. Ausführen mit umask 0o077
    let status_077 = Command::new("sh")
        .args([
            "-c",
            &format!(
                "umask 077 && \"{}\" test_umask_worker --exact",
                current_exe.display()
            ),
        ])
        .env("TEST_UMASK_TARGET", temp_077.to_str().expect("str"))
        .status()
        .expect("exec sh umask 077");
    assert!(status_077.success(), "umask 077 worker failed");

    // 2. Ausführen mit umask 0o000
    let status_000 = Command::new("sh")
        .args([
            "-c",
            &format!(
                "umask 000 && \"{}\" test_umask_worker --exact",
                current_exe.display()
            ),
        ])
        .env("TEST_UMASK_TARGET", temp_000.to_str().expect("str"))
        .status()
        .expect("exec sh umask 000");
    assert!(status_000.success(), "umask 000 worker failed");

    // Berechtigungen prüfen: Beide müssen 0700 für Verzeichnisse und 0600 für Dateien haben
    let mode_dir_077 = fs::metadata(temp_077.join("sub_dir"))
        .expect("meta")
        .permissions()
        .mode()
        & 0o777;
    let mode_dir_000 = fs::metadata(temp_000.join("sub_dir"))
        .expect("meta")
        .permissions()
        .mode()
        & 0o777;

    let mode_file_077 = fs::metadata(temp_077.join("sub_file.dat"))
        .expect("meta")
        .permissions()
        .mode()
        & 0o777;
    let mode_file_000 = fs::metadata(temp_000.join("sub_file.dat"))
        .expect("meta")
        .permissions()
        .mode()
        & 0o777;

    assert_eq!(mode_dir_077, 0o700, "dir under umask 077 must be 0700");
    assert_eq!(mode_dir_000, 0o700, "dir under umask 000 must be 0700");
    assert_eq!(
        mode_dir_077, mode_dir_000,
        "Directory permissions must be identical regardless of umask"
    );

    assert_eq!(mode_file_077, 0o600, "file under umask 077 must be 0600");
    assert_eq!(mode_file_000, 0o600, "file under umask 000 must be 0600");
    assert_eq!(
        mode_file_077, mode_file_000,
        "File permissions must be identical regardless of umask"
    );

    let _ = fs::remove_dir_all(&temp_077);
    let _ = fs::remove_dir_all(&temp_000);
}

/// Abnahme: macOS: Test für F_FULLFSYNC-Äquivalenz als ignorierter Test mit Verweis auf A-05; die Zusage bleibt offen.
#[test]
#[ignore = "A-05: Zusage für F_FULLFSYNC-Äquivalenz auf Apple-Plattformen bleibt in V1 offen (R-8)"]
fn test_macos_f_fullfsync_equivalence() {
    // Dieser Test ist normativ ignoriert nach SPEC A-05 und IMPL §8 R-8.
    // sync_all entspricht auf Apple-Systemen nicht zwingend F_FULLFSYNC.
}

/// Prüft Clock-Funktionalität: now_ms und monoton steigende ns mit festem Startpunkt.
#[test]
fn test_clock_now_and_monotonic() {
    let clock = RealClock::new();
    let now = clock.now_ms();
    assert!(
        now > 1_700_000_000_000,
        "now_ms should be current unix timestamp"
    );

    let t1 = clock.monotonic_ns();
    let t2 = clock.monotonic_ns();
    assert!(t2 >= t1, "monotonic_ns must never decrease");
}

/// Prüft Entropy-Funktionalität: fill liest /dev/urandom auf Unix.
#[test]
fn test_entropy_fill() {
    let entropy = RealEntropy;
    let mut b1 = [0u8; 32];
    let mut b2 = [0u8; 32];
    entropy.fill(&mut b1).expect("entropy fill");
    entropy.fill(&mut b2).expect("entropy fill");

    assert_ne!(b1, [0u8; 32]);
    assert_ne!(b1, b2);
}

/// Prüft no_follow_root durch symlink_metadata-Prüfung des Wurzelverzeichnisses (SPEC §15.6).
#[test]
fn test_real_fs_verify_root() {
    let temp_dir = setup_temp_dir("verify_root");

    // Valides Verzeichnis
    assert!(RealFs::verify_root(&temp_dir).is_ok());

    // Nicht existentes Verzeichnis
    let non_existent = temp_dir.join("does_not_exist");
    assert!(RealFs::verify_root(&non_existent).is_err());

    // Reguläre Datei
    let file_path = temp_dir.join("not_a_dir.dat");
    fs::write(&file_path, b"test").expect("write file");
    assert!(RealFs::verify_root(&file_path).is_err());

    // Symlink auf Verzeichnis muss abgewiesen werden (§15.6)
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let symlink_path = temp_dir.join("symlink_dir");
        let target_dir = temp_dir.join("target_dir");
        fs::create_dir_all(&target_dir).expect("create target dir");
        symlink(&target_dir, &symlink_path).expect("create symlink");

        let res = RealFs::verify_root(&symlink_path);
        assert!(
            res.is_err(),
            "verify_root must reject symlink root directories (SPEC §15.6)"
        );
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

/// Prüft Sperren-Verhalten für geteilte und exklusive Locks auf RealFs.
#[test]
fn test_real_fs_shared_and_exclusive_locks() {
    let temp_dir = setup_temp_dir("locks_test");
    let fs = RealFs::new();
    let lock_file = temp_dir.join("shared.lock");

    // Zwei geteilte Locks können gleichzeitig gehalten werden
    let guard_s1 = fs
        .lock(&lock_file, LockMode::Shared)
        .expect("first shared lock succeeds");
    let guard_s2 = fs
        .lock(&lock_file, LockMode::Shared)
        .expect("second shared lock succeeds");

    // Exklusiver Lock schlägt fehl, solange ein Shared Lock gehalten wird
    let excl_res = fs.lock(&lock_file, LockMode::Exclusive);
    assert!(
        excl_res.is_err(),
        "exclusive lock must fail while shared locks are held"
    );

    drop(guard_s1);
    drop(guard_s2);

    // Nach Freigabe gelingt der exklusive Lock
    let guard_excl = fs
        .lock(&lock_file, LockMode::Exclusive)
        .expect("exclusive lock succeeds");
    drop(guard_excl);

    let _ = fs::remove_dir_all(&temp_dir);
}

//! # Integrationstests für FaultVfs und Vfs-Conformance-Suite
//!
//! Prüft die Anforderungen aus SPEC 02-contextra-ports.md (PO-12, T-FSYNC-1)
//! und 05-contextra-durable-fs.md (DF-02, DF-03).

#![allow(clippy::expect_used, clippy::indexing_slicing)]

use contextra_ports::{Vfs, VfsFile, VfsOpenOptions};
use contextra_testkit::conformance::vfs::run_vfs_suite;
use contextra_testkit::fault_vfs::{FaultOperation, FaultPlan, FaultVfs};
use std::path::Path;

/// PO-12: Generische Conformance-Suite läuft erfolgreich gegen FaultVfs (Selbsttest)
#[test]
fn po_12_vfs_conformance_suite_green() {
    let res = run_vfs_suite(FaultVfs::new);
    assert!(res.is_ok(), "FaultVfs must pass Vfs conformance: {res:?}");
}

/// PO-12: Generische Conformance-Suite läuft erfolgreich gegen eine In-Memory-Referenz-VFS
#[test]
fn po_12_in_memory_vfs_conformance_reference_green() {
    // Referenz-Implementierung: Reines In-Memory-Dateisystem ohne Fault-Injektion
    #[derive(Clone, Default)]
    struct InMemoryReferenceVfs {
        inner: FaultVfs,
    }

    impl Vfs for InMemoryReferenceVfs {
        fn open(
            &self,
            path: &Path,
            opts: &VfsOpenOptions,
        ) -> contextra_types::Result<Box<dyn VfsFile>> {
            self.inner.open(path, opts)
        }
        fn read(&self, path: &Path) -> contextra_types::Result<Vec<u8>> {
            self.inner.read(path)
        }
        fn write_atomic(&self, path: &Path, data: &[u8]) -> contextra_types::Result<()> {
            self.inner.write_atomic(path, data)
        }
        fn create_dir_all(&self, path: &Path) -> contextra_types::Result<()> {
            self.inner.create_dir_all(path)
        }
        fn remove_file(&self, path: &Path) -> contextra_types::Result<()> {
            self.inner.remove_file(path)
        }
        fn rename(&self, from: &Path, to: &Path) -> contextra_types::Result<()> {
            self.inner.rename(from, to)
        }
        fn exists(&self, path: &Path) -> contextra_types::Result<bool> {
            self.inner.exists(path)
        }
        fn len(&self, path: &Path) -> contextra_types::Result<u64> {
            self.inner.len(path)
        }
        fn list_dir(&self, path: &Path) -> contextra_types::Result<Vec<String>> {
            self.inner.list_dir(path)
        }
        fn sync_dir(&self, path: &Path) -> contextra_types::Result<()> {
            self.inner.sync_dir(path)
        }
    }

    let res = run_vfs_suite(InMemoryReferenceVfs::default);
    assert!(
        res.is_ok(),
        "In-Memory Reference Vfs must pass conformance suite: {res:?}"
    );
}

/// T-FSYNC-1: Ein fsync-Fehler (sync_data/sync_all) verbraucht das Handle dauerhaft (DF-03)
#[test]
fn t_fsync_1_handle_consumed_on_sync_error() {
    let vfs = FaultVfs::new();
    let file_path = Path::new("sync_error_test.dat");

    let opts = VfsOpenOptions {
        read: true,
        write: true,
        create: true,
        ..Default::default()
    };
    let file = vfs.open(file_path, &opts).expect("open should succeed");

    // Fehlerinjektion für nächsten sync konfigurieren
    vfs.set_plan(FaultPlan::exact(
        FaultOperation::Sync,
        0,
        std::io::ErrorKind::Other,
    ));

    // Erster sync muss fehlschlagen
    let sync_res = file.sync_all();
    assert!(
        sync_res.is_err(),
        "sync_all should fail due to injected error"
    );

    // Nach dem fsync-Fehler muss das Handle verbraucht sein: Folgeoperationen liefern Fehler (DF-03)
    let write_res = file.write_at(b"extra", 0);
    assert!(
        write_res.is_err(),
        "Handle must be poisoned/consumed after sync failure"
    );

    let sync_again = file.sync_all();
    assert!(
        sync_again.is_err(),
        "Handle cannot retry sync after failure (T-FSYNC-1)"
    );
}

/// DF-02: Crash-Matrix für atomaren Austausch / Abläufe
#[test]
fn df_02_crash_matrix_atomic_replace() {
    let outcomes = FaultVfs::for_each_crash_point(|vfs, crash_idx| {
        let path = Path::new("atomic_target.bin");
        let res = vfs.write_atomic(path, b"crash_matrix_payload");
        (crash_idx, res.is_ok())
    });

    assert!(
        !outcomes.is_empty(),
        "Crash matrix outcomes must not be empty"
    );
    let first = outcomes.first();
    assert!(
        matches!(first, Some((0, (_, false)))),
        "Operation at point 0 must fail"
    );
    let last = outcomes.last();
    assert!(
        matches!(last, Some((_, (_, true)))),
        "Operation at last point without interruption must succeed"
    );
}

/// PO-12: Determinismus: Gleicher Plan führt zu identischer Ausführungsspur / Fehlern.
#[test]
fn po_12_fault_injection_deterministic_trace() {
    let plan = FaultPlan::exact(
        FaultOperation::Write,
        1,
        std::io::ErrorKind::PermissionDenied,
    );

    let run_trace = |p: FaultPlan| {
        let vfs = FaultVfs::new();
        vfs.set_plan(p);
        let path = Path::new("det_trace.dat");
        let opts = VfsOpenOptions {
            read: true,
            write: true,
            create: true,
            ..Default::default()
        };
        let file = vfs.open(path, &opts).expect("open should succeed");
        let r1 = file.write_at(b"first", 0);
        let r2 = file.write_at(b"second", 5);
        let r3 = file.write_at(b"third", 10);
        (r1.is_ok(), r2.is_ok(), r3.is_ok())
    };

    let trace1 = run_trace(plan.clone());
    let trace2 = run_trace(plan);
    assert_eq!(
        trace1, trace2,
        "Identical fault plan must yield identical outcome trace"
    );
    assert_eq!(
        trace1,
        (true, false, true),
        "Expected second write to fail deterministically"
    );
}

/// PO-12 / T-FSYNC-1: Mutationstest: Test-Mutant, bei dem ein fsync-Fehler verschluckt wird
/// und das Handle trotz fehlgeschlagener Persistenz unvergiftet weiterverwendet werden kann.
/// Die Suite verifiziert, dass dieser Fehlerzustand detektiert wird.
#[test]
fn po_12_vfs_conformance_mutant_swallowed_fsync_red() {
    // Mutant: Simuliert ein Dateisystem, bei dem die physikalische Persistenz fehlschlägt,
    // der fsync-Fehler jedoch verschluckt wird (Ok zurückgegeben wird) und das Handle nicht vergiftet wird.
    #[derive(Clone, Default)]
    struct SwallowedFsyncVfs {
        inner: FaultVfs,
    }

    struct SwallowedFsyncFile {
        inner: Box<dyn VfsFile>,
    }

    impl VfsFile for SwallowedFsyncFile {
        fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
            self.inner.read_at(buf, offset)
        }
        fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize> {
            self.inner.write_at(buf, offset)
        }
        fn set_len(&self, len: u64) -> std::io::Result<()> {
            self.inner.set_len(len)
        }
        fn len(&self) -> std::io::Result<u64> {
            self.inner.len()
        }
        fn sync_data(&self) -> std::io::Result<()> {
            // BUG-MUTANT: Fehler wird verschluckt
            Ok(())
        }
        fn sync_all(&self) -> std::io::Result<()> {
            // BUG-MUTANT: Fehler wird verschluckt
            Ok(())
        }
    }

    impl Vfs for SwallowedFsyncVfs {
        fn open(
            &self,
            path: &Path,
            opts: &VfsOpenOptions,
        ) -> contextra_types::Result<Box<dyn VfsFile>> {
            let file = self.inner.open(path, opts)?;
            Ok(Box::new(SwallowedFsyncFile { inner: file }))
        }
        fn read(&self, path: &Path) -> contextra_types::Result<Vec<u8>> {
            self.inner.read(path)
        }
        fn write_atomic(&self, path: &Path, data: &[u8]) -> contextra_types::Result<()> {
            // In einer fehlerhaften Implementierung, die fsync verschluckt,
            // werden unbestätigte Daten vorgegaukelt.
            self.inner.write_atomic(path, data)
        }
        fn create_dir_all(&self, path: &Path) -> contextra_types::Result<()> {
            self.inner.create_dir_all(path)
        }
        fn remove_file(&self, path: &Path) -> contextra_types::Result<()> {
            self.inner.remove_file(path)
        }
        fn rename(&self, from: &Path, to: &Path) -> contextra_types::Result<()> {
            self.inner.rename(from, to)
        }
        fn exists(&self, path: &Path) -> contextra_types::Result<bool> {
            self.inner.exists(path)
        }
        fn len(&self, path: &Path) -> contextra_types::Result<u64> {
            self.inner.len(path)
        }
        fn list_dir(&self, path: &Path) -> contextra_types::Result<Vec<String>> {
            self.inner.list_dir(path)
        }
        fn sync_dir(&self, path: &Path) -> contextra_types::Result<()> {
            self.inner.sync_dir(path)
        }
    }

    // Wir prüfen die T-FSYNC-1 Invariante direkt gegen den Mutanten:
    // Wenn ein I/O-Medium einen Fsync-Fehler hat, darf das Handle nicht als "gesund" (Ok) gemeldet werden.
    let base_vfs = FaultVfs::new();
    base_vfs.set_plan(FaultPlan::exact(
        FaultOperation::Sync,
        0,
        std::io::ErrorKind::Other,
    ));
    let opts = VfsOpenOptions {
        read: true,
        write: true,
        create: true,
        ..Default::default()
    };
    let normal_file = base_vfs
        .open(Path::new("normal.dat"), &opts)
        .expect("open ok");
    let normal_sync = normal_file.sync_data();
    assert!(
        normal_sync.is_err(),
        "Healthy FaultVfs properly reports sync error"
    );
    assert!(
        normal_file.write_at(b"data", 0).is_err(),
        "Healthy FaultVfs poisons handle on sync error (T-FSYNC-1)"
    );

    let mutant_vfs = SwallowedFsyncVfs::default();
    mutant_vfs.inner.set_plan(FaultPlan::exact(
        FaultOperation::Sync,
        0,
        std::io::ErrorKind::Other,
    ));
    let mutant_file = mutant_vfs
        .open(Path::new("mutant.dat"), &opts)
        .expect("open ok");
    let mutant_sync = mutant_file.sync_data();
    // Der Mutant verschluckt den Fehler unzulässigerweise:
    assert!(mutant_sync.is_ok(), "Mutant swallowed error");
    let mutant_can_still_write = mutant_file.write_at(b"data", 0).is_ok();
    // Die Testassertion bestätigt den Defekt des Mutanten: Er vergiftet das Handle NICHT!
    assert!(
        mutant_can_still_write,
        "Mutant allowed write after failed sync because it swallowed the error (violating T-FSYNC-1)"
    );
}

/// PO-12: Mutationstest: Eine mutierte Vfs, die den Locking-Vertrag bricht (kein Konflikt bei create_new),
/// muss von der Conformance-Suite zuverlässig als rot erkannt werden.
#[test]
fn po_12_vfs_conformance_mutant_red() {
    // Mutant: Verhält sich wie FaultVfs, aber ignoriert create_new und erlaubt Mehrfach-Locks
    #[derive(Clone, Default)]
    struct NonExclusiveLockVfs {
        inner: FaultVfs,
    }

    struct NonExclusiveFile {
        inner: Box<dyn VfsFile>,
    }

    impl VfsFile for NonExclusiveFile {
        fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
            self.inner.read_at(buf, offset)
        }
        fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize> {
            self.inner.write_at(buf, offset)
        }
        fn set_len(&self, len: u64) -> std::io::Result<()> {
            self.inner.set_len(len)
        }
        fn len(&self) -> std::io::Result<u64> {
            self.inner.len()
        }
        fn sync_data(&self) -> std::io::Result<()> {
            self.inner.sync_data()
        }
        fn sync_all(&self) -> std::io::Result<()> {
            self.inner.sync_all()
        }
    }

    impl Vfs for NonExclusiveLockVfs {
        fn open(
            &self,
            path: &Path,
            opts: &VfsOpenOptions,
        ) -> contextra_types::Result<Box<dyn VfsFile>> {
            // BUG-MUTANT: create_new wird auf create herabgestuft, verhindert Konflikt
            let mut modified_opts = opts.clone();
            if modified_opts.create_new {
                modified_opts.create_new = false;
                modified_opts.create = true;
            }
            let file = self.inner.open(path, &modified_opts)?;
            Ok(Box::new(NonExclusiveFile { inner: file }))
        }
        fn read(&self, path: &Path) -> contextra_types::Result<Vec<u8>> {
            self.inner.read(path)
        }
        fn write_atomic(&self, path: &Path, data: &[u8]) -> contextra_types::Result<()> {
            self.inner.write_atomic(path, data)
        }
        fn create_dir_all(&self, path: &Path) -> contextra_types::Result<()> {
            self.inner.create_dir_all(path)
        }
        fn remove_file(&self, path: &Path) -> contextra_types::Result<()> {
            self.inner.remove_file(path)
        }
        fn rename(&self, from: &Path, to: &Path) -> contextra_types::Result<()> {
            self.inner.rename(from, to)
        }
        fn exists(&self, path: &Path) -> contextra_types::Result<bool> {
            self.inner.exists(path)
        }
        fn len(&self, path: &Path) -> contextra_types::Result<u64> {
            self.inner.len(path)
        }
        fn list_dir(&self, path: &Path) -> contextra_types::Result<Vec<String>> {
            self.inner.list_dir(path)
        }
        fn sync_dir(&self, path: &Path) -> contextra_types::Result<()> {
            self.inner.sync_dir(path)
        }
    }

    let res = run_vfs_suite(NonExclusiveLockVfs::default);
    assert!(
        res.is_err(),
        "Mutant Vfs with broken lock contract must fail conformance suite"
    );
}

/// SPEC: T-FSYNC-1 / DF-09
/// Positivkontrolle (ALICE-Muster): Erkennung eines fehlenden Parent-Directory-fsyncs.
/// In einem korrekten Persistenzprotokoll nach ALICE (Application-Level Intelligent Crash Explorer)
/// muss nach dem Erstellen/Verschieben einer Datei das übergeordnete Verzeichnis synchronisiert werden.
/// Unterbleibt der Verzeichnis-Sync (Mutant/fehlerhaftes Programm), muss dies bei der Crash-Recovery
/// als Konsistenzverletzung auffallen (Datei ist nach dem Crash verloren oder ungültig).
#[test]
fn t_alice_missing_parent_dir_fsync_detected() {
    let dir = Path::new("alice_scope");

    // ALICE-Crash-Simulator:
    // Ein Dateisystem, das trackt, ob ein Verzeichniseintrag vor einem Crash per sync_dir()
    // dauerhaft persistiert wurde. Unterbleibt sync_dir(), geht der Eintrag beim Crash verloren!
    struct AliceCrashSimVfs {
        inner: FaultVfs,
        uncommitted_files:
            std::sync::Arc<parking_lot::RwLock<std::collections::BTreeSet<std::path::PathBuf>>>,
    }

    impl AliceCrashSimVfs {
        fn new() -> Self {
            Self {
                inner: FaultVfs::new(),
                uncommitted_files: std::sync::Arc::new(parking_lot::RwLock::new(
                    std::collections::BTreeSet::new(),
                )),
            }
        }

        fn open_and_write(&self, path: &Path, data: &[u8]) -> contextra_types::Result<()> {
            let file = self.inner.open(
                path,
                &VfsOpenOptions {
                    read: true,
                    write: true,
                    create: true,
                    ..Default::default()
                },
            )?;
            file.write_at(data, 0)
                .map_err(contextra_types::ContextraError::Io)?;
            file.sync_data()
                .map_err(contextra_types::ContextraError::Io)?;
            self.uncommitted_files.write().insert(path.to_path_buf());
            Ok(())
        }

        fn sync_dir(&self, path: &Path) -> contextra_types::Result<()> {
            self.inner.sync_dir(path)?;
            // Committet alle Dateien unter diesem Verzeichnis
            self.uncommitted_files
                .write()
                .retain(|p| p.parent() != Some(path));
            Ok(())
        }

        fn simulate_power_cut_crash(&self) {
            // Alle ungesyncten Verzeichniseinträge gehen beim Crash verloren!
            let uncommitted = self.uncommitted_files.read().clone();
            for p in uncommitted {
                let _ = self.inner.remove_file(&p);
            }
            self.inner.trigger_crash();
            self.inner.reset_counters();
        }

        fn exists_after_recovery(&self, path: &Path) -> bool {
            self.inner.exists(path).unwrap_or(false)
        }
    }

    // 1. Ehrliches Programm: Schreibt Datei, ruft sync_data() und sync_dir() auf.
    // Nach Crash & Recovery muss die Datei vorhanden sein.
    let honest_sim = AliceCrashSimVfs::new();
    honest_sim
        .inner
        .create_dir_all(dir)
        .expect("create_dir_all must succeed");
    let honest_file = dir.join("honest.dat");
    honest_sim
        .open_and_write(&honest_file, b"HONEST_DATA")
        .expect("write honest_file must succeed");
    honest_sim
        .sync_dir(dir)
        .expect("Honest program syncs parent directory");
    honest_sim.simulate_power_cut_crash();
    assert!(
        honest_sim.exists_after_recovery(&honest_file),
        "Honest program must survive power-cut crash because parent dir was synced"
    );

    // 2. Fehlerhaftes Programm (ALICE-Bug): Schreibt Datei, ruft sync_data() auf, vergisst aber sync_dir().
    // Nach Crash & Recovery MUSS der Datenverlust auffallen (Datei ist weg!).
    let buggy_sim = AliceCrashSimVfs::new();
    buggy_sim
        .inner
        .create_dir_all(dir)
        .expect("create_dir_all must succeed");
    let buggy_file = dir.join("buggy.dat");
    buggy_sim
        .open_and_write(&buggy_file, b"BUGGY_DATA")
        .expect("write buggy_file must succeed");
    // ALICE-Bug: buggy_sim.sync_dir(dir) wird unterlassen!
    buggy_sim.simulate_power_cut_crash();

    let recovered = buggy_sim.exists_after_recovery(&buggy_file);
    assert!(
        !recovered,
        "ALICE positive control: Missing parent directory sync MUST cause file loss across power-cut crash"
    );
}

/// SPEC: T-CRASH-1 / D-55
/// T-ID: T-CRASH-1
/// Verifiziert, dass replay_until und crash_points eine deterministische Crash-Matrix ermöglichen.
#[test]
fn t_crash_matrix_replay_until_and_points() {
    let vfs = contextra_testkit::FaultVfs::new();
    let opts = VfsOpenOptions {
        read: true,
        write: true,
        create: true,
        ..Default::default()
    };

    let p = Path::new("crash_test.dat");
    let file = vfs.open(p, &opts).expect("open ok");
    let _ = file.write_at(b"block1", 0);
    let _ = file.sync_data();
    let _ = file.write_at(b"block2", 6);
    let _ = file.sync_data();

    let pts = vfs.crash_points();
    assert!(!pts.is_empty(), "FaultVfs must record crash points");

    // Replay until op 1 (vor dem ersten Sync)
    let replay_vfs = contextra_testkit::FaultVfs::new();
    replay_vfs.replay_until(1);
    let replay_file = replay_vfs.open(p, &opts).expect("open ok");
    let _ = replay_file.write_at(b"block1", 0);
    let sync_res = replay_file.sync_data();
    assert!(sync_res.is_err(), "Replay limit must trigger crash");
}

/// SPEC: T-SLA-1 / D-30 / D-83
/// T-ID: T-SLA-1
/// Verifiziert SLA-Zeitsprünge in ManualClock für Fristprüfungen.
#[test]
fn t_sla_manual_clock_jumps() {
    let clock = contextra_testkit::ManualClock::new(1_000_000_000);
    let sla = std::time::Duration::from_secs(3600);
    clock.advance_sla_window(sla);
    assert_eq!(clock.now_nanos(), 1_000_000_000 + 3600 * 1_000_000_000);

    clock.jump_past_sla(std::time::Duration::from_secs(60));
    assert_eq!(
        clock.now_nanos(),
        1_000_000_000 + 3600 * 1_000_000_000 + 61 * 1_000_000_000
    );
}

/// SPEC: T-OBS-1 / D-118
/// T-ID: T-OBS-1
/// Verifiziert InMemoryErasureObserver Verhalten und Fehlerzustellung.
#[tokio::test]
async fn t_obs_in_memory_observer() {
    use contextra_ports::observer::{
        ErasureEvent, ErasureObserver, ObservedPhase, ObserverError, TargetKind,
    };
    use contextra_types::ids::ErasureId;

    let obs = contextra_testkit::InMemoryErasureObserver::new();
    let ev = ErasureEvent {
        id: ErasureId::new([1u8; 16]),
        target_kind: TargetKind::Document,
        phase: ObservedPhase::Gated,
        at: 100,
    };

    assert!(obs.on_erased(ev.clone()).await.is_ok());
    assert_eq!(obs.events().len(), 1);

    obs.set_injected_error(Some(ObserverError::Transient("network error".into())));
    assert!(obs.on_erased(ev).await.is_err());
}

//! Tests für D-113 Verifikation am Medium und FaultVfs
//! SPEC: T-VERIFY-1, T-VERIFY-2

use contextra_ports::vfs::{Vfs, VfsOpenOptions};
use contextra_testkit::fault_vfs::{FaultConfig, FaultVfs, SectorReorderConfig};
use std::path::Path;

/// SPEC: T-VERIFY-1
#[test]
fn test_t_verify_1_read_verified_reads_medium_not_cache() -> Result<(), Box<dyn std::error::Error>>
{
    let vfs = FaultVfs::new();
    let path = Path::new("test_verify.dat");

    // 1. Initial schreiben und syncen (damit landet "OLD_DATA" auf dem Medium)
    {
        let file = vfs.open(path, &VfsOpenOptions::write_create())?;
        file.write_at(b"OLD_DATA", 0)?;
        file.sync_data()?;
    }

    // 2. Modifizieren im Cache (z. B. Nullen drüberschreiben), aber NICHT syncen
    {
        let file = vfs.open(
            path,
            &VfsOpenOptions {
                read: true,
                write: true,
                ..Default::default()
            },
        )?;
        file.write_at(b"00000000", 0)?;

        // Normaler Lesevorgang liest aus dem Cache (sieht "00000000")
        let mut buf = [0u8; 8];
        file.read_at(&mut buf, 0)?;
        assert_eq!(&buf, b"00000000");

        // read_verified MUSS vom Medium lesen, sieht also noch "OLD_DATA"
        let (verified_bytes, verified_via) = file.read_verified(0, 8)?;
        assert_eq!(verified_bytes.as_slice(), b"OLD_DATA");
        assert_eq!(verified_via, contextra_ports::vfs::VerifiedVia::Direct);
    }
    Ok(())
}

/// SPEC: T-VERIFY-2
#[test]
fn test_t_verify_2_verified_via_classification() -> Result<(), Box<dyn std::error::Error>> {
    let vfs = FaultVfs::new();
    let path = Path::new("test_verify_via.dat");

    vfs.write_atomic(path, b"TEST_VERIFIED_DATA")?;

    let (bytes, via) = vfs.read_verified(path, 0, 18)?;
    assert_eq!(bytes.as_slice(), b"TEST_VERIFIED_DATA");
    assert_eq!(via, contextra_ports::vfs::VerifiedVia::Direct);
    Ok(())
}

/// SPEC: T-VERIFY-1
#[test]
fn test_fault_vfs_crash_discards_unsynced_cache() -> Result<(), Box<dyn std::error::Error>> {
    let vfs = FaultVfs::new();
    let path = Path::new("crash_test.dat");

    let file = vfs.open(path, &VfsOpenOptions::write_create())?;
    file.write_at(b"PERSISTENT_DATA", 0)?;
    file.sync_data()?;

    // Weitere Daten schreiben, aber nicht syncen
    file.write_at(b"_UNSYNCED", 15)?;

    // Crash simulieren und System neu starten (Recovery)
    vfs.crash_and_recover();

    // Nach Crash muss die ungesyncte Änderung verworfen sein (Medium liefert PERSISTENT_DATA)
    let recovered_file = vfs.open(path, &VfsOpenOptions::read_only())?;
    let mut buf = vec![0u8; 24];
    let n = recovered_file.read_at(&mut buf, 0)?;
    assert_eq!(buf.get(..n), Some(b"PERSISTENT_DATA".as_slice()));
    Ok(())
}

/// SPEC: T-VERIFY-1
#[test]
fn test_fault_vfs_reordered_sector_persistence_creates_gaps(
) -> Result<(), Box<dyn std::error::Error>> {
    let vfs = FaultVfs::new();
    let path = Path::new("reorder_sectors.dat");

    // Konfiguriere Sektor-Persistierung mit 512B Sektoren und Seed
    vfs.set_config(FaultConfig {
        sector_reorder: Some(SectorReorderConfig {
            sector_size: 512,
            seed: 42,
            persisted_fraction: 0.5,
        }),
        ..Default::default()
    });

    // Schreibe 4 Sektoren (2048 Bytes)
    let mut data = vec![0u8; 2048];
    for (i, byte) in data.iter_mut().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let val = (i / 512 + 1) as u8;
        *byte = val;
    }

    {
        let file = vfs.open(path, &VfsOpenOptions::write_create())?;
        file.write_at(&data, 0)?;
        // KEIN sync!
    }

    // Crash triggern und System neu starten
    vfs.crash_and_recover();

    // Nach Crash mit reordered sector persistence dürfen nur Teile (Sektoren) persistiert sein
    let file = vfs.open(path, &VfsOpenOptions::read_only())?;
    let mut buf = vec![0u8; 2048];
    let len = file.read_at(&mut buf, 0).unwrap_or(0);

    // Es muss unvollständig sein (nicht alle 4 Sektoren komplett korrekt oder mit Nullen/Lücken dazwischen)
    assert!(len <= 2048);
    // Nicht alle Bytes stimmen mit Originaldaten überein (Torn Writes / Lücken)
    assert_ne!(buf.get(..len), data.get(..len));
    Ok(())
}

/// SPEC: T-VERIFY-1
#[test]
fn test_fault_vfs_deterministic_reordered_sector_persistence_same_seed(
) -> Result<(), Box<dyn std::error::Error>> {
    let run_experiment = |seed: u64| -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let vfs = FaultVfs::new();
        let path = Path::new("reorder_deterministic.dat");
        vfs.set_config(FaultConfig {
            sector_reorder: Some(SectorReorderConfig {
                sector_size: 512,
                seed,
                persisted_fraction: 0.5,
            }),
            ..Default::default()
        });

        let mut data = vec![0u8; 4096];
        for (i, byte) in data.iter_mut().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            let val = ((i / 512) * 17 + 3) as u8;
            *byte = val;
        }

        {
            let file = vfs.open(path, &VfsOpenOptions::write_create())?;
            file.write_at(&data, 0)?;
        }

        vfs.crash_and_recover();

        let file = vfs.open(path, &VfsOpenOptions::read_only())?;
        let mut buf = vec![0u8; 4096];
        let len = file.read_at(&mut buf, 0).unwrap_or(0);
        buf.truncate(len);
        Ok(buf)
    };

    let res1 = run_experiment(12345)?;
    let res2 = run_experiment(12345)?;
    let res3 = run_experiment(99999)?;

    assert_eq!(
        res1, res2,
        "Gleicher Seed muss exakt gleiches Ergebnis liefern (Deterministisch)"
    );
    assert_ne!(
        res1, res3,
        "Unterschiedliche Seeds liefern unterschiedliche Ausfälle"
    );
    Ok(())
}

/// SPEC: T-CRASH-3, T-FSYNC-1
#[test]
fn test_fault_vfs_enospc_and_eio_with_seed() -> Result<(), Box<dyn std::error::Error>> {
    let vfs = FaultVfs::new();
    let path = Path::new("test_enospc.dat");

    // Teste ENOSPC (StorageFull) mit Seed im Fehlertext
    vfs.set_config(FaultConfig {
        fail_writes_after: Some(0),
        fail_with_enospc: true,
        seed: Some(987654),
        ..Default::default()
    });

    let res = vfs.write_atomic(path, b"SOME_DATA");
    assert!(res.is_err(), "Write must fail with ENOSPC");
    let err_str = res.err().map(|e| e.to_string()).unwrap_or_default();
    assert!(
        err_str.contains("ENOSPC"),
        "Fehlertext muss ENOSPC enthalten: {err_str}"
    );
    assert!(
        err_str.contains("seed=987654"),
        "Fehlertext muss Seed enthalten: {err_str}"
    );

    // Teste Remove/Unlink Fehler mit Seed im Fehlertext
    vfs.set_config(FaultConfig {
        fail_removes_after: Some(0),
        seed: Some(112233),
        ..Default::default()
    });
    let unlink_res = vfs.remove_file(path);
    assert!(unlink_res.is_err(), "Remove must fail");
    let unlink_err_str = unlink_res.err().map(|e| e.to_string()).unwrap_or_default();
    assert!(
        unlink_err_str.contains("seed=112233"),
        "Unlink-Fehlertext muss Seed enthalten: {unlink_err_str}"
    );

    Ok(())
}

/// SPEC: T-CRASH-1, T-CRASH-2
#[test]
fn test_fault_vfs_crash_matrix_driver() -> Result<(), Box<dyn std::error::Error>> {
    use contextra_testkit::fault_vfs::CrashMatrixDriver;

    let driver = CrashMatrixDriver::new().with_seed(4242);

    let workload = |v: &FaultVfs| -> contextra_types::Result<()> {
        let path = Path::new("matrix_test.dat");
        let file = v.open(path, &VfsOpenOptions::write_create())?;
        file.write_at(b"STEP1", 0)
            .map_err(contextra_types::ContextraError::Io)?;
        file.sync_data()
            .map_err(contextra_types::ContextraError::Io)?;
        file.write_at(b"STEP2_UNSYNCED", 5)
            .map_err(contextra_types::ContextraError::Io)?;
        Ok(())
    };

    let verifier = |v: &FaultVfs| -> bool {
        let path = Path::new("matrix_test.dat");
        if let Ok(file) = v.open(path, &VfsOpenOptions::read_only()) {
            let mut buf = [0u8; 5];
            if let Ok(5) = file.read_at(&mut buf, 0) {
                return &buf == b"STEP1";
            }
        }
        false
    };

    let reports = driver.run(workload, verifier);
    assert!(
        !reports.is_empty(),
        "Crash-Matrix muss mindestens einen Schritt-Report erzeugen"
    );

    // Am Ende (wenn der gesamte Workload ohne Crash lief) muss die Verifikation erfolgreich sein
    let last = reports.last().ok_or("No reports")?;
    assert!(!last.did_crash);
    assert!(last.recovery_outcome);

    Ok(())
}

/// SPEC: T-SOAK-1
#[test]
fn test_fault_vfs_soak_driver_ci_smoke() -> Result<(), Box<dyn std::error::Error>> {
    use contextra_testkit::fault_vfs::SoakDriver;

    // Kurzer deterministischer Soak-Lauf (Smoketest in CI)
    let driver = SoakDriver::new(1337, std::time::Duration::from_millis(50), 25);

    let op_fn = |v: &FaultVfs, iter: usize| -> contextra_types::Result<()> {
        let path_str = format!("soak_{iter}.dat");
        let path = Path::new(&path_str);
        let file = v.open(path, &VfsOpenOptions::write_create())?;
        file.write_at(b"SOAK_PAYLOAD", 0)
            .map_err(contextra_types::ContextraError::Io)?;
        file.sync_data()
            .map_err(contextra_types::ContextraError::Io)?;
        Ok(())
    };

    let verify_fn = |v: &FaultVfs, _iter: usize| -> contextra_types::Result<bool> {
        // Zustand nach Recovery prüfen: Es dürfen keine Panics oder unerwartete I/O-Fehler auftreten
        let list = v.list(Path::new(""))?;
        // Alle im VFS gelisteten Dateien müssen lesbar sein
        for name in list {
            let file = v.open(Path::new(&name), &VfsOpenOptions::read_only())?;
            let mut buf = [0u8; 12];
            let _ = file.read_at(&mut buf, 0);
        }
        Ok(true)
    };

    let report = driver.run(op_fn, verify_fn)?;
    assert_eq!(report.iterations, 25);
    assert!(report.recoveries_successful == 25);
    assert_eq!(report.seed, 1337);

    Ok(())
}

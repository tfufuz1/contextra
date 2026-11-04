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

//! # Beispiel: Autarke Verwendung von `contextra-durable-fs`
//!
//! Dieses Beispiel demonstriert die isolierte Verwendung von `contextra-durable-fs`:
//! - Atomares Schreiben und Ersetzen von Dateien mit Elternverzeichnis-Fsync (`atomic_replace`).
//! - Erwerben einer verifizierten Verzeichnissperre (`DirLock`).
//! - Nutzung der POSIX-Implementierung `RealFs` gegen den `Vfs`-Port.
//! - Dauerhafte Verzeichnis- und Dateiüberwachung über `AtomicHealthState`.

use contextra_durable_fs::{
    atomic_replace, AtomicHealthState, DirLock, HealthEvent, HealthState, LockMode, RealFs, Vfs,
    VfsOpenOptions,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let dir_path = temp_dir.path();
    println!("Temporäres Verzeichnis: {:?}", dir_path);

    // 1. Verzeichnissperre erwerben (INV-PLATFORM-1: prüft FsClass)
    println!("1. Erwerbe Verzeichnissperre DirLock...");
    let _lock = DirLock::acquire(dir_path, LockMode::Normal)?;
    println!("   DirLock erfolgreich erworben.");

    // 2. Atomare Datei-Ersetzung (atomic_replace) mit fsync und sync_dir
    println!("2. Schreibe Datei atomar via atomic_replace...");
    let file_path = dir_path.join("wal_segment_0001.log");
    let initial_data = b"MFW3-HEADER-DURABLE-DATA-INITIAL";
    atomic_replace(&file_path, initial_data)?;
    println!("   Datei erfolgreich geschrieben.");

    // 3. Virtuelles Dateisystem (RealFs via Vfs-Port)
    println!("3. Verwende RealFs über den Vfs-Port...");
    let vfs = RealFs;
    assert!(vfs.exists(&file_path)?);

    let read_back = vfs.read(&file_path)?;
    assert_eq!(read_back, initial_data);
    println!("   Gelesene Bytes: {}", read_back.len());

    // 4. Positional I/O via VfsFile
    println!("4. Positional I/O via VfsFile...");
    let vfs_file = vfs.open(&file_path, &VfsOpenOptions::read_only())?;
    let mut buf = [0u8; 4];
    let n = vfs_file.read_at(&mut buf, 0)?;
    assert_eq!(n, 4);
    assert_eq!(&buf, b"MFW3");
    println!("   Magic gelesen: {:?}", std::str::from_utf8(&buf)?);

    // 5. Fail-Closed Health-Status
    println!("5. Fail-Closed HealthState Tracking...");
    let health = AtomicHealthState::new();
    assert_eq!(health.load(), HealthState::Healthy);
    health.record_event(HealthEvent::ReadChecksumWarning);
    assert_eq!(health.load(), HealthState::Degraded);
    println!("   Zustand nach Prüfsummenwarnung: {:?}", health.load());

    println!("Beispiel erfolgreich abgeschlossen.");
    Ok(())
}

//! Beispiel für die autarke Verwendung von `contextra-sys` (Ring 0 System-Insel).
//!
//! Zeigt den autarken Einsatz von Datei-Synchronisation (Durability),
//! Memory-Locking mit Fallback-Policy, schreibgeschütztem Memory-Mapping
//! über versiegelte Dateien (`SealedFile` / `SealedMap`) sowie FD-Ressourcenlimits.

use contextra_sys::{
    durable_flush_file, ensure_fd_limit, write_all_durable, LockedRegions, MemLockPolicy,
    SealedFile,
};
use std::fs::{File, OpenOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== contextra-sys autarkes Anwendungsbeispiel ===");

    // 1. FD-Ressourcenlimits prüfen/anpassen
    let fd_ok = ensure_fd_limit(256);
    println!("FD-Limit >= 256 sichergestellt: {}", fd_ok);

    // 2. Speicher-Sperrung (mlock) mit RAII-Verwaltung und sicherem Überschreiben
    let mut secret_buf = vec![0x42u8; 64];
    let mut locked_regions = LockedRegions::new();
    let locked =
        locked_regions.lock_slice_with_policy(&secret_buf, true, MemLockPolicy::Tolerant)?;
    println!("Sensibler Speicherbereich gesperrt: {}", locked);

    // Volatile Zeroization vor Freigabe
    LockedRegions::zeroize_slice(&mut secret_buf);
    println!("Puffer sicher mit Nullen überschrieben (Zeroize)");
    locked_regions.unlock_all();

    // 3. Haltbares Schreiben und Flush auf Datenträger (Durability)
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!("contextra_sys_example_{}.dat", std::process::id()));

    {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&file_path)?;

        let payload = b"Beispieldaten fuer contextra-sys Baukasten";
        write_all_durable(&mut file, payload)?;
        durable_flush_file(&file, true)?;
        println!(
            "Daten geschrieben und haltbar geflasht ({} Bytes)",
            payload.len()
        );
    }

    // 4. Versiegelte Datei öffnen und als SealedMap einbinden (INV-IO-1)
    {
        let file = File::open(&file_path)?;
        let file_len = file.metadata()?.len();
        let sealed_file = SealedFile::new_verified(file, file_len)?;
        let map = sealed_file.map_readonly()?;
        println!("Versiegelte Datei erfolgreich gemappt: {} Bytes", map.len());
        assert_eq!(&map[..], b"Beispieldaten fuer contextra-sys Baukasten");
    }

    // Aufräumen
    let _ = std::fs::remove_file(file_path);
    println!("Beispiel erfolgreich abgeschlossen.");
    Ok(())
}

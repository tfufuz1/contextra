// FILE-CONTEXT
// STAND: 2026-10-28T00:00:00Z
// ZWECK: Read-only memory mapping facade over memmap2 (Ring 0 / SPEC 7.7)
// INVARIANTEN: Zero panic; typed io::Error propagation; read-only mapping
// SIEHE AUCH: AGENTS.md, docs/SPEC.md §7.7

//! # Modul: Nur-Lese-Speicherabbildung (`mmap`)
//!
//! Dieses Modul stellt eine sichere Fassade für speicherabgebildete Dateien (Memory-Mapped I/O)
//! bereit, basierend auf der Crate `memmap2`.
//!
//! ## Ring-Zugehörigkeit und Verwendung
//! - **Ring 0 (Kern / Unsafe-Insel):** Kapselt den `unsafe`-Aufruf `memmap2::Mmap::map(file)`.
//! - **Verwendung im System:** Wird von `contextra-vector` (DiskANN-Vektorindizes) und
//!   `contextra-store` (SSTables) verwendet, um große Datenmengen extrem speichereffizient
//!   direkt aus dem Betriebssystem-Seitencache (Page Cache) ohne Heap-Kopien (Zero-Copy) zu lesen.
//!
//! ## Sicherheitsinvariante: Striktes Read-Only
//! Da mehrere Threads oder Prozesse gleichzeitig auf die Speicherabbildung zugreifen können,
//! wird ausschließlich eine unveränderliche Speicherabbildung ([`memmap2::Mmap`]) erzeugt.
//! Schreibende Speicherabbildungen (`MmapMut`) sind hier bewusst ausgeschlossen, um
//! Data Races (Datenwettläufe) im Rust-Adressraum auszuschließen.

use std::fs::File;
use std::io;

// ANCHOR[contextra-sys:MMAP-READONLY-001]
/// Erstellt eine unveränderliche Nur-Lese-Speicherabbildung (Read-Only Memory Map) für eine geöffnete Datei.
///
/// # Ablauf
/// 1. Übergibt die geöffnete Dateireferenz `file` an `memmap2::Mmap::map`.
/// 2. Das Betriebssystem richtet über seine Paging-Tabellen virtuelle Adressen ein,
///    die direkt auf die Seiten im Dateisystemcache verweisen.
/// 3. Gibt bei Erfolg eine typisierte [`memmap2::Mmap`]-Struktur zurück, die wie ein `&[u8]`-Slice dereferenziert werden kann.
///
/// # Vorbedingungen
/// - `file` muss eine gültige, geöffnete Datei mit Leserechten referenzieren.
///
/// # Nachbedingungen
/// - Die Speicherabbildung bleibt gültig, solange die zurückgegebene Struktur existiert.
/// - Zugriffe auf den Speicherbereich erfolgen Zero-Copy direkt aus dem Betriebssystem-Cache.
///
/// # Fehlerfälle
/// Gibt einen [`std::io::Error`] zurück, wenn das Speicherabbilden durch das Betriebssystem
/// fehlschlägt (z. B. unzureichender virtueller Adressraum oder Berechtigungsfehler).
///
/// # Safety
/// Die Invariante wird hergestellt, da die Datei nur lesend eingebunden wird (`Mmap`, nicht `MmapMut`).
/// Modifikationen im Adressraum sind typtechnisch unmöglich.
///
/// # Beispiel
/// ```no_run
/// use std::fs::File;
/// use contextra_sys::mmap_readonly;
///
/// # fn run() -> std::io::Result<()> {
/// let file = File::open("vector_index.bin")?;
/// let mmap = mmap_readonly(&file)?;
/// println!("Dateigröße im Speicher: {} Bytes", mmap.len());
/// # Ok(())
/// # }
/// ```
pub fn mmap_readonly(file: &File) -> io::Result<memmap2::Mmap> {
    // SAFETY: `file` ist ein gültiger, geöffneter Read-Only-Dateideskriptor. Die Read-Only-Abbildung verhindert Data Races im Rust-Adressraum. Seitenfehler (Page Faults) werden transparent vom Betriebssystemkernel behandelt.
    unsafe { memmap2::Mmap::map(file) }
}

// ANCHOR[contextra-sys:MMAP-TESTS-001]
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    // ANCHOR[contextra-sys:MMAP-TEST-READ-001]
    // Prüft das Einlesen von Daten über eine Nur-Lese-Speicherabbildung und vergleicht den Inhalt exakt.
    #[test]
    fn test_mmap_readonly() -> Result<(), Box<dyn std::error::Error>> {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "contextra_sys_mmap_test_{}.bin",
            std::process::id()
        ));
        {
            let mut file = File::create(&path)?;
            file.write_all(b"contextra mmap readonly test payload")?;
        }
        let file = File::open(&path)?;
        let mmap = mmap_readonly(&file)?;
        assert_eq!(&mmap[..], b"contextra mmap readonly test payload");
        let _ = std::fs::remove_file(path);
        Ok(())
    }

    // ANCHOR[contextra-sys:MMAP-TEST-EMPTY-001]
    // Prüft das Abbilden einer leeren Datei (0 Bytes) auf eine leere Speicherabbildung.
    #[test]
    fn test_mmap_readonly_empty_file() -> Result<(), Box<dyn std::error::Error>> {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "contextra_sys_mmap_empty_{}.bin",
            std::process::id()
        ));
        {
            File::create(&path)?;
        }
        let file = File::open(&path)?;
        let mmap = mmap_readonly(&file)?;
        assert!(mmap.is_empty());
        let _ = std::fs::remove_file(path);
        Ok(())
    }

    // ANCHOR[contextra-sys:MMAP-TEST-LARGE-001]
    // Prüft das Abbilden einer größeren Datei (1 MiB), um mehrseitiges Paging zu verifizieren.
    #[test]
    fn test_mmap_readonly_large_file() -> Result<(), Box<dyn std::error::Error>> {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "contextra_sys_mmap_large_{}.bin",
            std::process::id()
        ));
        let payload = vec![0xABu8; 1024 * 1024]; // 1 MiB
        {
            let mut file = File::create(&path)?;
            file.write_all(&payload)?;
        }
        let file = File::open(&path)?;
        let mmap = mmap_readonly(&file)?;
        assert_eq!(mmap.len(), 1024 * 1024);
        assert_eq!(&mmap[..], &payload[..]);
        let _ = std::fs::remove_file(path);
        Ok(())
    }

    // ANCHOR[contextra-sys:MMAP-TEST-BOUNDS-001]
    // Prüft Zugriffsgrenzen (`get` mit Index/Slice) auf der erstellten Speicherabbildung.
    #[test]
    fn test_mmap_readonly_access_boundary() -> Result<(), Box<dyn std::error::Error>> {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "contextra_sys_mmap_bound_{}.bin",
            std::process::id()
        ));
        {
            let mut file = File::create(&path)?;
            file.write_all(b"short payload")?;
        }
        let file = File::open(&path)?;
        let mmap = mmap_readonly(&file)?;
        assert_eq!(mmap.get(..13), Some(&b"short payload"[..]));
        assert_eq!(mmap.get(13), None);
        let _ = std::fs::remove_file(path);
        Ok(())
    }
}

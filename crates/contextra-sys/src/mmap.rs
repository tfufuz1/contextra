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
///
/// Das Einbinden einer Datei in den Adressraum (`mmap`) ist inhärent unsicher (UB-Risiko),
/// da das Betriebssystem den Speicherbereich außerhalb der Kontrolle des Rust-Compilers verwaltet:
/// - **Dateimodifikation:** Modifiziert ein anderer Prozess oder Thread die unterliegende Datei schreibend
///   während der Lebensdauer des Mappings, kann dies zu undefiniertem Verhalten (UB / Data Races) führen.
/// - **Dateikürzung (Truncation):** Wird die Datei gekürzt, während das Mapping aktiv ist, führt ein
///   späterer Lesezugriff auf abgeschnittene Seiten zu einem Hardware-Signal (`SIGBUS` unter POSIX bzw.
///   Access Violation unter Windows), das den Prozess unweigerlich abstürzen lässt.
/// - **Vorbedingung des Aufrufers:** Der Aufrufer muss sicherstellen, dass die Datei während der gesamten
///   Gültigkeitsdauer der zurückgegebenen [`memmap2::Mmap`]-Instanz weder von außen gekürzt noch
///   inkonsistent modifiziert wird.
///
/// # Beispiel
/// ```no_run
/// use std::fs::File;
/// use contextra_sys::mmap_readonly;
///
/// # fn run() -> std::io::Result<()> {
/// let file = File::open("vector_index.bin")?;
/// // SAFETY: Die Datei wird exklusiv gelesen und während des Zugriffs nicht gekürzt oder verändert.
/// let mmap = unsafe { mmap_readonly(&file)? };
/// println!("Dateigröße im Speicher: {} Bytes", mmap.len());
/// # Ok(())
/// # }
/// ```
pub unsafe fn mmap_readonly(file: &File) -> io::Result<memmap2::Mmap> {
    // SAFETY: `file` referenziert eine geöffnete Datei. Der Aufrufer garantiert über den `# Safety`-Vertrag, dass die Datei nicht gekürzt oder nebenläufig mutiert wird. Das Mapping erfolgt strikt schreibgeschützt via `memmap2::Mmap::map`.
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
        // SAFETY: Die Datei ist temporär und wird während des Tests von keinem anderen Prozess modifiziert oder gekürzt.
        let mmap = unsafe { mmap_readonly(&file)? };
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
        // SAFETY: Leere Datei bleibt während des Mappings unverändert.
        let mmap = unsafe { mmap_readonly(&file)? };
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
        // SAFETY: 1-MiB-Testdatei wird während des Mappings von keinem Prozess mutiert.
        let mmap = unsafe { mmap_readonly(&file)? };
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
        // SAFETY: Testdatei wird während des Mappings nicht gekürzt.
        let mmap = unsafe { mmap_readonly(&file)? };
        assert_eq!(mmap.get(..13), Some(&b"short payload"[..]));
        assert_eq!(mmap.get(13), None);
        let _ = std::fs::remove_file(path);
        Ok(())
    }
}

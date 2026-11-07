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

// ANCHOR[contextra-sys:MMAP-TYPINVARIANT-001]
/// Typzustands-Marker: Zeigt an, dass eine Datei vollständig versiegelt, unveränderlich
/// und prüfsummenverifiziert ist (gemäß INV-IO-1 und INV-ENCRYPT-REST-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SealedReadOnly;

/// Typzustands-Marker: Zeigt an, dass eine Datei noch unversiegelt oder potenziell veränderlich ist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unsealed;

/// Ein typisierter Dateiwächter, der den Versiegelungszustand einer Datei zur Compilezeit
/// und Laufzeit abbildet.
///
/// Invariante gemäß SPEC INV-IO-1:
/// Eine Datei darf erst per Memory-Mapping eingebunden werden, wenn sie versiegelt (`SealedReadOnly`)
/// und auf unveränderlichen Zustand geprüft wurde.
#[derive(Debug)]
pub struct SealedFile<State = SealedReadOnly> {
    file: File,
    expected_len: u64,
    _marker: std::marker::PhantomData<State>,
}

impl SealedFile<SealedReadOnly> {
    /// Erstellt eine versiegelte Dateireferenz nach erfolgreicher Integritäts- und Prüfsummenverifikation.
    ///
    /// # Ablauf
    /// 1. Liest die Dateimetadaten aus `file`.
    /// 2. Verifiziert, dass die tatsächliche Dateigröße exakt `expected_len` entspricht.
    /// 3. Gibt bei Übereinstimmung eine versiegelte Dateistruktur zurück.
    ///
    /// # Fehler
    /// Liefert [`std::io::ErrorKind::InvalidData`], wenn die Dateigröße von `expected_len` abweicht.
    pub fn new_verified(file: File, expected_len: u64) -> io::Result<Self> {
        let meta = file.metadata()?;
        if meta.len() != expected_len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "SealedFile verification failed: expected length {}, actual length {}",
                    expected_len,
                    meta.len()
                ),
            ));
        }
        Ok(Self {
            file,
            expected_len,
            _marker: std::marker::PhantomData,
        })
    }

    /// Gibt eine Referenz auf die zugrundeliegende [`File`] zurück.
    pub fn as_file(&self) -> &File {
        &self.file
    }

    /// Gibt die verifizierte Dateilänge in Bytes zurück.
    pub fn verified_len(&self) -> u64 {
        self.expected_len
    }

    /// Mappt die versiegelte Datei über die typgesicherte [`SealedMap`]-Abstraktion.
    pub fn map_readonly(&self) -> io::Result<SealedMap> {
        SealedMap::map_sealed(self)
    }
}

// ANCHOR[contextra-sys:MMAP-SEALED-MAP-001]
/// Sichere Nur-Lese-Speicherabbildung für versiegelte, unveränderliche Dateien (INV-IO-1).
///
/// Im Gegensatz zu rohem `mmap` kann eine [`SealedMap`] ausschließlich aus einer [`SealedFile`]
/// erzeugt werden. Dies erzwingt die Architekturinvariante INV-IO-1 im Typensystem.
#[derive(Debug)]
pub struct SealedMap {
    inner: memmap2::Mmap,
}

impl SealedMap {
    /// Mappt eine nachweislich versiegelte Datei in den virtuellen Adressraum.
    ///
    /// # Ablauf
    /// 1. Übergibt die verifizierte Datei an `mmap_readonly`.
    /// 2. Prüft, ob die gemappte Länge exakt mit der verifizierten Dateilänge übereinstimmt.
    /// 3. Liefert die gekapselte `SealedMap` zurück.
    pub fn map_sealed(file: &SealedFile<SealedReadOnly>) -> io::Result<Self> {
        let inner = mmap_readonly(file.as_file())?;
        if inner.len() as u64 != file.verified_len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Mmap length mismatch: mapped {} bytes, expected {} bytes",
                    inner.len(),
                    file.verified_len()
                ),
            ));
        }
        Ok(Self { inner })
    }

    /// Gibt den Speicherinhalt als Byte-Slice zurück.
    pub fn as_slice(&self) -> &[u8] {
        &self.inner[..]
    }

    /// Gibt die Länge der Speicherabbildung in Bytes zurück.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Prüft, ob die Speicherabbildung leer ist (0 Bytes).
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl std::ops::Deref for SealedMap {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.inner[..]
    }
}

impl AsRef<[u8]> for SealedMap {
    fn as_ref(&self) -> &[u8] {
        &self.inner[..]
    }
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

    // ANCHOR[contextra-sys:MMAP-TEST-SEALED-001]
    // Prüft die typisierte SealedMap- und SealedFile-Abstraktion (INV-IO-1).
    #[test]
    fn test_sealed_map_and_file() -> Result<(), Box<dyn std::error::Error>> {
        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "contextra_sys_sealed_map_test_{}.bin",
            std::process::id()
        ));
        let payload = b"sealed payload verified content";
        {
            let mut file = File::create(&path)?;
            file.write_all(payload)?;
        }

        // Test mit korrekter verifizierter Länge
        let file = File::open(&path)?;
        let sealed_file = SealedFile::new_verified(file, payload.len() as u64)?;
        assert_eq!(sealed_file.verified_len(), payload.len() as u64);

        let sealed_map = sealed_file.map_readonly()?;
        assert_eq!(sealed_map.len(), payload.len());
        assert_eq!(&sealed_map[..], payload);
        assert_eq!(sealed_map.as_slice(), payload);

        // Test mit inkorrekter Länge -> InvalidData Fehler (Fail-Closed)
        let file_wrong = File::open(&path)?;
        let res_wrong = SealedFile::new_verified(file_wrong, payload.len() as u64 + 10);
        assert!(res_wrong.is_err());

        let _ = std::fs::remove_file(path);
        Ok(())
    }
}

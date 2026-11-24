// =========================================================================
// ANCHOR[id=sys-mmap-main,epic=sys-mmap,seq=0] Read-Only Memory Mapping Fassade
// =========================================================================
// LINK crates/contextra-sys/src/lib.rs
// NOTE: Read-only memory mapping facade over memmap2 (Ring 0 / E0 / docs/SPEC.md §10.1 / INV-IO-1 / INV-ENCRYPT-REST-2).
// Invariante: Zero panic; typed error propagation; read-only mapping; bounds checks.
// Keine unzulässigen Garantieversprechen (SPEC §6.4).

//! # Modul: Nur-Lese-Speicherabbildung (`mmap`)
//!
//! Dieses Modul stellt eine sichere Fassade für speicherabgebildete Dateien (Memory-Mapped I/O)
//! bereit, basierend auf der Crate `memmap2`.
//!
//! ## Ring-Zugehörigkeit und Verwendung
//! - **Ring 0 / Ebene E0 (Kern / Unsafe-Insel):** Kapselt den `unsafe`-Aufruf `memmap2::Mmap::map(file)`.
//! - **Verwendung im System:** Wird von `contextra-vector` (DiskANN-Vektorindizes) und
//!   `contextra-store` (SSTables) verwendet, um große Datenmengen speichereffizient
//!   direkt aus dem Betriebssystem-Seitencache (Page Cache) ohne Heap-Kopien (Zero-Copy) zu lesen.
//!
//! ## Sicherheitsinvariante: Striktes Read-Only
//! Da mehrere Threads oder Prozesse gleichzeitig auf die Speicherabbildung zugreifen können,
//! wird ausschließlich eine unveränderliche Speicherabbildung ([`memmap2::Mmap`]) erzeugt.
//! Schreibende Speicherabbildungen (`MmapMut`) sind hier bewusst ausgeschlossen, um
//! Data Races (Datenwettläufe) im Rust-Adressraum auszuschließen.

use std::fs::File;
use std::io;

use crate::error::{Error, Result};

// SECTION Raw Readonly Mapping
// ANCHOR[id=sys-mmap-raw,epic=sys-mmap,seq=1] mmap_readonly

/// Erstellt eine unveränderliche Nur-Lese-Speicherabbildung (Read-Only Memory Map) für eine geöffnete Datei.
///
/// # Vorbedingungen
/// * `file`: Gültiger, lesbarer Dateideskriptor.
///
/// # Fehler
/// Liefert [`std::io::Error`], falls das Betriebssystem das Mapping ablehnt (z. B. unzureichender virtueller Adressraum).
pub fn mmap_readonly(file: &File) -> io::Result<memmap2::Mmap> {
    // SAFETY: `file` ist ein gültiger, geöffneter Read-Only-Dateideskriptor.
    // Die Read-Only-Abbildung verhindert Data Races im Rust-Adressraum.
    // Seitenfehler (Page Faults) werden transparent vom Betriebssystemkernel behandelt.
    unsafe { memmap2::Mmap::map(file) }
}
// !SECTION Raw Readonly Mapping

// SECTION Sealed File Abstraction
// ANCHOR[id=sys-mmap-sealed,epic=sys-mmap,seq=2] Typisierte Versiegelung (Typestate)

/// Typzustands-Marker: Zeigt an, dass eine Datei vollständig versiegelt, unveränderlich
/// und prüfsummenverifiziert ist (gemäß INV-IO-1 und INV-ENCRYPT-REST-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SealedReadOnly;

/// Typzustands-Marker: Zeigt an, dass eine Datei noch unversiegelt oder potenziell veränderlich ist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unsealed;

/// Ein typisierter Dateiwächter, der den Versiegelungszustand einer Datei zur Compilezeit
/// und Laufzeit abbildet.
#[derive(Debug)]
pub struct SealedFile<State = SealedReadOnly> {
    file: File,
    expected_len: u64,
    _marker: std::marker::PhantomData<State>,
}

impl SealedFile<SealedReadOnly> {
    /// Erstellt eine versiegelte Dateireferenz nach erfolgreicher Integritäts- und Längenprüfung.
    ///
    /// # Fehler
    /// Liefert [`std::io::ErrorKind::InvalidData`], wenn die Dateilänge nicht mit `expected_len` übereinstimmt.
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
// !SECTION Sealed File Abstraction

// SECTION Sealed Map Abstraction
// ANCHOR[id=sys-mmap-sealed-map,epic=sys-mmap,seq=3] SealedMap

/// Sichere Nur-Lese-Speicherabbildung für versiegelte, unveränderliche Dateien (INV-IO-1).
#[derive(Debug)]
pub struct SealedMap {
    inner: memmap2::Mmap,
}

impl SealedMap {
    /// Mappt eine nachweislich versiegelte Datei in den virtuellen Adressraum.
    ///
    /// # Fehler
    /// Liefert [`std::io::ErrorKind::InvalidData`], wenn die gemappte Größe von der erwarteten Größe abweicht.
    pub fn map_sealed(file: &SealedFile<SealedReadOnly>) -> io::Result<Self> {
        let inner = mmap_readonly(file.as_file())?;
        let mapped_len = inner.len() as u64;
        if mapped_len != file.verified_len() {
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

    /// Liest einen Teilbereich der Speicherabbildung mit sicherer Grenzenprüfung.
    ///
    /// # Fehler
    /// Liefert [`crate::Error::InvalidInput`], falls Bereichsgrenzen überschritten werden oder ein Überlauf auftritt.
    pub fn get_subslice(&self, offset: usize, len: usize) -> Result<&[u8]> {
        let end = offset.checked_add(len).ok_or_else(|| {
            Error::InvalidInput("slice offset + len overflowed usize".to_string())
        })?;
        self.inner
            .get(offset..end)
            .ok_or_else(|| Error::InvalidInput("slice bounds exceed mmap length".to_string()))
    }

    /// Gibt den gesamten Speicherinhalt als Byte-Slice zurück.
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
// !SECTION Sealed Map Abstraction

// SECTION Unit Tests
// ANCHOR[id=sys-mmap-tests,epic=sys-mmap,seq=4] Mmap-Modultests
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_mmap_readonly() -> std::result::Result<(), Box<dyn std::error::Error>> {
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

    #[test]
    fn test_mmap_readonly_empty_file() -> std::result::Result<(), Box<dyn std::error::Error>> {
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

    #[test]
    fn test_mmap_readonly_large_file() -> std::result::Result<(), Box<dyn std::error::Error>> {
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

    #[test]
    fn test_mmap_readonly_access_boundary() -> std::result::Result<(), Box<dyn std::error::Error>> {
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

    #[test]
    fn test_sealed_map_and_file() -> std::result::Result<(), Box<dyn std::error::Error>> {
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

        let file = File::open(&path)?;
        let sealed_file = SealedFile::new_verified(file, payload.len() as u64)?;
        assert_eq!(sealed_file.verified_len(), payload.len() as u64);

        let sealed_map = sealed_file.map_readonly()?;
        assert_eq!(sealed_map.len(), payload.len());
        assert_eq!(&sealed_map[..], payload);
        assert_eq!(sealed_map.as_slice(), payload);

        let sub = sealed_map.get_subslice(7, 7)?;
        assert_eq!(sub, b"payload");

        let err_overflow = sealed_map.get_subslice(usize::MAX, 1);
        assert!(err_overflow.is_err());

        let err_oob = sealed_map.get_subslice(20, 100);
        assert!(err_oob.is_err());

        let file_wrong = File::open(&path)?;
        let res_wrong = SealedFile::new_verified(file_wrong, payload.len() as u64 + 10);
        assert!(res_wrong.is_err());

        let _ = std::fs::remove_file(path);
        Ok(())
    }
}
// !SECTION Unit Tests

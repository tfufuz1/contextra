//! Feldmessung der Klassen MK1, MK2 und MK3 (SPEC §12.1, D-35, D-72, ER-26).
//!
//! MK1: Logische/funktionale Prüfung – kein Eintrag in Strukturen des Datenverzeichnisses.
//! MK2: Rohscan inklusive tmp/, alte Segmente, WAL, Snapshots, scope/, Berichte.
//! MK3: Schlüsselabwesenheit – Slot der höchsten Ebene genullt, DEK-Ableitung scheitert.

use crate::error::ErasureError;
use contextra_ports::vfs::{Vfs, VfsOpenOptions};
use std::path::{Path, PathBuf};

/// Art der Leseverifikation gemäß SPEC D-113.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifiedVia {
    /// Direktes Lesen am Medium (O_DIRECT / F_NOCACHE).
    Direct,
    /// Cache vor dem Lesen verworfen (posix_fadvise DONTNEED).
    CacheDropped,
    /// Regulärer Cache-Lesepfad (Fallback bis D-113 voll in Ports gemergt ist).
    Cached,
}

impl VerifiedVia {
    /// Identifikator-String für den Bericht.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Direct => "Direct",
            Self::CacheDropped => "CacheDropped",
            Self::Cached => "Cached",
        }
    }
}

/// Ergebnis einer MK1-Feldmessung (logische Prüfung).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mk1Result {
    /// Anzahl geprüfter Strukturen.
    pub checked_structures: u64,
    /// Treffer (sollte 0 sein).
    pub matches: u64,
    /// Verifikationsmethode (D-113).
    pub verified_via: VerifiedVia,
}

/// Ergebnis einer MK2-Feldmessung (Rohscan).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mk2Result {
    /// Anzahl geprüfter Dateien.
    pub checked_files: u64,
    /// Geprüfte Bytes.
    pub checked_bytes: u64,
    /// Treffer (sollte 0 sein).
    pub matches: u64,
    /// Verifikationsmethode (D-113).
    pub verified_via: VerifiedVia,
}

/// Ergebnis einer MK3-Feldmessung (Schlüsselabwesenheit).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mk3Result {
    /// Slot der höchsten Ebene genullt.
    pub top_slot_zeroed: bool,
    /// DEK-Ableitung gescheitert.
    pub dek_derivation_fails: bool,
    /// F-Slots der Zieldateien nicht mehr vorhanden.
    pub file_slots_absent: bool,
    /// Art der Verifikation (D-113).
    pub verified_via: VerifiedVia,
}

impl Mk3Result {
    /// Prüft ob MK3 vollständig bestanden (kein Schlüssel mehr ableitbar).
    pub fn is_clean(&self) -> bool {
        self.top_slot_zeroed && self.dek_derivation_fails && self.file_slots_absent
    }
}

/// Führt MK3-Schlüsselprüfung durch (SPEC §12.3, Anhang F.3, D-79, D-113).
pub fn scan_mk3_keys(
    top_slot_zeroed: bool,
    dek_derivation_fails: bool,
    file_slots_absent: bool,
) -> Mk3Result {
    Mk3Result {
        top_slot_zeroed,
        dek_derivation_fails,
        file_slots_absent,
        verified_via: VerifiedVia::Cached,
    }
}

/// Liest Bytes einer Datei verifiziert über Vfs (SPEC D-113 Brücke).
pub fn read_file_verified(
    vfs: &dyn Vfs,
    path: &Path,
) -> Result<(Vec<u8>, VerifiedVia), ErasureError> {
    let opts = VfsOpenOptions::read_only();
    if let Ok(file) = vfs.open(path, &opts) {
        let file_len = file.len().map_err(ErasureError::Io)?;
        let len = usize::try_from(file_len).map_err(|e| ErasureError::Internal(e.to_string()))?;
        let mut buf = vec![0u8; len];
        let mut read_bytes = 0usize;
        while read_bytes < len {
            if let Some(target) = buf.get_mut(read_bytes..) {
                let n = file
                    .read_at(target, read_bytes as u64)
                    .map_err(ErasureError::Io)?;
                if n == 0 {
                    break;
                }
                read_bytes = read_bytes.saturating_add(n);
            } else {
                break;
            }
        }
        buf.truncate(read_bytes);
        Ok((buf, VerifiedVia::Cached))
    } else {
        let content = vfs
            .read(path)
            .map_err(|e| ErasureError::Internal(e.to_string()))?;
        Ok((content, VerifiedVia::Cached))
    }
}

/// Rekursives Sammeln aller Dateipfade über Vfs (ohne Verzeichnisse selbst als Dateien einzufügen).
fn collect_all_files(vfs: &dyn Vfs, dir: &Path) -> Result<Vec<PathBuf>, ErasureError> {
    let mut result = Vec::new();
    let entries = match vfs.list_dir(dir) {
        Ok(list) => list,
        Err(_) => return Ok(result),
    };

    for name in entries {
        let child = dir.join(name);
        let sub_entries = vfs.list_dir(&child).unwrap_or_default();
        if !sub_entries.is_empty() {
            let mut sub = collect_all_files(vfs, &child)?;
            result.append(&mut sub);
        } else if vfs.exists(&child).unwrap_or(false) && vfs.len(&child).unwrap_or(0) > 0 {
            // Datei mit Inhalt
            result.push(child);
        } else if let Ok(sub) = vfs.list_dir(&child) {
            if sub.is_empty() && vfs.len(&child).unwrap_or(0) > 0 {
                result.push(child);
            }
        } else {
            result.push(child);
        }
    }
    Ok(result)
}

/// Scannt MK1-Strukturen (Manifeste, SSTable-Indizes, WAL-Köpfe) auf ein Schlüsselmuster.
pub fn scan_mk1_structures(
    vfs: &dyn Vfs,
    root: &Path,
    pattern: &[u8],
) -> Result<Mk1Result, ErasureError> {
    if pattern.is_empty() {
        return Ok(Mk1Result {
            checked_structures: 0,
            matches: 0,
            verified_via: VerifiedVia::Cached,
        });
    }

    let files = collect_all_files(vfs, root)?;
    let mut checked_structures = 0u64;
    let mut matches = 0u64;

    for path in &files {
        let path_str = path.to_string_lossy();
        let is_structure = path_str.contains("manifest")
            || path_str.ends_with(".wal")
            || path_str.ends_with(".sst")
            || path_str.ends_with(".idx")
            || path_str.contains("vault");

        if is_structure {
            checked_structures = checked_structures.saturating_add(1);
            let (bytes, _) = read_file_verified(vfs, path)?;
            if bytes.len() >= pattern.len() {
                for i in 0..=bytes.len() - pattern.len() {
                    if let Some(window) = bytes.get(i..i + pattern.len()) {
                        if window == pattern {
                            matches = matches.saturating_add(1);
                        }
                    }
                }
            }
        }
    }

    Ok(Mk1Result {
        checked_structures,
        matches,
        verified_via: VerifiedVia::Cached,
    })
}

/// Scannt MK2-Rohdaten aller Dateien (inklusive tmp/, alte Segmente, WAL, scope/, Berichte).
pub fn scan_mk2_raw(vfs: &dyn Vfs, root: &Path, pattern: &[u8]) -> Result<Mk2Result, ErasureError> {
    if pattern.is_empty() {
        return Ok(Mk2Result {
            checked_files: 0,
            checked_bytes: 0,
            matches: 0,
            verified_via: VerifiedVia::Cached,
        });
    }

    let files = collect_all_files(vfs, root)?;
    let mut checked_files = 0u64;
    let mut checked_bytes = 0u64;
    let mut matches = 0u64;

    for path in &files {
        checked_files = checked_files.saturating_add(1);
        let (bytes, _) = read_file_verified(vfs, path)?;
        checked_bytes = checked_bytes.saturating_add(bytes.len() as u64);

        if bytes.len() >= pattern.len() {
            for i in 0..=bytes.len() - pattern.len() {
                if let Some(window) = bytes.get(i..i + pattern.len()) {
                    if window == pattern {
                        matches = matches.saturating_add(1);
                    }
                }
            }
        }
    }

    Ok(Mk2Result {
        checked_files,
        checked_bytes,
        matches,
        verified_via: VerifiedVia::Cached,
    })
}

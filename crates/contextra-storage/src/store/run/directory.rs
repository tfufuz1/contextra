//! # DirectoryBlock: Inhaltsverzeichnis aller Metadaten- und Index-Blöcke
//!
//! Format (SPEC §9.5):
//! `count varint | (kind u16 | offset u64 | len u32 | required u8) × count`
//! Unbekannte optionale Blöcke (`required = 0`) werden ignoriert,
//! unbekannte Pflichtblöcke (`required = 1`) verweigern das Öffnen.
//! Vor jedem Zugriff werden Offset und Länge gegen Dateilänge und Überlappung geprüft.
//! Keine Integer-Casts (M-02 = 0).

use super::block::block_kind;
use super::error::RunError;
use crate::codec::varint::{decode_u64, encode_u64};

/// Ein Eintrag im Inhaltsverzeichnis eines Runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectoryEntry {
    /// Art des Blocks (z. B. Index=1, Filter=2, Meta=5, Extension=0x1000+id).
    pub kind: u16,
    /// Byte-Offset des Blocks in der Run-Datei.
    pub offset: u64,
    /// Physische Länge des Blocks inklusive Rahmen und Prüfsumme.
    pub len: u32,
    /// Pflicht-Flag: 1 verlangt Unterstützung, 0 ist optional.
    pub required: u8,
}

impl DirectoryEntry {
    /// Feste Länge eines Verzeichniseintrags in Bytes (2 + 8 + 4 + 1 = 15).
    pub const ENTRY_LEN: usize = 15;

    /// Erstellt einen neuen Verzeichniseintrag.
    pub const fn new(kind: u16, offset: u64, len: u32, required: u8) -> Self {
        Self {
            kind,
            offset,
            len,
            required,
        }
    }

    /// Kodiert den Eintrag in ein Byte-Array.
    pub fn encode(&self) -> [u8; Self::ENTRY_LEN] {
        let mut out = [0u8; Self::ENTRY_LEN];
        let k = self.kind.to_le_bytes();
        let off = self.offset.to_le_bytes();
        let l = self.len.to_le_bytes();

        if let Some(s) = out.get_mut(0..2) {
            s.copy_from_slice(&k);
        }
        if let Some(s) = out.get_mut(2..10) {
            s.copy_from_slice(&off);
        }
        if let Some(s) = out.get_mut(10..14) {
            s.copy_from_slice(&l);
        }
        if let Some(b) = out.get_mut(14) {
            *b = self.required;
        }
        out
    }

    /// Dekodiert einen Eintrag aus einem Byte-Ausschnitt.
    pub fn decode(bytes: &[u8]) -> Result<Self, RunError> {
        let slice = bytes
            .get(0..Self::ENTRY_LEN)
            .ok_or_else(|| RunError::Corrupt("Directory-Eintrag zu kurz (< 15 B)".into()))?;

        let mut k_bytes = [0u8; 2];
        let mut off_bytes = [0u8; 8];
        let mut l_bytes = [0u8; 4];

        if let Some(s) = slice.get(0..2) {
            k_bytes.copy_from_slice(s);
        }
        if let Some(s) = slice.get(2..10) {
            off_bytes.copy_from_slice(s);
        }
        if let Some(s) = slice.get(10..14) {
            l_bytes.copy_from_slice(s);
        }
        let required = *slice
            .get(14)
            .ok_or_else(|| RunError::Corrupt("Required-Flag fehlt".into()))?;

        Ok(Self {
            kind: u16::from_le_bytes(k_bytes),
            offset: u64::from_le_bytes(off_bytes),
            len: u32::from_le_bytes(l_bytes),
            required,
        })
    }
}

/// Das vollständige Inhaltsverzeichnis eines Runs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DirectoryBlock {
    /// Liste aller enthaltenen Blöcke.
    pub entries: Vec<DirectoryEntry>,
}

impl DirectoryBlock {
    /// Erstellt ein neues leeres Inhaltsverzeichnis.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Fügt einen neuen Block in das Verzeichnis ein.
    pub fn add(&mut self, kind: u16, offset: u64, len: u32, required: u8) {
        self.entries.push(DirectoryEntry::new(kind, offset, len, required));
    }

    /// Sucht einen Eintrag anhand seiner Block-Art.
    pub fn find(&self, kind: u16) -> Option<&DirectoryEntry> {
        self.entries.iter().find(|e| e.kind == kind)
    }

    /// Kodiert den Directory-Block in ein Byte-Vektor.
    pub fn encode(&self) -> Vec<u8> {
        let count_u64 = u64::try_from(self.entries.len()).unwrap_or(0);
        let mut var_buf = [0u8; 10];
        let var_len = encode_u64(count_u64, &mut var_buf);

        let cap = var_len.saturating_add(
            self.entries
                .len()
                .saturating_mul(DirectoryEntry::ENTRY_LEN),
        );
        let mut out = Vec::with_capacity(cap);
        if let Some(s) = var_buf.get(..var_len) {
            out.extend_from_slice(s);
        }
        for entry in &self.entries {
            out.extend_from_slice(&entry.encode());
        }
        out
    }

    /// Dekodiert und validiert das Verzeichnis gegen Dateilänge und Überlappungen.
    pub fn decode(bytes: &[u8], file_len: u64) -> Result<Self, RunError> {
        let (count_u64, consumed) = decode_u64(bytes)
            .map_err(|e| RunError::Corrupt(format!("Directory-Count varint ungültig: {e}")))?;

        let count = usize::try_from(count_u64)
            .map_err(|_| RunError::Limit("Directory-Count übersteigt Adressraum".into()))?;

        let mut offset = consumed;
        let mut entries = Vec::with_capacity(count);

        for _ in 0..count {
            let next_offset = offset
                .checked_add(DirectoryEntry::ENTRY_LEN)
                .ok_or_else(|| RunError::Limit("Offset-Überlauf beim Lesen des Directory".into()))?;

            let entry_bytes = bytes
                .get(offset..next_offset)
                .ok_or_else(|| RunError::Corrupt("Directory-Daten unerwartet kurz".into()))?;

            let entry = DirectoryEntry::decode(entry_bytes)?;
            entries.push(entry);
            offset = next_offset;
        }

        let dir = Self { entries };
        dir.validate(file_len)?;
        Ok(dir)
    }

    /// Validiert alle Einträge: Grenzen innerhalb der Datei und Überlappungsfreiheit.
    pub fn validate(&self, file_len: u64) -> Result<(), RunError> {
        let mut intervals: Vec<(u64, u64, u16, u8)> = Vec::with_capacity(self.entries.len());

        for entry in &self.entries {
            let len_u64 = u64::from(entry.len);
            let end = entry
                .offset
                .checked_add(len_u64)
                .ok_or_else(|| RunError::Corrupt("Block-Offset überläuft u64".into()))?;

            if end > file_len {
                return Err(RunError::Corrupt(format!(
                    "Block kind {} ragt über Dateiende hinaus: end {end} > file_len {file_len}",
                    entry.kind
                )));
            }

            intervals.push((entry.offset, end, entry.kind, entry.required));
        }

        // Sortieren nach Start-Offset
        intervals.sort_by_key(|&(start, _, _, _)| start);

        let mut prev_end: u64 = 0;
        for (idx, &(start, end, kind, _)) in intervals.iter().enumerate() {
            if idx > 0 && start < prev_end {
                return Err(RunError::Corrupt(format!(
                    "Block kind {kind} überlappt mit vorherigem Block (start {start} < prev_end {prev_end})"
                )));
            }
            prev_end = end;
        }

        Ok(())
    }

    /// Prüft, ob alle Pflichtblöcke (`required = 1`) vom Reader unterstützt werden.
    pub fn check_required(&self, supported_kinds: &[u16]) -> Result<(), RunError> {
        for entry in &self.entries {
            if entry.required == 1 && !supported_kinds.contains(&entry.kind) {
                return Err(RunError::Unsupported(format!(
                    "Unbekannter Pflichtblock im Run: kind {}",
                    entry.kind
                )));
            }
        }
        Ok(())
    }
}

/// Standardmäßig vom Kern unterstützte Block-Arten.
pub fn is_supported_kind(k: u16) -> bool {
    matches!(
        k,
        block_kind::INDEX
            | block_kind::FILTER
            | block_kind::PREFIX_FILTER
            | block_kind::RANGE_DEL
            | block_kind::META
    ) || k >= block_kind::EXTENSION_BASE
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn test_directory_roundtrip_and_validation() {
        let mut dir = DirectoryBlock::new();
        dir.add(block_kind::INDEX, 100, 50, 1);
        dir.add(block_kind::META, 150, 30, 1);
        dir.add(block_kind::FILTER, 50, 40, 0);

        let enc = dir.encode();
        let dec = DirectoryBlock::decode(&enc, 200).unwrap_or_default();
        assert_eq!(dec.entries.len(), 3);

        assert_eq!(dec.find(block_kind::INDEX).map(|e| e.offset), Some(100));
        assert_eq!(dec.find(block_kind::META).map(|e| e.offset), Some(150));
        assert_eq!(dec.find(block_kind::FILTER).map(|e| e.offset), Some(50));
    }

    #[test]
    fn test_directory_overlap_detected() {
        let mut dir = DirectoryBlock::new();
        dir.add(block_kind::INDEX, 100, 60, 1);
        dir.add(block_kind::META, 150, 30, 1); // 150 < 100 + 60 = 160 -> Überlappung!

        let enc = dir.encode();
        assert!(DirectoryBlock::decode(&enc, 300).is_err());
    }

    #[test]
    fn test_directory_past_file_len_detected() {
        let mut dir = DirectoryBlock::new();
        dir.add(block_kind::INDEX, 100, 50, 1);

        let enc = dir.encode();
        assert!(DirectoryBlock::decode(&enc, 120).is_err()); // 150 > 120
    }
}

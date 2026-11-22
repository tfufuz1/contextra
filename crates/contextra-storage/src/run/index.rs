//! # IndexBlock: Index über alle Datenblöcke eines Runs
//!
//! Format (SPEC §9.5):
//! `count varint | (sep_len varint | separator | offset u64 | len u32 | entries u32 | weights[k] u64 | max_stamp u64) × count`
//! `weights[0]` sind logische Bytes, `weights[1..k]` stammen vom Weigher.
//! Trenner ist der kürzeste Schlüssel >= letzter Schlüssel des Blocks und < erster des nächsten.
//! Keine Integer-Casts (M-02 = 0).

use super::error::RunError;
use crate::codec::varint::{decode_u64, encode_u64};

/// Ein Eintrag im Indexblock, der einen Datenblock beschreibt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexEntry {
    /// Trennschlüssel für die Bereichsnavigation.
    pub separator: Vec<u8>,
    /// Dateioffset des Datenblocks.
    pub offset: u64,
    /// Gesamtlänge des Datenblocks auf der Platte.
    pub len: u32,
    /// Anzahl der Einträge im Datenblock.
    pub entries: u32,
    /// Gewichte (k Einträge; weights[0] = logische Bytes).
    pub weights: Vec<u64>,
    /// Höchster Commit-Stamp in diesem Datenblock.
    pub max_stamp: u64,
}

impl IndexEntry {
    /// Erstellt einen neuen Index-Eintrag.
    pub fn new(
        separator: Vec<u8>,
        offset: u64,
        len: u32,
        entries: u32,
        weights: Vec<u64>,
        max_stamp: u64,
    ) -> Self {
        Self {
            separator,
            offset,
            len,
            entries,
            weights,
            max_stamp,
        }
    }
}

/// Der vollständige Index-Block eines Runs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IndexBlock {
    /// Liste aller Datenblock-Einträge in Schlüsselreihenfolge.
    pub entries: Vec<IndexEntry>,
}

impl IndexBlock {
    /// Erstellt einen leeren Indexblock.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Fügt einen Datenblock-Eintrag hinzu.
    pub fn add(&mut self, entry: IndexEntry) {
        self.entries.push(entry);
    }

    /// Kodiert den Indexblock nach SPEC §9.5.
    pub fn encode(&self, weights_k: usize) -> Vec<u8> {
        let count_u64 = u64::try_from(self.entries.len()).unwrap_or(0);
        let mut var_buf = [0u8; 10];
        let var_len = encode_u64(count_u64, &mut var_buf);

        let mut out = Vec::new();
        if let Some(s) = var_buf.get(..var_len) {
            out.extend_from_slice(s);
        }

        for entry in &self.entries {
            // sep_len varint | separator
            let sep_len_u64 = u64::try_from(entry.separator.len()).unwrap_or(0);
            let slen = encode_u64(sep_len_u64, &mut var_buf);
            if let Some(s) = var_buf.get(..slen) {
                out.extend_from_slice(s);
            }
            out.extend_from_slice(&entry.separator);

            // offset u64 | len u32 | entries u32
            out.extend_from_slice(&entry.offset.to_le_bytes());
            out.extend_from_slice(&entry.len.to_le_bytes());
            out.extend_from_slice(&entry.entries.to_le_bytes());

            // weights[k] u64
            for i in 0..weights_k {
                let w = entry.weights.get(i).copied().unwrap_or(0);
                out.extend_from_slice(&w.to_le_bytes());
            }

            // max_stamp u64
            out.extend_from_slice(&entry.max_stamp.to_le_bytes());
        }

        out
    }

    /// Dekodiert einen Indexblock.
    pub fn decode(bytes: &[u8], weights_k: usize) -> Result<Self, RunError> {
        let (count_u64, consumed) = decode_u64(bytes)
            .map_err(|e| RunError::Corrupt(format!("Index-Count varint ungültig: {e}")))?;

        let count = usize::try_from(count_u64)
            .map_err(|_| RunError::Limit("Index-Count übersteigt Adressraum".into()))?;

        let mut offset = consumed;
        let mut entries = Vec::with_capacity(count);

        for _ in 0..count {
            let remaining = bytes
                .get(offset..)
                .ok_or_else(|| RunError::Corrupt("Index-Puffer zu kurz".into()))?;
            let (slen_u64, slen_consumed) = decode_u64(remaining)
                .map_err(|e| RunError::Corrupt(format!("sep_len varint ungültig: {e}")))?;
            offset = offset
                .checked_add(slen_consumed)
                .ok_or_else(|| RunError::Limit("Offset-Überlauf".into()))?;

            let slen = usize::try_from(slen_u64)
                .map_err(|_| RunError::Limit("sep_len übersteigt Adressraum".into()))?;
            let sep_end = offset
                .checked_add(slen)
                .ok_or_else(|| RunError::Limit("Offset-Überlauf".into()))?;
            let sep_slice = bytes
                .get(offset..sep_end)
                .ok_or_else(|| RunError::Corrupt("Trenn-Bytes fehlen".into()))?;
            let separator = sep_slice.to_vec();
            offset = sep_end;

            // offset u64 (8 B) + len u32 (4 B) + entries u32 (4 B) = 16 B
            let fixed_end = offset
                .checked_add(16)
                .ok_or_else(|| RunError::Limit("Offset-Überlauf".into()))?;
            let fixed_slice = bytes
                .get(offset..fixed_end)
                .ok_or_else(|| RunError::Corrupt("Index feste Felder fehlen".into()))?;

            let mut off_b = [0u8; 8];
            let mut len_b = [0u8; 4];
            let mut ent_b = [0u8; 4];
            if let Some(s) = fixed_slice.get(0..8) {
                off_b.copy_from_slice(s);
            }
            if let Some(s) = fixed_slice.get(8..12) {
                len_b.copy_from_slice(s);
            }
            if let Some(s) = fixed_slice.get(12..16) {
                ent_b.copy_from_slice(s);
            }

            let block_offset = u64::from_le_bytes(off_b);
            let block_len = u32::from_le_bytes(len_b);
            let block_entries = u32::from_le_bytes(ent_b);
            offset = fixed_end;

            // weights[k] u64
            let mut weights = Vec::with_capacity(weights_k);
            for _ in 0..weights_k {
                let w_end = offset
                    .checked_add(8)
                    .ok_or_else(|| RunError::Limit("Offset-Überlauf".into()))?;
                let w_slice = bytes
                    .get(offset..w_end)
                    .ok_or_else(|| RunError::Corrupt("Gewicht-Bytes fehlen".into()))?;
                let mut w_b = [0u8; 8];
                w_b.copy_from_slice(w_slice);
                weights.push(u64::from_le_bytes(w_b));
                offset = w_end;
            }

            // max_stamp u64
            let stamp_end = offset
                .checked_add(8)
                .ok_or_else(|| RunError::Limit("Offset-Überlauf".into()))?;
            let stamp_slice = bytes
                .get(offset..stamp_end)
                .ok_or_else(|| RunError::Corrupt("max_stamp fehlt".into()))?;
            let mut stamp_b = [0u8; 8];
            stamp_b.copy_from_slice(stamp_slice);
            let max_stamp = u64::from_le_bytes(stamp_b);
            offset = stamp_end;

            entries.push(IndexEntry::new(
                separator,
                block_offset,
                block_len,
                block_entries,
                weights,
                max_stamp,
            ));
        }

        Ok(Self { entries })
    }

    /// Führt eine Binärsuche nach dem ersten Block durch, dessen Trenner >= `key` ist.
    pub fn search(&self, key: &[u8]) -> Option<usize> {
        if self.entries.is_empty() {
            return None;
        }
        let mut low: usize = 0;
        let mut high: usize = self.entries.len();

        while low < high {
            let mid = low.checked_add(high.saturating_sub(low) / 2)?;
            let entry = self.entries.get(mid)?;
            if entry.separator.as_slice() < key {
                low = mid.checked_add(1)?;
            } else {
                high = mid;
            }
        }

        if low < self.entries.len() {
            Some(low)
        } else {
            None
        }
    }
}

/// Erzeugt den kürzesten Trenner >= `last_key` und < `next_first_key`.
pub fn shortest_separator(last_key: &[u8], next_first_key: Option<&[u8]>) -> Vec<u8> {
    match next_first_key {
        None => last_key.to_vec(),
        Some(next) => {
            let min_len = std::cmp::min(last_key.len(), next.len());
            let mut diff_idx: Option<usize> = None;
            for i in 0..min_len {
                if last_key.get(i) != next.get(i) {
                    diff_idx = Some(i);
                    break;
                }
            }

            if let Some(i) = diff_idx {
                let l_byte = last_key.get(i).copied().unwrap_or(0);
                let n_byte = next.get(i).copied().unwrap_or(0);

                if i.saturating_add(1) < last_key.len()
                    && (l_byte.saturating_add(1) < n_byte
                        || (l_byte.saturating_add(1) == n_byte && next.len() > i.saturating_add(1)))
                {
                    let mut sep = Vec::with_capacity(i.saturating_add(1));
                    if let Some(slice) = last_key.get(..i) {
                        sep.extend_from_slice(slice);
                    }
                    sep.push(l_byte.saturating_add(1));
                    return sep;
                }
            }
            last_key.to_vec()
        }
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn test_index_roundtrip_and_search() {
        let mut index = IndexBlock::new();
        index.add(IndexEntry::new(b"d".to_vec(), 0, 100, 10, vec![1000, 5], 50));
        index.add(IndexEntry::new(b"m".to_vec(), 100, 120, 12, vec![1200, 6], 80));
        index.add(IndexEntry::new(b"z".to_vec(), 220, 150, 15, vec![1500, 8], 90));

        let encoded = index.encode(2);
        let decoded = IndexBlock::decode(&encoded, 2).unwrap();

        assert_eq!(decoded.entries.len(), 3);
        assert_eq!(decoded.search(b"b"), Some(0)); // <= "d"
        assert_eq!(decoded.search(b"d"), Some(0));
        assert_eq!(decoded.search(b"e"), Some(1)); // <= "m"
        assert_eq!(decoded.search(b"p"), Some(2)); // <= "z"
        assert_eq!(decoded.search(b"{"), None);   // > "z"
    }

    #[test]
    fn test_shortest_separator() {
        assert_eq!(shortest_separator(b"apple", Some(b"banana")), b"b".to_vec());
        assert_eq!(shortest_separator(b"apple", Some(b"apply")), b"apple".to_vec());
        assert_eq!(shortest_separator(b"xyz", None), b"xyz".to_vec());
    }
}

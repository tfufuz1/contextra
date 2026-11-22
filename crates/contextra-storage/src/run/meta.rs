//! # MetaBlock: Metadaten und Statistiken eines Runs
//!
//! Format in fester Feldreihenfolge (SPEC §9.5):
//! `space_id u16 | run_id u64 | entries u64 | min_stamp u64 | max_stamp u64 |`
//! `range_markers u32 | erase_markers u32 | created_ms u64 | header_len u16 |`
//! `weights_k u8 | weigher_version u32 | feature_bits u64 |`
//! `min_key (len varint + bytes) | max_key (len varint + bytes) |`
//! `blob_count varint | (blob_no u64, bytes u64) × blob_count`
//! `min_key` und `max_key` umfassen auch Bereichsmarker (SPEC §9.5).
//! Keine Integer-Casts (M-02 = 0).

use super::error::RunError;
use crate::codec::varint::{decode_u64, encode_u64};

/// Strukturierte Repräsentation der Run-Metadaten.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetaBlock {
    /// Space-Kennung.
    pub space_id: u16,
    /// Eindeutige Run-ID aus der Hochwassermarke.
    pub run_id: u64,
    /// Gesamtzahl der enthaltenen Daten-Einträge.
    pub entries: u64,
    /// Kleinster Commit-Stamp im Run.
    pub min_stamp: u64,
    /// Größter Commit-Stamp im Run.
    pub max_stamp: u64,
    /// Anzahl an Bereichslöschungen (RangeDelete).
    pub range_markers: u32,
    /// Anzahl an physischen Bereichslöschungen (EraseRange).
    pub erase_markers: u32,
    /// Erstellungszeitpunkt in Millisekunden seit Unix-Epoche.
    pub created_ms: u64,
    /// Feste Headerlänge je Eintrag im Space.
    pub header_len: u16,
    /// Anzahl der geführten Gewichte k (1..=4).
    pub weights_k: u8,
    /// Version des Zählverfahrens des Weigher.
    pub weigher_version: u32,
    /// Gesetzte Feature-Bits für diesen Run.
    pub feature_bits: u64,
    /// Kleinster Schlüssel (inklusive Bereichsmarker).
    pub min_key: Vec<u8>,
    /// Größter Schlüssel (inklusive Bereichsmarker).
    pub max_key: Vec<u8>,
    /// Liste der referenzierten Blobs und deren referenzierte Bytes: (blob_no, bytes).
    pub blobs: Vec<(u64, u64)>,
}

impl MetaBlock {
    /// Kodiert den MetaBlock nach der festen SPEC §9.5 Feldreihenfolge.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();

        // 1. space_id u16
        out.extend_from_slice(&self.space_id.to_le_bytes());
        // 2. run_id u64
        out.extend_from_slice(&self.run_id.to_le_bytes());
        // 3. entries u64
        out.extend_from_slice(&self.entries.to_le_bytes());
        // 4. min_stamp u64
        out.extend_from_slice(&self.min_stamp.to_le_bytes());
        // 5. max_stamp u64
        out.extend_from_slice(&self.max_stamp.to_le_bytes());
        // 6. range_markers u32
        out.extend_from_slice(&self.range_markers.to_le_bytes());
        // 7. erase_markers u32
        out.extend_from_slice(&self.erase_markers.to_le_bytes());
        // 8. created_ms u64
        out.extend_from_slice(&self.created_ms.to_le_bytes());
        // 9. header_len u16
        out.extend_from_slice(&self.header_len.to_le_bytes());
        // 10. weights_k u8
        out.push(self.weights_k);
        // 11. weigher_version u32
        out.extend_from_slice(&self.weigher_version.to_le_bytes());
        // 12. feature_bits u64
        out.extend_from_slice(&self.feature_bits.to_le_bytes());

        // 13. min_key (len varint + bytes)
        let mut var_buf = [0u8; 10];
        let min_len_u64 = u64::try_from(self.min_key.len()).unwrap_or(0);
        let mlen = encode_u64(min_len_u64, &mut var_buf);
        if let Some(s) = var_buf.get(..mlen) {
            out.extend_from_slice(s);
        }
        out.extend_from_slice(&self.min_key);

        // 14. max_key (len varint + bytes)
        let max_len_u64 = u64::try_from(self.max_key.len()).unwrap_or(0);
        let xlen = encode_u64(max_len_u64, &mut var_buf);
        if let Some(s) = var_buf.get(..xlen) {
            out.extend_from_slice(s);
        }
        out.extend_from_slice(&self.max_key);

        // 15. blob_count varint
        let blob_count_u64 = u64::try_from(self.blobs.len()).unwrap_or(0);
        let blen = encode_u64(blob_count_u64, &mut var_buf);
        if let Some(s) = var_buf.get(..blen) {
            out.extend_from_slice(s);
        }

        // 16. (blob_no u64, bytes u64) × blob_count
        for &(blob_no, bytes) in &self.blobs {
            out.extend_from_slice(&blob_no.to_le_bytes());
            out.extend_from_slice(&bytes.to_le_bytes());
        }

        out
    }

    /// Dekodiert den MetaBlock aus Bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, RunError> {
        // Feste Kopfgröße bis feature_bits:
        // space_id(2) + run_id(8) + entries(8) + min_stamp(8) + max_stamp(8) +
        // range_markers(4) + erase_markers(4) + created_ms(8) + header_len(2) +
        // weights_k(1) + weigher_version(4) + feature_bits(8) = 65 Bytes
        const FIXED_LEN: usize = 2 + 8 + 8 + 8 + 8 + 4 + 4 + 8 + 2 + 1 + 4 + 8;
        let fixed_slice = bytes
            .get(0..FIXED_LEN)
            .ok_or_else(|| RunError::Corrupt("MetaBlock zu kurz für feste Felder".into()))?;

        let mut offset = 0usize;
        let space_id = read_u16_le(fixed_slice, &mut offset)?;
        let run_id = read_u64_le(fixed_slice, &mut offset)?;
        let entries = read_u64_le(fixed_slice, &mut offset)?;
        let min_stamp = read_u64_le(fixed_slice, &mut offset)?;
        let max_stamp = read_u64_le(fixed_slice, &mut offset)?;
        let range_markers = read_u32_le(fixed_slice, &mut offset)?;
        let erase_markers = read_u32_le(fixed_slice, &mut offset)?;
        let created_ms = read_u64_le(fixed_slice, &mut offset)?;
        let header_len = read_u16_le(fixed_slice, &mut offset)?;
        let weights_k = *fixed_slice
            .get(offset)
            .ok_or_else(|| RunError::Corrupt("weights_k fehlt".into()))?;
        offset = offset.saturating_add(1);
        let weigher_version = read_u32_le(fixed_slice, &mut offset)?;
        let feature_bits = read_u64_le(fixed_slice, &mut offset)?;

        // min_key
        let rem1 = bytes
            .get(FIXED_LEN..)
            .ok_or_else(|| RunError::Corrupt("Meta min_key fehlt".into()))?;
        let (min_len_u64, min_consumed) = decode_u64(rem1)
            .map_err(|e| RunError::Corrupt(format!("Meta min_key_len varint: {e}")))?;
        let min_len = usize::try_from(min_len_u64)
            .map_err(|_| RunError::Limit("min_key_len übersteigt Adressraum".into()))?;
        let mut cur_pos = FIXED_LEN.saturating_add(min_consumed);

        let min_key_end = cur_pos
            .checked_add(min_len)
            .ok_or_else(|| RunError::Limit("Offset-Überlauf".into()))?;
        let min_key = bytes
            .get(cur_pos..min_key_end)
            .ok_or_else(|| RunError::Corrupt("Meta min_key Bytes fehlen".into()))?
            .to_vec();
        cur_pos = min_key_end;

        // max_key
        let rem2 = bytes
            .get(cur_pos..)
            .ok_or_else(|| RunError::Corrupt("Meta max_key fehlt".into()))?;
        let (max_len_u64, max_consumed) = decode_u64(rem2)
            .map_err(|e| RunError::Corrupt(format!("Meta max_key_len varint: {e}")))?;
        let max_len = usize::try_from(max_len_u64)
            .map_err(|_| RunError::Limit("max_key_len übersteigt Adressraum".into()))?;
        cur_pos = cur_pos.saturating_add(max_consumed);

        let max_key_end = cur_pos
            .checked_add(max_len)
            .ok_or_else(|| RunError::Limit("Offset-Überlauf".into()))?;
        let max_key = bytes
            .get(cur_pos..max_key_end)
            .ok_or_else(|| RunError::Corrupt("Meta max_key Bytes fehlen".into()))?
            .to_vec();
        cur_pos = max_key_end;

        // blob_count
        let rem3 = bytes
            .get(cur_pos..)
            .ok_or_else(|| RunError::Corrupt("Meta blob_count fehlt".into()))?;
        let (blob_count_u64, blob_consumed) = decode_u64(rem3)
            .map_err(|e| RunError::Corrupt(format!("Meta blob_count varint: {e}")))?;
        let blob_count = usize::try_from(blob_count_u64)
            .map_err(|_| RunError::Limit("blob_count übersteigt Adressraum".into()))?;
        cur_pos = cur_pos.saturating_add(blob_consumed);

        let mut blobs = Vec::with_capacity(blob_count);
        for _ in 0..blob_count {
            let next_pos = cur_pos
                .checked_add(16)
                .ok_or_else(|| RunError::Limit("Offset-Überlauf".into()))?;
            let blob_slice = bytes
                .get(cur_pos..next_pos)
                .ok_or_else(|| RunError::Corrupt("Meta Blob-Eintrag fehlt".into()))?;

            let mut b_no = [0u8; 8];
            let mut b_bytes = [0u8; 8];
            if let Some(s) = blob_slice.get(0..8) {
                b_no.copy_from_slice(s);
            }
            if let Some(s) = blob_slice.get(8..16) {
                b_bytes.copy_from_slice(s);
            }
            blobs.push((u64::from_le_bytes(b_no), u64::from_le_bytes(b_bytes)));
            cur_pos = next_pos;
        }

        Ok(Self {
            space_id,
            run_id,
            entries,
            min_stamp,
            max_stamp,
            range_markers,
            erase_markers,
            created_ms,
            header_len,
            weights_k,
            weigher_version,
            feature_bits,
            min_key,
            max_key,
            blobs,
        })
    }
}

fn read_u16_le(src: &[u8], offset: &mut usize) -> Result<u16, RunError> {
    let next = offset.saturating_add(2);
    let slice = src
        .get(*offset..next)
        .ok_or_else(|| RunError::Corrupt("u16 fehlt".into()))?;
    *offset = next;
    let mut b = [0u8; 2];
    b.copy_from_slice(slice);
    Ok(u16::from_le_bytes(b))
}

fn read_u32_le(src: &[u8], offset: &mut usize) -> Result<u32, RunError> {
    let next = offset.saturating_add(4);
    let slice = src
        .get(*offset..next)
        .ok_or_else(|| RunError::Corrupt("u32 fehlt".into()))?;
    *offset = next;
    let mut b = [0u8; 4];
    b.copy_from_slice(slice);
    Ok(u32::from_le_bytes(b))
}

fn read_u64_le(src: &[u8], offset: &mut usize) -> Result<u64, RunError> {
    let next = offset.saturating_add(8);
    let slice = src
        .get(*offset..next)
        .ok_or_else(|| RunError::Corrupt("u64 fehlt".into()))?;
    *offset = next;
    let mut b = [0u8; 8];
    b.copy_from_slice(slice);
    Ok(u64::from_le_bytes(b))
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn test_metablock_roundtrip() {
        let meta = MetaBlock {
            space_id: 42,
            run_id: 1001,
            entries: 500,
            min_stamp: 1,
            max_stamp: 75,
            range_markers: 2,
            erase_markers: 1,
            created_ms: 1_700_000_000_000,
            header_len: 32,
            weights_k: 2,
            weigher_version: 1,
            feature_bits: 0,
            min_key: b"alpha".to_vec(),
            max_key: b"omega".to_vec(),
            blobs: vec![(0x71, 4096), (0x72, 8192)],
        };

        let encoded = meta.encode();
        let decoded = MetaBlock::decode(&encoded).unwrap();
        assert_eq!(decoded, meta);
    }
}

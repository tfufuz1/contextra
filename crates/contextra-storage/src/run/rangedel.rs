//! # RangeDelBlock: Bereichslöschungen und Erase-Marker
//!
//! Format (SPEC §9.5):
//! `count varint | (start (len varint + bytes) | end_flag u8 | [end (len varint + bytes)] | tag u64) × count`
//! `end_flag`: 0 = begrenzt, 1 = unbegrenzt bis Space-Ende.
//! Sortiert und fragmentiert; beim Öffnen vollständig im Speicher geladen (klein).
//! Keine Integer-Casts (M-02 = 0).

use super::error::RunError;
use crate::codec::tag::Tag;
use crate::codec::varint::{decode_u64, encode_u64};
use std::cmp::Ordering;

/// Ein einzelner Bereichsmarker (RangeDelete oder EraseRange).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeMarker {
    /// Startschlüssel (inklusiv).
    pub start: Vec<u8>,
    /// Endschlüssel (exklusiv), oder `None` für unbegrenzt.
    pub end: Option<Vec<u8>>,
    /// Tag: `(stamp << 8) | kind`.
    pub tag: Tag,
}

impl RangeMarker {
    /// Erstellt einen neuen begrenzten Marker.
    pub fn new(start: Vec<u8>, end: Vec<u8>, tag: Tag) -> Self {
        Self {
            start,
            end: Some(end),
            tag,
        }
    }

    /// Erstellt einen neuen unbegrenzten Marker bis zum Space-Ende.
    pub fn unbounded(start: Vec<u8>, tag: Tag) -> Self {
        Self {
            start,
            end: None,
            tag,
        }
    }

    /// Prüft, ob ein Schlüssel innerhalb dieses Bereichs liegt.
    pub fn covers_key(&self, key: &[u8]) -> bool {
        if key < self.start.as_slice() {
            return false;
        }
        match &self.end {
            Some(e) => key < e.as_slice(),
            None => true,
        }
    }

    /// Stamp des Markers.
    pub const fn stamp(&self) -> u64 {
        self.tag.stamp()
    }
}

impl Ord for RangeMarker {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.start.cmp(&other.start) {
            Ordering::Equal => {
                // Bei gleichem Start: größter Stamp zuerst
                other.tag.0.cmp(&self.tag.0)
            }
            ord => ord,
        }
    }
}

impl PartialOrd for RangeMarker {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Der RangeDel-Block eines Runs im Speicher.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RangeDelBlock {
    /// Sortierte Marker.
    pub markers: Vec<RangeMarker>,
}

impl RangeDelBlock {
    /// Erstellt einen leeren Block.
    pub fn new() -> Self {
        Self {
            markers: Vec::new(),
        }
    }

    /// Fügt einen Marker hinzu.
    pub fn add(&mut self, marker: RangeMarker) {
        self.markers.push(marker);
    }

    /// Fragmentiert und sortiert die Marker für kanonische Speicherung.
    pub fn prepare(&mut self) {
        self.markers.sort();
    }

    /// Kodiert den Block nach SPEC §9.5.
    pub fn encode(&self) -> Vec<u8> {
        let count_u64 = u64::try_from(self.markers.len()).unwrap_or(0);
        let mut var_buf = [0u8; 10];
        let var_len = encode_u64(count_u64, &mut var_buf);

        let mut out = Vec::new();
        if let Some(s) = var_buf.get(..var_len) {
            out.extend_from_slice(s);
        }

        for m in &self.markers {
            // start: len varint + bytes
            let slen_u64 = u64::try_from(m.start.len()).unwrap_or(0);
            let slen = encode_u64(slen_u64, &mut var_buf);
            if let Some(s) = var_buf.get(..slen) {
                out.extend_from_slice(s);
            }
            out.extend_from_slice(&m.start);

            // end_flag
            match &m.end {
                Some(end_bytes) => {
                    out.push(0u8); // 0 = begrenzt
                    let elen_u64 = u64::try_from(end_bytes.len()).unwrap_or(0);
                    let elen = encode_u64(elen_u64, &mut var_buf);
                    if let Some(s) = var_buf.get(..elen) {
                        out.extend_from_slice(s);
                    }
                    out.extend_from_slice(end_bytes);
                }
                None => {
                    out.push(1u8); // 1 = unbegrenzt
                }
            }

            // tag u64
            out.extend_from_slice(&m.tag.0.to_le_bytes());
        }

        out
    }

    /// Dekodiert einen RangeDel-Block.
    pub fn decode(bytes: &[u8]) -> Result<Self, RunError> {
        let (count_u64, consumed) = decode_u64(bytes)
            .map_err(|e| RunError::Corrupt(format!("RangeDel-Count varint ungültig: {e}")))?;

        let count = usize::try_from(count_u64)
            .map_err(|_| RunError::Limit("RangeDel-Count übersteigt Adressraum".into()))?;

        let mut offset = consumed;
        let mut markers = Vec::with_capacity(count);

        for _ in 0..count {
            // start
            let remaining = bytes
                .get(offset..)
                .ok_or_else(|| RunError::Corrupt("RangeDel-Daten zu kurz für start".into()))?;
            let (slen_u64, slen_consumed) = decode_u64(remaining)
                .map_err(|e| RunError::Corrupt(format!("RangeDel start_len varint: {e}")))?;
            offset = offset
                .checked_add(slen_consumed)
                .ok_or_else(|| RunError::Limit("Offset-Überlauf".into()))?;

            let slen = usize::try_from(slen_u64)
                .map_err(|_| RunError::Limit("Start-Länge übersteigt Adressraum".into()))?;
            let start_end_offset = offset
                .checked_add(slen)
                .ok_or_else(|| RunError::Limit("Offset-Überlauf".into()))?;
            let start_bytes = bytes
                .get(offset..start_end_offset)
                .ok_or_else(|| RunError::Corrupt("RangeDel start Bytes fehlen".into()))?;
            let start = start_bytes.to_vec();
            offset = start_end_offset;

            // end_flag
            let end_flag = *bytes
                .get(offset)
                .ok_or_else(|| RunError::Corrupt("RangeDel end_flag fehlt".into()))?;
            offset = offset
                .checked_add(1)
                .ok_or_else(|| RunError::Limit("Offset-Überlauf".into()))?;

            let end = if end_flag == 0 {
                // Begrenzt: len varint + bytes
                let rem = bytes
                    .get(offset..)
                    .ok_or_else(|| RunError::Corrupt("RangeDel end_len fehlt".into()))?;
                let (elen_u64, elen_consumed) = decode_u64(rem)
                    .map_err(|e| RunError::Corrupt(format!("RangeDel end_len varint: {e}")))?;
                offset = offset
                    .checked_add(elen_consumed)
                    .ok_or_else(|| RunError::Limit("Offset-Überlauf".into()))?;

                let elen = usize::try_from(elen_u64)
                    .map_err(|_| RunError::Limit("End-Länge übersteigt Adressraum".into()))?;
                let end_end_offset = offset
                    .checked_add(elen)
                    .ok_or_else(|| RunError::Limit("Offset-Überlauf".into()))?;
                let end_bytes = bytes
                    .get(offset..end_end_offset)
                    .ok_or_else(|| RunError::Corrupt("RangeDel end Bytes fehlen".into()))?;
                let e = end_bytes.to_vec();
                offset = end_end_offset;
                Some(e)
            } else if end_flag == 1 {
                None
            } else {
                return Err(RunError::Corrupt(format!(
                    "Ungültiges RangeDel end_flag: {end_flag}"
                )));
            };

            // tag u64
            let tag_bytes_slice = bytes
                .get(offset..offset.saturating_add(8))
                .ok_or_else(|| RunError::Corrupt("RangeDel tag u64 fehlt".into()))?;
            let mut tag_arr = [0u8; 8];
            tag_arr.copy_from_slice(tag_bytes_slice);
            let tag = Tag::from_le_bytes(tag_arr);
            offset = offset
                .checked_add(8)
                .ok_or_else(|| RunError::Limit("Offset-Überlauf".into()))?;

            markers.push(RangeMarker { start, end, tag });
        }

        let mut block = Self { markers };
        block.prepare();
        Ok(block)
    }

    /// Prüft, ob ein Eintrag mit `key` durch einen Bereichsmarker verdeckt wird.
    ///
    /// Ein Marker mit Stamp `S` verdeckt Versionen mit `stamp < S` (SPEC §6.2, §11).
    pub fn is_deleted(&self, key: &[u8], version_stamp: u64) -> bool {
        for m in &self.markers {
            if m.covers_key(key) && m.stamp() > version_stamp {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;
    use crate::codec::tag::TagKind;

    #[test]
    fn test_rangedel_roundtrip_and_coverage() {
        let mut block = RangeDelBlock::new();
        let t1 = Tag::new(10, TagKind::RangeDelete);
        let t2 = Tag::new(20, TagKind::EraseRange);

        block.add(RangeMarker::new(b"a".to_vec(), b"m".to_vec(), t1));
        block.add(RangeMarker::unbounded(b"x".to_vec(), t2));
        block.prepare();

        let encoded = block.encode();
        let decoded = RangeDelBlock::decode(&encoded).unwrap();

        assert_eq!(decoded.markers.len(), 2);
        assert!(decoded.is_deleted(b"c", 5));  // c liegt in [a, m) und stamp 5 < 10
        assert!(!decoded.is_deleted(b"c", 10)); // stamp 10 nicht < 10
        assert!(!decoded.is_deleted(b"p", 5));  // p liegt nicht in [a, m)
        assert!(decoded.is_deleted(b"z", 15)); // z liegt in [x, unbegrenzt) und 15 < 20
    }
}

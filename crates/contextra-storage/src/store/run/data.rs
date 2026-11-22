//! # DataBlock: Präfixkomprimierte Schlüssel-Wert-Einträge
//!
//! Format (SPEC §9.5):
//! `shared varint | unshared varint | vlen varint | tag u64 | key_suffix[unshared] | value[vlen]`
//! Neustartpunkt alle 16 Einträge (dort `shared = 0`).
//! Am Ende: `restart_offsets u32[n] | n u32`.
//! Ziel 4 KiB; ein einzelner größerer Eintrag erhält einen eigenen Block.
//! Keine Integer-Casts (M-02 = 0).

use super::error::RunError;
use crate::codec::tag::Tag;
use crate::codec::varint::{decode_u64, encode_u64};

/// Standard-Neustartintervall für Präfixkompression (SPEC §6.6, §9.5).
pub const RESTART_INTERVAL: usize = 16;

/// Standard-Blockzielgröße (4 KiB, SPEC §6.6, §9.5).
pub const TARGET_BLOCK_BYTES: usize = 4096;

/// Ein dekodierter Eintrag in einem DataBlock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataEntry {
    /// Vollständiger Nutzerschlüssel.
    pub user_key: Vec<u8>,
    /// Tag: `(stamp << 8) | kind`.
    pub tag: Tag,
    /// Wert (Header + Körper bzw. BlobRef). Bei Delete leer.
    pub value: Vec<u8>,
}

/// Ersteller für einen Datenblock mit Präfixkompression.
#[derive(Debug, Clone)]
pub struct DataBlockBuilder {
    restart_interval: usize,
    buffer: Vec<u8>,
    restarts: Vec<u32>,
    entries: u32,
    last_key: Vec<u8>,
    first_key: Option<Vec<u8>>,
}

impl DataBlockBuilder {
    /// Erstellt einen neuen DataBlockBuilder mit Standard-Intervall (16).
    pub fn new() -> Self {
        Self::with_restart_interval(RESTART_INTERVAL)
    }

    /// Erstellt einen neuen DataBlockBuilder mit konfigurierbarem Neustart-Intervall.
    pub fn with_restart_interval(interval: usize) -> Self {
        let mut builder = Self {
            restart_interval: if interval == 0 { 16 } else { interval },
            buffer: Vec::with_capacity(TARGET_BLOCK_BYTES),
            restarts: Vec::new(),
            entries: 0,
            last_key: Vec::new(),
            first_key: None,
        };
        // Erster Eintrag ist immer ein Neustartpunkt an Offset 0
        builder.restarts.push(0);
        builder
    }

    /// Prüft, ob noch keine Einträge vorhanden sind.
    pub const fn is_empty(&self) -> bool {
        self.entries == 0
    }

    /// Anzahl der Einträge im aktuellen Block.
    pub const fn entries_count(&self) -> u32 {
        self.entries
    }

    /// Erster Schlüssel im Block.
    pub fn first_key(&self) -> Option<&[u8]> {
        self.first_key.as_deref()
    }

    /// Letzter Schlüssel im Block.
    pub fn last_key(&self) -> Option<&[u8]> {
        if self.entries == 0 {
            None
        } else {
            Some(&self.last_key)
        }
    }

    /// Geschätzte physische Größe des fertigen Blocks in Bytes.
    pub fn estimated_size(&self) -> usize {
        // Puffer + Neustarts (je 4 B) + count (4 B)
        self.buffer.len().saturating_add(
            self.restarts
                .len()
                .saturating_mul(4)
                .saturating_add(4),
        )
    }

    /// Fügt einen neuen Eintrag hinzu.
    pub fn add(&mut self, key: &[u8], tag: Tag, value: &[u8]) {
        if self.first_key.is_none() {
            self.first_key = Some(key.to_vec());
        }

        let entries_usize = usize::try_from(self.entries).unwrap_or(0);
        let is_restart = entries_usize > 0 && (entries_usize % self.restart_interval == 0);

        let shared = if is_restart {
            let offset_u32 = u32::try_from(self.buffer.len()).unwrap_or(u32::MAX);
            self.restarts.push(offset_u32);
            0usize
        } else {
            common_prefix(&self.last_key, key)
        };

        let unshared = key.len().saturating_sub(shared);

        // Varints kodieren: shared, unshared, vlen
        append_varint(&mut self.buffer, shared);
        append_varint(&mut self.buffer, unshared);
        append_varint(&mut self.buffer, value.len());

        // tag u64 (8 B little-endian)
        self.buffer.extend_from_slice(&tag.0.to_le_bytes());

        // key_suffix[unshared]
        if let Some(suffix) = key.get(shared..) {
            self.buffer.extend_from_slice(suffix);
        }

        // value[vlen]
        self.buffer.extend_from_slice(value);

        self.last_key.clear();
        self.last_key.extend_from_slice(key);
        self.entries = self.entries.saturating_add(1);
    }

    /// Schließt den Block ab und hängt die Neustart-Tabelle an.
    pub fn finish(mut self) -> Vec<u8> {
        let count_u32 = u32::try_from(self.restarts.len()).unwrap_or(0);
        for &offset in &self.restarts {
            self.buffer.extend_from_slice(&offset.to_le_bytes());
        }
        self.buffer.extend_from_slice(&count_u32.to_le_bytes());
        self.buffer
    }
}

/// Leser für einen präfixkomprimierten Datenblock.
#[derive(Debug, Clone)]
pub struct DataBlockReader<'a> {
    data: &'a [u8],
    restarts_offset: usize,
    restart_count: usize,
}

impl<'a> DataBlockReader<'a> {
    /// Erstellt einen Leser aus den Rohbytes der Blocknutzlast.
    pub fn new(data: &'a [u8]) -> Result<Self, RunError> {
        if data.len() < 4 {
            return Err(RunError::Corrupt("DataBlock zu kurz (< 4 B)".into()));
        }
        let count_offset = data.len().saturating_sub(4);
        let count_slice = data.get(count_offset..).ok_or_else(|| RunError::Corrupt("Puffergrenzen ungültig".into()))?;
        let mut count_arr = [0u8; 4];
        count_arr.copy_from_slice(count_slice);
        let restart_count = usize::try_from(u32::from_le_bytes(count_arr))
            .map_err(|_| RunError::Limit("Neustartanzahl übersteigt Adressraum".into()))?;
        let table_len = restart_count.checked_mul(4)
            .ok_or_else(|| RunError::Limit("Multiplikationsüberlauf bei Neustarttabelle".into()))?;
        if count_offset < table_len {
            return Err(RunError::Corrupt("DataBlock zu kurz für Neustarttabelle".into()));
        }
        let restarts_offset = count_offset.saturating_sub(table_len);
        Ok(Self { data, restarts_offset, restart_count })
    }

    fn read_restart(&self, idx: usize) -> Result<usize, RunError> {
        if idx >= self.restart_count {
            return Err(RunError::Corrupt("Neustartindex außerhalb der Grenzen".into()));
        }
        let byte_pos = self.restarts_offset.saturating_add(idx.saturating_mul(4));
        let slice = self.data.get(byte_pos..byte_pos.saturating_add(4))
            .ok_or_else(|| RunError::Corrupt("Neustart-Offset fehlt".into()))?;
        let mut b = [0u8; 4];
        b.copy_from_slice(slice);
        usize::try_from(u32::from_le_bytes(b)).map_err(|_| RunError::Limit("Offset übersteigt Adressraum".into()))
    }

    /// Sucht einen Eintrag mit passendem `user_key`.
    pub fn get(&self, user_key: &[u8]) -> Result<Option<DataEntry>, RunError> {
        if self.restart_count == 0 {
            return Ok(None);
        }

        // 1. Binärsuche über Neustartpunkte
        let mut low: usize = 0;
        let mut high: usize = self.restart_count;

        while low < high {
            let mid = low.checked_add(high.saturating_sub(low) / 2)
                .ok_or_else(|| RunError::Limit("Überlauf bei Binärsuche".into()))?;
            let offset = self.read_restart(mid)?;
            let key = self.read_restart_key(offset)?;

            if key.as_slice() < user_key {
                low = mid.saturating_add(1);
            } else {
                high = mid;
            }
        }

        // Startpunkt für sequentiellen Scan
        let scan_restart = if low == 0 { 0 } else { low.saturating_sub(1) };
        let mut cur_offset = self.read_restart(scan_restart)?;
        let mut cur_key = Vec::new();

        while cur_offset < self.restarts_offset {
            let (entry, next_offset) = self.decode_entry(cur_offset, &mut cur_key)?;
            cur_offset = next_offset;

            if entry.user_key.as_slice() == user_key {
                return Ok(Some(entry));
            }
            if entry.user_key.as_slice() > user_key {
                // Einträge sind sortiert, weiterlesen unnötig
                break;
            }
        }

        Ok(None)
    }

    fn read_restart_key(&self, offset: usize) -> Result<Vec<u8>, RunError> {
        let mut key = Vec::new();
        let (entry, _) = self.decode_entry(offset, &mut key)?;
        Ok(entry.user_key)
    }

    fn decode_entry(
        &self,
        offset: usize,
        current_key: &mut Vec<u8>,
    ) -> Result<(DataEntry, usize), RunError> {
        let slice = self
            .data
            .get(offset..self.restarts_offset)
            .ok_or_else(|| RunError::Corrupt("Offset außerhalb der Nutzlast".into()))?;

        // 1. shared, unshared, vlen
        let (shared, c1) = decode_usize(slice, "shared")?;
        let slice2 = slice.get(c1..).ok_or_else(|| RunError::Corrupt("Daten fehlen nach shared".into()))?;
        let (unshared, c2) = decode_usize(slice2, "unshared")?;
        let slice3 = slice2.get(c2..).ok_or_else(|| RunError::Corrupt("Daten fehlen nach unshared".into()))?;
        let (vlen, c3) = decode_usize(slice3, "vlen")?;

        let hdr_consumed = c1.saturating_add(c2).saturating_add(c3);

        // 4. tag u64 (8 B)
        let slice4 = slice
            .get(hdr_consumed..)
            .ok_or_else(|| RunError::Corrupt("Daten fehlen für tag u64".into()))?;
        let tag_slice = slice4
            .get(0..8)
            .ok_or_else(|| RunError::Corrupt("tag u64 zu kurz".into()))?;
        let mut tag_arr = [0u8; 8];
        tag_arr.copy_from_slice(tag_slice);
        let tag = Tag::from_le_bytes(tag_arr);

        let data_start = hdr_consumed.saturating_add(8);

        // 5. key_suffix[unshared]
        let suffix_end = data_start
            .checked_add(unshared)
            .ok_or_else(|| RunError::Limit("Überlauf bei suffix_end".into()))?;
        let suffix = slice
            .get(data_start..suffix_end)
            .ok_or_else(|| RunError::Corrupt("key_suffix fehlt".into()))?;

        current_key.truncate(shared);
        current_key.extend_from_slice(suffix);
        let user_key = current_key.clone();

        // 6. value[vlen]
        let val_end = suffix_end
            .checked_add(vlen)
            .ok_or_else(|| RunError::Limit("Überlauf bei val_end".into()))?;
        let value = slice
            .get(suffix_end..val_end)
            .ok_or_else(|| RunError::Corrupt("value Bytes fehlen".into()))?
            .to_vec();

        let total_consumed = offset.saturating_add(val_end);
        Ok((DataEntry { user_key, tag, value }, total_consumed))
    }

    /// Liefert einen Iterator über alle Einträge im Datenblock.
    pub fn iter(&self) -> DataBlockIter<'a> {
        DataBlockIter {
            reader: self.clone(),
            offset: 0,
            current_key: Vec::new(),
        }
    }
}

/// Iterator über alle Einträge eines Datenblocks.
#[derive(Debug, Clone)]
pub struct DataBlockIter<'a> {
    reader: DataBlockReader<'a>,
    offset: usize,
    current_key: Vec<u8>,
}

impl<'a> Iterator for DataBlockIter<'a> {
    type Item = Result<DataEntry, RunError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.reader.restarts_offset {
            return None;
        }
        match self.reader.decode_entry(self.offset, &mut self.current_key) {
            Ok((entry, next_offset)) => {
                self.offset = next_offset;
                Some(Ok(entry))
            }
            Err(e) => {
                self.offset = self.reader.restarts_offset;
                Some(Err(e))
            }
        }
    }
}

fn common_prefix(a: &[u8], b: &[u8]) -> usize {
    let limit = std::cmp::min(a.len(), b.len());
    let mut i = 0usize;
    while i < limit {
        if a.get(i) != b.get(i) {
            break;
        }
        i = i.saturating_add(1);
    }
    i
}

fn append_varint(buf: &mut Vec<u8>, val: usize) {
    let mut var_buf = [0u8; 10];
    let v_u64 = u64::try_from(val).unwrap_or(0);
    let len = encode_u64(v_u64, &mut var_buf);
    if let Some(s) = var_buf.get(..len) {
        buf.extend_from_slice(s);
    }
}

fn decode_usize(slice: &[u8], name: &'static str) -> Result<(usize, usize), RunError> {
    let (v_u64, consumed) = decode_u64(slice)
        .map_err(|e| RunError::Corrupt(format!("{name} varint: {e}")))?;
    let v = usize::try_from(v_u64)
        .map_err(|_| RunError::Limit(format!("{name} übersteigt Adressraum")))?;
    Ok((v, consumed))
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;
    use crate::codec::tag::TagKind;

    #[test]
    fn test_datablock_roundtrip_prefix_compression() {
        let mut builder = DataBlockBuilder::with_restart_interval(4);
        let t1 = Tag::new(1, TagKind::Put);
        let t2 = Tag::new(2, TagKind::Put);
        let t3 = Tag::new(3, TagKind::Put);

        builder.add(b"apple", t1, b"val_apple");
        builder.add(b"application", t2, b"val_app");
        builder.add(b"apply", t3, b"val_apply");

        let encoded = builder.finish();
        let reader = DataBlockReader::new(&encoded).unwrap();

        let e1 = reader.get(b"apple").unwrap().unwrap();
        assert_eq!(e1.user_key, b"apple");
        assert_eq!(e1.value, b"val_apple");

        let e2 = reader.get(b"application").unwrap().unwrap();
        assert_eq!(e2.user_key, b"application");
        assert_eq!(e2.value, b"val_app");

        assert!(reader.get(b"banana").unwrap().is_none());

        let all: Vec<_> = reader.iter().map(|r| r.unwrap().user_key).collect();
        assert_eq!(all, vec![b"apple".to_vec(), b"application".to_vec(), b"apply".to_vec()]);
    }
}

//! Postings-Strukturen und VByte-Komprimierung nach SPEC Anhang F.7.

use contextra_types::ContextraError;
type Result<T> = std::result::Result<T, ContextraError>;

pub const BLOCK_SIZE: usize = 128;

pub fn encode_vbyte(mut val: u64, out: &mut Vec<u8>) {
    loop {
        let byte = (val & 0x7F) as u8;
        val >>= 7;
        if val == 0 {
            out.push(byte | 0x80);
            break;
        } else {
            out.push(byte);
        }
    }
}

pub fn decode_vbyte(slice: &[u8]) -> Option<(u64, usize)> {
    let mut val: u64 = 0;
    let mut shift = 0;
    for (i, &b) in slice.iter().enumerate() {
        if shift >= 64 {
            return None;
        }
        val |= ((b & 0x7F) as u64) << shift;
        if b & 0x80 != 0 {
            return Some((val, i + 1));
        }
        shift += 7;
    }
    None
}

#[derive(Clone, Debug, PartialEq)]
pub struct PostingBlock {
    pub max_doc_ord: u32,
    pub max_tf_norm: u16,
    pub doc_ords: Vec<u32>,
    pub tfs: Vec<u32>,
    pub positions: Option<Vec<Vec<u32>>>,
}

impl PostingBlock {
    pub fn new() -> Self {
        Self {
            max_doc_ord: 0,
            max_tf_norm: 0,
            doc_ords: Vec::with_capacity(BLOCK_SIZE),
            tfs: Vec::with_capacity(BLOCK_SIZE),
            positions: None,
        }
    }

    pub fn encode(&self, has_positions: bool) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&self.max_doc_ord.to_be_bytes());
        out.extend_from_slice(&self.max_tf_norm.to_be_bytes());
        let count = u16::try_from(self.doc_ords.len()).unwrap_or(u16::MAX);
        out.extend_from_slice(&count.to_be_bytes());

        let mut last_ord = 0u32;
        for &ord in &self.doc_ords {
            let delta = ord.saturating_sub(last_ord);
            encode_vbyte(delta as u64, &mut out);
            last_ord = ord;
        }

        for &tf in &self.tfs {
            encode_vbyte(tf as u64, &mut out);
        }

        if has_positions {
            if let Some(ref pos_list) = self.positions {
                for doc_positions in pos_list {
                    encode_vbyte(doc_positions.len() as u64, &mut out);
                    let mut last_pos = 0u32;
                    for &pos in doc_positions {
                        let delta = pos.saturating_sub(last_pos);
                        encode_vbyte(delta as u64, &mut out);
                        last_pos = pos;
                    }
                }
            }
        }

        out
    }

    pub fn decode(bytes: &[u8], has_positions: bool) -> Result<(Self, usize)> {
        if bytes.len() < 8 {
            return Err(ContextraError::Index("Corrupted block".into()));
        }
        let max_doc_ord = bytes.get(..4).and_then(|s| s.try_into().ok()).map(u32::from_be_bytes).ok_or_else(|| ContextraError::Index("Corrupted max_doc_ord".into()))?;
        let max_tf_norm = bytes.get(4..6).and_then(|s| s.try_into().ok()).map(u16::from_be_bytes).ok_or_else(|| ContextraError::Index("Corrupted max_tf_norm".into()))?;
        let count = bytes.get(6..8).and_then(|s| s.try_into().ok()).map(u16::from_be_bytes).map(|c| c as usize).ok_or_else(|| ContextraError::Index("Corrupted count".into()))?;

        let mut offset = 8;
        let mut doc_ords = Vec::with_capacity(count);
        let mut last_ord = 0u32;

        for _ in 0..count {
            let slice = bytes.get(offset..).ok_or_else(|| ContextraError::Index("Corrupted slice".into()))?;
            let (delta, read) = decode_vbyte(slice).ok_or_else(|| ContextraError::Index("Corrupted delta".into()))?;
            offset += read;
            let delta_u32 = u32::try_from(delta).unwrap_or(u32::MAX);
            let ord = last_ord.saturating_add(delta_u32);
            doc_ords.push(ord);
            last_ord = ord;
        }

        let mut tfs = Vec::with_capacity(count);
        for _ in 0..count {
            let slice = bytes.get(offset..).ok_or_else(|| ContextraError::Index("Corrupted slice".into()))?;
            let (tf, read) = decode_vbyte(slice).ok_or_else(|| ContextraError::Index("Corrupted tf".into()))?;
            offset += read;
            let tf_u32 = u32::try_from(tf).unwrap_or(u32::MAX);
            tfs.push(tf_u32);
        }

        let positions = if has_positions {
            let mut all_pos = Vec::with_capacity(count);
            for _ in 0..count {
                let slice = bytes.get(offset..).ok_or_else(|| ContextraError::Index("Corrupted slice".into()))?;
                let (pcount, read) = decode_vbyte(slice).ok_or_else(|| ContextraError::Index("Corrupted pos count".into()))?;
                offset += read;
                let pcount_usize = usize::try_from(pcount).unwrap_or(usize::MAX);
                let mut doc_pos = Vec::with_capacity(pcount_usize);
                let mut last_pos = 0u32;
                for _ in 0..pcount_usize {
                    let pslice = bytes.get(offset..).ok_or_else(|| ContextraError::Index("Corrupted pslice".into()))?;
                    let (delta, pread) = decode_vbyte(pslice).ok_or_else(|| ContextraError::Index("Corrupted pos delta".into()))?;
                    offset += pread;
                    let delta_u32 = u32::try_from(delta).unwrap_or(u32::MAX);
                    let pos = last_pos.saturating_add(delta_u32);
                    doc_pos.push(pos);
                    last_pos = pos;
                }
                all_pos.push(doc_pos);
            }
            Some(all_pos)
        } else {
            None
        };

        Ok((
            Self {
                max_doc_ord,
                max_tf_norm,
                doc_ords,
                tfs,
                positions,
            },
            offset,
        ))
    }
}

impl Default for PostingBlock {
    fn default() -> Self {
        Self::new()
    }
}

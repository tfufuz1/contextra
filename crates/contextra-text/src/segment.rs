//! Segment-Format und I/O nach SPEC Anhang F.7 und D-71.

use contextra_types::{ContextraError, Result, TermTag};
use std::collections::BTreeMap;

pub const POSTING_BLOCK_SIZE: usize = 128;
pub const FLAG_POSITIONS: u8 = 0x01;

/// Big-Endian Segmentheader (SPEC Anhang F.7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentHeader {
    pub bucket: u8,
    pub bucket_count: u8,
    pub analyzer_id: u8,
    pub analyzer_version: u16,
    pub lang: [u8; 3],
    pub positions: bool,
    pub doc_count: u32,
    pub term_count: u32,
    pub total_len: u64,
}

impl SegmentHeader {
    pub fn to_bytes(&self) -> [u8; 28] {
        let mut buf = [0u8; 28];
        if let Some(b) = buf.get_mut(0) {
            *b = self.bucket;
        }
        if let Some(b) = buf.get_mut(1) {
            *b = self.bucket_count;
        }
        if let Some(b) = buf.get_mut(2) {
            *b = self.analyzer_id;
        }
        if let Some(dst) = buf.get_mut(3..5) {
            dst.copy_from_slice(&self.analyzer_version.to_be_bytes());
        }
        if let Some(dst) = buf.get_mut(5..8) {
            dst.copy_from_slice(&self.lang);
        }
        if let Some(b) = buf.get_mut(8) {
            *b = if self.positions { FLAG_POSITIONS } else { 0 };
        }
        if let Some(dst) = buf.get_mut(9..13) {
            dst.copy_from_slice(&self.doc_count.to_be_bytes());
        }
        if let Some(dst) = buf.get_mut(13..17) {
            dst.copy_from_slice(&self.term_count.to_be_bytes());
        }
        if let Some(dst) = buf.get_mut(17..25) {
            dst.copy_from_slice(&self.total_len.to_be_bytes());
        }
        buf
    }

    pub fn from_bytes(slice: &[u8]) -> Result<Self> {
        if slice.len() < 28 {
            return Err(ContextraError::InvalidInput(
                "Segment header too short".into(),
            ));
        }
        let bucket = *slice
            .first()
            .ok_or_else(|| ContextraError::InvalidInput("Missing bucket".into()))?;
        let bucket_count = *slice
            .get(1)
            .ok_or_else(|| ContextraError::InvalidInput("Missing bucket_count".into()))?;
        let analyzer_id = *slice
            .get(2)
            .ok_or_else(|| ContextraError::InvalidInput("Missing analyzer_id".into()))?;

        let v_bytes: [u8; 2] = slice
            .get(3..5)
            .and_then(|s| s.try_into().ok())
            .ok_or_else(|| ContextraError::InvalidInput("Invalid version".into()))?;
        let analyzer_version = u16::from_be_bytes(v_bytes);

        let lang: [u8; 3] = slice
            .get(5..8)
            .and_then(|s| s.try_into().ok())
            .ok_or_else(|| ContextraError::InvalidInput("Invalid lang".into()))?;
        let flags = *slice.get(8).unwrap_or(&0);
        let positions = (flags & FLAG_POSITIONS) != 0;

        let dc_bytes: [u8; 4] = slice
            .get(9..13)
            .and_then(|s| s.try_into().ok())
            .ok_or_else(|| ContextraError::InvalidInput("Invalid doc_count".into()))?;
        let doc_count = u32::from_be_bytes(dc_bytes);

        let tc_bytes: [u8; 4] = slice
            .get(13..17)
            .and_then(|s| s.try_into().ok())
            .ok_or_else(|| ContextraError::InvalidInput("Invalid term_count".into()))?;
        let term_count = u32::from_be_bytes(tc_bytes);

        let tl_bytes: [u8; 8] = slice
            .get(17..25)
            .and_then(|s| s.try_into().ok())
            .ok_or_else(|| ContextraError::InvalidInput("Invalid total_len".into()))?;
        let total_len = u64::from_be_bytes(tl_bytes);

        Ok(Self {
            bucket,
            bucket_count,
            analyzer_id,
            analyzer_version,
            lang,
            positions,
            doc_count,
            term_count,
            total_len,
        })
    }
}

/// Dokumentzuordnung und Bindung (D-69, F.7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentEntry {
    pub doc_ord: u32,
    pub doclen: u32,
    pub binding_envelope: Vec<u8>,
}

/// Einzelposting innerhalb eines 128er-Blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentPosting {
    pub doc_ord: u32,
    pub tf: u32,
    pub positions: Vec<u32>,
}

/// Metadaten eines 128er-Posting-Blocks für Block-Max WAND (D-81, F.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockMeta {
    pub max_doc_ord: u32,
    pub max_tf_norm: u16,
}

/// Block mit bis zu 128 Postings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentPostingBlock {
    pub meta: BlockMeta,
    pub postings: Vec<SegmentPosting>,
}

/// Wörterbucheintrag im Segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictionaryEntry {
    pub term_tag: TermTag,
    pub df: u32,
    pub blocks: Vec<SegmentPostingBlock>,
}

/// Builder zur Erstellung von Segmentdateien im Speicher oder persistent (SPEC F.7).
pub struct SegmentBuilder {
    bucket: u8,
    bucket_count: u8,
    analyzer_id: u8,
    analyzer_version: u16,
    lang: [u8; 3],
    positions: bool,
    documents: Vec<DocumentEntry>,
    postings_by_term: BTreeMap<TermTag, Vec<SegmentPosting>>,
}

impl SegmentBuilder {
    pub fn new(
        bucket: u8,
        bucket_count: u8,
        analyzer_id: u8,
        analyzer_version: u16,
        lang: [u8; 3],
        positions: bool,
    ) -> Self {
        Self {
            bucket,
            bucket_count,
            analyzer_id,
            analyzer_version,
            lang,
            positions,
            documents: Vec::new(),
            postings_by_term: BTreeMap::new(),
        }
    }

    pub fn add_document(&mut self, doclen: u32, binding_envelope: Vec<u8>) -> u32 {
        let doc_ord = u32::try_from(self.documents.len()).unwrap_or(u32::MAX);
        self.documents.push(DocumentEntry {
            doc_ord,
            doclen,
            binding_envelope,
        });
        doc_ord
    }

    pub fn add_term_postings(&mut self, term_tag: TermTag, postings: Vec<SegmentPosting>) {
        self.postings_by_term.insert(term_tag, postings);
    }

    pub fn build(self) -> Vec<u8> {
        let mut out = Vec::new();

        let doc_count = u32::try_from(self.documents.len()).unwrap_or(u32::MAX);
        let term_count = u32::try_from(self.postings_by_term.len()).unwrap_or(u32::MAX);
        let total_len: u64 = self.documents.iter().map(|d| d.doclen as u64).sum();

        let header = SegmentHeader {
            bucket: self.bucket,
            bucket_count: self.bucket_count,
            analyzer_id: self.analyzer_id,
            analyzer_version: self.analyzer_version,
            lang: self.lang,
            positions: self.positions,
            doc_count,
            term_count,
            total_len,
        };

        out.extend_from_slice(&header.to_bytes());

        out.extend_from_slice(&doc_count.to_be_bytes());
        for doc in &self.documents {
            out.extend_from_slice(&doc.doclen.to_be_bytes());
            let b_len = u16::try_from(doc.binding_envelope.len()).unwrap_or(u16::MAX);
            out.extend_from_slice(&b_len.to_be_bytes());
            out.extend_from_slice(&doc.binding_envelope);
        }

        let bitmap_bytes = (doc_count as usize + 7) / 8;
        out.resize(out.len() + bitmap_bytes, 0);

        out.extend_from_slice(&term_count.to_be_bytes());
        for (term_tag, mut postings) in self.postings_by_term {
            postings.sort_by_key(|p| p.doc_ord);
            let df = u32::try_from(postings.len()).unwrap_or(u32::MAX);
            out.extend_from_slice(term_tag.as_bytes());
            out.extend_from_slice(&df.to_be_bytes());

            let chunks = postings.chunks(POSTING_BLOCK_SIZE);
            let block_count = u32::try_from(chunks.len()).unwrap_or(u32::MAX);
            out.extend_from_slice(&block_count.to_be_bytes());

            for chunk in chunks {
                let max_doc_ord = chunk.iter().map(|p| p.doc_ord).max().unwrap_or(0);
                let max_tf = chunk.iter().map(|p| p.tf).max().unwrap_or(0);
                let max_tf_norm = max_tf.min(u16::MAX as u32) as u16;

                out.extend_from_slice(&max_doc_ord.to_be_bytes());
                out.extend_from_slice(&max_tf_norm.to_be_bytes());

                let count_in_block = u8::try_from(chunk.len()).unwrap_or(u8::MAX);
                out.push(count_in_block);

                let mut prev_doc_ord = 0u32;
                for p in chunk {
                    let delta = p.doc_ord.saturating_sub(prev_doc_ord);
                    out.extend_from_slice(&delta.to_be_bytes());
                    out.extend_from_slice(&p.tf.to_be_bytes());
                    prev_doc_ord = p.doc_ord;

                    if self.positions {
                        let pos_count = u16::try_from(p.positions.len()).unwrap_or(u16::MAX);
                        out.extend_from_slice(&pos_count.to_be_bytes());
                        let mut prev_pos = 0u32;
                        for pos in &p.positions {
                            let p_delta = pos.saturating_sub(prev_pos);
                            out.extend_from_slice(&p_delta.to_be_bytes());
                            prev_pos = *pos;
                        }
                    }
                }
            }
        }

        out
    }
}

pub struct SegmentReader<'a> {
    data: &'a [u8],
    pub header: SegmentHeader,
    doc_table_offset: usize,
    bitmap_offset: usize,
    dictionary_offset: usize,
}

impl<'a> SegmentReader<'a> {
    pub fn open(data: &'a [u8]) -> Result<Self> {
        let header = SegmentHeader::from_bytes(data)?;
        let mut cursor = 28usize;

        let dc_slice: [u8; 4] = data
            .get(cursor..cursor + 4)
            .and_then(|s| s.try_into().ok())
            .ok_or_else(|| ContextraError::InvalidInput("Truncated doc table count".into()))?;
        let doc_count = u32::from_be_bytes(dc_slice) as usize;
        cursor += 4;

        let doc_table_offset = cursor;
        for _ in 0..doc_count {
            cursor += 4;
            let bl_slice: [u8; 2] = data
                .get(cursor..cursor + 2)
                .and_then(|s| s.try_into().ok())
                .ok_or_else(|| ContextraError::InvalidInput("Truncated doc table entry".into()))?;
            let b_len = u16::from_be_bytes(bl_slice) as usize;
            cursor += 2;
            if data.get(cursor..cursor + b_len).is_none() {
                return Err(ContextraError::InvalidInput(
                    "Truncated binding envelope".into(),
                ));
            }
            cursor += b_len;
        }

        let bitmap_offset = cursor;
        let bitmap_bytes = (header.doc_count as usize + 7) / 8;
        if data.get(cursor..cursor + bitmap_bytes).is_none() {
            return Err(ContextraError::InvalidInput("Truncated bitmap".into()));
        }
        cursor += bitmap_bytes;

        let dictionary_offset = cursor;

        Ok(Self {
            data,
            header,
            doc_table_offset,
            bitmap_offset,
            dictionary_offset,
        })
    }

    pub fn is_deleted(&self, doc_ord: u32) -> bool {
        let byte_idx = (doc_ord / 8) as usize;
        let bit_idx = (doc_ord % 8) as u8;
        if let Some(&b) = self.data.get(self.bitmap_offset.saturating_add(byte_idx)) {
            (b & (1 << bit_idx)) != 0
        } else {
            true
        }
    }

    pub fn get_binding(&self, target_doc_ord: u32) -> Result<Option<(u32, &'a [u8])>> {
        if target_doc_ord >= self.header.doc_count || self.is_deleted(target_doc_ord) {
            return Ok(None);
        }

        let mut cursor = self.doc_table_offset;
        for ord in 0..self.header.doc_count {
            let dl_slice: [u8; 4] = self
                .data
                .get(cursor..cursor + 4)
                .and_then(|s| s.try_into().ok())
                .ok_or_else(|| ContextraError::InvalidInput("Corrupt doc table length".into()))?;
            let doclen = u32::from_be_bytes(dl_slice);
            cursor += 4;

            let bl_slice: [u8; 2] = self
                .data
                .get(cursor..cursor + 2)
                .and_then(|s| s.try_into().ok())
                .ok_or_else(|| {
                    ContextraError::InvalidInput("Corrupt doc table binding length".into())
                })?;
            let b_len = u16::from_be_bytes(bl_slice) as usize;
            cursor += 2;

            let env = self
                .data
                .get(cursor..cursor + b_len)
                .ok_or_else(|| ContextraError::InvalidInput("Corrupt doc table envelope".into()))?;
            cursor += b_len;

            if ord == target_doc_ord {
                return Ok(Some((doclen, env)));
            }
        }

        Ok(None)
    }

    pub fn get_postings(&self, target_term: &TermTag) -> Result<Option<Vec<SegmentPostingBlock>>> {
        let mut cursor = self.dictionary_offset;
        let tc_slice: [u8; 4] = match self
            .data
            .get(cursor..cursor + 4)
            .and_then(|s| s.try_into().ok())
        {
            Some(s) => s,
            None => return Ok(None),
        };
        let term_count = u32::from_be_bytes(tc_slice) as usize;
        cursor += 4;

        for _ in 0..term_count {
            let tag_slice: [u8; 16] = self
                .data
                .get(cursor..cursor + 16)
                .and_then(|s| s.try_into().ok())
                .ok_or_else(|| ContextraError::InvalidInput("Truncated dictionary tag".into()))?;
            let current_tag = TermTag::new(tag_slice);
            cursor += 16;

            cursor += 4; // skip df

            let bc_slice: [u8; 4] = self
                .data
                .get(cursor..cursor + 4)
                .and_then(|s| s.try_into().ok())
                .ok_or_else(|| ContextraError::InvalidInput("Truncated block count".into()))?;
            let block_count = u32::from_be_bytes(bc_slice) as usize;
            cursor += 4;

            let is_match = current_tag == *target_term;
            let mut matched_blocks = if is_match {
                Vec::with_capacity(block_count)
            } else {
                Vec::new()
            };

            for _ in 0..block_count {
                let mdo_slice: [u8; 4] = self
                    .data
                    .get(cursor..cursor + 4)
                    .and_then(|s| s.try_into().ok())
                    .ok_or_else(|| ContextraError::InvalidInput("Truncated max_doc_ord".into()))?;
                let max_doc_ord = u32::from_be_bytes(mdo_slice);
                cursor += 4;

                let mtn_slice: [u8; 2] = self
                    .data
                    .get(cursor..cursor + 2)
                    .and_then(|s| s.try_into().ok())
                    .ok_or_else(|| ContextraError::InvalidInput("Truncated max_tf_norm".into()))?;
                let max_tf_norm = u16::from_be_bytes(mtn_slice);
                cursor += 2;

                let count_in_block = *self.data.get(cursor).ok_or_else(|| {
                    ContextraError::InvalidInput("Truncated count in block".into())
                })? as usize;
                cursor += 1;

                let mut postings = Vec::with_capacity(count_in_block);
                let mut prev_doc_ord = 0u32;

                for _ in 0..count_in_block {
                    let d_slice: [u8; 4] = self
                        .data
                        .get(cursor..cursor + 4)
                        .and_then(|s| s.try_into().ok())
                        .ok_or_else(|| ContextraError::InvalidInput("Truncated delta".into()))?;
                    let delta = u32::from_be_bytes(d_slice);
                    cursor += 4;

                    let tf_slice: [u8; 4] = self
                        .data
                        .get(cursor..cursor + 4)
                        .and_then(|s| s.try_into().ok())
                        .ok_or_else(|| ContextraError::InvalidInput("Truncated tf".into()))?;
                    let tf = u32::from_be_bytes(tf_slice);
                    cursor += 4;

                    let doc_ord = prev_doc_ord.saturating_add(delta);
                    prev_doc_ord = doc_ord;

                    let mut positions = Vec::new();
                    if self.header.positions {
                        let pc_slice: [u8; 2] = self
                            .data
                            .get(cursor..cursor + 2)
                            .and_then(|s| s.try_into().ok())
                            .ok_or_else(|| {
                                ContextraError::InvalidInput("Truncated pos count".into())
                            })?;
                        let pos_count = u16::from_be_bytes(pc_slice) as usize;
                        cursor += 2;

                        let mut prev_pos = 0u32;
                        for _ in 0..pos_count {
                            let pd_slice: [u8; 4] = self
                                .data
                                .get(cursor..cursor + 4)
                                .and_then(|s| s.try_into().ok())
                                .ok_or_else(|| {
                                    ContextraError::InvalidInput("Truncated pos delta".into())
                                })?;
                            let p_delta = u32::from_be_bytes(pd_slice);
                            cursor += 4;
                            let pos = prev_pos.saturating_add(p_delta);
                            prev_pos = pos;
                            positions.push(pos);
                        }
                    }

                    if !self.is_deleted(doc_ord) {
                        postings.push(SegmentPosting {
                            doc_ord,
                            tf,
                            positions,
                        });
                    }
                }

                if is_match {
                    matched_blocks.push(SegmentPostingBlock {
                        meta: BlockMeta {
                            max_doc_ord,
                            max_tf_norm,
                        },
                        postings,
                    });
                }
            }

            if is_match {
                return Ok(Some(matched_blocks));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_builder_roundtrip() {
        let mut builder = SegmentBuilder::new(0, 1, 1, 1, *b"de ", true);
        let env1 = vec![1, 2, 3, 4];
        let doc0 = builder.add_document(100, env1.clone());
        let env2 = vec![5, 6, 7, 8];
        let doc1 = builder.add_document(150, env2.clone());

        let term_tag = TermTag::new([9u8; 16]);
        builder.add_term_postings(
            term_tag,
            vec![
                SegmentPosting {
                    doc_ord: doc0,
                    tf: 2,
                    positions: vec![10, 25],
                },
                SegmentPosting {
                    doc_ord: doc1,
                    tf: 1,
                    positions: vec![40],
                },
            ],
        );

        let bytes = builder.build();
        assert!(bytes.len() > 28);

        let reader = SegmentReader::open(&bytes).expect("open segment");
        assert_eq!(reader.header.doc_count, 2);
        assert_eq!(reader.header.term_count, 1);
        assert_eq!(reader.header.total_len, 250);

        let doc0_info = reader.get_binding(doc0).unwrap().unwrap();
        assert_eq!(doc0_info.0, 100);
        assert_eq!(doc0_info.1, env1.as_slice());

        let blocks = reader.get_postings(&term_tag).unwrap().unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].postings.len(), 2);
        assert_eq!(blocks[0].postings[0].doc_ord, 0);
        assert_eq!(blocks[0].postings[0].tf, 2);
        assert_eq!(blocks[0].postings[0].positions, vec![10, 25]);
    }
}

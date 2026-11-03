//! Segmentformat nach SPEC Anhang F.7 und D-71.

use contextra_crypto::aead::BlockSealer;
use contextra_crypto::hierarchy::{FileKey, FileKind};
use contextra_types::ContextraError;

type Result<T> = std::result::Result<T, ContextraError>;

use crate::postings::PostingBlock;

pub const MAGIC: &[u8; 8] = b"CTXFILE\0";
pub const FORMAT_VERSION: u16 = 1;
pub const FILE_KIND_TEXT: u8 = 4;
pub const ALG_IDS: [u8; 4] = [1, 0, 1, 1];

#[derive(Clone, Debug, PartialEq)]
pub struct DictEntry {
    pub term_tag: [u8; 16],
    pub df: u32,
    pub posting_off: u64,
    pub posting_len: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DocEntry {
    pub doc_ord: u32,
    pub doclen: u32,
    pub binding_envelope: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SegmentHeader {
    pub bucket: u8,
    pub bucket_count: u8,
    pub analyzer_id: u8,
    pub analyzer_version: u16,
    pub lang: [u8; 3],
    pub flags: u8,
    pub doc_count: u32,
    pub term_count: u32,
    pub total_len: u64,
}

impl SegmentHeader {
    pub fn encode(&self) -> [u8; 32] {
        let mut out = [0u8; 32];
        out[0] = self.bucket;
        out[1] = self.bucket_count;
        out[2] = self.analyzer_id;
        if let Some(dest) = out.get_mut(3..5) {
            dest.copy_from_slice(&self.analyzer_version.to_be_bytes());
        }
        if let Some(dest) = out.get_mut(5..8) {
            dest.copy_from_slice(&self.lang);
        }
        out[8] = self.flags;
        if let Some(dest) = out.get_mut(9..13) {
            dest.copy_from_slice(&self.doc_count.to_be_bytes());
        }
        if let Some(dest) = out.get_mut(13..17) {
            dest.copy_from_slice(&self.term_count.to_be_bytes());
        }
        if let Some(dest) = out.get_mut(17..25) {
            dest.copy_from_slice(&self.total_len.to_be_bytes());
        }
        out
    }

    pub fn decode(bytes: &[u8; 32]) -> Self {
        let bucket = bytes[0];
        let bucket_count = bytes[1];
        let analyzer_id = bytes[2];
        let analyzer_version = u16::from_be_bytes([bytes[3], bytes[4]]);
        let mut lang = [0u8; 3];
        if let Some(src) = bytes.get(5..8) {
            lang.copy_from_slice(src);
        }
        let flags = bytes[8];
        let doc_count = u32::from_be_bytes([bytes[9], bytes[10], bytes[11], bytes[12]]);
        let term_count = u32::from_be_bytes([bytes[13], bytes[14], bytes[15], bytes[16]]);
        let total_len = u64::from_be_bytes([
            bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23], bytes[24],
        ]);

        Self {
            bucket,
            bucket_count,
            analyzer_id,
            analyzer_version,
            lang,
            flags,
            doc_count,
            term_count,
            total_len,
        }
    }
}

pub struct SegmentWriter {
    pub header: SegmentHeader,
    pub dictionary: Vec<DictEntry>,
    pub postings: Vec<PostingBlock>,
    pub docs: Vec<DocEntry>,
}

impl SegmentWriter {
    pub fn new(
        bucket: u8,
        bucket_count: u8,
        analyzer_id: u8,
        analyzer_version: u16,
        lang: [u8; 3],
        positions: bool,
    ) -> Self {
        let flags = if positions { 0x01 } else { 0x00 };
        Self {
            header: SegmentHeader {
                bucket,
                bucket_count,
                analyzer_id,
                analyzer_version,
                lang,
                flags,
                doc_count: 0,
                term_count: 0,
                total_len: 0,
            },
            dictionary: Vec::new(),
            postings: Vec::new(),
            docs: Vec::new(),
        }
    }

    pub fn flags(&self) -> u8 {
        self.header.flags
    }

    pub fn doc_count(&self) -> u32 {
        self.header.doc_count
    }

    pub fn add_document(&mut self, doclen: u32, binding_envelope: Vec<u8>) -> u32 {
        let ord = u32::try_from(self.docs.len()).unwrap_or(u32::MAX);
        self.docs.push(DocEntry {
            doc_ord: ord,
            doclen,
            binding_envelope,
        });
        self.header.doc_count = u32::try_from(self.docs.len()).unwrap_or(u32::MAX);
        self.header.total_len += doclen as u64;
        ord
    }

    pub fn add_term(&mut self, term_tag: [u8; 16], df: u32, block: PostingBlock) {
        let has_positions = (self.header.flags & 0x01) != 0;
        let encoded_block = block.encode(has_positions);
        let posting_off = self.postings.len() as u64;
        let posting_len = u32::try_from(encoded_block.len()).unwrap_or(u32::MAX);
        self.dictionary.push(DictEntry {
            term_tag,
            df,
            posting_off,
            posting_len,
        });
        self.postings.push(block);
    }

    pub fn serialize_payload(&mut self) -> Vec<u8> {
        self.dictionary.sort_by(|a, b| a.term_tag.cmp(&b.term_tag));
        self.header.term_count = u32::try_from(self.dictionary.len()).unwrap_or(u32::MAX);

        let mut payload = Vec::new();
        payload.extend_from_slice(&self.header.encode());

        let dict_len = u32::try_from(self.dictionary.len()).unwrap_or(u32::MAX);
        payload.extend_from_slice(&dict_len.to_be_bytes());
        for entry in &self.dictionary {
            payload.extend_from_slice(&entry.term_tag);
            payload.extend_from_slice(&entry.df.to_be_bytes());
            payload.extend_from_slice(&entry.posting_off.to_be_bytes());
            payload.extend_from_slice(&entry.posting_len.to_be_bytes());
        }

        let doc_count = u32::try_from(self.docs.len()).unwrap_or(u32::MAX);
        payload.extend_from_slice(&doc_count.to_be_bytes());
        for doc in &self.docs {
            payload.extend_from_slice(&doc.doc_ord.to_be_bytes());
            payload.extend_from_slice(&doc.doclen.to_be_bytes());
            let env_len = u16::try_from(doc.binding_envelope.len()).unwrap_or(u16::MAX);
            payload.extend_from_slice(&env_len.to_be_bytes());
            payload.extend_from_slice(&doc.binding_envelope);
        }

        let postings_count = u32::try_from(self.postings.len()).unwrap_or(u32::MAX);
        payload.extend_from_slice(&postings_count.to_be_bytes());
        let has_positions = (self.header.flags & 0x01) != 0;
        for block in &self.postings {
            let enc = block.encode(has_positions);
            let len = u32::try_from(enc.len()).unwrap_or(u32::MAX);
            payload.extend_from_slice(&len.to_be_bytes());
            payload.extend_from_slice(&enc);
        }

        let bitmap_len = (self.header.doc_count as usize).div_ceil(8);
        let bitmap_len_u32 = u32::try_from(bitmap_len).unwrap_or(u32::MAX);
        payload.extend_from_slice(&bitmap_len_u32.to_be_bytes());
        payload.resize(payload.len() + bitmap_len, 0);

        payload
    }

    pub fn seal_to_bytes(&mut self, file_id: [u8; 16], file_key: FileKey) -> Result<Vec<u8>> {
        let mut sealer = BlockSealer::new(file_key, FileKind::TextSegment, file_id, None);
        let mut out = Vec::new();

        let mut head48 = [0u8; 48];
        head48[0..8].copy_from_slice(MAGIC);
        head48[8..10].copy_from_slice(&FORMAT_VERSION.to_be_bytes());
        head48[10] = FILE_KIND_TEXT;
        head48[11..15].copy_from_slice(&ALG_IDS);
        head48[15..31].copy_from_slice(&file_id);
        head48[31..39].copy_from_slice(&0u64.to_be_bytes());
        head48[39] = 0;

        let tag = sealer.header_tag(&head48);
        out.extend_from_slice(&head48);
        out.extend_from_slice(&tag);

        let plain = self.serialize_payload();
        let block = sealer.seal(&plain).map_err(|_| ContextraError::Index("Block seal failed".into()))?;

        let ct_len = u32::try_from(block.ciphertext.len()).unwrap_or(u32::MAX);
        out.extend_from_slice(&ct_len.to_be_bytes());
        out.extend_from_slice(&block.ciphertext);
        out.extend_from_slice(&block.tag);

        Ok(out)
    }
}

pub struct SegmentReader {
    pub header: SegmentHeader,
    pub dictionary: Vec<DictEntry>,
    pub docs: Vec<DocEntry>,
    pub postings: Vec<PostingBlock>,
    pub delete_bitmap: Vec<u8>,
}

impl SegmentReader {
    pub fn open_from_bytes(bytes: &[u8], file_id: [u8; 16], file_key: FileKey) -> Result<Self> {
        if bytes.len() < 64 + 4 + 16 {
            return Err(ContextraError::Index("Segment file too small".into()));
        }

        let magic = bytes.get(0..8).ok_or_else(|| ContextraError::Index("Corrupted magic".into()))?;
        if magic != MAGIC {
            return Err(ContextraError::Index("Invalid magic".into()));
        }
        let kind = bytes.get(10).copied().ok_or_else(|| ContextraError::Index("Corrupted kind".into()))?;
        if kind != FILE_KIND_TEXT {
            return Err(ContextraError::Index("Invalid file kind".into()));
        }

        let mut offset = 64;
        let ct_len = u32::from_be_bytes([
            *bytes.get(offset).ok_or_else(|| ContextraError::Index("Corrupted offset".into()))?,
            *bytes.get(offset + 1).ok_or_else(|| ContextraError::Index("Corrupted offset".into()))?,
            *bytes.get(offset + 2).ok_or_else(|| ContextraError::Index("Corrupted offset".into()))?,
            *bytes.get(offset + 3).ok_or_else(|| ContextraError::Index("Corrupted offset".into()))?,
        ]) as usize;
        offset += 4;

        let ct = bytes.get(offset..offset + ct_len).ok_or_else(|| ContextraError::Index("Corrupted ct".into()))?;
        offset += ct_len;
        let tag = bytes.get(offset..offset + 16).ok_or_else(|| ContextraError::Index("Corrupted tag".into()))?;

        use aes_gcm_siv::aead::{Aead, KeyInit, Payload};
        use aes_gcm_siv::{Aes256GcmSiv, Key, Nonce};

        let cipher = Aes256GcmSiv::new(Key::<Aes256GcmSiv>::from_slice(file_key.as_bytes()));
        let nonce = [0u8; 12];
        let mut aad = Vec::with_capacity(1 + 16 + 8);
        aad.push(FileKind::TextSegment as u8);
        aad.extend_from_slice(&file_id);
        aad.extend_from_slice(&0u64.to_be_bytes());

        let mut ct_and_tag = ct.to_vec();
        ct_and_tag.extend_from_slice(tag);

        let plain = cipher
            .decrypt(
                Nonce::from_slice(&nonce),
                Payload {
                    msg: &ct_and_tag,
                    aad: &aad,
                },
            )
            .map_err(|_| ContextraError::Index("Block decryption failed".into()))?;

        if plain.len() < 32 {
            return Err(ContextraError::Index("Plaintext too small".into()));
        }
        let mut head_buf = [0u8; 32];
        if let Some(src) = plain.get(0..32) {
            head_buf.copy_from_slice(src);
        }
        let header = SegmentHeader::decode(&head_buf);

        let mut p_off = 32;
        let term_count = u32::from_be_bytes([
            *plain.get(p_off).ok_or_else(|| ContextraError::Index("Corrupted p_off".into()))?,
            *plain.get(p_off + 1).ok_or_else(|| ContextraError::Index("Corrupted p_off".into()))?,
            *plain.get(p_off + 2).ok_or_else(|| ContextraError::Index("Corrupted p_off".into()))?,
            *plain.get(p_off + 3).ok_or_else(|| ContextraError::Index("Corrupted p_off".into()))?,
        ]) as usize;
        p_off += 4;

        let mut dictionary = Vec::with_capacity(term_count);
        for _ in 0..term_count {
            let mut term_tag = [0u8; 16];
            let tag_slice = plain.get(p_off..p_off + 16).ok_or_else(|| ContextraError::Index("Corrupted tag".into()))?;
            term_tag.copy_from_slice(tag_slice);
            p_off += 16;

            let df = u32::from_be_bytes([
                *plain.get(p_off).ok_or_else(|| ContextraError::Index("Corrupted df".into()))?,
                *plain.get(p_off + 1).ok_or_else(|| ContextraError::Index("Corrupted df".into()))?,
                *plain.get(p_off + 2).ok_or_else(|| ContextraError::Index("Corrupted df".into()))?,
                *plain.get(p_off + 3).ok_or_else(|| ContextraError::Index("Corrupted df".into()))?,
            ]);
            p_off += 4;

            let posting_off = u64::from_be_bytes([
                *plain.get(p_off).ok_or_else(|| ContextraError::Index("Corrupted posting_off".into()))?,
                *plain.get(p_off + 1).ok_or_else(|| ContextraError::Index("Corrupted posting_off".into()))?,
                *plain.get(p_off + 2).ok_or_else(|| ContextraError::Index("Corrupted posting_off".into()))?,
                *plain.get(p_off + 3).ok_or_else(|| ContextraError::Index("Corrupted posting_off".into()))?,
                *plain.get(p_off + 4).ok_or_else(|| ContextraError::Index("Corrupted posting_off".into()))?,
                *plain.get(p_off + 5).ok_or_else(|| ContextraError::Index("Corrupted posting_off".into()))?,
                *plain.get(p_off + 6).ok_or_else(|| ContextraError::Index("Corrupted posting_off".into()))?,
                *plain.get(p_off + 7).ok_or_else(|| ContextraError::Index("Corrupted posting_off".into()))?,
            ]);
            p_off += 8;

            let posting_len = u32::from_be_bytes([
                *plain.get(p_off).ok_or_else(|| ContextraError::Index("Corrupted posting_len".into()))?,
                *plain.get(p_off + 1).ok_or_else(|| ContextraError::Index("Corrupted posting_len".into()))?,
                *plain.get(p_off + 2).ok_or_else(|| ContextraError::Index("Corrupted posting_len".into()))?,
                *plain.get(p_off + 3).ok_or_else(|| ContextraError::Index("Corrupted posting_len".into()))?,
            ]);
            p_off += 4;

            dictionary.push(DictEntry {
                term_tag,
                df,
                posting_off,
                posting_len,
            });
        }

        let doc_count = u32::from_be_bytes([
            *plain.get(p_off).ok_or_else(|| ContextraError::Index("Corrupted doc_count".into()))?,
            *plain.get(p_off + 1).ok_or_else(|| ContextraError::Index("Corrupted doc_count".into()))?,
            *plain.get(p_off + 2).ok_or_else(|| ContextraError::Index("Corrupted doc_count".into()))?,
            *plain.get(p_off + 3).ok_or_else(|| ContextraError::Index("Corrupted doc_count".into()))?,
        ]) as usize;
        p_off += 4;

        let mut docs = Vec::with_capacity(doc_count);
        for _ in 0..doc_count {
            let doc_ord = u32::from_be_bytes([
                *plain.get(p_off).ok_or_else(|| ContextraError::Index("Corrupted doc_ord".into()))?,
                *plain.get(p_off + 1).ok_or_else(|| ContextraError::Index("Corrupted doc_ord".into()))?,
                *plain.get(p_off + 2).ok_or_else(|| ContextraError::Index("Corrupted doc_ord".into()))?,
                *plain.get(p_off + 3).ok_or_else(|| ContextraError::Index("Corrupted doc_ord".into()))?,
            ]);
            p_off += 4;

            let doclen = u32::from_be_bytes([
                *plain.get(p_off).ok_or_else(|| ContextraError::Index("Corrupted doclen".into()))?,
                *plain.get(p_off + 1).ok_or_else(|| ContextraError::Index("Corrupted doclen".into()))?,
                *plain.get(p_off + 2).ok_or_else(|| ContextraError::Index("Corrupted doclen".into()))?,
                *plain.get(p_off + 3).ok_or_else(|| ContextraError::Index("Corrupted doclen".into()))?,
            ]);
            p_off += 4;

            let env_len = u16::from_be_bytes([
                *plain.get(p_off).ok_or_else(|| ContextraError::Index("Corrupted env_len".into()))?,
                *plain.get(p_off + 1).ok_or_else(|| ContextraError::Index("Corrupted env_len".into()))?,
            ]) as usize;
            p_off += 2;

            let binding_envelope = plain.get(p_off..p_off + env_len).ok_or_else(|| ContextraError::Index("Corrupted env".into()))?.to_vec();
            p_off += env_len;

            docs.push(DocEntry {
                doc_ord,
                doclen,
                binding_envelope,
            });
        }

        let postings_count = u32::from_be_bytes([
            *plain.get(p_off).ok_or_else(|| ContextraError::Index("Corrupted postings_count".into()))?,
            *plain.get(p_off + 1).ok_or_else(|| ContextraError::Index("Corrupted postings_count".into()))?,
            *plain.get(p_off + 2).ok_or_else(|| ContextraError::Index("Corrupted postings_count".into()))?,
            *plain.get(p_off + 3).ok_or_else(|| ContextraError::Index("Corrupted postings_count".into()))?,
        ]) as usize;
        p_off += 4;

        let has_positions = (header.flags & 0x01) != 0;
        let mut postings = Vec::with_capacity(postings_count);
        for _ in 0..postings_count {
            let block_len = u32::from_be_bytes([
                *plain.get(p_off).ok_or_else(|| ContextraError::Index("Corrupted block_len".into()))?,
                *plain.get(p_off + 1).ok_or_else(|| ContextraError::Index("Corrupted block_len".into()))?,
                *plain.get(p_off + 2).ok_or_else(|| ContextraError::Index("Corrupted block_len".into()))?,
                *plain.get(p_off + 3).ok_or_else(|| ContextraError::Index("Corrupted block_len".into()))?,
            ]) as usize;
            p_off += 4;
            let block_bytes = plain.get(p_off..p_off + block_len).ok_or_else(|| ContextraError::Index("Corrupted block".into()))?;
            p_off += block_len;
            let (block, _) = PostingBlock::decode(block_bytes, has_positions)?;
            postings.push(block);
        }

        let bitmap_len = u32::from_be_bytes([
            *plain.get(p_off).ok_or_else(|| ContextraError::Index("Corrupted bitmap_len".into()))?,
            *plain.get(p_off + 1).ok_or_else(|| ContextraError::Index("Corrupted bitmap_len".into()))?,
            *plain.get(p_off + 2).ok_or_else(|| ContextraError::Index("Corrupted bitmap_len".into()))?,
            *plain.get(p_off + 3).ok_or_else(|| ContextraError::Index("Corrupted bitmap_len".into()))?,
        ]) as usize;
        p_off += 4;

        let delete_bitmap = plain.get(p_off..p_off + bitmap_len).ok_or_else(|| ContextraError::Index("Corrupted bitmap".into()))?.to_vec();

        Ok(Self {
            header,
            dictionary,
            docs,
            postings,
            delete_bitmap,
        })
    }
}

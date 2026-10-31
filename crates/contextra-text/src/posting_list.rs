// FILE-CONTEXT: In-Memory Resident Posting List & Index (IP-10a).
// ZWECK: Stellt kompakte, resident gehaltene Postinglisten pro Term bereit.
// CONSISTENCY CONTRACT:
// 1. Source of Truth ist und bleibt die LSM StorageEngine (Persistenz, Crash Recovery).
// 2. Der ResidentPostingIndex dient als In-Memory Read-Cache zur Vermeidung teurer LSM scan_prefix_at Scans.
// 3. Updates (upsert/delete) werden per RCU (Read-Copy-Update) inkrementell in den residenten Index eingepflegt.
// 4. Bei Cache-Misses (z.B. nach Neustart) werden Postinglisten lazy aus dem LSM Store geladen.
// 5. MVCC-Sichtbarkeit und Tombstones werden weiterhin dynamisch pro Anfrage evaluiert.
// INVARIANTEN: Zero-Panic Doctrine, repr(C, align(8)) Layout für Postings, RCU-Locking ohne globale Exklusivlocks.

//! Kompakte, speicherresidente Posting-Listen und Block-Max-Metadaten.
//!
//! # Ring-Zuordnung
//! Ring 0 (Kern).
//!
//! # Funktionsweise
//! Verwaltet Invertierte Listen pro Term mit Delta-Varint-Kompression und
//! Block-Max-Zusammenfassungen (à 64 Einträge) zur Beschleunigung des WAND-Algorithmus.
//! Aktualisierungen erfolgen über RCU (Read-Copy-Update) für sperrenfreie Lesezugriffe.

use contextra_types::{ContextraError, DocId};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// ANCHOR[contextra-text:POST-001]
/// Magische Bytes für das kompakte V2-Posting-Listen-Format (`PL\x02\x00`).
pub const POSTING_LIST_V2_MAGIC: &[u8; 4] = b"PL\x02\x00";

// ANCHOR[contextra-text:POST-002]
/// Blockgröße für die Block-Max-WAND-Zerlegung (64 Postings pro Block).
pub const BLOCK_SIZE: usize = 64;

#[cfg(not(feature = "docid-128"))]
pub type DocIdRaw = u64;

#[cfg(feature = "docid-128")]
pub type DocIdRaw = u128;

// ANCHOR[contextra-text:POST-003]
/// Metadaten eines 64er-Posting-Blocks zur Berechnung oberer BM25-Schranken in Block-Max WAND.
#[repr(C, align(8))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PostingBlockInfo {
    /// Höchste Dokument-ID in diesem Block (ermöglicht schnelles Überspringen).
    pub max_doc_id: DocIdRaw,
    /// Maximale Termfrequenz innerhalb dieses Blocks.
    pub max_tf: u32,
    /// Minimale Dokumentlänge innerhalb dieses Blocks (maximiert die BM25-Obergrenze).
    pub min_doc_len: u32,
}

// ANCHOR[contextra-text:POST-004]
/// Kompakte Repräsentation eines einzelnen Vorkommens (Posting) im Index.
///
/// 8-Byte ausgerichtet (`repr(C, align(8))`) für optimale CPU-Cacheline-Ausnutzung (16 Bytes Gesamtgröße).
#[repr(C, align(8))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Posting {
    /// Dokument-ID des Eintrags.
    pub doc_id: DocIdRaw,
    /// Termfrequenz (Häufigkeit des Terms in diesem Dokument).
    pub tf: u32,
    /// Gesamtlänge des Dokuments in Token.
    pub doc_len: u32,
}

impl Posting {
    #[inline]
    pub fn new(doc_id: DocId, tf: u32, doc_len: u32) -> Self {
        Self {
            doc_id: doc_id.inner(),
            tf,
            doc_len,
        }
    }

    #[inline]
    pub fn doc_id(&self) -> DocId {
        DocId::new(self.doc_id)
    }
}

/// Computes Block-Max metadata for a sequence of postings.
fn compute_blocks(postings: &[Posting]) -> Vec<PostingBlockInfo> {
    if postings.is_empty() {
        return Vec::new();
    }
    let mut blocks = Vec::with_capacity(postings.len().div_ceil(BLOCK_SIZE));
    for chunk in postings.chunks(BLOCK_SIZE) {
        let max_doc_id = chunk.last().map(|p| p.doc_id).unwrap_or(0);
        let mut max_tf = 0u32;
        let mut min_doc_len = u32::MAX;
        for p in chunk {
            if p.tf > max_tf {
                max_tf = p.tf;
            }
            if p.doc_len < min_doc_len {
                min_doc_len = p.doc_len;
            }
        }
        if min_doc_len == u32::MAX {
            min_doc_len = 0;
        }
        blocks.push(PostingBlockInfo {
            max_doc_id,
            max_tf,
            min_doc_len,
        });
    }
    blocks
}

// ANCHOR[contextra-text:POST-005]
/// Zusammenhängende, nach DocId sortierte Posting-Liste für einen Begriff.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PostingList {
    postings: Vec<Posting>,
    blocks: Vec<PostingBlockInfo>,
}

impl PostingList {
    // ANCHOR[contextra-text:POST-006]
    /// Erzeugt eine sortierte und deduplizierte Posting-Liste und berechnet die Block-Metadaten.
    pub fn new(mut postings: Vec<Posting>) -> Self {
        postings.sort_unstable_by_key(|p| p.doc_id);
        postings.dedup_by_key(|p| p.doc_id);
        let blocks = compute_blocks(&postings);
        Self { postings, blocks }
    }

    // ANCHOR[contextra-text:POST-007]
    /// Erzeugt eine leere Posting-Liste.
    pub fn empty() -> Self {
        Self {
            postings: Vec::new(),
            blocks: Vec::new(),
        }
    }

    // ANCHOR[contextra-text:POST-008]
    /// Liefert die Postings als Slice.
    #[inline]
    pub fn as_slice(&self) -> &[Posting] {
        &self.postings
    }

    // ANCHOR[contextra-text:POST-009]
    /// Liefert die Block-Max-Metadaten.
    #[inline]
    pub fn blocks(&self) -> &[PostingBlockInfo] {
        &self.blocks
    }

    // ANCHOR[contextra-text:POST-010]
    /// Liefert die Anzahl der Einträge in der Liste.
    #[inline]
    pub fn len(&self) -> usize {
        self.postings.len()
    }

    // ANCHOR[contextra-text:POST-011]
    /// Prüft, ob die Liste leer ist.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.postings.is_empty()
    }

    // ANCHOR[contextra-text:POST-012]
    /// Fügt ein Posting per Copy-On-Write (RCU) ein oder aktualisiert einen bestehenden Eintrag.
    pub fn upsert(&self, posting: Posting) -> Self {
        let mut new_postings = self.postings.clone();
        match new_postings.binary_search_by_key(&posting.doc_id, |p| p.doc_id) {
            Ok(idx) => {
                new_postings[idx] = posting;
            }
            Err(idx) => {
                new_postings.insert(idx, posting);
            }
        }
        let blocks = compute_blocks(&new_postings);
        Self {
            postings: new_postings,
            blocks,
        }
    }

    // ANCHOR[contextra-text:POST-013]
    /// Entfernt ein Posting per Copy-On-Write (RCU).
    pub fn remove(&self, doc_id: DocId) -> Self {
        let mut new_postings = self.postings.clone();
        if let Ok(idx) = new_postings.binary_search_by_key(&doc_id.inner(), |p| p.doc_id) {
            new_postings.remove(idx);
        }
        let blocks = compute_blocks(&new_postings);
        Self {
            postings: new_postings,
            blocks,
        }
    }

    // ANCHOR[contextra-text:POST-014]
    /// Kodiert die Posting-Liste in das kompakte Binärformat mit Delta-Varint-Kompression.
    pub fn encode_compact(&self) -> Result<Vec<u8>, ContextraError> {
        let mut buf = Vec::with_capacity(4 + 8 + self.postings.len() * 8);
        buf.extend_from_slice(POSTING_LIST_V2_MAGIC);
        encode_varint(self.postings.len() as u64, &mut buf);

        let mut prev_doc_id = 0u64;
        for p in &self.postings {
            let delta = (p.doc_id as u128)
                .checked_sub(prev_doc_id as u128)
                .ok_or_else(|| {
                    ContextraError::Storage(
                        "Unsorted or overflow doc_id in posting list".to_string(),
                    )
                })? as u64;
            encode_varint(delta, &mut buf);
            encode_varint(p.tf as u64, &mut buf);
            encode_varint(p.doc_len as u64, &mut buf);
            #[cfg(not(feature = "docid-128"))]
            {
                prev_doc_id = p.doc_id;
            }
            #[cfg(feature = "docid-128")]
            {
                prev_doc_id = p.doc_id as u64;
            }
        }

        Ok(buf)
    }

    // ANCHOR[contextra-text:POST-015]
    /// Dekodiert eine Posting-Liste aus dem Delta-Varint-Format mit Fallback auf Legacy-Bincode.
    pub fn decode_compact(bytes: &[u8]) -> Result<Self, ContextraError> {
        if bytes.starts_with(POSTING_LIST_V2_MAGIC) {
            let mut offset = POSTING_LIST_V2_MAGIC.len();
            let count = decode_varint(bytes, &mut offset)?;
            if count > 100_000_000 {
                return Err(ContextraError::Storage(format!(
                    "Oversized posting list count: {}",
                    count
                )));
            }
            let mut postings = Vec::with_capacity(count as usize);
            let mut prev_doc_id = 0u64;

            for _ in 0..count {
                let delta = decode_varint(bytes, &mut offset)?;
                let tf_u64 = decode_varint(bytes, &mut offset)?;
                let doc_len_u64 = decode_varint(bytes, &mut offset)?;

                let doc_id = prev_doc_id.checked_add(delta).ok_or_else(|| {
                    ContextraError::Storage("DocId overflow during delta decoding".to_string())
                })?;
                let tf: u32 = tf_u64
                    .try_into()
                    .map_err(|_| ContextraError::Storage("TF overflow in varint".to_string()))?;
                let doc_len: u32 = doc_len_u64.try_into().map_err(|_| {
                    ContextraError::Storage("DocLen overflow in varint".to_string())
                })?;

                postings.push(Posting {
                    doc_id: doc_id as DocIdRaw,
                    tf,
                    doc_len,
                });
                prev_doc_id = doc_id;
            }

            if offset != bytes.len() {
                return Err(ContextraError::Storage(format!(
                    "Extra trailing bytes after posting list decoding: {} remaining",
                    bytes.len() - offset
                )));
            }

            let blocks = compute_blocks(&postings);
            Ok(Self { postings, blocks })
        } else {
            // Legacy V1 bincode fallback
            bincode::deserialize::<PostingList>(bytes)
                .map_err(|e| ContextraError::Storage(format!("bincode legacy decode error: {}", e)))
        }
    }
}

fn encode_varint(mut val: u64, buf: &mut Vec<u8>) {
    while val >= 0x80 {
        buf.push((val as u8 & 0x7f) | 0x80);
        val >>= 7;
    }
    buf.push(val as u8);
}

fn decode_varint(bytes: &[u8], offset: &mut usize) -> Result<u64, ContextraError> {
    let mut result = 0u64;
    let mut shift = 0u32;
    while *offset < bytes.len() {
        let byte = bytes[*offset];
        *offset += 1;
        result |= ((byte & 0x7f) as u64) << shift;
        if (byte & 0x80) == 0 {
            return Ok(result);
        }
        shift += 7;
        if shift >= 64 {
            return Err(ContextraError::Storage("Varint overflow".to_string()));
        }
    }
    Err(ContextraError::Storage(
        "Unexpected EOF while decoding varint".to_string(),
    ))
}

// ANCHOR[contextra-text:POST-016]
/// Speicherresidenter Lese-Cache für Posting-Listen mit RCU-Verwaltung.
#[derive(Default)]
pub struct ResidentPostingIndex {
    index: RwLock<HashMap<String, Arc<PostingList>>>,
}

impl ResidentPostingIndex {
    // ANCHOR[contextra-text:POST-017]
    /// Erzeugt einen leeren residenten Posting-Index.
    pub fn new() -> Self {
        Self {
            index: RwLock::new(HashMap::new()),
        }
    }

    // ANCHOR[contextra-text:POST-018]
    /// Liefert eine referenzgezählte Posting-Liste für den Begriff, falls im Cache vorhanden.
    pub fn get(&self, term: &str) -> Option<Arc<PostingList>> {
        self.index.read().get(term).cloned()
    }

    // ANCHOR[contextra-text:POST-019]
    /// Fügt eine Posting-Liste in den Index ein oder ersetzt sie.
    pub fn insert_list(&self, term: String, list: PostingList) {
        self.index.write().insert(term, Arc::new(list));
    }

    // ANCHOR[contextra-text:POST-020]
    /// Fügt ein einzelnes Posting per RCU (Copy-On-Write) in die Posting-Liste des Begriffs ein.
    pub fn upsert_posting(&self, term: &str, posting: Posting) {
        let mut guard = self.index.write();
        let existing = guard.get(term).cloned();
        let updated = match existing {
            Some(list) => list.upsert(posting),
            None => PostingList::new(vec![posting]),
        };
        guard.insert(term.to_string(), Arc::new(updated));
    }

    // ANCHOR[contextra-text:POST-021]
    /// Entfernt eine Dokument-ID aus den Posting-Listen mehrerer Begriffe per RCU.
    pub fn remove_posting_from_terms(&self, terms: &[String], doc_id: DocId) {
        let mut guard = self.index.write();
        for term in terms {
            if let Some(list) = guard.get(term).cloned() {
                let updated = list.remove(doc_id);
                if updated.is_empty() {
                    guard.remove(term);
                } else {
                    guard.insert(term.clone(), Arc::new(updated));
                }
            }
        }
    }

    // ANCHOR[contextra-text:POST-022]
    /// Entfernt die angegebenen Begriffe aus dem Cache (z. B. bei Rollback).
    pub fn remove_terms(&self, terms: &[String]) {
        let mut guard = self.index.write();
        for term in terms {
            guard.remove(term);
        }
    }

    // ANCHOR[contextra-text:POST-023]
    /// Leert den gesamten Cache (z. B. bei vollständigem Index-Neuladen).
    pub fn clear(&self) {
        self.index.write().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR[contextra-text:TEST-POST-001]
    // Prüft, dass PostingList Eingaben automatisch nach DocId sortiert und Duplikate dedupliziert.
    #[test]
    fn test_posting_list_sorted_and_dedup() {
        let p1 = Posting::new(DocId::new(10), 2, 100);
        let p2 = Posting::new(DocId::new(5), 1, 50);
        let p3 = Posting::new(DocId::new(10), 3, 120);

        let list = PostingList::new(vec![p1, p2, p3]);
        assert_eq!(list.len(), 2);
        assert_eq!(list.as_slice()[0].doc_id(), DocId::new(5));
        assert_eq!(list.as_slice()[1].doc_id(), DocId::new(10));
    }

    #[test]
    fn test_posting_list_upsert_and_remove() {
        let list = PostingList::empty();
        let list = list.upsert(Posting::new(DocId::new(1), 2, 20));
        let list = list.upsert(Posting::new(DocId::new(3), 1, 15));
        let list = list.upsert(Posting::new(DocId::new(2), 4, 30));

        assert_eq!(list.len(), 3);
        assert_eq!(list.as_slice()[0].doc_id(), DocId::new(1));
        assert_eq!(list.as_slice()[1].doc_id(), DocId::new(2));
        assert_eq!(list.as_slice()[2].doc_id(), DocId::new(3));

        // Update doc 2
        let list = list.upsert(Posting::new(DocId::new(2), 5, 35));
        assert_eq!(list.len(), 3);
        assert_eq!(list.as_slice()[1].tf, 5);

        // Remove doc 1
        let list = list.remove(DocId::new(1));
        assert_eq!(list.len(), 2);
        assert_eq!(list.as_slice()[0].doc_id(), DocId::new(2));
    }

    #[test]
    fn test_resident_posting_index_rcu() {
        let index = ResidentPostingIndex::new();
        let term = "search".to_string();

        index.upsert_posting(&term, Posting::new(DocId::new(100), 1, 10));
        index.upsert_posting(&term, Posting::new(DocId::new(50), 2, 20));

        let plist = index.get(&term).unwrap();
        assert_eq!(plist.len(), 2);
        assert_eq!(plist.as_slice()[0].doc_id(), DocId::new(50));
        assert_eq!(plist.as_slice()[1].doc_id(), DocId::new(100));

        index.remove_posting_from_terms(std::slice::from_ref(&term), DocId::new(50));
        let plist_after = index.get(&term).unwrap();
        assert_eq!(plist_after.len(), 1);
        assert_eq!(plist_after.as_slice()[0].doc_id(), DocId::new(100));

        index.remove_posting_from_terms(std::slice::from_ref(&term), DocId::new(100));
        assert!(index.get(&term).is_none());
    }

    #[test]
    fn test_delta_varint_encode_decode_roundtrip() {
        let p1 = Posting::new(DocId::new(10), 2, 100);
        let p2 = Posting::new(DocId::new(100), 5, 250);
        let p3 = Posting::new(DocId::new(1000), 1, 50);

        let plist = PostingList::new(vec![p1, p2, p3]);
        let bytes = plist.encode_compact().expect("encoding should succeed");
        assert!(bytes.starts_with(POSTING_LIST_V2_MAGIC));

        let decoded = PostingList::decode_compact(&bytes).expect("decoding should succeed");
        assert_eq!(decoded.len(), 3);
        assert_eq!(decoded.as_slice(), plist.as_slice());
        assert_eq!(decoded.blocks(), plist.blocks());
    }

    #[test]
    fn test_delta_varint_encode_decode_comprehensive_roundtrip() {
        // Test 1: Empty posting list
        let empty_list = PostingList::empty();
        let empty_bytes = empty_list.encode_compact().expect("encode empty");
        let decoded_empty = PostingList::decode_compact(&empty_bytes).expect("decode empty");
        assert_eq!(decoded_empty.len(), 0);
        assert!(decoded_empty.is_empty());
        assert!(decoded_empty.blocks().is_empty());

        // Test 2: Single posting
        let single_list = PostingList::new(vec![Posting::new(DocId::new(42), 7, 120)]);
        let single_bytes = single_list.encode_compact().expect("encode single");
        let decoded_single = PostingList::decode_compact(&single_bytes).expect("decode single");
        assert_eq!(decoded_single.len(), 1);
        assert_eq!(decoded_single.as_slice()[0].doc_id(), DocId::new(42));
        assert_eq!(decoded_single.as_slice()[0].tf, 7);
        assert_eq!(decoded_single.as_slice()[0].doc_len, 120);

        // Test 3: Multi-block posting list across block boundaries (> BLOCK_SIZE)
        let count = BLOCK_SIZE * 3 + 15; // 207 postings
        let mut postings = Vec::with_capacity(count);
        let mut curr_doc_id = 5u64;
        for i in 0..count {
            // Mix small delta (+1) and large delta (+1000)
            let delta = if i % 5 == 0 { 1000 } else { 1 };
            curr_doc_id += delta;
            let tf = ((i % 20) + 1) as u32;
            let doc_len = (50 + (i % 100)) as u32;
            postings.push(Posting::new(DocId::new(curr_doc_id), tf, doc_len));
        }

        let multi_list = PostingList::new(postings);
        let multi_bytes = multi_list.encode_compact().expect("encode multi-block");
        assert!(multi_bytes.starts_with(POSTING_LIST_V2_MAGIC));

        let decoded_multi = PostingList::decode_compact(&multi_bytes).expect("decode multi-block");
        assert_eq!(decoded_multi.len(), count);
        assert_eq!(decoded_multi.as_slice(), multi_list.as_slice());
        assert_eq!(decoded_multi.blocks(), multi_list.blocks());

        // Verify block-max metadata upper bounds match decoded postings
        let blocks = decoded_multi.blocks();
        assert_eq!(blocks.len(), count.div_ceil(BLOCK_SIZE));
        for (idx, block) in blocks.iter().enumerate() {
            let start = idx * BLOCK_SIZE;
            let end = (start + BLOCK_SIZE).min(count);
            let chunk = &decoded_multi.as_slice()[start..end];

            let chunk_max_doc = chunk.last().unwrap().doc_id;
            let chunk_max_tf = chunk.iter().map(|p| p.tf).max().unwrap();
            let chunk_min_dl = chunk.iter().map(|p| p.doc_len).min().unwrap();

            assert_eq!(block.max_doc_id, chunk_max_doc);
            assert_eq!(block.max_tf, chunk_max_tf);
            assert_eq!(block.min_doc_len, chunk_min_dl);
        }
    }

    #[test]
    fn test_delta_varint_legacy_bincode_fallback() {
        let p1 = Posting::new(DocId::new(10), 2, 100);
        let p2 = Posting::new(DocId::new(100), 5, 250);
        let plist = PostingList::new(vec![p1, p2]);

        let legacy_bytes = bincode::serialize(&plist).expect("bincode serialize");
        assert!(!legacy_bytes.starts_with(POSTING_LIST_V2_MAGIC));

        let decoded = PostingList::decode_compact(&legacy_bytes).expect("fallback decode");
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded.as_slice(), plist.as_slice());
    }

    #[test]
    fn test_delta_varint_corrupted_payload_no_panic() {
        let invalid_bytes = b"PL\x02\x00\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff";
        let res = PostingList::decode_compact(invalid_bytes);
        assert!(res.is_err());
    }
}

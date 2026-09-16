// FILE-CONTEXT: In-Memory Resident Posting List & Index (IP-10a).
// ZWECK: Stellt kompakte, resident gehaltene Postinglisten pro Term bereit.
// CONSISTENCY CONTRACT:
// 1. Source of Truth ist und bleibt die LSM StorageEngine (Persistenz, Crash Recovery).
// 2. Der ResidentPostingIndex dient als In-Memory Read-Cache zur Vermeidung teurer LSM scan_prefix_at Scans.
// 3. Updates (upsert/delete) werden per RCU (Read-Copy-Update) inkrementell in den residenten Index eingepflegt.
// 4. Bei Cache-Misses (z.B. nach Neustart) werden Postinglisten lazy aus dem LSM Store geladen.
// 5. MVCC-Sichtbarkeit und Tombstones werden weiterhin dynamisch pro Anfrage evaluiert.
// INVARIANTEN: Zero-Panic Doctrine, repr(C, align(8)) Layout für Postings, RCU-Locking ohne globale Exklusivlocks.

use memfuse_core::DocId;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Compact representation of a single posting in a posting list.
/// Packed with 8-byte alignment (16 bytes total) for optimal CPU cache alignment.
#[repr(C, align(8))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Posting {
    pub doc_id: u64,
    pub tf: u32,
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

/// A contiguous, sorted sequence of postings for a specific term.
#[derive(Debug, Clone, Default)]
pub struct PostingList {
    postings: Vec<Posting>,
}

impl PostingList {
    /// Creates a new `PostingList` from a vector of postings, ensuring sorted order by DocId.
    pub fn new(mut postings: Vec<Posting>) -> Self {
        postings.sort_unstable_by_key(|p| p.doc_id);
        postings.dedup_by_key(|p| p.doc_id);
        Self { postings }
    }

    /// Creates an empty `PostingList`.
    pub fn empty() -> Self {
        Self {
            postings: Vec::new(),
        }
    }

    /// Returns a slice of postings.
    #[inline]
    pub fn as_slice(&self) -> &[Posting] {
        &self.postings
    }

    /// Returns the number of postings.
    #[inline]
    pub fn len(&self) -> usize {
        self.postings.len()
    }

    /// Returns true if the posting list is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.postings.is_empty()
    }

    /// Inserts or updates a posting for a doc_id using RCU/copy-on-write semantics.
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
        Self {
            postings: new_postings,
        }
    }

    /// Removes a posting for a doc_id using RCU/copy-on-write semantics.
    pub fn remove(&self, doc_id: DocId) -> Self {
        let mut new_postings = self.postings.clone();
        if let Ok(idx) = new_postings.binary_search_by_key(&doc_id.inner(), |p| p.doc_id) {
            new_postings.remove(idx);
        }
        Self {
            postings: new_postings,
        }
    }
}

/// In-Memory resident index mapping terms to their posting lists.
/// Uses RCU (Read-Copy-Update) via `Arc<PostingList>` for lock-free read access under `RwLock`.
#[derive(Default)]
pub struct ResidentPostingIndex {
    index: RwLock<HashMap<String, Arc<PostingList>>>,
}

impl ResidentPostingIndex {
    pub fn new() -> Self {
        Self {
            index: RwLock::new(HashMap::new()),
        }
    }

    /// Retrieves an Arc to the PostingList for a given term if cached.
    pub fn get(&self, term: &str) -> Option<Arc<PostingList>> {
        self.index.read().get(term).cloned()
    }

    /// Inserts or replaces a PostingList for a given term.
    pub fn insert_list(&self, term: String, list: PostingList) {
        self.index.write().insert(term, Arc::new(list));
    }

    /// Upserts a single posting for a term using RCU (Copy-On-Write).
    pub fn upsert_posting(&self, term: &str, posting: Posting) {
        let mut guard = self.index.write();
        let existing = guard.get(term).cloned();
        let updated = match existing {
            Some(list) => list.upsert(posting),
            None => PostingList::new(vec![posting]),
        };
        guard.insert(term.to_string(), Arc::new(updated));
    }

    /// Removes a posting for a doc_id across a list of terms using RCU.
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

    /// Removes specified terms from the in-memory cache (e.g., on transaction rollback).
    pub fn remove_terms(&self, terms: &[String]) {
        let mut guard = self.index.write();
        for term in terms {
            guard.remove(term);
        }
    }

    /// Clears the in-memory cache (e.g., on full re-sync).
    pub fn clear(&self) {
        self.index.write().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

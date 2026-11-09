//! Fault Injection and Input Limits Test for `MemoryTextIndex` (INV-FSYNC-1, Zero-Panic).
//!
//! Validates:
//! - UTF-8 invalid bytes are rejected cleanly without panicking.
//! - Document payloads exceeding `MAX_DOC_BYTES` return `CapacityExceeded`.
//! - Oversized queries return `CapacityExceeded`.
//! - Inspect API under feature `inspect` accurately reports presence and tombstones.

use contextra_text::contract::DocKey;
use contextra_text::error::TextError;
use contextra_text::memory_index::{MemoryTextIndex, MAX_DOC_BYTES};
use contextra_text::traits::{Index, Maintain};

#[test]
fn test_invalid_utf8_payload_returns_error() {
    let mut index = MemoryTextIndex::new();
    let invalid_utf8 = vec![0xFF, 0xFE, 0xFD];
    let res = Index::<DocKey, str>::insert(&mut index, DocKey::from(1u64), &invalid_utf8);
    assert!(matches!(res, Err(TextError::InvalidInput(_))));
}

#[test]
fn test_oversized_payload_returns_capacity_exceeded() {
    let mut index = MemoryTextIndex::new();
    let oversized = vec![b'a'; MAX_DOC_BYTES + 1];
    let res = Index::<DocKey, str>::insert(&mut index, DocKey::from(2u64), &oversized);
    assert!(matches!(
        res,
        Err(TextError::CapacityExceeded {
            requested,
            maximum
        }) if requested == MAX_DOC_BYTES + 1 && maximum == MAX_DOC_BYTES
    ));
}

#[test]
fn test_oversized_query_returns_capacity_exceeded() {
    let index = MemoryTextIndex::new();
    let oversized_query = "q".repeat(MAX_DOC_BYTES + 1);
    let res = Index::<DocKey, str>::search(&index, &oversized_query, 10);
    assert!(matches!(
        res,
        Err(TextError::CapacityExceeded {
            requested,
            maximum
        }) if requested == MAX_DOC_BYTES + 1 && maximum == MAX_DOC_BYTES
    ));
}

#[cfg(feature = "inspect")]
#[test]
fn test_inspect_key_lifecycle() {
    let mut index = MemoryTextIndex::new();
    let key = DocKey::from(42u64);

    // Initial inspection: not in index
    let insp0 = index.inspect_key(&key);
    assert!(!insp0.in_doc_lengths);
    assert!(!insp0.is_tombstone);
    assert_eq!(insp0.posting_occurrences, 0);

    // Insert doc
    Index::<DocKey, str>::insert(&mut index, key, b"rust index test").unwrap();
    let insp1 = index.inspect_key(&key);
    assert!(insp1.in_doc_lengths);
    assert!(!insp1.is_tombstone);
    assert_eq!(insp1.posting_occurrences, 3);

    // Tombstone removal
    Index::<DocKey, str>::remove(&mut index, &key).unwrap();
    let insp2 = index.inspect_key(&key);
    assert!(insp2.in_doc_lengths);
    assert!(insp2.is_tombstone);
    assert_eq!(insp2.posting_occurrences, 3);

    // Physical compact
    Maintain::compact(&mut index).unwrap();
    let insp3 = index.inspect_key(&key);
    assert!(!insp3.in_doc_lengths);
    assert!(!insp3.is_tombstone);
    assert_eq!(insp3.posting_occurrences, 0);
}

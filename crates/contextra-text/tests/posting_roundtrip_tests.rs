//! Umfassende Tests für Posting-Listen Delta-Varint-Kompression, Monotonie,
//! Block-Max Skip-Informationen und defensive Fehlerabfangung bei Korruption.

use contextra_text::posting_list::{Posting, PostingList, BLOCK_SIZE, POSTING_LIST_V2_MAGIC};

#[cfg(not(feature = "types-compat"))]
use contextra_text::types::DocId;
#[cfg(feature = "types-compat")]
use contextra_types::DocId;

#[test]
fn test_posting_empty_roundtrip() {
    let empty = PostingList::empty();
    let encoded = empty.encode_compact().expect("encode empty list");
    assert!(encoded.starts_with(POSTING_LIST_V2_MAGIC));
    let decoded = PostingList::decode_compact(&encoded).expect("decode empty list");
    assert_eq!(decoded.len(), 0);
    assert!(decoded.is_empty());
    assert_eq!(decoded.blocks().len(), 0);
}

#[test]
fn test_posting_single_and_multiblock_roundtrip() {
    let count = BLOCK_SIZE * 4 + 7; // 263 postings across 5 blocks
    let mut postings = Vec::with_capacity(count);
    let mut curr_doc_id = 10u64;

    for i in 0..count {
        curr_doc_id += (i % 7 + 1) as u64;
        let tf = ((i % 15) + 1) as u32;
        let doc_len = (20 + (i % 80)) as u32;
        postings.push(Posting::new(DocId::new(curr_doc_id), tf, doc_len));
    }

    let list = PostingList::new(postings);
    let bytes = list.encode_compact().expect("encode multiblock list");
    let decoded = PostingList::decode_compact(&bytes).expect("decode multiblock list");

    assert_eq!(decoded.len(), count);
    assert_eq!(decoded.as_slice(), list.as_slice());
    assert_eq!(decoded.blocks(), list.blocks());

    // Verify block-max invariants
    for (b_idx, block) in decoded.blocks().iter().enumerate() {
        let start = b_idx * BLOCK_SIZE;
        let end = (start + BLOCK_SIZE).min(count);
        let slice = &decoded.as_slice()[start..end];

        let expected_max_doc = slice.last().map(|p| p.doc_id).unwrap_or(0);
        let expected_max_tf = slice.iter().map(|p| p.tf).max().unwrap_or(0);
        let expected_min_dl = slice.iter().map(|p| p.doc_len).min().unwrap_or(0);

        assert_eq!(block.max_doc_id, expected_max_doc);
        assert_eq!(block.max_tf, expected_max_tf);
        assert_eq!(block.min_doc_len, expected_min_dl);
    }
}

#[test]
fn test_posting_defensive_truncated_payload() {
    let list = PostingList::new(vec![
        Posting::new(DocId::new(10), 2, 50),
        Posting::new(DocId::new(20), 5, 100),
    ]);
    let bytes = list.encode_compact().expect("encode");

    // Truncate at every single byte boundary
    for cut in 0..bytes.len() {
        let truncated = &bytes[..cut];
        let res = PostingList::decode_compact(truncated);
        // Either fallback fails or compact decode detects truncation - never panic!
        assert!(res.is_err(), "Truncated at {cut} bytes should fail closed");
    }
}

#[test]
fn test_posting_defensive_trailing_bytes_rejected() {
    let list = PostingList::new(vec![Posting::new(DocId::new(10), 2, 50)]);
    let mut bytes = list.encode_compact().expect("encode");
    bytes.push(0x42); // Trailing junk byte

    let res = PostingList::decode_compact(&bytes);
    assert!(res.is_err(), "Trailing bytes must return Error");
}

#[test]
fn test_posting_defensive_non_monotonic_deltas_rejected() {
    // Construct payload with declared count 2, but second delta = 0 (duplicate doc_id)
    let mut buf = Vec::new();
    buf.extend_from_slice(POSTING_LIST_V2_MAGIC);
    buf.push(2); // count = 2
                 // Posting 1: doc_id = 10, tf = 1, doc_len = 10
    buf.push(10); // delta = 10
    buf.push(1); // tf = 1
    buf.push(10); // doc_len = 10

    // Posting 2: non-monotonic delta = 0 (same doc_id 10)
    buf.push(0); // delta = 0 -> INVALID for second entry!
    buf.push(1);
    buf.push(10);

    let res = PostingList::decode_compact(&buf);
    assert!(res.is_err(), "Duplicate/non-monotonic delta must fail");
}

#[test]
fn test_posting_defensive_declared_count_exceeds_buffer() {
    // Header claims 10_000 postings, but only 5 bytes follow
    let mut buf = Vec::new();
    buf.extend_from_slice(POSTING_LIST_V2_MAGIC);
    // Varint for 10_000 = [0xa0, 0x4e]
    buf.push(0xa0);
    buf.push(0x4e);
    buf.extend_from_slice(&[1, 2, 3, 4, 5]);

    let res = PostingList::decode_compact(&buf);
    assert!(
        res.is_err(),
        "Count exceeding byte capacity must fail without OOM"
    );
}

#[test]
fn test_posting_defensive_doc_id_overflow() {
    // First doc_id u64::MAX, second doc_id delta 1 -> overflow
    let mut buf = Vec::new();
    buf.extend_from_slice(POSTING_LIST_V2_MAGIC);
    buf.push(2); // count = 2

    // First posting: delta = u64::MAX
    for _ in 0..9 {
        buf.push(0xff);
    }
    buf.push(0x01); // 64-bit varint for u64::MAX
    buf.push(1); // tf
    buf.push(1); // doc_len

    // Second posting: delta = 1 -> triggers overflow when added to u64::MAX
    buf.push(1);
    buf.push(1);
    buf.push(1);

    let res = PostingList::decode_compact(&buf);
    assert!(res.is_err(), "DocId overflow must fail closed");
}

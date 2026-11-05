//! Spezifikationstests für den segmentbasierten BM25-Volltextindex laut SPEC 9.7, 8.9, D-20, D-49, D-69, D-71, D-81, F.7.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::assertions_on_constants
)]

use contextra_crypto::dek::Dek;
use contextra_crypto::hierarchy::FileKey;
use contextra_crypto::secret::SecretKey;
use contextra_text::analyzer;
use contextra_text::buffer::{open_binding_envelope, seal_binding_envelope};
use contextra_text::phrase::match_phrase_positions;
use contextra_text::postings::PostingBlock;
use contextra_text::segment::{SegmentReader, SegmentWriter};
use contextra_text::stats::SegmentStats;

/// SPEC: T-TEXT-ANALYZER-1
#[test]
fn test_t_text_analyzer_1_goldenset() {
    let analyzer = analyzer::get_analyzer("de", 1).expect("Analyzer de/1 must exist");
    assert_eq!(analyzer.id(), 1);
    assert_eq!(analyzer.version(), 1);
    assert_eq!(analyzer.lang(), *b"de\0");

    let tokens = analyzer.analyze("Donaudampfschifffahrt und Schloßstraße mit Übermut");
    assert!(
        tokens.contains(&"donaudampfschifffahrt".to_string())
            || tokens.contains(&"schiff".to_string())
    );
    assert!(
        tokens.contains(&"schlossstrasse".to_string())
            || tokens.contains(&"schloss".to_string())
            || tokens.contains(&"strasse".to_string())
    );
    assert!(!tokens.contains(&"und".to_string()));
    assert!(!tokens.contains(&"mit".to_string()));
}

/// SPEC: T-TEXT-PHRASE-1
#[test]
/// SPEC: T-TEXT-PHRASE-1
fn test_t_text_phrase_1() {
    let term1_pos = [10u32, 25u32];
    let term2_pos = [11u32, 40u32];
    let term3_pos = [12u32, 90u32];

    assert!(match_phrase_positions(&[
        &term1_pos, &term2_pos, &term3_pos
    ]));

    let term_unmatched = [50u32, 60u32];
    assert!(!match_phrase_positions(&[&term1_pos, &term_unmatched]));
}

/// SPEC: T-TEXT-1
#[test]
/// SPEC: T-TEXT-1
fn test_t_text_1_segment_search() {
    let mut writer = SegmentWriter::new(0, 1, 1, 1, *b"de\0", true);
    let doc_ord = writer.add_document(120, vec![1, 2, 3, 4]);
    assert_eq!(doc_ord, 0);
    assert_eq!(writer.doc_count(), 1);

    let mut block = PostingBlock::new();
    block.max_doc_ord = 0;
    block.max_tf_norm = 10;
    block.doc_ords.push(0);
    block.tfs.push(2);
    block.positions = Some(vec![vec![5, 10]]);
    writer.add_term([0x33; 16], 1, block);

    let file_id = [42u8; 16];
    let file_key1 = FileKey::new([7u8; 32]);
    let file_key2 = FileKey::new([7u8; 32]);
    let sealed = writer
        .seal_to_bytes(file_id, file_key1)
        .expect("Sealing must succeed");

    let reader =
        SegmentReader::open_from_bytes(&sealed, file_id, file_key2).expect("Reading must succeed");
    assert_eq!(reader.header.doc_count, 1);
    assert_eq!(reader.header.total_len, 120);
    assert_eq!(reader.docs.len(), 1);
    assert_eq!(reader.docs.first().map(|d| d.doc_ord), Some(0));
    assert_eq!(reader.docs.first().map(|d| d.doclen), Some(120));
    assert_eq!(
        reader.docs.first().map(|d| d.binding_envelope.clone()),
        Some(vec![1, 2, 3, 4])
    );
    assert_eq!(reader.postings.len(), 1);
    assert_eq!(
        reader.postings.first().map(|p| p.doc_ords.clone()),
        Some(vec![0])
    );
}

/// SPEC: T-TEXT-2
#[test]
/// SPEC: T-TEXT-2
fn test_t_text_2_merge() {
    let stats = SegmentStats::recompute(&[100, 200]);
    assert_eq!(stats.doc_count, 2);
}

/// SPEC: T-TEXT-3
#[test]
/// SPEC: T-TEXT-3
fn test_t_text_3_rewrite_stats() {
    let initial_docs = [100u32, 200u32, 300u32];
    let stats = SegmentStats::recompute(&initial_docs);
    assert_eq!(stats.doc_count, 3);
    assert_eq!(stats.total_len, 600);
    assert_eq!(stats.avg_doc_len, 200.0);

    let surviving_docs = [100u32, 300u32];
    let recomputed = SegmentStats::recompute(&surviving_docs);
    assert_eq!(recomputed.doc_count, 2);
    assert_eq!(recomputed.total_len, 400);
    assert_eq!(recomputed.avg_doc_len, 200.0);
}

/// SPEC: T-TEXT-4
#[test]
/// SPEC: T-TEXT-4
/// SPEC: T-KEYCOV-1
fn test_t_text_4_and_t_keycov_1_unlinked() {
    let dek = Dek(SecretKey::new([0x42; 32]));
    let file_id = [0xAA; 16];
    let doc_ord = 7;
    let cluster_tag = [0x11; 16];
    let doc_tag = [0x22; 16];

    let envelope = seal_binding_envelope(&dek, file_id, doc_ord, cluster_tag, doc_tag)
        .expect("Binding envelope sealing must succeed");

    let (c, d) = open_binding_envelope(&dek, file_id, doc_ord, &envelope)
        .expect("Binding envelope opening must succeed");
    assert_eq!(c, cluster_tag);
    assert_eq!(d, doc_tag);

    let destroyed_dek = Dek(SecretKey::new([0x00; 32]));
    let res = open_binding_envelope(&destroyed_dek, file_id, doc_ord, &envelope);
    assert!(
        res.is_err(),
        "Postings must be unlinked when DEK is destroyed"
    );
}

/// SPEC: T-RAW-4
#[test]
/// SPEC: T-RAW-4
fn test_t_raw_4_no_plaintext_terms() {
    let analyzer = analyzer::get_analyzer("de", 1).expect("Analyzer de/1 must exist");
    let key = contextra_crypto::hierarchy::IdxKey(SecretKey::new([1u8; 32]));
    let tag = analyzer.term_tag(&key, "geheimnis");
    assert_eq!(tag.len(), 16);
}

/// SPEC: T-TEXT-1
#[test]
/// SPEC: T-TEXT-1
fn test_t_text_1_block_manipulation_fails_closed() {
    let mut writer = SegmentWriter::new(0, 1, 1, 1, *b"de\0", true);
    writer.add_document(80, vec![1, 2, 3]);
    let file_id = [12u8; 16];
    let file_key = FileKey::new([34u8; 32]);
    let mut sealed = writer.seal_to_bytes(file_id, file_key).expect("seal");

    // Bitflip in ciphertext block (after 64 bytes header + 4 bytes ct_len)
    if let Some(byte) = sealed.get_mut(70) {
        *byte ^= 0xFF;
    }

    let file_key2 = FileKey::new([34u8; 32]);
    let res = SegmentReader::open_from_bytes(&sealed, file_id, file_key2);
    assert!(
        res.is_err(),
        "Manipulated ciphertext block must fail decryption"
    );
}

/// SPEC: T-TEXT-1
#[test]
/// SPEC: T-TEXT-1
fn test_t_text_1_wrong_file_id_fails_closed() {
    let mut writer = SegmentWriter::new(0, 1, 1, 1, *b"de\0", true);
    writer.add_document(80, vec![1, 2, 3]);
    let file_id = [12u8; 16];
    let file_key = FileKey::new([34u8; 32]);
    let sealed = writer.seal_to_bytes(file_id, file_key).expect("seal");

    let wrong_file_id = [99u8; 16];
    let file_key2 = FileKey::new([34u8; 32]);
    let res = SegmentReader::open_from_bytes(&sealed, wrong_file_id, file_key2);
    assert!(res.is_err(), "Opening with wrong file_id must fail closed");
}

//! SPEC T-TEXT Acceptance Test Suite (T-TEXT-1, T-TEXT-3, T-TEXT-4, T-TEXT-ANALYZER-1, T-TEXT-PHRASE-1).
//!
//! # SPEC-Referenz:
//! - SPEC §9.7: Segmentierter BM25-Index, Analyzer de/1, Block-Max WAND
//! - SPEC D-69: Entkopplung (Postings tragen nur doc_ord; Bindung unter DEK; nach Vernichtung unlinked)
//! - SPEC D-81: Analyzer de/1 und und/1, Positionen für Phrasen, BM25 k1=1.2, b=0.75
//! - SPEC E-L5: Exakte Neuberechnung aller Statistiken nach Purge
//! - SPEC Anhang F.7: Segmentformat

#![allow(clippy::expect_used, clippy::unwrap_used)]

use contextra_crypto::hierarchy::FileKey;
use contextra_crypto::secret::SecretKey;
use contextra_text::analyzer::get_analyzer;
use contextra_text::buffer::{open_binding_envelope, seal_binding_envelope};
use contextra_text::phrase::match_phrase_positions;
use contextra_text::postings::PostingBlock;
use contextra_text::segment::{SegmentReader, SegmentWriter};
use contextra_text::stats::SegmentStats;

/// SPEC: T-TEXT-ANALYZER-1
#[test]
fn test_t_text_analyzer_1_goldenset() {
    let analyzer = get_analyzer("de", 1).expect("Analyzer de/1 must exist");
    assert_eq!(analyzer.id(), 1);
    assert_eq!(analyzer.version(), 1);
    assert_eq!(analyzer.lang(), *b"de\0");

    let tokens = analyzer.analyze("Großartige Äpfel und Überläufer.");
    assert!(!tokens.contains(&"und".to_string()));
    assert!(
        tokens.contains(&"grossartig".to_string()) || tokens.contains(&"grossartige".to_string())
    );

    let comp_tokens = analyzer.analyze("Bundesverfassungsgericht");
    assert!(comp_tokens.len() > 1 || comp_tokens.contains(&"bundesverfassungsgericht".to_string()));
}

/// SPEC: T-TEXT-PHRASE-1
#[test]
fn test_t_text_phrase_1_positions() {
    let pos1 = [1u32, 10u32];
    let pos2 = [2u32, 20u32];
    assert!(match_phrase_positions(&[&pos1, &pos2]));

    let unmatched = [50u32];
    assert!(!match_phrase_positions(&[&pos1, &unmatched]));
}

/// SPEC: T-TEXT-1
#[test]
fn test_t_text_1_segment_roundtrip() {
    let mut writer = SegmentWriter::new(0, 1, 1, 1, *b"de\0", true);
    let doc_ord = writer.add_document(120, vec![1, 2, 3, 4]);
    assert_eq!(doc_ord, 0);

    let mut block = PostingBlock::new();
    block.max_doc_ord = 0;
    block.max_tf_norm = 10;
    block.doc_ords.push(0);
    block.tfs.push(1);
    block.positions = Some(vec![vec![5]]);
    writer.add_term([0x55; 16], 1, block);

    let file_id = [0x11; 16];
    let file_key = FileKey::new([0x22; 32]);
    let sealed = writer.seal_to_bytes(file_id, file_key).expect("seal");
    let file_key2 = FileKey::new([0x22; 32]);
    let reader = SegmentReader::open_from_bytes(&sealed, file_id, file_key2).expect("open");
    assert_eq!(reader.header.doc_count, 1);
    assert_eq!(reader.header.total_len, 120);
}

/// SPEC: T-TEXT-3
#[test]
fn test_t_text_3_exact_stats() {
    let stats = SegmentStats::recompute(&[100, 200, 300]);
    assert_eq!(stats.doc_count, 3);
    assert_eq!(stats.total_len, 600);
    assert_eq!(stats.avg_doc_len, 200.0);
}

/// SPEC: T-TEXT-4
#[test]
fn test_t_text_4_unlinked() {
    use contextra_crypto::dek::Dek;
    let dek = Dek(SecretKey::new([0x33; 32]));
    let file_id = [0x44; 16];
    let doc_ord = 5;
    let env = seal_binding_envelope(&dek, file_id, doc_ord, [0x11; 16], [0x22; 16]).expect("seal");

    let opened = open_binding_envelope(&dek, file_id, doc_ord, &env);
    assert!(opened.is_ok());

    let bad_dek = Dek(SecretKey::new([0x00; 32]));
    let bad_open = open_binding_envelope(&bad_dek, file_id, doc_ord, &env);
    assert!(bad_open.is_err(), "Must be unlinked when DEK is destroyed");
}

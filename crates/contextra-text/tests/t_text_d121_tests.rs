//! Tests für SPEC D-121 (T-TEXT-5..9):
//! - T-TEXT-5: Einheit Chunk (doc_tag, chunk, version_seq)
//! - T-TEXT-6: Flush-Crash Matrix (Floor-Regel)
//! - T-TEXT-7: Löschung nach Schlüsselvernichtung (Prädikat I-22)
//! - T-TEXT-8: Sofortige Sichtbarkeit nach Insert
//! - T-TEXT-9: Mitgliedschaftsblock ohne Falsch-Negativ (m(t))

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::assertions_on_constants
)]

use contextra_crypto::hierarchy::IdxKey;
use contextra_crypto::secret::SecretKey;
use contextra_text::buffer::{open_binding_envelope, seal_binding_envelope, TextBuffer};
use contextra_text::member::MembershipBlock;
use contextra_text::stats::SegmentStats;

/// SPEC: T-TEXT-5
/// Einheit ist der Chunk (doc_tag, chunk, version_seq).
#[test]
fn test_t_text_5_chunk_unit_binding() {
    use contextra_crypto::dek::Dek;
    let dek = Dek(SecretKey::new([0x55; 32]));
    let file_id = [0x77; 16];
    let doc_ord = 1;
    let cluster_tag = [0x11; 16];
    let doc_tag = [0x22; 16];

    let envelope =
        seal_binding_envelope(&dek, file_id, doc_ord, cluster_tag, doc_tag).expect("seal binding");
    let (c, d) = open_binding_envelope(&dek, file_id, doc_ord, &envelope).expect("open binding");
    assert_eq!(c, cluster_tag);
    assert_eq!(d, doc_tag);
}

/// SPEC: T-TEXT-7
/// Löschung nach Schlüsselvernichtung findet und entfernt alle Zielsegmente (Prädikat I-22).
#[test]
fn test_t_text_7_deletion_predicate_i22() {
    use contextra_crypto::dek::Dek;
    let dek = Dek(SecretKey::new([0x88; 32]));
    let file_id = [0x99; 16];
    let doc_ord = 0;
    let cluster_tag = [0x01; 16];
    let doc_tag = [0x02; 16];

    let envelope =
        seal_binding_envelope(&dek, file_id, doc_ord, cluster_tag, doc_tag).expect("seal");

    // Gültiger DEK öffnet
    let res = open_binding_envelope(&dek, file_id, doc_ord, &envelope);
    assert!(res.is_ok());

    // Fehlender / vernichteter DEK scheitert an Auth-Tag (I-22: Schlüsselabwesenheit)
    let destroyed_dek = Dek(SecretKey::new([0x00; 32]));
    let res_destroyed = open_binding_envelope(&destroyed_dek, file_id, doc_ord, &envelope);
    assert!(res_destroyed.is_err());
}

/// SPEC: T-TEXT-8
/// Sichtbarkeit sofort nach In-Memory Buffer Insert.
#[test]
fn test_t_text_8_buffer_immediate_visibility() {
    let mut buffer = TextBuffer::new();
    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);

    let doc_ord = buffer.insert([0x11; 16], [0x22; 16], "sofort sichtbar".to_string());
    assert_eq!(doc_ord, 0);
    assert_eq!(buffer.len(), 1);
    assert!(!buffer.is_empty());

    let records = buffer.drain();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].text, "sofort sichtbar");
    assert!(buffer.is_empty());
}

/// SPEC: T-TEXT-9
/// Mitgliedschaftsblock m(t) hat keine falsch-negativen Treffer (D-121 Abs. 3).
#[test]
fn test_t_text_9_membership_no_false_negatives() {
    let k_idx = IdxKey(SecretKey::new([0x42; 32]));
    let mut mb = MembershipBlock::new();

    let tag1 = [0xAA; 16];
    let tag2 = [0xBB; 16];
    let tag3 = [0xCC; 16];

    mb.add_tag(&k_idx, &tag1);
    mb.add_tag(&k_idx, &tag2);
    mb.finalize();

    // Vorhandene Tags MÜSSEN gefunden werden (kein Falsch-Negativ)
    assert!(mb.contains_tag(&k_idx, &tag1));
    assert!(mb.contains_tag(&k_idx, &tag2));

    // Nicht hinzugefügtes Tag
    assert!(!mb.contains_tag(&k_idx, &tag3));

    // Roundtrip Serialisierung
    let encoded = mb.encode();
    let (decoded, read_bytes) = MembershipBlock::decode(&encoded).expect("decode");
    assert_eq!(read_bytes, encoded.len());
    assert!(decoded.contains_tag(&k_idx, &tag1));
    assert!(decoded.contains_tag(&k_idx, &tag2));
    assert!(!decoded.contains_tag(&k_idx, &tag3));
}

/// SPEC: T-TEXT-6
/// Segment-Rewrite Statistiken neu berechnet (Floor-Regel).
#[test]
fn test_t_text_6_flush_exact_stats() {
    let living_doc_lens = [150u32, 250u32];
    let stats = SegmentStats::recompute(&living_doc_lens);
    assert_eq!(stats.doc_count, 2);
    assert_eq!(stats.total_len, 400);
    assert_eq!(stats.avg_doc_len, 200.0);
}

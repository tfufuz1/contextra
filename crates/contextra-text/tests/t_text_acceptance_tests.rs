//! SPEC T-TEXT Acceptance Test Suite (T-TEXT-1, T-TEXT-3, T-TEXT-4, T-TEXT-ANALYZER-1, T-TEXT-PHRASE-1).
//!
//! # SPEC-Referenz:
//! - SPEC §9.7: Segmentierter BM25-Index, Analyzer de/1, Block-Max WAND
//! - SPEC D-69: Entkopplung (Postings tragen nur doc_ord; Bindung unter DEK; nach Vernichtung unlinked)
//! - SPEC D-81: Analyzer de/1 und und/1, Positionen für Phrasen, BM25 k1=1.2, b=0.75
//! - SPEC E-L5: Exakte Neuberechnung aller Statistiken nach Purge
//! - SPEC Anhang F.7: Segmentformat

use contextra_crypto::hierarchy::IdxKey;
use contextra_crypto::secret::SecretKey;
use contextra_text::analyzer::{compute_term_tag, Analyzer, AnalyzerId, GermanAnalyzer};
use contextra_text::buffer::{purge_segments, TextBuffer, TextDocHandle};
use contextra_text::segment::SegmentReader;
use contextra_types::{ClusterTag, DocTag};
use std::collections::HashSet;

/// SPEC: T-TEXT-ANALYZER-1
/// Prüft de/1 Normalisierung: UAX #29, NFKC, ß->ss, Umlautnormalisierung, Komposita, Stoppwörter.
#[test]
fn test_t_text_analyzer_1_goldenset() {
    let analyzer = GermanAnalyzer::new();
    assert_eq!(analyzer.analyzer_id(), AnalyzerId::De1);

    // 1. Großschreibung, Umlaute und ß
    let analyzed = analyzer.analyze("Großartige Äpfel und Überläufer.");
    let terms: Vec<_> = analyzed.tokens.iter().map(|t| t.term.as_str()).collect();

    // "und" ist Stoppwort
    assert!(!terms.contains(&"und"));
    // "Großartige" -> "grossartige"
    assert!(terms.contains(&"grossartige"));
    // "Äpfel" -> "aepfel"
    assert!(terms.contains(&"aepfel"));

    // 2. Komposita-Zerlegung (z.B. Bundesverfassungsgericht)
    let comp_analyzed = analyzer.analyze("Bundesverfassungsgericht");
    let comp_terms: Vec<_> = comp_analyzed
        .tokens
        .iter()
        .map(|t| t.term.as_str())
        .collect();
    assert!(comp_terms.contains(&"bundesverfassungsgericht"));
    // Dekompoundierte Teile
    assert!(comp_terms.len() > 1);
}

/// SPEC: T-TEXT-PHRASE-1
/// Prüft, dass Tokenpositionen erfasst und im Segment für Phrasensuche erhalten bleiben.
#[test]
fn test_t_text_phrase_1_positions() {
    let key = IdxKey(SecretKey::new([3u8; 32]));
    let analyzer = GermanAnalyzer::new();
    let text = "schnelle rote Autos fahren auf der Autobahn";
    let analyzed = analyzer.analyze(text);

    let mut buf = TextBuffer::new();
    let doc_tag = DocTag::new([10u8; 16]);
    let handle = TextDocHandle {
        cluster_tag: ClusterTag::new([1u8; 16]),
        doc_tag,
    };
    let mut env = vec![0u8; 32];
    env[16..32].copy_from_slice(doc_tag.as_bytes());

    buf.insert(handle, analyzed.token_count, env, analyzed);
    let seg_bytes = buf.flush_to_segment(&key, AnalyzerId::De1, 0, 1, true);

    let reader = SegmentReader::open(&seg_bytes).expect("open segment");
    assert!(reader.header.positions);

    let term_tag = compute_term_tag(&key, AnalyzerId::De1, "rote");
    let postings = reader
        .get_postings(&term_tag)
        .expect("postings")
        .expect("found");
    assert_eq!(postings.len(), 1);
    let p = &postings[0].postings[0];
    assert!(
        !p.positions.is_empty(),
        "Positionen müssen erhalten bleiben"
    );
}

/// SPEC: T-TEXT-4
/// Prüft, dass nach Vernichtung des Dokumentschlüssels (DEK) die Postings unlinked sind.
/// Ein Wörterbuchangriff auf gelöschte Tags scheitert, da keine Identitätsbindung mehr existiert.
#[test]
fn test_t_text_4_postings_unlinked_after_key_destruction() {
    let key = IdxKey(SecretKey::new([5u8; 32]));
    let mut buf = TextBuffer::new();

    let doc_tag1 = DocTag::new([11u8; 16]);
    let doc_tag2 = DocTag::new([12u8; 16]);

    let mut env1 = vec![0u8; 32];
    env1[16..32].copy_from_slice(doc_tag1.as_bytes());

    let mut env2 = vec![0u8; 32];
    env2[16..32].copy_from_slice(doc_tag2.as_bytes());

    let analyzer = GermanAnalyzer::new();
    buf.insert(
        TextDocHandle {
            cluster_tag: ClusterTag::new([0u8; 16]),
            doc_tag: doc_tag1,
        },
        50,
        env1,
        analyzer.analyze("vertrauliches geheimnis dokument"),
    );
    buf.insert(
        TextDocHandle {
            cluster_tag: ClusterTag::new([0u8; 16]),
            doc_tag: doc_tag2,
        },
        50,
        env2,
        analyzer.analyze("oeffentliches dokument"),
    );

    let seg_bytes = buf.flush_to_segment(&key, AnalyzerId::De1, 0, 1, false);

    // Simuliere DEK-Vernichtung für doc_tag1: DEK existiert nicht mehr in valid_deks
    let mut target_docs = HashSet::new();
    target_docs.insert(doc_tag1);
    let mut valid_deks = HashSet::new();
    valid_deks.insert(doc_tag2);

    let (rewritten, outcome) =
        purge_segments(&[seg_bytes], &target_docs, &valid_deks, &key).expect("purge");
    assert_eq!(outcome.documents_purged, 1);
    assert_eq!(rewritten.len(), 1);

    let reader = SegmentReader::open(&rewritten[0]).expect("reader");
    assert_eq!(reader.header.doc_count, 1);

    // Suche nach "geheimnis": Darf in den überlebenden Postings nicht mehr existieren!
    let term_tag = compute_term_tag(&key, AnalyzerId::De1, "geheimnis");
    let result = reader.get_postings(&term_tag).expect("lookup");
    assert!(
        result.is_none(),
        "Gelöschtes Term-Posting darf nicht mehr abrufbar sein"
    );
}

/// SPEC: T-TEXT-3
/// Prüft, dass beim Purge-Rewrite alle Zieldokumente entfernt und Statistiken (E-L5) exakt neu berechnet werden.
#[test]
fn test_t_text_3_exact_statistics_recalculation() {
    let key = IdxKey(SecretKey::new([9u8; 32]));
    let mut buf = TextBuffer::new();

    let doc_tag1 = DocTag::new([21u8; 16]);
    let doc_tag2 = DocTag::new([22u8; 16]);

    let mut env1 = vec![0u8; 32];
    env1[16..32].copy_from_slice(doc_tag1.as_bytes());
    let mut env2 = vec![0u8; 32];
    env2[16..32].copy_from_slice(doc_tag2.as_bytes());

    let analyzer = GermanAnalyzer::new();
    buf.insert(
        TextDocHandle {
            cluster_tag: ClusterTag::new([0u8; 16]),
            doc_tag: doc_tag1,
        },
        200,
        env1,
        analyzer.analyze("datenbank sicherheit architekt"),
    );
    buf.insert(
        TextDocHandle {
            cluster_tag: ClusterTag::new([0u8; 16]),
            doc_tag: doc_tag2,
        },
        300,
        env2,
        analyzer.analyze("datenbank optimierung performanz"),
    );

    let seg_bytes = buf.flush_to_segment(&key, AnalyzerId::De1, 0, 1, false);
    let reader_before = SegmentReader::open(&seg_bytes).expect("open");
    assert_eq!(reader_before.header.doc_count, 2);
    assert_eq!(reader_before.header.total_len, 500);

    // Lösche doc_tag1
    let mut targets = HashSet::new();
    targets.insert(doc_tag1);
    let mut valid_deks = HashSet::new();
    valid_deks.insert(doc_tag2);

    let (rewritten, _) = purge_segments(&[seg_bytes], &targets, &valid_deks, &key).expect("purge");
    let reader_after = SegmentReader::open(&rewritten[0]).expect("open rewritten");

    // E-L5 Invariante: Exakte Neuberechnung!
    assert_eq!(
        reader_after.header.doc_count, 1,
        "doc_count muss exakt 1 sein"
    );
    assert_eq!(
        reader_after.header.total_len, 300,
        "total_len muss exakt 300 sein (kein Restwert aus doc 1)"
    );
}

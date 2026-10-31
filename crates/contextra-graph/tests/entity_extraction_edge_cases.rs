//! Edge case tests for rule-based entity extraction (`crates/contextra-graph/src/entity_extraction.rs`).
//!
//! Covers genuine edge cases not in `entity_extraction_tests.rs`:
//! - Empty text and whitespace/punctuation only
//! - Non-ASCII and Unicode handling (German umlauts, accented characters, multi-byte emojis, CJK/RTL)
//! - Very long texts (tokenization and memory stress)
//! - Exact confidence boundary thresholding
//! - Determinism across repeated extractions

use contextra_graph::entity_extraction::{
    ingest_extracted_relations, EntityExtractor, ExtractedRelation, RuleBasedEntityExtractor,
};
use contextra_graph::CsrGraph;
use contextra_types::DocId;

#[test]
fn test_empty_and_whitespace_and_punctuation_only_input() {
    let extractor = RuleBasedEntityExtractor::new(0.50);

    // Completely empty string
    assert!(extractor.extract("").is_empty());

    // Only whitespace
    assert!(extractor.extract("   \t\n\r\n   ").is_empty());

    // Only sentence delimiters and punctuation
    assert!(extractor.extract("... ??? !!! ;;; ...\n\n").is_empty());

    // Triggers present, but no entity words on either side
    assert!(extractor.extract("arbeitet bei.").is_empty());
    assert!(extractor.extract("works at").is_empty());
    assert!(extractor.extract("... wohnt in ...").is_empty());
}

#[test]
fn test_unicode_umlauts_accents_emojis_and_multilingual() {
    let extractor = RuleBasedEntityExtractor::new(0.50);

    // German umlauts and Eszett in entities
    let text_umlauts = "Jürgen Müller arbeitet bei Großbäckerei Schäfer in München.";
    let res = extractor.extract(text_umlauts);
    assert!(!res.is_empty(), "Should extract umlaut entities");
    assert_eq!(res[0].source, "Jürgen Müller");
    assert_eq!(res[0].target, "Großbäckerei Schäfer");
    assert_eq!(res[0].relation_type, "WORKS_AT");

    // French accents
    let text_en = "Amélie Poulain works for Société Générale in Paris.";
    let res_fr = extractor.extract(text_en);
    assert!(!res_fr.is_empty());
    assert_eq!(res_fr[0].source, "Amélie Poulain");
    assert_eq!(res_fr[0].target, "Société Générale");

    // Emojis interspersed: should not panic or cause slicing issues
    let text_emojis = "🚀 Dr. Alex arbeitet bei 🏢 DeepTech 🌐.";
    let res_emojis = extractor.extract(text_emojis);
    assert!(!res_emojis.is_empty());
    assert_eq!(res_emojis[0].relation_type, "WORKS_AT");

    // Non-Latin characters (CJK, Hebrew) without uppercase distinctions
    let text_cjk = "田中 arbeitet bei ソニー.";
    // Should not panic, clean_entity_name gracefully handles non-Latin
    let _res_cjk = extractor.extract(text_cjk);
}

#[test]
fn test_very_long_input_stress_and_bounded_execution() {
    let extractor = RuleBasedEntityExtractor::new(0.50);

    // Generate a long text with thousands of sentences and tokens
    let mut long_text = String::with_capacity(100_000);
    for i in 0..1_000 {
        if i % 100 == 0 {
            long_text.push_str(&format!("Forscher_{i} arbeitet bei Institut_{i}. "));
        } else {
            long_text.push_str("Dies ist ein irrelevanter Füllsatz ohne Entitätsmuster. ");
        }
    }

    let results = extractor.extract(&long_text);
    assert_eq!(results.len(), 10);
    for (idx, rel) in results.iter().enumerate() {
        let i = idx * 100;
        assert_eq!(rel.source, format!("Forscher_{i}"));
        assert_eq!(rel.target, format!("Institut_{i}"));
        assert_eq!(rel.relation_type, "WORKS_AT");
    }
}

#[test]
fn test_strict_confidence_boundary_precision() {
    // base confidences:
    // FOUNDED: 0.90
    // WORKS_AT, LIVES_IN, LEADS: 0.85
    // PARTNERED_WITH: 0.80
    // CONNECTED_TO: 0.75

    let extractor_85 = RuleBasedEntityExtractor::new(0.85);

    let text_80 = "Company A partnered with Company B.";
    let res_80 = extractor_85.extract(text_80);
    assert!(
        res_80.is_empty(),
        "0.80 must be filtered out when min is 0.85"
    );

    let text_85 = "Developer works at Enterprise.";
    let res_85 = extractor_85.extract(text_85);
    assert_eq!(res_85.len(), 1, "0.85 must be included when min is 0.85");
    assert_eq!(res_85[0].relation_type, "WORKS_AT");

    // Ingest threshold exact boundary
    let graph = CsrGraph::new();
    let relations = vec![
        ExtractedRelation::new("SubjA", "ObjA", "REL", 0.70),
        ExtractedRelation::new("SubjB", "ObjB", "REL", 0.70001),
    ];

    let ingested = ingest_extracted_relations(&graph, &relations, 0.70, Some(DocId::from(1u64)))
        .expect("ingest must succeed");
    assert_eq!(
        ingested.len(),
        2,
        "Both relations meeting or exceeding 0.70 must be ingested"
    );

    let ingested_strict =
        ingest_extracted_relations(&graph, &relations, 0.700005, Some(DocId::from(2u64)))
            .expect("ingest must succeed");
    assert_eq!(
        ingested_strict.len(),
        1,
        "Only 0.70001 meets 0.700005 threshold"
    );
}

#[test]
fn test_extraction_determinism_repeated_runs() {
    let extractor = RuleBasedEntityExtractor::default();
    let text =
        "Ada Lovelace founded AnalyticalEngine Corp in London. Charles Babbage leads EngineTeam.";

    let res1 = extractor.extract(text);
    let res2 = extractor.extract(text);

    assert_eq!(
        res1, res2,
        "Extraction must be strictly deterministic across runs"
    );
    assert_eq!(res1.len(), 2);
}

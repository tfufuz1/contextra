//! Integrationstest-Suite: `structured_golden_tests.rs`
//!
//! # Zweck & Testabdeckung
//! Verifiziert das Verhalten, Zero-Panic-Invariante und deterministische Ausgabe
//! von `contextra-ingest` fuer dieses Modul/Format gemaess Spezifikation.
//!
// FILE-CONTEXT
// ZWECK: Golden-Tests für strukturierte Extrakte (Json, Jsonl, Xml, Yaml, Toml).
// INVARIANTEN: Zero-Panic, All-features / All-targets Konformität.
// STAND: TS:2026-10-31T00:00:00Z

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use contextra_ingest::error::IngestError;
use contextra_ingest::extractor::{DocumentExtractor, ExtractContext, ResourceBudget};
use contextra_ingest::format::FormatId;
use contextra_ingest::formats::structured::StructuredExtractor;
use contextra_ingest::ir::{BlockKind, WarningCode};
use contextra_ingest::limits::{ExtractPolicy, IngestLimits};
use contextra_ingest::registry::ExtractorRegistry;
use std::collections::BTreeMap;

fn create_ctx<'a>(
    limits: &'a IngestLimits,
    policy: &'a ExtractPolicy,
    budget: &'a ResourceBudget,
    registry: &'a ExtractorRegistry,
) -> ExtractContext<'a> {
    ExtractContext {
        limits,
        policy,
        budget,
        registry,
        filename: None,
    }
}

fn extract_records(
    extractor: &StructuredExtractor,
    format: FormatId,
    bytes: &[u8],
) -> Vec<(String, String)> {
    let limits = IngestLimits::default();
    let policy = ExtractPolicy::default();
    let budget = ResourceBudget::new(limits.clone());
    let reg = ExtractorRegistry::new_empty();
    let ctx = create_ctx(&limits, &policy, &budget, &reg);

    let ir = extractor
        .extract(format, bytes, &ctx)
        .expect("Extraction should succeed");
    ir.blocks
        .into_iter()
        .filter_map(|b| match b.kind {
            BlockKind::Record { path, value } => Some((path, value)),
            _ => None,
        })
        .collect()
}

// ANCHOR[contextra-ingest:TST-0417]
// Testet die Funktionsfaehigkeit von `test_golden_cross_format_logical_equivalence`.
#[test]
fn test_golden_cross_format_logical_equivalence() {
    let extractor = StructuredExtractor::new();

    let json_bytes = include_bytes!("fixtures/structured/sample.json");
    let yaml_bytes = include_bytes!("fixtures/structured/sample.yaml");
    let toml_bytes = include_bytes!("fixtures/structured/sample.toml");

    let json_records = extract_records(&extractor, FormatId::Json, json_bytes);
    let yaml_records = extract_records(&extractor, FormatId::Yaml, yaml_bytes);
    let toml_records = extract_records(&extractor, FormatId::Toml, toml_bytes);

    let json_map: BTreeMap<_, _> = json_records.into_iter().collect();
    let yaml_map: BTreeMap<_, _> = yaml_records.into_iter().collect();
    let toml_map: BTreeMap<_, _> = toml_records.into_iter().collect();

    assert_eq!(json_map["title"], "Golden Dataset");
    assert_eq!(json_map["[\"kunden_überschrift\"]"], "Musterkunde ÄÖÜäöüß");
    assert_eq!(json_map["adresse.plz"], "12345");
    assert_eq!(json_map["adresse.stadt"], "Berlin");
    assert_eq!(json_map["bestellungen[0].pos"], "1");
    assert_eq!(json_map["bestellungen[0].produkt"], "Laptop");

    assert_eq!(json_map, yaml_map);
    assert_eq!(json_map, toml_map);
}

// ANCHOR[contextra-ingest:TST-0418]
// Testet die Funktionsfaehigkeit von `test_special_character_keys`.
#[test]
fn test_special_character_keys() {
    let extractor = StructuredExtractor::new();
    let json_src = br#"{
        "user.name": "Alice",
        "zip code": "12345",
        "3d_model": "Mesh",
        "quote\"in\"key": "Val",
        "": "EmptyKey"
    }"#;

    let records = extract_records(&extractor, FormatId::Json, json_src);
    let map: BTreeMap<_, _> = records.into_iter().collect();

    assert_eq!(map["[\"user.name\"]"], "Alice");
    assert_eq!(map["[\"zip code\"]"], "12345");
    assert_eq!(map["[\"3d_model\"]"], "Mesh");
    assert_eq!(map["[\"quote\\\"in\\\"key\"]"], "Val");
    assert_eq!(map["[\"\"]"], "EmptyKey");
}

// ANCHOR[contextra-ingest:TST-0419]
// Testet die Funktionsfaehigkeit von `test_german_umlauts_in_keys_and_values`.
#[test]
fn test_german_umlauts_in_keys_and_values() {
    let extractor = StructuredExtractor::new();
    let json_src = r#"{
        "überschrift": "München und Düsseldorf",
        "äquivalenz": "großschreibung ÄÖÜß"
    }"#
    .as_bytes();

    let records = extract_records(&extractor, FormatId::Json, json_src);
    let map: BTreeMap<_, _> = records.into_iter().collect();

    assert_eq!(map["[\"überschrift\"]"], "München und Düsseldorf");
    assert_eq!(map["[\"äquivalenz\"]"], "großschreibung ÄÖÜß");
}

// ANCHOR[contextra-ingest:TST-0420]
// Testet die Funktionsfaehigkeit von `test_duplicate_json_keys`.
#[test]
fn test_duplicate_json_keys() {
    let extractor = StructuredExtractor::new();
    let json_src = br#"{
        "key": "First Value",
        "key": "Second Value"
    }"#;

    let limits = IngestLimits::default();
    let policy = ExtractPolicy::default();
    let budget = ResourceBudget::new(limits.clone());
    let reg = ExtractorRegistry::new_empty();
    let ctx = create_ctx(&limits, &policy, &budget, &reg);

    let ir = extractor
        .extract(FormatId::Json, json_src, &ctx)
        .expect("Duplicate keys should succeed");

    assert_eq!(ir.blocks.len(), 2);
    assert_eq!(ir.warnings.len(), 1);
    assert_eq!(ir.warnings[0].code, WarningCode::FeatureLimited);

    let r0 = match &ir.blocks[0].kind {
        BlockKind::Record { path, value } => (path.as_str(), value.as_str()),
        _ => panic!("Expected Record"),
    };
    let r1 = match &ir.blocks[1].kind {
        BlockKind::Record { path, value } => (path.as_str(), value.as_str()),
        _ => panic!("Expected Record"),
    };

    assert_eq!(r0, ("key", "First Value"));
    assert_eq!(r1, ("key", "Second Value"));
}

// ANCHOR[contextra-ingest:TST-0421]
// Testet die Funktionsfaehigkeit von `test_empty_containers`.
#[test]
fn test_empty_containers() {
    let extractor = StructuredExtractor::new();
    let json_src = br#"{
        "obj": {},
        "arr": []
    }"#;

    let records = extract_records(&extractor, FormatId::Json, json_src);
    let map: BTreeMap<_, _> = records.into_iter().collect();

    assert_eq!(map["obj"], "{}");
    assert_eq!(map["arr"], "[]");
}

// ANCHOR[contextra-ingest:TST-0422]
// Testet die Funktionsfaehigkeit von `test_jsonl_broken_lines_and_all_invalid`.
#[test]
fn test_jsonl_broken_lines_and_all_invalid() {
    let extractor = StructuredExtractor::new();

    // 1. Teils beschädigtes JSONL
    let jsonl_src = b"{\"valid\": 1}\n{broken_json}\n{\"valid\": 2}\n";
    let limits = IngestLimits::default();
    let policy = ExtractPolicy::default();
    let budget = ResourceBudget::new(limits.clone());
    let reg = ExtractorRegistry::new_empty();
    let ctx = create_ctx(&limits, &policy, &budget, &reg);

    let ir = extractor
        .extract(FormatId::Jsonl, jsonl_src, &ctx)
        .expect("JSONL extraction should succeed");
    assert_eq!(ir.blocks.len(), 2);
    assert_eq!(ir.warnings.len(), 1);
    assert_eq!(ir.warnings[0].code, WarningCode::FeatureLimited);

    // 2. Komplett beschädigtes JSONL -> Corrupt
    let corrupt_src = b"{broken_1}\n{broken_2}\n";
    let err = extractor
        .extract(FormatId::Jsonl, corrupt_src, &ctx)
        .unwrap_err();
    assert!(matches!(
        err,
        IngestError::Corrupt {
            format: Some(FormatId::Jsonl),
            ..
        }
    ));
}

// ANCHOR[contextra-ingest:TST-0423]
// Testet die Funktionsfaehigkeit von `test_yaml_multi_document_prefixing`.
#[test]
fn test_yaml_multi_document_prefixing() {
    let extractor = StructuredExtractor::new();
    let yaml_src = b"---\ntitle: Doc 1\nval: 100\n---\ntitle: Doc 2\nval: 200\n";

    let records = extract_records(&extractor, FormatId::Yaml, yaml_src);
    let map: BTreeMap<_, _> = records.into_iter().collect();

    assert_eq!(map["doc[0].title"], "Doc 1");
    assert_eq!(map["doc[0].val"], "100");
    assert_eq!(map["doc[1].title"], "Doc 2");
    assert_eq!(map["doc[1].val"], "200");
}

// ANCHOR[contextra-ingest:TST-0424]
// Testet die Funktionsfaehigkeit von `test_toml_array_of_tables`.
#[test]
fn test_toml_array_of_tables() {
    let extractor = StructuredExtractor::new();
    let toml_src = b"title = \"Shop\"\n\n[[items]]\nname = \"A\"\n\n[[items]]\nname = \"B\"\n";

    let records = extract_records(&extractor, FormatId::Toml, toml_src);
    let map: BTreeMap<_, _> = records.into_iter().collect();

    assert_eq!(map["title"], "Shop");
    assert_eq!(map["items[0].name"], "A");
    assert_eq!(map["items[1].name"], "B");
}

// ANCHOR[contextra-ingest:TST-0425]
// Testet die Funktionsfaehigkeit von `test_xml_attributes_namespaces_cdata`.
#[test]
fn test_xml_attributes_namespaces_cdata() {
    let extractor = StructuredExtractor::new();
    let xml_src = r#"<ns:root xmlns:ns="http://example.com" ns:version="2.0">
        <title>XML Title</title>
        <ns:item id="1"><![CDATA[Raw CDATA Content]]></ns:item>
        <ns:item id="2">Plain Content</ns:item>
    </ns:root>"#
        .as_bytes();

    let records = extract_records(&extractor, FormatId::Xml, xml_src);
    let map: BTreeMap<_, _> = records.into_iter().collect();

    assert_eq!(map["[\"ns:root\"][\"@xmlns:ns\"]"], "http://example.com");
    assert_eq!(map["[\"ns:root\"][\"@ns:version\"]"], "2.0");
    assert_eq!(map["[\"ns:root\"].title"], "XML Title");
    assert_eq!(map["[\"ns:root\"][\"ns:item\"][0].@id"], "1");
    assert_eq!(map["[\"ns:root\"][\"ns:item\"][0]"], "Raw CDATA Content");
    assert_eq!(map["[\"ns:root\"][\"ns:item\"][1].@id"], "2");
    assert_eq!(map["[\"ns:root\"][\"ns:item\"][1]"], "Plain Content");
}

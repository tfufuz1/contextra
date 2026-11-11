//! Golden-Extraktionstests (WP-2).
//!
//! # Zweck & Invarianten
//! Verifiziert, dass die Ingest-Pipeline für den Eingabe-Korpus deterministische,
//! bitidentische IR- und Chunk-Ergebnisse liefert (INV-1).
//! Ein Generator `generate_golden_outputs` (mit `#[ignore]` markiert) erzeugt
//! die Golden-Dateien und `MANIFEST.sha256`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use contextra_ingest::chunking::types::DocId;
use contextra_ingest::error::IngestError;
use contextra_ingest::pipeline::{ingest, IngestConfig, IngestRequest};
use contextra_ingest::registry::ExtractorRegistry;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct GoldenSnapshot {
    source_file: String,
    detected_format: String,
    title: Option<String>,
    canonical_hash: String,
    chunk_count: usize,
    chunks_summary: Vec<ChunkSummary>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ChunkSummary {
    index: usize,
    content: String,
    token_count: usize,
    metadata: Option<serde_json::Value>,
}

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("corpus")
}

fn golden_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("golden")
}

fn run_extraction_for_file(path: &Path) -> Option<GoldenSnapshot> {
    let bytes = fs::read(path).expect("corpus file readable");
    let file_name = path.file_name().unwrap().to_str().unwrap().to_string();

    let registry = ExtractorRegistry::with_defaults();
    let req = IngestRequest {
        bytes: &bytes,
        filename: Some(&file_name),
        mime: None,
        format_override: None,
        doc_id: DocId::new(100),
    };
    let cfg = IngestConfig::default();

    match ingest(&req, &cfg, &registry) {
        Ok(res) => {
            let chunks_summary = res
                .chunks
                .iter()
                .enumerate()
                .map(|(i, c)| ChunkSummary {
                    index: i,
                    content: c.content.clone(),
                    token_count: c.token_count,
                    metadata: c.metadata.clone(),
                })
                .collect();

            Some(GoldenSnapshot {
                source_file: file_name,
                detected_format: format!("{:?}", res.format),
                title: res.meta.title,
                canonical_hash: res.canonical_hash,
                chunk_count: res.chunks.len(),
                chunks_summary,
            })
        }
        Err(IngestError::UnsupportedFormat { .. }) => None,
        Err(e) => panic!("Ingestion error on corpus file {file_name}: {e:?}"),
    }
}

/// Ständiger Prüftest: stellt sicher, dass jede Datei im Korpus
/// genau die bitidentische Golden-JSON-Ausgabe erzeugt, sofern
/// das Format im aktuellen Feature-Satz aktiv ist.
#[test]
fn test_golden_outputs_bitidentical() {
    let c_root = corpus_root();
    let g_root = golden_root();

    assert!(c_root.exists(), "Corpus root exists");
    assert!(g_root.exists(), "Golden root exists");

    let mut entries: Vec<_> = fs::read_dir(&c_root)
        .expect("read corpus dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();

    entries.sort();
    assert!(!entries.is_empty(), "Corpus contains test files");

    for path in entries {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let golden_file = g_root.join(format!("{file_name}.golden.json"));
        assert!(
            golden_file.exists(),
            "Golden file missing for corpus input: {file_name}"
        );

        if let Some(actual_snapshot) = run_extraction_for_file(&path) {
            let expected_json = fs::read_to_string(&golden_file).expect("read golden json");
            let expected_snapshot: GoldenSnapshot =
                serde_json::from_str(&expected_json).expect("valid golden json schema");

            assert_eq!(
                actual_snapshot, expected_snapshot,
                "Bitidentical Golden invariant violated for: {file_name}"
            );
        }
    }
}

/// Generator-Test: erzeugt oder aktualisiert alle Golden-Dateien und `MANIFEST.sha256`.
/// Normalerweise `#[ignore]`.
#[test]
#[ignore]
fn generate_golden_outputs() {
    let c_root = corpus_root();
    let g_root = golden_root();
    fs::create_dir_all(&g_root).expect("create golden dir");

    let mut entries: Vec<_> = fs::read_dir(&c_root)
        .expect("read corpus dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();
    entries.sort();

    let mut manifest_lines = Vec::new();

    for path in &entries {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let bytes = fs::read(path).unwrap();
        let hash = blake3::hash(&bytes).to_hex().to_string();
        manifest_lines.push(format!("{hash}  corpus/{file_name}"));

        if let Some(snapshot) = run_extraction_for_file(path) {
            let golden_file = g_root.join(format!("{file_name}.golden.json"));
            let json_data = serde_json::to_string_pretty(&snapshot).unwrap();
            fs::write(&golden_file, &json_data).unwrap();

            let golden_hash = blake3::hash(json_data.as_bytes()).to_hex().to_string();
            manifest_lines.push(format!("{golden_hash}  golden/{file_name}.golden.json"));
        }
    }

    manifest_lines.sort();
    let manifest_path = g_root.join("MANIFEST.sha256");
    fs::write(manifest_path, manifest_lines.join("\n") + "\n").unwrap();
}

//! Fuzzing-Regressionstest-Suite (WP-4).
//!
//! # Zweck & Invarianten
//! Führt alle 14 Fuzzing-Ziele mit ihren jeweiligen Seed-Korpora aus,
//! um sicherzustellen, dass kein Seed zu einem Panic, Crash oder unzulässigen
//! Fehlerzustand führt (Zero-Panic Invariante INV-5).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use contextra_ingest::chunking::types::DocId;
use contextra_ingest::detect::{detect, DetectHints};
use contextra_ingest::format::FormatId;
use contextra_ingest::pipeline::{ingest, IngestConfig, IngestRequest};
use contextra_ingest::registry::ExtractorRegistry;
use std::fs;
use std::path::PathBuf;

fn fuzz_corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fuzz")
        .join("corpus")
}

fn for_each_seed<F: Fn(&[u8], &str)>(target_name: &str, test_fn: F) {
    let target_dir = fuzz_corpus_root().join(target_name);
    if !target_dir.exists() {
        return;
    }

    let entries = fs::read_dir(&target_dir).expect("read fuzz target corpus dir");
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let data = fs::read(&path).expect("read seed file");
            let file_name = path.file_name().unwrap().to_str().unwrap();
            test_fn(&data, file_name);
        }
    }
}

fn run_pipeline_for_format(format: FormatId, data: &[u8]) {
    let registry = ExtractorRegistry::with_defaults();
    let req = IngestRequest {
        bytes: data,
        filename: Some("fuzz_sample"),
        mime: None,
        format_override: Some(format),
        doc_id: DocId::new(1),
    };
    let cfg = IngestConfig::default();
    let _ = ingest(&req, &cfg, &registry);
}

#[test]
fn test_fuzz_seeds_detect() {
    for_each_seed("fuzz_detect", |data, _name| {
        let hints = DetectHints::default();
        let _ = detect(data, &hints);
    });
}

#[test]
fn test_fuzz_seeds_text() {
    for_each_seed("fuzz_text", |data, _name| {
        run_pipeline_for_format(FormatId::Txt, data);
        run_pipeline_for_format(FormatId::Markdown, data);
    });
}

#[test]
fn test_fuzz_seeds_web() {
    for_each_seed("fuzz_web", |data, _name| {
        run_pipeline_for_format(FormatId::Html, data);
    });
}

#[test]
fn test_fuzz_seeds_docx() {
    for_each_seed("fuzz_docx", |data, _name| {
        run_pipeline_for_format(FormatId::Docx, data);
    });
}

#[test]
fn test_fuzz_seeds_pptx() {
    for_each_seed("fuzz_pptx", |data, _name| {
        run_pipeline_for_format(FormatId::Pptx, data);
    });
}

#[test]
fn test_fuzz_seeds_sheets() {
    for_each_seed("fuzz_sheets", |data, _name| {
        run_pipeline_for_format(FormatId::Csv, data);
        run_pipeline_for_format(FormatId::Tsv, data);
    });
}

#[test]
fn test_fuzz_seeds_odf() {
    for_each_seed("fuzz_odf", |data, _name| {
        run_pipeline_for_format(FormatId::Odt, data);
    });
}

#[test]
fn test_fuzz_seeds_pdf() {
    for_each_seed("fuzz_pdf", |data, _name| {
        run_pipeline_for_format(FormatId::Pdf, data);
    });
}

#[test]
fn test_fuzz_seeds_ole() {
    for_each_seed("fuzz_ole", |data, _name| {
        run_pipeline_for_format(FormatId::Doc, data);
        run_pipeline_for_format(FormatId::Ppt, data);
        run_pipeline_for_format(FormatId::Msg, data);
    });
}

#[test]
fn test_fuzz_seeds_email() {
    for_each_seed("fuzz_email", |data, _name| {
        run_pipeline_for_format(FormatId::Eml, data);
        run_pipeline_for_format(FormatId::Mbox, data);
    });
}

#[test]
fn test_fuzz_seeds_structured() {
    for_each_seed("fuzz_structured", |data, _name| {
        run_pipeline_for_format(FormatId::Json, data);
        run_pipeline_for_format(FormatId::Yaml, data);
        run_pipeline_for_format(FormatId::Toml, data);
        run_pipeline_for_format(FormatId::Xml, data);
    });
}

#[test]
fn test_fuzz_seeds_pipeline() {
    let registry = ExtractorRegistry::with_defaults();
    let cfg = IngestConfig::default();

    for_each_seed("fuzz_pipeline", |data, name| {
        let req = IngestRequest {
            bytes: data,
            filename: Some(name),
            mime: None,
            format_override: None,
            doc_id: DocId::new(99),
        };
        let _ = ingest(&req, &cfg, &registry);
    });
}

#[test]
#[cfg(feature = "zip-container")]
fn test_fuzz_seeds_safe_archive() {
    use contextra_ingest::limits::IngestLimits;
    use contextra_ingest::safe::archive::validate_zip_archive;
    use std::io::Cursor;

    let limits = IngestLimits::default();
    for_each_seed("fuzz_safe_archive", |data, _name| {
        let _ = validate_zip_archive(Cursor::new(data), &limits);
    });
}

#[test]
#[cfg(feature = "xml")]
fn test_fuzz_seeds_safe_xml() {
    use contextra_ingest::extractor::ResourceBudget;
    use contextra_ingest::limits::IngestLimits;
    use contextra_ingest::safe::xml::parse_xml;

    let limits = IngestLimits::default();
    let budget = ResourceBudget::new(limits);
    for_each_seed("fuzz_safe_xml", |data, _name| {
        let _ = parse_xml(data, &budget);
    });
}

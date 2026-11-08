//! Criterion-Benchmark-Suite für contextra-ingest (SPEC v4.0 B8, Gate G6).
//!
//! Benchmarkt Parsing, Extraktion und Chunking für Text/Markdown,
//! Tabellendaten (CSV) und strukturierte Daten (JSON).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use contextra_ingest::chunking::types::DocId;
use contextra_ingest::format::FormatId;
use contextra_ingest::pipeline::{ingest, IngestConfig, IngestRequest};
use contextra_ingest::registry::ExtractorRegistry;

fn bench_markdown_ingest(c: &mut Criterion) {
    let registry = ExtractorRegistry::with_defaults();
    let config = IngestConfig::default();

    let sample_text = r#"
# Technische Architekturbeschreibung
Contextra ist eine hochgradig modulare Bibliothek zur Indizierung und semantischen Suche.

## Baukasten-Architektur
Jede Komponente ist ein autarker Baustein, der strengen Entkopplungsmustern folgt.
Abhängigkeiten zeigen stets nur nach unten. Keine zyklischen Abhängigkeiten.

### Segment-Speicher
Der LSM-Baum speichert Schlüssel-Werte-Paare in unveränderlichen SSTables.

### Chunking
Das Chunking erfolgt strukturbewusst unter Berücksichtigung von Dokumenthierarchien.
- Ebene 1: Kapitel
- Ebene 2: Abschnitte
- Ebene 3: Absätze und Listen
"#;

    c.bench_function("bm_ingest_markdown_text", |b| {
        b.iter(|| {
            let req = IngestRequest {
                bytes: sample_text.as_bytes(),
                filename: Some("architektur.md"),
                mime: Some("text/markdown"),
                format_override: Some(FormatId::Markdown),
                doc_id: DocId::new(1),
            };
            let out = ingest(black_box(&req), black_box(&config), black_box(&registry)).unwrap();
            black_box(out);
        });
    });
}

fn bench_csv_table_ingest(c: &mut Criterion) {
    let registry = ExtractorRegistry::with_defaults();
    let config = IngestConfig::default();

    let mut csv_data = String::from("id,name,role,department,score\n");
    for i in 0..100 {
        csv_data.push_str(&format!("{i},Mitarbeiter_{i},Engineer,F&E,{}.5\n", i % 10));
    }

    c.bench_function("bm_ingest_csv_table", |b| {
        b.iter(|| {
            let req = IngestRequest {
                bytes: csv_data.as_bytes(),
                filename: Some("personal.csv"),
                mime: Some("text/csv"),
                format_override: Some(FormatId::Csv),
                doc_id: DocId::new(2),
            };
            let out = ingest(black_box(&req), black_box(&config), black_box(&registry)).unwrap();
            black_box(out);
        });
    });
}

fn bench_json_structured_ingest(c: &mut Criterion) {
    let registry = ExtractorRegistry::with_defaults();
    let config = IngestConfig::default();

    let json_data = r#"{
        "projekt": "Contextra",
        "version": "4.0",
        "bausteine": [
            {"name": "contextra-types", "ring": 0},
            {"name": "contextra-store", "ring": 1},
            {"name": "contextra-vector", "ring": 2},
            {"name": "contextra-text", "ring": 2},
            {"name": "contextra-ingest", "ring": 4}
        ],
        "parameter": {
            "max_tokens": 512,
            "min_tokens": 64,
            "overlap": 32
        }
    }"#;

    c.bench_function("bm_ingest_json_structured", |b| {
        b.iter(|| {
            let req = IngestRequest {
                bytes: json_data.as_bytes(),
                filename: Some("config.json"),
                mime: Some("application/json"),
                format_override: Some(FormatId::Json),
                doc_id: DocId::new(3),
            };
            let out = ingest(black_box(&req), black_box(&config), black_box(&registry)).unwrap();
            black_box(out);
        });
    });
}

criterion_group!(
    benches,
    bench_markdown_ingest,
    bench_csv_table_ingest,
    bench_json_structured_ingest
);
criterion_main!(benches);

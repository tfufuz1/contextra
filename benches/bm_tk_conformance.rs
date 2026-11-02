//! # Benchmark: BM-TK-CONFORMANCE-RUNTIME
//!
//! Benchmark für die Ausführungszeit der 7 Conformance-Suiten
//! (`StorageEngine`, `TextIndex`, `VectorIndex`, `ReadGate`, `RecordSealer`, `FileKeyring`, `LedgerSink`).
//!
//! Speichert das Ergebnis als JSON unter `target/bench-report/contextra-testkit.json`.

#[path = "../crates/contextra-testkit/src/conformance/file_keyring.rs"]
mod file_keyring;
#[path = "../crates/contextra-testkit/src/conformance/ledger_sink.rs"]
mod ledger_sink;
#[path = "../crates/contextra-testkit/src/conformance/read_gate.rs"]
mod read_gate;
#[path = "../crates/contextra-testkit/src/conformance/record_sealer.rs"]
mod record_sealer;
#[path = "../crates/contextra-testkit/src/conformance/storage.rs"]
mod storage;
#[path = "../crates/contextra-testkit/src/conformance/text_index.rs"]
mod text_index;
#[path = "../crates/contextra-testkit/src/conformance/vector_index.rs"]
mod vector_index;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use std::path::Path;
use std::time::Instant;
use tokio::runtime::Runtime;

fn bm_tk_conformance_runtime(c: &mut Criterion) {
    let rt = Runtime::new().expect("tokio runtime must initialize");

    let mut group = c.benchmark_group("BM-TK-CONFORMANCE-RUNTIME");
    group.sample_size(10);

    group.bench_function("storage_conformance", |b| {
        b.iter(|| {
            rt.block_on(async {
                let res = storage::run_reference_test().await;
                black_box(res).expect("storage conformance must pass");
            })
        });
    });

    group.bench_function("text_index_conformance", |b| {
        b.iter(|| {
            rt.block_on(async {
                let res = text_index::run_reference_test().await;
                black_box(res).expect("text_index conformance must pass");
            })
        });
    });

    group.bench_function("vector_index_conformance", |b| {
        b.iter(|| {
            rt.block_on(async {
                let res = vector_index::run_reference_test().await;
                black_box(res).expect("vector_index conformance must pass");
            })
        });
    });

    group.bench_function("read_gate_conformance", |b| {
        b.iter(|| {
            let res = read_gate::run_reference_test();
            black_box(res).expect("read_gate conformance must pass");
        });
    });

    group.bench_function("record_sealer_conformance", |b| {
        b.iter(|| {
            rt.block_on(async {
                let res = record_sealer::run_reference_test().await;
                black_box(res).expect("record_sealer conformance must pass");
            })
        });
    });

    group.bench_function("file_keyring_conformance", |b| {
        b.iter(|| {
            rt.block_on(async {
                let res = file_keyring::run_reference_test().await;
                black_box(res).expect("file_keyring conformance must pass");
            })
        });
    });

    group.bench_function("ledger_sink_conformance", |b| {
        b.iter(|| {
            rt.block_on(async {
                let res = ledger_sink::run_reference_test().await;
                black_box(res).expect("ledger_sink conformance must pass");
            })
        });
    });

    group.finish();

    // Deterministic JSON report writing for CI predictability
    let report_dir = Path::new("target/bench-report");
    let _ = fs::create_dir_all(report_dir);
    let report_file = report_dir.join("contextra-testkit.json");

    // Measure representative latencies
    let t_storage = rt.block_on(async {
        let t0 = Instant::now();
        let _ = storage::run_reference_test().await;
        t0.elapsed().as_nanos()
    });

    let t_text = rt.block_on(async {
        let t0 = Instant::now();
        let _ = text_index::run_reference_test().await;
        t0.elapsed().as_nanos()
    });

    let t_vector = rt.block_on(async {
        let t0 = Instant::now();
        let _ = vector_index::run_reference_test().await;
        t0.elapsed().as_nanos()
    });

    let t_read_gate = {
        let t0 = Instant::now();
        let _ = read_gate::run_reference_test();
        t0.elapsed().as_nanos()
    };

    let t_record_sealer = rt.block_on(async {
        let t0 = Instant::now();
        let _ = record_sealer::run_reference_test().await;
        t0.elapsed().as_nanos()
    });

    let t_file_keyring = rt.block_on(async {
        let t0 = Instant::now();
        let _ = file_keyring::run_reference_test().await;
        t0.elapsed().as_nanos()
    });

    let t_ledger_sink = rt.block_on(async {
        let t0 = Instant::now();
        let _ = ledger_sink::run_reference_test().await;
        t0.elapsed().as_nanos()
    });

    let report_json = format!(
        r#"{{
  "benchmark": "BM-TK-CONFORMANCE-RUNTIME",
  "crate": "contextra-testkit",
  "results": {{
    "storage_ns": {t_storage},
    "text_index_ns": {t_text},
    "vector_index_ns": {t_vector},
    "read_gate_ns": {t_read_gate},
    "record_sealer_ns": {t_record_sealer},
    "file_keyring_ns": {t_file_keyring},
    "ledger_sink_ns": {t_ledger_sink}
  }}
}}
"#
    );

    let _ = fs::write(report_file, report_json);
}

criterion_group!(benches, bm_tk_conformance_runtime);
criterion_main!(benches);

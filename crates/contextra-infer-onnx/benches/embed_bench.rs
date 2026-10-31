// FILE-CONTEXT
// STAND: 2026-10-31
// ZWECK: Criterion Benchmark für lokale ONNX Text-Embeddings (Ring 2).

//! # Benchmark `embed_bench`
//!
//! Misst den Durchsatz und die Latenz von `TextEmbedder::embed_async` bei 100 aufeinanderfolgenden
//! Embedding-Aufrufen mittels Criterion.
//!
//! Falls keine Testmodelle unter `tests/data` hinterlegt sind, wird der Benchmark übersprungen.

#[cfg(feature = "onnx")]
use contextra_infer_onnx::TextEmbedder;
#[cfg(feature = "onnx")]
use criterion::{criterion_group, criterion_main, Criterion};
#[cfg(feature = "onnx")]
use std::path::PathBuf;
#[cfg(feature = "onnx")]
use tokio::runtime::Runtime;

// ANCHOR[contextra-infer-onnx:BENCH-EMBED-FN]
/// Führt den Durchsatz-Benchmark für 100 asynchrone Embedding-Aufrufe aus.
#[cfg(feature = "onnx")]
fn bench_embed(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut model_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    model_dir.push("tests/data");

    if !model_dir.join("model.onnx").exists() || !model_dir.join("tokenizer.json").exists() {
        println!("Skipping benchmark because ONNX model or tokenizer is missing in tests/data");
        return;
    }

    let embedder = TextEmbedder::load(&model_dir).expect("Failed to load TextEmbedder");

    c.bench_function("embed_async_100_calls", |b| {
        b.iter(|| {
            rt.block_on(async {
                for _ in 0..100 {
                    let _ = embedder
                        .embed_async("This is a benchmark test sentence for ONNX embeddings.")
                        .await
                        .unwrap();
                }
            })
        })
    });
}

#[cfg(feature = "onnx")]
criterion_group!(benches, bench_embed);
#[cfg(feature = "onnx")]
criterion_main!(benches);

/// Leerer Eintrittspunkt, falls das Feature `onnx` nicht aktiv ist.
#[cfg(not(feature = "onnx"))]
fn main() {}

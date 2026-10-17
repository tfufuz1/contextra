#[cfg(feature = "onnx")]
use contextra_infer_onnx::TextEmbedder;
#[cfg(feature = "onnx")]
use criterion::{criterion_group, criterion_main, Criterion};
#[cfg(feature = "onnx")]
use std::path::PathBuf;
#[cfg(feature = "onnx")]
use tokio::runtime::Runtime;

#[cfg(feature = "onnx")]
fn bench_embed(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut model_dir = manifest_dir.join("tests").join("data");
    if !model_dir.join("model.onnx").exists() {
        model_dir = manifest_dir.join("tests").join("fixtures");
    }

    if !model_dir.join("model.onnx").exists() || !model_dir.join("tokenizer.json").exists() {
        println!("Skipping benchmark because ONNX model or tokenizer is missing in tests/data and tests/fixtures");
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

    c.bench_function("embed_batch_100_texts", |b| {
        let texts = vec!["This is a benchmark test sentence for ONNX embeddings."; 100];
        b.iter(|| {
            rt.block_on(async {
                let _ = embedder.embed_batch_async(&texts).await.unwrap();
            })
        })
    });
}

#[cfg(feature = "onnx")]
criterion_group!(benches, bench_embed);
#[cfg(feature = "onnx")]
criterion_main!(benches);

#[cfg(not(feature = "onnx"))]
fn main() {}

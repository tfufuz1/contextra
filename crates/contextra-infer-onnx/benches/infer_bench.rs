use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use contextra_infer_onnx::traits::{Embedder, Reranker};
use contextra_infer_onnx::{FakeEmbedder, FakeReranker};

fn bench_embeddings(c: &mut Criterion) {
    let embedder = FakeEmbedder::new(384).expect("valid embedder");
    let mut group = c.benchmark_group("fake_embedder_batch");

    for size in [1, 8, 32, 64] {
        let texts: Vec<&str> = vec!["Contextra autarker Baustein Inferenz Benchmark"; size];
        group.bench_with_input(BenchmarkId::from_parameter(size), &texts, |b, inputs| {
            b.iter(|| {
                embedder.embed_batch(inputs).expect("embed ok");
            });
        });
    }
    group.finish();
}

fn bench_reranking(c: &mut Criterion) {
    let reranker = FakeReranker::new();
    let query = "contextra information retrieval";
    let docs = vec![
        "contextra information retrieval with fast bm25 and embeddings",
        "unrelated text with zero matches",
        "contextra retrieval engine benchmark",
    ];

    c.bench_function("fake_reranker_score", |b| {
        b.iter(|| {
            reranker.score(query, &docs).expect("score ok");
        });
    });
}

criterion_group!(benches, bench_embeddings, bench_reranking);
criterion_main!(benches);

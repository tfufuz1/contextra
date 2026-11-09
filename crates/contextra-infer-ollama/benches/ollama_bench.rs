use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_prompt_building(c: &mut Criterion) {
    let mut group = c.benchmark_group("prompt");
    let doc = "Dies ist ein längeres Dokument für RAG-Kontext. ".repeat(50);
    let chunk = "Spezifischer Chunk-Inhalt für RAG Retrieval.".repeat(5);
    let query = "Was steht im Dokument über RAG?";

    group.bench_function("build_rag_prompt", |b| {
        b.iter(|| {
            contextra_infer_ollama::build_rag_prompt(
                black_box(&doc),
                black_box(&chunk),
                black_box(query),
            );
        })
    });

    group.bench_function("xml_escape", |b| {
        b.iter(|| {
            contextra_infer_ollama::xml_escape(black_box(&doc));
        })
    });

    group.finish();
}

fn bench_response_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");
    let response = "Based on our model analysis, the calculated importance is 0.85.";

    group.bench_function("parse_importance_score", |b| {
        b.iter(|| {
            contextra_infer_ollama::parse_importance_score_response(black_box(response));
        })
    });

    group.finish();
}

criterion_group!(benches, bench_prompt_building, bench_response_parsing);
criterion_main!(benches);

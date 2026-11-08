use criterion::{black_box, criterion_group, criterion_main, Criterion};

use contextra_infer_ollama::importance::parse_importance_score_response;
use contextra_infer_ollama::model_info::known_dimension;
use contextra_infer_ollama::xml_escape::xml_escape;

fn bench_xml_escape(c: &mut Criterion) {
    let payload = "This is a <test> with & special \"characters\" and 'single' quotes across multiple lines.\nNext line <tag attr=\"value\">content</tag>";
    c.bench_function("xml_escape", |b| b.iter(|| xml_escape(black_box(payload))));
}

fn bench_importance_response_parsing(c: &mut Criterion) {
    let response =
        "Based on comprehensive analysis of memory utility, the rating is 0.85 (High utility).";
    c.bench_function("parse_importance_score_response", |b| {
        b.iter(|| parse_importance_score_response(black_box(response)))
    });
}

fn bench_known_dimension(c: &mut Criterion) {
    c.bench_function("known_dimension", |b| {
        b.iter(|| known_dimension(black_box("nomic-embed-text")))
    });
}

criterion_group!(
    benches,
    bench_xml_escape,
    bench_importance_response_parsing,
    bench_known_dimension
);
criterion_main!(benches);

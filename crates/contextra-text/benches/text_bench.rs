use contextra_text::bm25::BM25;
use contextra_text::tokenizer::{DefaultTokenizer, GermanMorphTokenizer, Tokenizer};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_tokenizer(c: &mut Criterion) {
    let default_tok = DefaultTokenizer;
    let german_tok = GermanMorphTokenizer::try_new().expect("GermanMorphTokenizer init");
    let sample_text = "Die Donaudampfschifffahrtselektrizitätenhauptbetriebswerkbauunterbeamtengesellschaft und moderne Volltextsuche";

    c.bench_function("default_tokenizer_sample", |b| {
        b.iter(|| default_tok.tokenize(black_box(sample_text)));
    });

    c.bench_function("german_morph_tokenizer_sample", |b| {
        b.iter(|| german_tok.tokenize(black_box(sample_text)));
    });
}

fn bench_bm25_scoring(c: &mut Criterion) {
    let bm25 = BM25::default();

    c.bench_function("bm25_score_term", |b| {
        b.iter(|| {
            let score = bm25.score_term(
                black_box(3),
                black_box(120),
                black_box(100.0),
                black_box(15),
                black_box(1_000),
            );
            black_box(score);
        });
    });
}

criterion_group!(
    benches,
    bench_tokenizer,
    bench_bm25_scoring,
    bench_analyzer_pipeline
);
criterion_main!(benches);

fn bench_analyzer_pipeline(c: &mut Criterion) {
    use contextra_text::tokenizer::{Analyzer, AnalyzerPipeline};

    let p_en = AnalyzerPipeline::default_english();
    let p_de = AnalyzerPipeline::default_german();

    let short_text = "Die Softwarearchitektur von Contextra ist modular und deterministisch.";
    let long_text = "Das Bundesverfassungsgericht entschied über die Verfassungsmäßigkeit der Regelung zur Vorratsdatenspeicherung und Datenschutzrichtlinie. ".repeat(50);

    c.bench_function("analyzer_pipeline_short_de", |b| {
        b.iter(|| p_de.analyze(black_box(short_text)));
    });

    c.bench_function("analyzer_pipeline_long_de", |b| {
        b.iter(|| p_de.analyze(black_box(&long_text)));
    });

    c.bench_function("analyzer_pipeline_short_en", |b| {
        b.iter(|| p_en.analyze(black_box(short_text)));
    });

    c.bench_function("analyzer_pipeline_long_en", |b| {
        b.iter(|| p_en.analyze(black_box(&long_text)));
    });
}

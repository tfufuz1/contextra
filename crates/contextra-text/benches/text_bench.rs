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

criterion_group!(benches, bench_tokenizer, bench_bm25_scoring);
criterion_main!(benches);

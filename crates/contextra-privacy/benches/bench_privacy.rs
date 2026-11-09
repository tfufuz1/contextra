#![allow(clippy::expect_used, clippy::unwrap_used)]

use contextra_privacy::{
    BulkExfiltrationDetector, EgressVault, RuleBasedEntityRecognizer, SessionId,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::time::Duration;

fn bench_entity_recognizer(c: &mut Criterion) {
    let recognizer = RuleBasedEntityRecognizer::new();
    let text = "Sehr geehrte Damen und Herren, bitte überweisen Sie den Betrag von 1.500 EUR an IBAN DE89370400440532013000 (BIC: DEUTDEDDFXX) für Max Mustermann (E-Mail: max.mustermann@example.com, Tel: +49 170 1234567, St-Nr: 12345678903).";

    let mut group = c.benchmark_group("entity_recognizer");
    group.throughput(Throughput::Bytes(text.len() as u64));

    group.bench_function("detect_mixed_pii", |b| {
        b.iter(|| {
            let res = recognizer.recognize(black_box(text));
            black_box(res);
        })
    });

    group.bench_function("detect_detailed_mixed_pii", |b| {
        b.iter(|| {
            let res = recognizer.recognize_detailed(black_box(text));
            black_box(res);
        })
    });

    let zalgo_text = "a\u{0300}\u{0301}d\u{0302}m\u{0303}i\u{0304}n\u{0305}@\u{0306}e\u{0307}x\u{0308}a\u{0309}m\u{030A}p\u{030B}l\u{030C}e\u{030D}.\u{030E}c\u{030F}o\u{0310}m\u{0311}";
    group.throughput(Throughput::Bytes(zalgo_text.len() as u64));
    group.bench_function("detect_evasion_zalgo", |b| {
        b.iter(|| {
            let res = recognizer.recognize(black_box(zalgo_text));
            black_box(res);
        })
    });
    group.finish();
}

fn bench_egress_vault(c: &mut Criterion) {
    let vault = EgressVault::try_default().expect("valid vault");
    let recognizer = RuleBasedEntityRecognizer::new();
    let text = "Contact alice@example.com regarding IBAN DE89370400440532013000 or credit card 4532-0150-1234-5678.";

    let mut group = c.benchmark_group("egress_vault");
    group.throughput(Throughput::Bytes(text.len() as u64));

    group.bench_function("sanitize_and_vault", |b| {
        b.iter(|| {
            let res = vault.sanitize_and_vault(black_box(text), &recognizer);
            black_box(res).expect("sanitization succeeds");
        })
    });
    group.finish();
}

fn bench_bulk_exfiltration(c: &mut Criterion) {
    let detector = BulkExfiltrationDetector::new(100_000, Duration::from_secs(60));
    let session = SessionId::from("bench_session");

    let mut group = c.benchmark_group("bulk_exfiltration");
    group.bench_function("record_and_check", |b| {
        b.iter(|| {
            let res = detector.record_and_check(black_box(session.clone()), black_box(128));
            black_box(res);
        })
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_entity_recognizer,
    bench_egress_vault,
    bench_bulk_exfiltration
);
criterion_main!(benches);

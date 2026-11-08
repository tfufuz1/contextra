//! Criterion-Benchmark für `contextra-avv-generator`.
//!
//! Misst die Durchlaufzeiten für:
//! - Markdown-Escaping (`escape_markdown`)
//! - Gesamtes AVV-Rendering (`render_avv_markdown`)

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use contextra_avv_generator::{
    default_technical_measures, render_avv_markdown, template::escape_markdown, AvvContext,
    TenantId,
};

fn bench_escape_markdown(c: &mut Criterion) {
    let input = "Kanzlei *Müller* & Co\n## Hacker-Titel mit [Links](http://example.com) und `Code`";

    c.bench_function("escape_markdown", |b| {
        b.iter(|| {
            let res = escape_markdown(black_box(input));
            black_box(res)
        });
    });
}

fn bench_render_avv(c: &mut Criterion) {
    let ctx = AvvContext::new(
        "Musterkanzlei AG",
        "Contextra Hosting GmbH",
        TenantId(100),
        default_technical_measures(),
        vec![
            "EU-Cloud-Region-Frankfurt".to_string(),
            "Backup-Vault-Nürnberg".to_string(),
        ],
        14,
    )
    .with_contract_date("2026-11-08")
    .with_subprocessors_allowed(true);

    c.bench_function("render_avv_markdown", |b| {
        b.iter(|| {
            let md = render_avv_markdown(black_box(&ctx));
            black_box(md)
        });
    });
}

criterion_group!(benches, bench_escape_markdown, bench_render_avv);
criterion_main!(benches);

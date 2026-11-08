//! Criterion benchmarks for `contextra-audit-export`.

use contextra_audit_export::{
    bsi_mapping_table, render_bsi_mapping_markdown, render_register_csv, render_register_json,
    render_register_markdown, testkit::InMemoryProcessingRegisterSource, ProcessingRegisterSource,
    TenantId,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_audit_exports(c: &mut Criterion) {
    let Ok(tenant_id) = TenantId::try_new(42) else {
        return;
    };
    let source = InMemoryProcessingRegisterSource::with_sample_data_for(tenant_id);
    let Ok(entries) = source.collect_entries(tenant_id) else {
        return;
    };

    c.bench_function("render_register_json", |b| {
        b.iter(|| {
            if let Ok(res) = render_register_json(black_box(&entries)) {
                black_box(res);
            }
        });
    });

    c.bench_function("render_register_markdown", |b| {
        b.iter(|| {
            if let Ok(res) = render_register_markdown(black_box(&entries), black_box("2026-11-08"))
            {
                black_box(res);
            }
        });
    });

    c.bench_function("render_register_csv", |b| {
        b.iter(|| {
            if let Ok(res) = render_register_csv(black_box(&entries)) {
                black_box(res);
            }
        });
    });

    let bsi = bsi_mapping_table();
    c.bench_function("render_bsi_mapping_markdown", |b| {
        b.iter(|| {
            let res = render_bsi_mapping_markdown(black_box(&bsi));
            black_box(res);
        });
    });
}

criterion_group!(benches, bench_audit_exports);
criterion_main!(benches);

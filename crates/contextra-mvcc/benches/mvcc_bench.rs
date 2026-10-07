#![allow(clippy::expect_used, clippy::unwrap_used)]

use contextra_mvcc::{SeqLogEntry, SnapshotRegistry};
use contextra_types::DocId;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::Arc;

fn bench_snapshot_registry_min_active(c: &mut Criterion) {
    let mut group = c.benchmark_group("snapshot_registry_min_active");

    for density in [10, 1000, 100_000] {
        let registry = Arc::new(SnapshotRegistry::new());
        let _guards: Vec<_> = (1..=density)
            .map(|i| registry.acquire(|| i as u64 * 10))
            .collect();

        group.bench_with_input(
            BenchmarkId::new("min_active_seqno", density),
            &registry,
            |b, reg| {
                b.iter(|| black_box(reg.min_active_seqno()));
            },
        );
    }

    group.finish();
}

fn bench_snapshot_registry_register_drop(c: &mut Criterion) {
    let mut group = c.benchmark_group("snapshot_registry_register_drop");

    for density in [10, 1000, 100_000] {
        let registry = Arc::new(SnapshotRegistry::new());
        let _active_guards: Vec<_> = (1..=density)
            .map(|i| registry.register(i as u64 * 10))
            .collect();

        group.bench_with_input(
            BenchmarkId::new("register_and_drop", density),
            &registry,
            |b, reg| {
                b.iter(|| {
                    let lease = reg.acquire(|| 55);
                    black_box(&lease);
                    drop(lease);
                });
            },
        );
    }

    group.finish();
}

fn bench_seq_log_entry_is_visible(c: &mut Criterion) {
    let doc_id = DocId::from_key("bench_doc").expect("valid doc id");
    let entry = SeqLogEntry {
        doc_id,
        insert_seq: 100,
        delete_seq: Some(500),
    };

    c.bench_function("seq_log_entry_is_visible_o1", |b| {
        b.iter(|| {
            black_box(entry.is_visible(black_box(250)));
        });
    });
}

criterion_group!(
    benches,
    bench_snapshot_registry_min_active,
    bench_snapshot_registry_register_drop,
    bench_seq_log_entry_is_visible
);
criterion_main!(benches);

//! # Benchmarks für contextra-mvcc
//!
//! **Ring:** Ring 0 (Transaktionsisolation)
//!
//! Misst die Latenz und Skalierbarkeit zentraler Concurrency-Control-Operationen:
//! - Lock-freies Auslesen der minimal aktiven Sequenznummer (`min_active_seqno`) unter variierender Snapshot-Dichte.
//! - Registrierung und Freigabe von RAII-Snapshot-Guards (`register_and_drop`).
//! - O(1)-Sichtbarkeitsprüfung von Sequenzlog-Einträgen (`is_visible`).

#![allow(clippy::expect_used, clippy::unwrap_used)]

use contextra_mvcc::{SeqLogEntry, SnapshotRegistry};
use contextra_types::DocId;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::Arc;

// ANCHOR[contextra-mvcc:BENCH-001]
/// Benchmark für das lock-freie Auslesen von `min_active_seqno` bei 10, 1.000 und 100.000 aktiven Snapshots.
fn bench_snapshot_registry_min_active(c: &mut Criterion) {
    let mut group = c.benchmark_group("snapshot_registry_min_active");

    for density in [10, 1000, 100_000] {
        let registry = Arc::new(SnapshotRegistry::new());
        let _guards: Vec<_> = (1..=density)
            .map(|i| registry.register(u64::try_from(i).unwrap_or(0) * 10))
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

// ANCHOR[contextra-mvcc:BENCH-002]
/// Benchmark für die Registrierung und den sofortigen Drop eines `SnapshotGuard`.
fn bench_snapshot_registry_register_drop(c: &mut Criterion) {
    let mut group = c.benchmark_group("snapshot_registry_register_drop");

    for density in [10, 1000, 100_000] {
        let registry = Arc::new(SnapshotRegistry::new());
        let _active_guards: Vec<_> = (1..=density)
            .map(|i| registry.register(u64::try_from(i).unwrap_or(0) * 10))
            .collect();

        group.bench_with_input(
            BenchmarkId::new("register_and_drop", density),
            &registry,
            |b, reg| {
                b.iter(|| {
                    let guard = reg.register(55);
                    black_box(&guard);
                    drop(guard);
                });
            },
        );
    }

    group.finish();
}

// ANCHOR[contextra-mvcc:BENCH-003]
/// Benchmark für die O(1)-Sichtbarkeitsprüfung `SeqLogEntry::is_visible`.
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

// ANCHOR[contextra-mvcc:BENCH-ROOT]
//! # Benchmarks für `contextra-mvcc`
//!
//! Misst die Durchsatz- und Latenzleistung kritischer Kernoperationen:
//! - Lock-freies Auslesen der minimal aktiven Sequenznummer (`min_active_seqno`) über AtomicU64 (Acquire-Ordering).
//! - Registrierung und Freigabe von SnapshotGuards unter verschiedenen Snapshot-Dichten (10 bis 100.000 aktive Snapshots).
//! - O(1)-Sichtbarkeitsprüfung von `SeqLogEntry::is_visible`.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use contextra_mvcc::{SeqLogEntry, SnapshotRegistry};
use contextra_types::DocId;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::sync::Arc;

// Benchmark für die lock-freie Abfrage von min_active_seqno bei verschiedenen Snapshot-Dichten.
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

// Benchmark für den Registrierungs- und Deregistrierungszyklus (Register + Drop) eines SnapshotGuards.
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

// Benchmark für die O(1)-Sichtbarkeitsprüfung eines SeqLogEntry.
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

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

// Benchmark für die SSI-Validierung und Konfliktprüfung je nach Größe der Lesemenge.
fn bench_ssi_validation_by_read_set_size(c: &mut Criterion) {
    use contextra_mvcc::ssi::{ReadSet, SequenceLogSsiValidator, SsiValidator};
    use contextra_types::TxId;

    let mut group = c.benchmark_group("ssi_validation_by_read_set_size");

    for size in [10, 100, 1000] {
        let validator = SequenceLogSsiValidator::new();
        // Commits registrieren
        for i in 0..size {
            let key = format!("bench_key_{i}").into_bytes();
            validator.record_commit_key(&key, 50);
        }

        let mut read_set = ReadSet::new();
        for i in 0..size {
            let key = format!("bench_key_{i}").into_bytes();
            read_set.record_read(key, 100); // snapshot 100 > commit 50 => kein Konflikt
        }

        let tx = TxId::new(42);
        group.bench_with_input(
            BenchmarkId::new("validate_disjoint", size),
            &(&validator, &read_set),
            |b, (val, rs)| {
                b.iter(|| {
                    black_box(val.validate(tx, black_box(rs)).is_ok());
                });
            },
        );
    }

    group.finish();
}

// Benchmark für Commit-Durchsatz (Commits/s via validate_and_record).
fn bench_ssi_commit_throughput(c: &mut Criterion) {
    use contextra_mvcc::ssi::{ReadSet, SequenceLogSsiValidator};
    use contextra_types::TxId;

    c.bench_function("ssi_commit_throughput", |b| {
        let validator = SequenceLogSsiValidator::new();
        let read_set = ReadSet::new();
        let key = b"bench_write_key".as_slice();
        let write_keys = [key];
        let mut seq = 100u64;

        b.iter(|| {
            seq += 1;
            black_box(
                validator
                    .validate_and_record(TxId::new(seq), &read_set, write_keys, seq)
                    .is_ok(),
            );
        });
    });
}

criterion_group!(
    benches,
    bench_snapshot_registry_min_active,
    bench_snapshot_registry_register_drop,
    bench_seq_log_entry_is_visible,
    bench_ssi_validation_by_read_set_size,
    bench_ssi_commit_throughput
);
criterion_main!(benches);

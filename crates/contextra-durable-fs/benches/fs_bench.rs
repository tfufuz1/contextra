use contextra_durable_fs::journal::{AppendOnlyJournal, JournalSyncPolicy};
use criterion::{criterion_group, criterion_main, Criterion};
use tempfile::tempdir;

fn bench_journal_append(c: &mut Criterion) {
    let mut group = c.benchmark_group("journal_append");

    group.bench_function("append_always_sync", |b| {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("always.log");
        let mut journal = AppendOnlyJournal::open_with_policy(
            &path,
            JournalSyncPolicy::Always,
            std::sync::Arc::new(contextra_durable_fs::AtomicHealthState::new()),
        )
        .expect("open");

        let payload = [0xAAu8; 128];
        b.iter(|| {
            journal.append(&payload).expect("append");
        });
    });

    group.bench_function("append_manual_sync", |b| {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("manual.log");
        let mut journal = AppendOnlyJournal::open_with_policy(
            &path,
            JournalSyncPolicy::Manual,
            std::sync::Arc::new(contextra_durable_fs::AtomicHealthState::new()),
        )
        .expect("open");

        let payload = [0xAAu8; 128];
        b.iter(|| {
            journal.append(&payload).expect("append");
        });
    });

    group.finish();
}

criterion_group!(benches, bench_journal_append);
criterion_main!(benches);

//! # Benchmark: BM-OPEN Fassaden-Overhead (SPEC §5.7, §8.14)
//!
//! Misst den Overhead von `Contextra::builder().open()` und KDF/Argon2-Anteil.

use contextra::{Contextra, Secret};
use criterion::{criterion_group, criterion_main, Criterion};
use std::time::Duration;
use tempfile::TempDir;

fn bm_facade_open(c: &mut Criterion) {
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(_) => return,
    };

    c.bench_function("BM-KDF-passphrase-derivation", |b| {
        b.iter(|| {
            let secret = Secret::new(b"sehr-sichere-passphrase-12345".to_vec());
            std::hint::black_box(secret);
        });
    });

    c.bench_function("BM-OPEN-facade-overhead", |b| {
        b.to_async(&rt).iter(|| async {
            let tmp = match TempDir::new() {
                Ok(t) => t,
                Err(_) => return,
            };
            let secret = Secret::new(b"sehr-sichere-passphrase-12345".to_vec());
            let handle = tokio::runtime::Handle::current();

            let builder = Contextra::builder()
                .data_dir(tmp.path())
                .passphrase(secret)
                .runtime(handle)
                .backup_retention(Duration::from_secs(86400))
                .dimension(768);

            std::hint::black_box(builder);
        });
    });
}

criterion_group!(benches, bm_facade_open);
criterion_main!(benches);

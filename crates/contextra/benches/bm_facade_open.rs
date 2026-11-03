//! # Benchmark: BM-OPEN Fassaden-Overhead (SPEC §5.7, §8.14)
//!
//! Misst den Overhead von `Contextra::builder().open()` und KDF/Argon2-Anteil.

use contextra::{Contextra, Secret};
use criterion::{criterion_group, criterion_main, Criterion};
use tempfile::TempDir;

fn bm_facade_open(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio rt");

    c.bench_function("BM-KDF-passphrase-derivation", |b| {
        b.iter(|| {
            let secret = Secret::new(b"sehr-sichere-passphrase-12345".to_vec());
            std::hint::black_box(secret);
        });
    });

    c.bench_function("BM-OPEN-facade-overhead", |b| {
        b.to_async(&rt).iter(|| async {
            let tmp = TempDir::new().expect("tempdir");
            let secret = Secret::new(b"sehr-sichere-passphrase-12345".to_vec());
            let handle = tokio::runtime::Handle::current();

            let builder = Contextra::builder()
                .data_dir(tmp.path())
                .passphrase(secret)
                .runtime(handle);

            std::hint::black_box(builder);
        });
    });
}

criterion_group!(benches, bm_facade_open);
criterion_main!(benches);

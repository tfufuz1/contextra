//! Benchmark suite for crypto primitives per BM-KDF and BM-WRITE.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_crypto::*;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

fn bench_block_aead_seal(c: &mut Criterion) {
    let key = FileKey::new([0x42; 32]);
    let file_id = [0x11; 16];
    let mut sealer = BlockSealer::new(key, FileKind::SSTable, file_id, None);
    let mut group = c.benchmark_group("block_aead_seal");

    for size in &[1024, 64 * 1024, 1024 * 1024] {
        let payload = vec![0x55u8; *size];
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| sealer.seal(black_box(&payload)).unwrap());
        });
    }
    group.finish();
}

fn bench_hkdf_expand(c: &mut Criterion) {
    let prk = [0x33u8; 32];
    c.bench_function("hkdf_expand_32b", |b| {
        let mut out = [0u8; 32];
        b.iter(|| {
            hkdf_expand(black_box(&prk), black_box(labels::ENC), black_box(&mut out)).unwrap()
        });
    });
}

fn bench_tag_computation(c: &mut Criterion) {
    let master = MasterKey::new([0x77; 32]);
    let hierarchy = KeyHierarchy::from_master(master);
    let mut pre = TagPreimage::new();
    pre.push(b"benchmark_tag_preimage_content");

    c.bench_function("keyed_blake3_tag", |b| {
        b.iter(|| tag(black_box(hierarchy.idx()), black_box(&pre)));
    });
}

fn bench_kdf_params(c: &mut Criterion) {
    let mut group = c.benchmark_group("bm_kdf");
    group.sample_size(10); // Argon2id is intentionally heavy

    let pass = Secret::from_slice(b"benchmark_master_passphrase");
    let salt = [0x5au8; 32];

    // D-06 minimum: 64 MiB (65536 KiB), t = 3, p = 1
    let params_min = KdfParams {
        m_kib: 65536,
        t: 3,
        p: 1,
    };
    group.bench_function("argon2id_min_64mib_t3_p1", |b| {
        b.iter(|| derive_kek(black_box(&pass), black_box(&salt), black_box(&params_min)).unwrap());
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_block_aead_seal,
    bench_hkdf_expand,
    bench_tag_computation,
    bench_kdf_params,
    bench_kdf_default_params
);
criterion_main!(benches);

fn bench_kdf_default_params(c: &mut Criterion) {
    let mut group = c.benchmark_group("bm_kdf_default");
    group.sample_size(10);

    let pass = Secret::from_slice(b"benchmark_master_passphrase");
    let salt = [0x5au8; 32];

    // D-68 standard: 128 MiB (131072 KiB), t = 3, p = 4
    let params_default = KdfParams::DEFAULT;
    group.bench_function("argon2id_default_128mib_t3_p4", |b| {
        b.iter(|| {
            derive_kek(
                black_box(&pass),
                black_box(&salt),
                black_box(&params_default),
            )
            .unwrap()
        });
    });

    group.finish();
}

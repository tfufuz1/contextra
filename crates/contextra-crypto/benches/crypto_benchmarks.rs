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

criterion_group!(
    benches,
    bench_block_aead_seal,
    bench_hkdf_expand,
    bench_tag_computation
);
criterion_main!(benches);

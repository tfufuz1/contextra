use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use contextra_crypto::aead::{open_record, seal_record_with_nonce};
use contextra_crypto::anti_tamper::{compute_keyed_mac, verify_keyed_mac};
use contextra_crypto::dek::Dek;
use contextra_crypto::secret::SecretKey;

fn bench_aead_block_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("aead_aes_256_gcm_siv");

    let secret = SecretKey::new([0x42u8; 32]);
    let dek = Dek::new(secret);
    let doc = [0x01u8; 16];
    let key_bytes = b"benchmark_aad_key";
    let nonce = [0x09u8; 12];

    for size in [64, 512, 4096, 65536].iter() {
        let plaintext = vec![0xABu8; *size];
        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("seal", size), &plaintext, |b, data| {
            b.iter(|| {
                seal_record_with_nonce(
                    black_box(&dek),
                    black_box(doc),
                    black_box(key_bytes),
                    black_box(data),
                    black_box(nonce),
                )
            });
        });

        let env = seal_record_with_nonce(&dek, doc, key_bytes, &plaintext, nonce);
        group.bench_with_input(BenchmarkId::new("open", size), &env, |b, envelope| {
            b.iter(|| {
                open_record(black_box(&dek), black_box(key_bytes), black_box(envelope))
                    .expect("open")
            });
        });
    }

    group.finish();
}

fn bench_blake3_mac_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("blake3_keyed_mac");
    let key = [0x55u8; 32];

    for size in [64, 512, 4096, 65536].iter() {
        let data = vec![0xCDu8; *size];
        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(BenchmarkId::new("mac", size), &data, |b, d| {
            b.iter(|| compute_keyed_mac(black_box(&key), black_box(d)));
        });

        let tag = compute_keyed_mac(&key, &data);
        group.bench_with_input(BenchmarkId::new("verify", size), &data, |b, d| {
            b.iter(|| verify_keyed_mac(black_box(&key), black_box(d), black_box(&tag)));
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_aead_block_throughput,
    bench_blake3_mac_throughput
);
criterion_main!(benches);

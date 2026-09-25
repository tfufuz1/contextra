// FILE-CONTEXT
// ZWECK: Criterion benchmark suite for contextra-crypto throughput and latency measurement.
// INVARIANTEN: Measures AES-256-GCM-SIV throughput at 1KB/64KB/1MB/16MB, HKDF derivation, HMAC throughput, and nonce overhead.
// STAND: TS:2026-08-30T19:50:00Z (SESSION: 20260830)

use contextra_crypto::CryptoKey;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

fn bench_aes_256_gcm_siv_encrypt(c: &mut Criterion) {
    let km = CryptoKey::try_new("bench-passphrase", b"bench-salt-123456").unwrap();
    let mut group = c.benchmark_group("aes_256_gcm_siv_encrypt");

    for size in &[1024, 64 * 1024, 1024 * 1024, 16 * 1024 * 1024] {
        let payload = vec![0x42u8; *size];
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| km.encrypt_auto_nonce(black_box(&payload)).unwrap());
        });
    }
    group.finish();
}

fn bench_aes_256_gcm_siv_decrypt(c: &mut Criterion) {
    let km = CryptoKey::try_new("bench-passphrase", b"bench-salt-123456").unwrap();
    let mut group = c.benchmark_group("aes_256_gcm_siv_decrypt");

    for size in &[1024, 64 * 1024, 1024 * 1024, 16 * 1024 * 1024] {
        let payload = vec![0x42u8; *size];
        let (ct, nonce) = km.encrypt_auto_nonce(&payload).unwrap();
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                km.decrypt_auto_nonce(black_box(&ct), black_box(&nonce))
                    .unwrap()
            });
        });
    }
    group.finish();
}

fn bench_hkdf_derivation(c: &mut Criterion) {
    c.bench_function("hkdf_key_derivation_latency", |b| {
        b.iter(|| {
            CryptoKey::try_new(
                black_box("user-provided-passphrase"),
                black_box(b"salt-for-hkdf-bench-123"),
            )
            .unwrap()
        });
    });
}

fn bench_hmac_integrity(c: &mut Criterion) {
    let km = CryptoKey::try_new("bench-passphrase", b"bench-salt-123456").unwrap();
    c.bench_function("hmac_sha256_integrity_key_derivation", |b| {
        b.iter(|| km.integrity_key().unwrap());
    });
}

// ADR-082 Performance-Nachweis Benchmarks

use contextra_crypto::kv_segment::{KvSegment, TenantIsolatedKvStore};
use contextra_types::TenantId;

fn bench_kv_insert_n_segments(c: &mut Criterion) {
    let mut group = c.benchmark_group("kv_insert_n_segments");
    for n in [10usize, 100, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            let store = TenantIsolatedKvStore::new();
            let tenant = TenantId::try_new(1).unwrap();
            // Prefill
            for id in 0..n as u64 {
                store.insert_segment(tenant, KvSegment::new(tenant, id, vec![0u8; 256]));
            }
            b.iter(|| {
                // Benchmark: insert eines weiteren Segments
                store.insert_segment(tenant, KvSegment::new(tenant, n as u64, vec![0u8; 256]));
            });
        });
    }
    group.finish();
}

fn bench_kv_get_concurrent_n_tenants(c: &mut Criterion) {
    let mut group = c.benchmark_group("kv_get_concurrent_n_tenants");
    for n in [4usize, 16, 32] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            let store = std::sync::Arc::new(TenantIsolatedKvStore::new());
            for t in 1..=n as u64 {
                let tenant = TenantId::try_new(t).unwrap();
                store.insert_segment(tenant, KvSegment::new(tenant, 1, vec![0u8; 256]));
            }
            b.iter(|| {
                let handles: Vec<_> = (1..=n as u64)
                    .map(|t| {
                        let s = std::sync::Arc::clone(&store);
                        std::thread::spawn(move || {
                            let tenant = TenantId::try_new(t).unwrap();
                            let _ = s.get_segment_bytes(tenant, 1);
                        })
                    })
                    .collect();
                for h in handles {
                    h.join().unwrap();
                }
            });
        });
    }
    group.finish();
}

fn bench_kv_evict_lock_held_duration(c: &mut Criterion) {
    c.bench_function("kv_evict_lru_lock_held_duration", |b| {
        b.iter(|| {
            let store = TenantIsolatedKvStore::new();
            let tenant = TenantId::try_new(1).unwrap();
            for id in 0..10u64 {
                store.insert_segment(tenant, KvSegment::new(tenant, id, vec![0xFFu8; 65_536]));
            }
            // Eviction mit Deferred-Drop — Zeroize läuft NACH Lock-Freigabe
            store.evict_lru_fair(500_000)
        });
    });
}

criterion_group!(
    benches,
    bench_aes_256_gcm_siv_encrypt,
    bench_aes_256_gcm_siv_decrypt,
    bench_hkdf_derivation,
    bench_hmac_integrity,
    bench_kv_insert_n_segments,
    bench_kv_get_concurrent_n_tenants,
    bench_kv_evict_lock_held_duration
);
criterion_main!(benches);

// FILE-CONTEXT
// ZWECK: Criterion benchmark for cryptographic deletion proof issuance and verification latency.
// INVARIANTEN: Measures cryptographic portion only (proof creation + signature verification + audit export)
//              without storage physical cleanup overhead.
// STAND: TS:2026-09-25T15:10:00Z

use contextra_crypto::deletion_proof::{
    DeletionLayer, DeletionProof, DeletionScope, ExcludedScope, LayerCleanupProof,
};
use contextra_types::{TenantId, TxId};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_deletion_proof_creation_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("deletion_proof_creation_latency");
    let proof_key = vec![0x42u8; 32];
    let tenant_id = TenantId::try_new(1).unwrap();
    let scope = DeletionScope::Tenant { tenant_id };
    let tx_id = TxId(10);
    let receipt = Some([0x12u8; 32]);

    for num_keys in [1, 100, 10_000, 1_000_000] {
        let keys: Vec<Vec<u8>> = (0..num_keys)
            .map(|i| (i as u64).to_le_bytes().repeat(4))
            .collect();

        if num_keys >= 1_000_000 {
            group.sample_size(10);
        } else if num_keys >= 10_000 {
            group.sample_size(20);
        } else {
            group.sample_size(50);
        }

        group.bench_with_input(BenchmarkId::from_parameter(num_keys), &num_keys, |b, _| {
            b.iter(|| {
                let cleanup_proofs = vec![
                    LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0)
                        .unwrap(),
                    LayerCleanupProof::new_after_verified_empty(DeletionLayer::HnswIndex, 0)
                        .unwrap(),
                ];
                DeletionProof::create_with_wal_receipt(
                    black_box(scope.clone()),
                    black_box(keys.clone()),
                    black_box(tx_id),
                    black_box(cleanup_proofs),
                    black_box(vec![ExcludedScope::LlmParameterMemory]),
                    black_box(receipt),
                    black_box(&proof_key),
                )
                .unwrap()
            });
        });
    }
    group.finish();
}

fn bench_deletion_proof_verification_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("deletion_proof_verification_latency");
    let proof_key = vec![0x42u8; 32];
    let tenant_id = TenantId::try_new(1).unwrap();
    let scope = DeletionScope::Tenant { tenant_id };
    let tx_id = TxId(10);
    let receipt = Some([0x12u8; 32]);

    for num_keys in [1, 100, 10_000, 1_000_000] {
        let keys: Vec<Vec<u8>> = (0..num_keys)
            .map(|i| (i as u64).to_le_bytes().repeat(4))
            .collect();

        if num_keys >= 1_000_000 {
            group.sample_size(10);
        } else if num_keys >= 10_000 {
            group.sample_size(20);
        } else {
            group.sample_size(50);
        }

        let cleanup_proofs = vec![
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::HnswIndex, 0).unwrap(),
        ];
        let proof = DeletionProof::create_with_wal_receipt(
            scope.clone(),
            keys,
            tx_id,
            cleanup_proofs,
            vec![ExcludedScope::LlmParameterMemory],
            receipt,
            &proof_key,
        )
        .unwrap();

        group.bench_with_input(BenchmarkId::from_parameter(num_keys), &num_keys, |b, _| {
            b.iter(|| {
                black_box(proof.verify(black_box(&proof_key)).unwrap());
            });
        });
    }
    group.finish();
}

fn bench_deletion_proof_full_issuance_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("deletion_proof_full_issuance_latency");
    let proof_key = vec![0x42u8; 32];
    let tenant_id = TenantId::try_new(1).unwrap();
    let scope = DeletionScope::Tenant { tenant_id };
    let tx_id = TxId(10);
    let receipt = Some([0x12u8; 32]);

    for num_keys in [1, 100, 10_000, 1_000_000] {
        let keys: Vec<Vec<u8>> = (0..num_keys)
            .map(|i| (i as u64).to_le_bytes().repeat(4))
            .collect();

        if num_keys >= 1_000_000 {
            group.sample_size(10);
        } else if num_keys >= 10_000 {
            group.sample_size(20);
        } else {
            group.sample_size(50);
        }

        group.bench_with_input(BenchmarkId::from_parameter(num_keys), &num_keys, |b, _| {
            b.iter(|| {
                let cleanup_proofs = vec![
                    LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0)
                        .unwrap(),
                    LayerCleanupProof::new_after_verified_empty(DeletionLayer::HnswIndex, 0)
                        .unwrap(),
                ];
                let proof = DeletionProof::create_with_wal_receipt(
                    black_box(scope.clone()),
                    black_box(keys.clone()),
                    black_box(tx_id),
                    black_box(cleanup_proofs),
                    black_box(vec![ExcludedScope::LlmParameterMemory]),
                    black_box(receipt),
                    black_box(&proof_key),
                )
                .unwrap();

                let verified = proof.verify(black_box(&proof_key)).unwrap();
                let audit_json = proof.export_for_audit().unwrap();
                black_box((verified, audit_json))
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_deletion_proof_creation_latency,
    bench_deletion_proof_verification_latency,
    bench_deletion_proof_full_issuance_latency
);
criterion_main!(benches);

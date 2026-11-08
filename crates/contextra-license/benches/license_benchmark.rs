//! Criterion-Benchmark für `contextra-license` (SPEC v4.0 §2 B8).
//!
//! Misst den Durchsatz und die Latenz von:
//! - Fast-Ring Bypass Prüfung (Schritt 1)
//! - 7-stufige Verifikation mit gültigem Signatur- und Hash-Check
//! - Konstruktion und Deserialisierung signierter Lizenz-Payloads

#![allow(clippy::unwrap_used, clippy::expect_used)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use contextra_license::{
    FeatureRing, FixedClock, LicenseGate, OpenFastGate, SignedActivation, SignedLicenseGate,
};
use ed25519_dalek::SigningKey;
use std::sync::Arc;

fn bench_fast_ring_check(c: &mut Criterion) {
    let gate = OpenFastGate;
    c.bench_function("fast_ring_check_open_gate", |b| {
        b.iter(|| gate.check_ring(black_box(FeatureRing::Fast)))
    });

    let signed_gate = SignedLicenseGate::no_activation();
    c.bench_function("fast_ring_check_signed_gate", |b| {
        b.iter(|| signed_gate.check_ring(black_box(FeatureRing::Fast)))
    });
}

fn bench_signed_activation_7_steps(c: &mut Criterion) {
    let signing_key = SigningKey::from_bytes(&[7u8; 32]);
    let verifying_key = signing_key.verifying_key();

    let installation_hash = [42u8; 32];
    let expires_at_unix = 2_000_000_000;
    let activation = SignedActivation::create_signed(
        FeatureRing::Sovereign,
        installation_hash,
        expires_at_unix,
        &signing_key,
    );

    let clock = Arc::new(FixedClock(1_500_000_000_000_000_000));
    let gate = SignedLicenseGate::from_activation_with_clock(activation, verifying_key, clock)
        .with_local_installation_id(installation_hash);

    c.bench_function("signed_activation_verify_sovereign", |b| {
        b.iter(|| gate.check_ring(black_box(FeatureRing::Sovereign)))
    });
}

fn bench_from_signed_payload(c: &mut Criterion) {
    let (payload_bytes, sig, vk_bytes) = SignedLicenseGate::create_test_signed_payload(
        vec![FeatureRing::Sovereign, FeatureRing::Compliance],
        Some(2_000_000_000),
    );
    let vk = ed25519_dalek::VerifyingKey::from_bytes(&vk_bytes).unwrap();

    c.bench_function("from_signed_payload_parse_verify", |b| {
        b.iter(|| {
            SignedLicenseGate::from_signed_payload(
                black_box(&payload_bytes),
                black_box(&sig),
                black_box(vk),
            )
        })
    });
}

criterion_group!(
    benches,
    bench_fast_ring_check,
    bench_signed_activation_7_steps,
    bench_from_signed_payload
);
criterion_main!(benches);

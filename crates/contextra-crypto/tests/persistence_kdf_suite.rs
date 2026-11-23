//! `kdf_suite`: HKDF-SHA256 als `Kdf` (SPEC §4, §16.2; algo_set 1; IMPL WP-2.4).
//!
//! Testvektoren: berechnet mit der Python-Standardbibliothek (`hmac`, `hashlib`) nach dem
//! Algorithmus aus RFC 5869 §2 (Extract mit Null-Salt der Hash-Länge, Expand mit
//! `info = Kontext-Konkatenation`, L = 32). Die RFC-Texte selbst waren nicht abrufbar;
//! der Algorithmus ist unverändert. Die Vektoren sind damit nachvollziehbar, aber keine
//! veröffentlichten Vektoren.

use contextra_crypto::persistence::HkdfSha256Kdf;
use contextra_ports::persistence::Kdf;

const KEY_0_31: [u8; 32] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31,
];
const KEY_42: [u8; 32] = [0x42; 32];

const EXPECTED_SEAL_5: &str = "0923c55788706da36d8bb2f61a037c8177d899b39d63ffc689351eb0ab63f2df";
const EXPECTED_REDO_ONE: &str = "1704cdb1a8ab1b799316e1828e8a02894c7ba1b3617b86287c403f55bef0dd76";

fn hex32(s: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (i, slot) in out.iter_mut().enumerate() {
        let pair = s.get(2 * i..2 * i + 2).unwrap_or("00");
        *slot = u8::from_str_radix(pair, 16).unwrap_or(0);
    }
    out
}

#[test]
fn derive_matches_reference_vector_two_parts() {
    let kdf = HkdfSha256Kdf;
    let ctx: [&[u8]; 2] = [b"ctxs seal v1", &5u32.to_le_bytes()];
    assert_eq!(kdf.derive(&KEY_0_31, &ctx), hex32(EXPECTED_SEAL_5));
}

#[test]
fn derive_matches_reference_vector_single_part() {
    let kdf = HkdfSha256Kdf;
    let ctx: [&[u8]; 1] = [b"ctxs redo v1"];
    assert_eq!(kdf.derive(&KEY_42, &ctx), hex32(EXPECTED_REDO_ONE));
}

#[test]
fn derive_is_deterministic_and_context_separated() {
    let kdf = HkdfSha256Kdf;
    let a = kdf.derive(&KEY_42, &[b"ctxs seal v1"]);
    assert_eq!(a, kdf.derive(&KEY_42, &[b"ctxs seal v1"]));
    assert_ne!(a, kdf.derive(&KEY_42, &[b"ctxs blob v1"]));
    assert_ne!(a, kdf.derive(&KEY_0_31, &[b"ctxs seal v1"]));
}

#[test]
fn derive_is_sensitive_to_part_order() {
    let kdf = HkdfSha256Kdf;
    assert_ne!(
        kdf.derive(&KEY_42, &[b"ab", b"cd"]),
        kdf.derive(&KEY_42, &[b"cd", b"ab"])
    );
}

#[test]
fn derive_output_is_not_all_zero() {
    let kdf = HkdfSha256Kdf;
    assert_ne!(kdf.derive(&KEY_42, &[b"x"]), [0u8; 32]);
}

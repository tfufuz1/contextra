#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::panic
)]

//! # Referenz-AEAD für Tests: XChaCha20-Poly1305
//!
//! Nur Testcode. Der Kern führt keine Krypto-Crate (SPEC §3.5); diese Implementierung
//! liegt im Testbaum und wird gegen öffentliche Vektoren geprüft:
//! - RFC 8439 §2.8.2 (AEAD_CHACHA20_POLY1305),
//! - draft-irtf-cfrg-xchacha-03 §2.2.1 (HChaCha20) und Anhang A.3.1 (AEAD_XCHACHA20_POLY1305).
//!
//! Die Golden-Fälle `seal_frame` stammen aus `tools/reference/generator.py`.

use super::{SealCipher, SealNonce};

const SIGMA: [u32; 4] = [0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574];

fn quarter(s: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
    s[a] = s[a].wrapping_add(s[b]);
    s[d] = (s[d] ^ s[a]).rotate_left(16);
    s[c] = s[c].wrapping_add(s[d]);
    s[b] = (s[b] ^ s[c]).rotate_left(12);
    s[a] = s[a].wrapping_add(s[b]);
    s[d] = (s[d] ^ s[a]).rotate_left(8);
    s[c] = s[c].wrapping_add(s[d]);
    s[b] = (s[b] ^ s[c]).rotate_left(7);
}

fn double_rounds(s: &mut [u32; 16]) {
    for _ in 0..10 {
        quarter(s, 0, 4, 8, 12);
        quarter(s, 1, 5, 9, 13);
        quarter(s, 2, 6, 10, 14);
        quarter(s, 3, 7, 11, 15);
        quarter(s, 0, 5, 10, 15);
        quarter(s, 1, 6, 11, 12);
        quarter(s, 2, 7, 8, 13);
        quarter(s, 3, 4, 9, 14);
    }
}

fn le_words(bytes: &[u8]) -> Vec<u32> {
    bytes
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
        .collect()
}

/// HChaCha20: 32-Byte-Unterschlüssel aus Schlüssel und 16 Byte Nonce.
pub fn hchacha20(key: &[u8; 32], nonce16: &[u8]) -> [u8; 32] {
    let mut s = [0u32; 16];
    s[..4].copy_from_slice(&SIGMA);
    for (i, w) in le_words(key).iter().enumerate() {
        s[4 + i] = *w;
    }
    for (i, w) in le_words(nonce16).iter().enumerate() {
        s[12 + i] = *w;
    }
    double_rounds(&mut s);
    let picked = [s[0], s[1], s[2], s[3], s[12], s[13], s[14], s[15]];
    let mut out = [0u8; 32];
    for (i, w) in picked.iter().enumerate() {
        out[4 * i..4 * i + 4].copy_from_slice(&w.to_le_bytes());
    }
    out
}

fn chacha_block(key: &[u8; 32], counter: u32, nonce12: &[u8]) -> [u8; 64] {
    let mut init = [0u32; 16];
    init[..4].copy_from_slice(&SIGMA);
    for (i, w) in le_words(key).iter().enumerate() {
        init[4 + i] = *w;
    }
    init[12] = counter;
    for (i, w) in le_words(nonce12).iter().enumerate() {
        init[13 + i] = *w;
    }
    let mut s = init;
    double_rounds(&mut s);
    let mut out = [0u8; 64];
    for i in 0..16 {
        let v = s[i].wrapping_add(init[i]);
        out[4 * i..4 * i + 4].copy_from_slice(&v.to_le_bytes());
    }
    out
}

fn chacha_xor(key: &[u8; 32], counter: u32, nonce12: &[u8], data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    for (idx, chunk) in data.chunks(64).enumerate() {
        let ctr = counter.wrapping_add(u32::try_from(idx).unwrap());
        let ks = chacha_block(key, ctr, nonce12);
        out.extend(chunk.iter().zip(ks.iter()).map(|(a, b)| a ^ b));
    }
    out
}

const M44: u64 = 0xfffffffffff;
const M42: u64 = 0x3ffffffffff;

fn le64(b: &[u8]) -> u64 {
    u64::from_le_bytes(b.try_into().unwrap())
}

/// Ein Poly1305-Block in 44/44/42-Bit-Limbs (poly1305-donna-64).
fn absorb(st: &mut [u64; 3], r: [u64; 3], m: &[u8; 16], hibit: u64) {
    let t0 = le64(&m[0..8]);
    let t1 = le64(&m[8..16]);
    let h0 = st[0] + (t0 & M44);
    let h1 = st[1] + (((t0 >> 44) | (t1 << 20)) & M44);
    let h2 = st[2] + ((t1 >> 24) | hibit);
    let s1 = r[1] * 20;
    let s2 = r[2] * 20;
    let (h0, h1, h2) = (u128::from(h0), u128::from(h1), u128::from(h2));
    let d0 = h0 * u128::from(r[0]) + h1 * u128::from(s2) + h2 * u128::from(s1);
    let d1 = h0 * u128::from(r[1]) + h1 * u128::from(r[0]) + h2 * u128::from(s2);
    let d2 = h0 * u128::from(r[2]) + h1 * u128::from(r[1]) + h2 * u128::from(r[0]);
    let c0 = d0 >> 44;
    let g0 = (d0 as u64) & M44;
    let d1 = d1 + c0;
    let c1 = d1 >> 44;
    let g1 = (d1 as u64) & M44;
    let d2 = d2 + c1;
    let c2 = d2 >> 42;
    let g2 = (d2 as u64) & M42;
    let g0 = g0 + (c2 as u64) * 5;
    let c3 = g0 >> 44;
    st[0] = g0 & M44;
    st[1] = g1 + c3;
    st[2] = g2;
}

fn poly1305(key: &[u8; 32], msg: &[u8]) -> [u8; 16] {
    let t0 = le64(&key[0..8]);
    let t1 = le64(&key[8..16]);
    let r = [
        t0 & 0xffc0fffffff,
        ((t0 >> 44) | (t1 << 20)) & 0xfffffc0ffff,
        (t1 >> 24) & 0x00ffffffc0f,
    ];
    let pad = (le64(&key[16..24]), le64(&key[24..32]));
    let mut st = [0u64; 3];
    for chunk in msg.chunks(16) {
        let mut block = [0u8; 16];
        block[..chunk.len()].copy_from_slice(chunk);
        if chunk.len() == 16 {
            absorb(&mut st, r, &block, 1 << 40);
        } else {
            block[chunk.len()] = 1;
            absorb(&mut st, r, &block, 0);
        }
    }
    let (mut h0, mut h1, mut h2) = (st[0], st[1], st[2]);
    let mut c = h1 >> 44;
    h1 &= M44;
    h2 += c;
    c = h2 >> 42;
    h2 &= M42;
    h0 += c * 5;
    c = h0 >> 44;
    h0 &= M44;
    h1 += c;
    c = h1 >> 44;
    h1 &= M44;
    h2 += c;
    c = h2 >> 42;
    h2 &= M42;
    h0 += c * 5;
    c = h0 >> 44;
    h0 &= M44;
    h1 += c;
    let mut g0 = h0 + 5;
    c = g0 >> 44;
    g0 &= M44;
    let mut g1 = h1 + c;
    c = g1 >> 44;
    g1 &= M44;
    let g2 = h2.wrapping_add(c).wrapping_sub(1u64 << 42);
    let mask = (g2 >> 63).wrapping_sub(1);
    g0 &= mask;
    g1 &= mask;
    let g2 = g2 & mask;
    let nm = !mask;
    h0 = (h0 & nm) | g0;
    h1 = (h1 & nm) | g1;
    h2 = (h2 & nm) | g2;
    let h = u128::from(h0) | (u128::from(h1) << 44) | (u128::from(h2) << 88);
    let p = u128::from(pad.0) | (u128::from(pad.1) << 64);
    h.wrapping_add(p).to_le_bytes()
}

fn mac_data(aad: &[u8], ct: &[u8]) -> Vec<u8> {
    let mut m = Vec::with_capacity(aad.len() + ct.len() + 48);
    m.extend_from_slice(aad);
    m.resize(m.len().next_multiple_of(16), 0);
    m.extend_from_slice(ct);
    m.resize(m.len().next_multiple_of(16), 0);
    m.extend_from_slice(&u64::try_from(aad.len()).unwrap().to_le_bytes());
    m.extend_from_slice(&u64::try_from(ct.len()).unwrap().to_le_bytes());
    m
}

fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// AEAD_CHACHA20_POLY1305 (RFC 8439 §2.8) mit 12-Byte-Nonce. Liefert `ciphertext ‖ tag`.
pub fn chacha20poly1305_seal(key: &[u8; 32], nonce12: &[u8], aad: &[u8], plain: &[u8]) -> Vec<u8> {
    let otk_block = chacha_block(key, 0, nonce12);
    let mut otk = [0u8; 32];
    otk.copy_from_slice(&otk_block[..32]);
    let ct = chacha_xor(key, 1, nonce12, plain);
    let tag = poly1305(&otk, &mac_data(aad, &ct));
    let mut out = ct;
    out.extend_from_slice(&tag);
    out
}

/// Gegenstück zu [`chacha20poly1305_seal`]; `None` bei falschem Tag.
pub fn chacha20poly1305_open(
    key: &[u8; 32],
    nonce12: &[u8],
    aad: &[u8],
    sealed: &[u8],
) -> Option<Vec<u8>> {
    if sealed.len() < 16 {
        return None;
    }
    let (ct, tag) = sealed.split_at(sealed.len() - 16);
    let otk_block = chacha_block(key, 0, nonce12);
    let mut otk = [0u8; 32];
    otk.copy_from_slice(&otk_block[..32]);
    let expect = poly1305(&otk, &mac_data(aad, ct));
    if !ct_eq(&expect, tag) {
        return None;
    }
    Some(chacha_xor(key, 1, nonce12, ct))
}

/// XChaCha20-Poly1305 als [`SealCipher`]: HChaCha20 leitet den Unterschlüssel ab,
/// ChaCha20-Poly1305 mit `0u32 ‖ nonce[16..24]` als Nonce versiegelt.
pub struct XChaCha20Poly1305Ref;

fn split_nonce(key: &[u8; 32], nonce: &SealNonce) -> ([u8; 32], [u8; 12]) {
    let sub = hchacha20(key, &nonce[..16]);
    let mut n12 = [0u8; 12];
    n12[4..].copy_from_slice(&nonce[16..24]);
    (sub, n12)
}

impl SealCipher for XChaCha20Poly1305Ref {
    fn seal(&self, key: &[u8; 32], nonce: &SealNonce, aad: &[u8], plain: &[u8]) -> Vec<u8> {
        let (sub, n12) = split_nonce(key, nonce);
        chacha20poly1305_seal(&sub, &n12, aad, plain)
    }

    fn open(
        &self,
        key: &[u8; 32],
        nonce: &SealNonce,
        aad: &[u8],
        sealed: &[u8],
    ) -> Option<Vec<u8>> {
        let (sub, n12) = split_nonce(key, nonce);
        chacha20poly1305_open(&sub, &n12, aad, sealed)
    }
}

#[cfg(test)]
mod vector_tests {
    use super::*;

    fn h(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    /// RFC 8439 §2.8.2 (Quelle: rfc-editor.org/rfc/rfc8439, Abschnitt 2.8.2).
    #[test]
    fn rfc8439_2_8_2_chacha20_poly1305() {
        let key: [u8; 32] = h("808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f")
            .try_into()
            .unwrap();
        let nonce = h("070000004041424344454647");
        let aad = h("50515253c0c1c2c3c4c5c6c7");
        let plain = b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it.";
        let want = h(concat!(
            "d31a8d34648e60db7b86afbc53ef7ec2a4aded51296e08fea9e2b5a736ee62d6",
            "3dbea45e8ca9671282fafb69da92728b1a71de0a9e060b2905d6a5b67ecd3b36",
            "92ddbd7f2d778b8c9803aee328091b58fab324e4fad675945585808b4831d7bc",
            "3ff4def08e4b7a9de576d26586cec64b6116",
            "1ae10b594f09e26a7e902ecbd0600691"
        ));
        assert_eq!(chacha20poly1305_seal(&key, &nonce, &aad, plain), want);
        assert_eq!(
            chacha20poly1305_open(&key, &nonce, &aad, &want).as_deref(),
            Some(&plain[..])
        );
    }

    /// draft-irtf-cfrg-xchacha-03 §2.2.1.
    #[test]
    fn xchacha_draft_2_2_1_hchacha20() {
        let key: [u8; 32] = h("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f")
            .try_into()
            .unwrap();
        let got = hchacha20(&key, &h("000000090000004a0000000031415927"));
        assert_eq!(
            got.to_vec(),
            h("82413b4227b27bfed30e42508a877d73a0f9e4d58a74a853c12ec41326d3ecdc")
        );
    }

    /// draft-irtf-cfrg-xchacha-03 Anhang A.3.1.
    #[test]
    fn xchacha_draft_a31_aead() {
        let key: [u8; 32] = h("808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f")
            .try_into()
            .unwrap();
        let nonce: SealNonce = h("404142434445464748494a4b4c4d4e4f5051525354555657")
            .try_into()
            .unwrap();
        let aad = h("50515253c0c1c2c3c4c5c6c7");
        let plain = h(concat!(
            "4c616469657320616e642047656e746c656d656e206f662074686520636c6173",
            "73206f66202739393a204966204920636f756c64206f6666657220796f75206f",
            "6e6c79206f6e652074697020666f7220746865206675747572652c2073756e73",
            "637265656e20776f756c642062652069742e"
        ));
        let want = h(concat!(
            "bd6d179d3e83d43b9576579493c0e939572a1700252bfaccbed2902c21396cbb",
            "731c7f1b0b4aa6440bf3a82f4eda7e39ae64c6708c54c216cb96b72e1213b452",
            "2f8c9ba40db5d945b11b69b982c1bb9e3f3fac2bc369488f76b2383565d3fff9",
            "21f9664c97637da9768812f615c68b13b52e",
            "c0875924c1c7987947deafd8780acf49"
        ));
        assert_eq!(XChaCha20Poly1305Ref.seal(&key, &nonce, &aad, &plain), want);
        assert_eq!(
            XChaCha20Poly1305Ref.open(&key, &nonce, &aad, &want),
            Some(plain)
        );
    }
}

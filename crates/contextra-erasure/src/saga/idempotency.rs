//! Idempotenz der Lösch-Saga (SPEC §7.4, §11.2, D-119, I-15, ER-18).

use blake3::Hasher;

/// Berechnet den Open-Lookup-Key nach D-119:
/// `open_key = BLAKE3-keyed(K_idx, "ctx/erasure-open/v1" || kanonisches_Ziel || mode)`
pub fn derive_open_key(k_idx: &[u8; 32], canonical_target: &[u8], mode_byte: u8) -> [u8; 32] {
    let mut hasher = Hasher::new_keyed(k_idx);
    hasher.update(b"ctx/erasure-open/v1");
    hasher.update(canonical_target);
    hasher.update(&[mode_byte]);
    *hasher.finalize().as_bytes()
}

/// Prüft ob zwei `EraseRequest`-Anfragen dieselbe Löschung darstellen.
/// Gleiches Ziel, gleicher Modus → gleiche ErasureId.
pub fn is_same_request(a_erasure_id: [u8; 16], b_erasure_id: [u8; 16]) -> bool {
    a_erasure_id == b_erasure_id
}

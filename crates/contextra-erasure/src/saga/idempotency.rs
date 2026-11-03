//! Idempotenz der Lösch-Saga (SPEC §7.4, I-15, ER-18).

/// Prüft ob zwei `EraseRequest`-Anfragen dieselbe Löschung darstellen.
/// Gleiches Ziel, gleicher Modus → gleiche ErasureId.
pub fn is_same_request(a_erasure_id: [u8; 16], b_erasure_id: [u8; 16]) -> bool {
    a_erasure_id == b_erasure_id
}

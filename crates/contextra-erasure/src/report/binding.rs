//! # Witness Binding & Hash-Chaining (INV-ERA-7 / WP-9).
//!
//! Ein Löschbericht bindet kryptografisch deterministisch die Zeugen, auf die er sich
//! stützt, über eine BLAKE3-Hash-Kette (INV-ERA-7).

use crate::types_local::LayerWitness;

/// Berechnet den verketteten Hash über eine Liste von `LayerWitness`-Objekten.
pub fn compute_witnesses_chain_hash(witnesses: &[LayerWitness]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    for (idx, w) in witnesses.iter().enumerate() {
        hasher.update(&(idx as u64).to_le_bytes());
        hasher.update(w.layer().as_bytes());
        hasher.update(w.hash());
        hasher.update(&w.files().to_le_bytes());
        hasher.update(&w.bytes().to_le_bytes());
    }
    *hasher.finalize().as_bytes()
}

/// Verifiziert, ob die gegebenen Zeugen mit dem gebundenen Hash übereinstimmen.
pub fn verify_witnesses_binding(witnesses: &[LayerWitness], expected_hash: &[u8; 32]) -> bool {
    let computed = compute_witnesses_chain_hash(witnesses);
    &computed == expected_hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_witnesses_hash_chain_determinism_and_tamper() {
        let w1 = LayerWitness::from_hash([0x11; 32]);
        let w2 = LayerWitness::from_hash([0x22; 32]);

        let chain = compute_witnesses_chain_hash(&[w1.clone(), w2.clone()]);
        assert!(verify_witnesses_binding(&[w1.clone(), w2.clone()], &chain));

        // Vertauschte Reihenfolge bricht die Bindung (INV-ERA-7)
        assert!(!verify_witnesses_binding(&[w2.clone(), w1.clone()], &chain));

        // Manipulierter Zeuge bricht die Bindung
        let w3 = LayerWitness::from_hash([0x33; 32]);
        assert!(!verify_witnesses_binding(&[w1, w3], &chain));
    }
}

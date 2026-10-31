//! # DSGVO-Löschbeweis-Subsystem (`deletion_proof/`)
//!
//! Dieses Modul implementiert kryptographische Löschzertifikate (Deletion Proofs)
//! zum rechtssicheren Nachweis der physischen und kryptographischen Löschung nach Art. 17 DSGVO.
//!
//! ## Submodule
//! - `types.rs`: Datenstrukturen und Enums (DeletionProof, DeletionLayer, etc.).
//! - `creation.rs`: Erzeugung und Signierung von Beweisen.
//! - `verification.rs`: Interne und externe Validierungslogik.
//! - `verifier.rs`: Standalone-Prüfer für Auditoren.
//! - `chain.rs`: Verkettung aufeinanderfolgender Beweise.
//! - `key_derivation.rs`: Schlüsselableitung für Signatur und Prüfsummen.
// FILE-CONTEXT
// ZWECK: Directory module declaration and re-exports for cryptographic deletion proofs (GDPR Art. 17).
// INVARIANTEN: All public interfaces from legacy deletion_proof.rs re-exported without breaking changes.

#![forbid(unsafe_code)]

//! Cryptographic Deletion Proof module for Contextra.

pub mod chain;
pub mod creation;
pub mod key_derivation;
pub mod types;
pub mod verification;
pub mod verifier;

pub use chain::*;
pub use creation::*;
pub use key_derivation::*;
pub use types::*;
pub use verification::*;
pub use verifier::*;

pub use crate::ed25519_proof::DeletionProofKeyPair;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ed25519_proof::DeletionProofKeyPair;
    use contextra_types::{DocId, TenantId, TxId};

    #[test]
    fn test_v3_create_and_verify_external_roundtrip() {
        let mut rng = rand::rngs::OsRng;
        let keypair = DeletionProofKeyPair::generate_with_rng(&mut rng);

        let proof = DeletionProof::create_v3(
            DeletionScope::Document {
                doc_id: DocId::from(20),
                tenant_id: TenantId(100),
            },
            vec![b"key_a".to_vec()],
            TxId(500),
            vec![LayerCleanupProof::for_test(DeletionLayer::LsmMemtable)],
            vec![ExcludedScope::ConsolidatedAndDistilled],
            1700000000,
            &[],
            keypair.signing_key(),
        )
        .expect("create_v3 proof");

        let res = proof.verify_external(&keypair.verifying_key);
        assert!(res.is_ok());

        let v_res =
            DeletionProofVerifier::verify_standalone(&proof, &keypair.verifying_key_bytes());
        assert!(v_res.is_valid);
    }
}

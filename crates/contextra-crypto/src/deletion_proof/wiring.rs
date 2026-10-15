//! Unified wiring facade for deletion proof generation, verification, audit chain anchoring, and revocation log management.

#![forbid(unsafe_code)]

use super::issuer::{IssIssuedProof, IssRequest, ProofIssuer};
use super::policy::{verify_with_policy, PolicyReport, ProofPolicy};
use super::proof_v4::ProofV4;
use super::signer::ProofSigner;
use crate::audit_chain::{AuditChain, AuditChainAnchor};
use crate::deletion_proof::probe::WitStoreProbe;
use crate::error::CryptoError;
use crate::revocation_log::RevocationLog;
use contextra_types::ContextraError;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Input dependencies required to construct [`DeletionProofWiring`].
pub struct DeletionProofWiringDeps {
    /// Storage directory path for proofs, audit chain log, and revocation log.
    pub storage_dir: PathBuf,
    /// Ed25519 proof signer implementation.
    pub signer: Arc<dyn ProofSigner>,
    /// Database instance identifier (16 bytes).
    pub db_instance_id: [u8; 16],
    /// Clock port for timestamping.
    pub clock: Arc<dyn contextra_ports::Clock>,
    /// Pre-initialized `RevocationLog`, if available. If `None`, one will be opened at `storage_dir`.
    pub revocation_log: Option<Arc<RevocationLog>>,
    /// Ed25519 signing key for signing revocation log entries if opened internally.
    pub revocation_signing_key: Option<ed25519_dalek::SigningKey>,
    /// Ed25519 verifying key for verifying revocation log integrity.
    pub revocation_verifying_key: ed25519_dalek::VerifyingKey,
    /// Flag indicating if existing key files are present in `storage_dir`.
    pub has_existing_keys: bool,
}

/// Unified facade bundling signer, proof issuer, verifier, audit chain anchor, and revocation log.
pub struct DeletionProofWiring {
    signer: Arc<dyn ProofSigner>,
    issuer: Arc<ProofIssuer>,
    audit_chain: Arc<Mutex<AuditChain>>,
    audit_anchor: Arc<Mutex<Option<AuditChainAnchor>>>,
    revocation_log: Arc<RevocationLog>,
    public_key: [u8; 32],
}

impl DeletionProofWiring {
    /// Constructs a fully wired [`DeletionProofWiring`] instance per Entrypoint Contract (Rule 7).
    pub fn build(deps: DeletionProofWiringDeps) -> Result<Self, ContextraError> {
        let public_key = deps.signer.public_key();

        // 1. Open or initialize ProofIssuer
        let (issuer, _warnings) =
            ProofIssuer::open(&deps.storage_dir, deps.signer.clone(), deps.db_instance_id)
                .map_err(|e| ContextraError::Crypto(e.to_string()))?;

        // 2. Open or initialize AuditChain
        let audit_chain = AuditChain::new();
        let anchor = audit_chain.anchor();

        // 3. Obtain or initialize RevocationLog
        let revocation_log = match deps.revocation_log {
            Some(log) => log,
            None => {
                let rev_path = deps.storage_dir.join("revocation.log");
                let log = RevocationLog::open_or_create_if_fresh(
                    &rev_path,
                    deps.clock,
                    deps.revocation_signing_key,
                    deps.revocation_verifying_key,
                    deps.has_existing_keys,
                )
                .map_err(|e| ContextraError::Crypto(e.to_string()))?;
                Arc::new(log)
            }
        };

        Ok(Self {
            signer: deps.signer,
            issuer: Arc::new(issuer),
            audit_chain: Arc::new(Mutex::new(audit_chain)),
            audit_anchor: Arc::new(Mutex::new(anchor)),
            revocation_log,
            public_key,
        })
    }

    /// Returns the active proof signer.
    pub fn signer(&self) -> Arc<dyn ProofSigner> {
        self.signer.clone()
    }

    /// Returns the bundled proof issuer.
    pub fn issuer(&self) -> Arc<ProofIssuer> {
        self.issuer.clone()
    }

    /// Returns the audit chain thread-safe reference.
    pub fn audit_chain(&self) -> Arc<Mutex<AuditChain>> {
        self.audit_chain.clone()
    }

    /// Returns the audit chain anchor state.
    pub fn audit_anchor(&self) -> Arc<Mutex<Option<AuditChainAnchor>>> {
        self.audit_anchor.clone()
    }

    /// Returns the revocation log.
    pub fn revocation_log(&self) -> Arc<RevocationLog> {
        self.revocation_log.clone()
    }

    /// Returns the 32-byte Ed25519 public key of the active signer.
    pub fn public_key(&self) -> [u8; 32] {
        self.public_key
    }

    /// Issues a version 4 deletion proof after executing counter-checks, persisting to store, and appending to audit chain.
    pub fn issue_proof(
        &self,
        req: IssRequest,
        probes: &[&dyn WitStoreProbe],
    ) -> Result<IssIssuedProof, CryptoError> {
        let issued = self.issuer.issue(req, probes)?;

        // Append entry to AuditChain and update AuditChainAnchor
        if let Ok(mut chain_guard) = self.audit_chain.lock() {
            let doc_id = match issued.proof.scope {
                super::types::DeletionScope::Document { doc_id, .. } => doc_id,
                _ => contextra_types::DocId(0),
            };
            let tx_id = contextra_types::TxId(issued.proof.deleted_after_tx);
            let rules_hash = [0u8; 32];
            let commitment = Some(issued.proof.deleted_keys_hash);

            let _ = chain_guard.append(
                "deletion_proof_v4",
                rules_hash,
                crate::audit_chain::DataClass::Restricted,
                doc_id,
                tx_id,
                commitment,
            );

            if let Ok(mut anchor_guard) = self.audit_anchor.lock() {
                *anchor_guard = chain_guard.anchor();
            }
        }

        Ok(issued)
    }

    /// Verifies a version 4 deletion proof in a **completely autonomous, stateless manner**:
    /// Requires strictly the 32-byte public key and the proof document, without database state or KeyManager.
    pub fn verify_external(public_key: &[u8; 32], proof: &ProofV4) -> PolicyReport {
        let policy = ProofPolicy::default();
        verify_with_policy(proof, public_key, &policy)
    }

    /// Verifies a version 4 deletion proof with a custom [`ProofPolicy`], without database state or KeyManager.
    pub fn verify_external_with_policy(
        public_key: &[u8; 32],
        proof: &ProofV4,
        policy: &ProofPolicy,
    ) -> PolicyReport {
        verify_with_policy(proof, public_key, policy)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::deletion_proof::signer::ProofKeyPairSigner;
    use contextra_ports::SystemClock;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    use tempfile::tempdir;

    #[test]
    fn test_deletion_proof_wiring_build_and_components() {
        let tmp = tempdir().expect("tempdir failed");
        let signer = Arc::new(ProofKeyPairSigner::generate(&mut OsRng));

        let rev_sk = SigningKey::generate(&mut OsRng);
        let rev_vk = rev_sk.verifying_key();
        let clock = Arc::new(SystemClock::new());

        let deps = DeletionProofWiringDeps {
            storage_dir: tmp.path().to_path_buf(),
            signer: signer.clone(),
            db_instance_id: [1u8; 16],
            clock,
            revocation_log: None,
            revocation_signing_key: Some(rev_sk),
            revocation_verifying_key: rev_vk,
            has_existing_keys: false,
        };

        let wiring = DeletionProofWiring::build(deps).expect("build failed");

        assert_eq!(wiring.public_key(), signer.public_key());
        assert!(wiring.revocation_log().verify_integrity().is_ok());
        assert!(wiring.audit_anchor().lock().unwrap().is_none());
    }
}

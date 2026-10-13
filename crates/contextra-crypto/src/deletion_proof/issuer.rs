//! Single entry point for issuing, verifying, and persisting version 4 deletion proofs (Spec C3, C7).

#![forbid(unsafe_code)]

use super::builder::ProofBuilder;
use super::exclusions::mandatory_exclusions_for;
use super::policy::{verify_with_policy, ProofPolicy};
use super::proof_store::{ProofStore, ProofStoreWarning};
use super::proof_v4::{ProofClaimV4, ProofV4};
use super::signer::ProofSigner;
use super::types::{DeletionLayer, DeletionScope, ExcludedScope};
use crate::deletion_proof::claim_text::WitAssurance;
use crate::deletion_proof::evidence::WitLayerEvidence;
use crate::deletion_proof::probe::WitStoreProbe;
use crate::deletion_witness::LayerWitness;
use crate::error::{CryptoError, Result};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn required_layers_for_assurance(assurance: WitAssurance) -> Vec<u16> {
    match assurance {
        WitAssurance::S0 => vec![1],
        WitAssurance::S1 => vec![1, 3, 20, 21, 26],
        WitAssurance::S2 => vec![1, 2, 3, 20, 21, 22, 23, 26],
        WitAssurance::S3 => vec![1, 2, 3, 4, 20, 21, 22, 23, 26],
        WitAssurance::S4 => vec![1, 2, 3, 4, 20, 21, 22, 23, 25, 26],
    }
}

/// Request parameters for issuing a version 4 deletion proof via [`ProofIssuer`].
#[derive(Debug)]
pub struct IssRequest {
    /// Deletion target scope.
    pub scope: DeletionScope,
    /// Storage scope string description.
    pub storage_scope: String,
    /// Length-prefixed BLAKE3 hash of deleted keys.
    pub deleted_keys_hash: [u8; 32],
    /// Footprint hash of affected storage bounds.
    pub footprint_hash: [u8; 32],
    /// Flag indicating whether the scope was confirmed empty without key deletion.
    pub observed_empty: bool,
    /// TxId sequence number after which deletion was completed.
    pub deleted_after_tx: u64,
    /// Proof stage (1 = Logical, 2 = Physical).
    pub stage: u8,
    /// Optional proof ID superseded by this proof.
    pub supersedes: Option<[u8; 32]>,
    /// Target assurance level.
    pub assurance: WitAssurance,
    /// Per-layer physical sanitization witnesses.
    pub witnesses: Vec<LayerWitness>,
    /// Additional explicitly excluded scopes.
    pub extra_exclusions: Vec<ExcludedScope>,
    /// Opaque environment TLV metadata bytes.
    pub environment_tlv: Vec<u8>,
    /// WAL chain receipt hash, if available.
    pub wal_chain_receipt: Option<[u8; 32]>,
}

/// Result of issuing a version 4 deletion proof via [`ProofIssuer`].
#[derive(Debug, Clone)]
pub struct IssIssuedProof {
    /// Issued version 4 deletion proof.
    pub proof: ProofV4,
    /// BLAKE3 chain hash after saving to ProofStore.
    pub chain_hash: [u8; 32],
}

/// Single entry point for issuing, verifying against store probes, and persisting version 4 deletion proofs.
pub struct ProofIssuer {
    dir: PathBuf,
    store: Arc<Mutex<ProofStore>>,
    signer: Arc<dyn ProofSigner>,
    db_instance_id: String,
}

impl ProofIssuer {
    /// Opens or initializes a `ProofIssuer` instance at `dir`.
    pub fn open(
        dir: impl AsRef<Path>,
        signer: Arc<dyn ProofSigner>,
        db_instance_id: [u8; 16],
    ) -> Result<(Self, Vec<ProofStoreWarning>)> {
        let dir_path = dir.as_ref().to_path_buf();
        let (store, warnings) = ProofStore::open(&dir_path)?;
        let db_instance_id_str = hex_encode(&db_instance_id);

        Ok((
            Self {
                dir: dir_path,
                store: Arc::new(Mutex::new(store)),
                signer,
                db_instance_id: db_instance_id_str,
            },
            warnings,
        ))
    }

    /// Issues a version 4 deletion proof from `req` after executing counter-checks 1–5 (C3, C7).
    pub fn issue(&self, req: IssRequest, probes: &[&dyn WitStoreProbe]) -> Result<IssIssuedProof> {
        // Compute expected proof_id for idempotency check
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.db_instance_id.as_bytes());
        hasher.update(&[req.stage]);
        hasher.update(&req.deleted_after_tx.to_be_bytes());
        hasher.update(&req.deleted_keys_hash);
        let proof_id = *hasher.finalize().as_bytes();
        let proof_id_hex = hex_encode(&proof_id);

        let proofs_dir = self.dir.join("proofs");
        if proofs_dir.exists() {
            if let Ok(entries) = fs::read_dir(&proofs_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                        if filename.contains(&proof_id_hex) && filename.ends_with(".json") {
                            if let Ok(json_str) = fs::read_to_string(&path) {
                                if let Ok(existing_proof) =
                                    serde_json::from_str::<ProofV4>(&json_str)
                                {
                                    let store_guard = self.store.lock().map_err(|_| {
                                        CryptoError::Crypto(
                                            "Failed to acquire ProofStore lock".to_string(),
                                        )
                                    })?;
                                    let chain_hash = store_guard.head_chain_hash();
                                    drop(store_guard);
                                    return Ok(IssIssuedProof {
                                        proof: existing_proof,
                                        chain_hash,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // Counter-checks 1–5:
        // (a) Witness store_generation check with up to 3 probe retries
        for witness in &req.witnesses {
            let layer = witness.layer();
            let expected_gen = witness.store_generation();
            let mut gen_match = false;

            for _retry in 0..3 {
                for probe in probes {
                    if probe.current_generation(layer) == expected_gen {
                        gen_match = true;
                        break;
                    }
                }
                if gen_match {
                    break;
                }
            }

            if !gen_match {
                return Err(CryptoError::Crypto(
                    "WitnessStale: store_generation mismatch across probes (ConcurrentModification)".to_string(),
                ));
            }
        }

        // (b) scope_write_seq check against probes
        for witness in &req.witnesses {
            let scope_hash = witness.scope_hash();
            for probe in probes {
                let probe_seq = probe.scope_write_seq(scope_hash);
                if probe_seq != 0 && probe_seq != witness.scope_write_seq() {
                    return Err(CryptoError::Crypto(
                        "ScopeWriteSeqMismatch: witness scope_write_seq deviates from store probe"
                            .to_string(),
                    ));
                }
            }
        }

        // (c) scope_hash and footprint_hash match request
        for witness in &req.witnesses {
            if witness.scope_hash() != &req.deleted_keys_hash {
                return Err(CryptoError::Crypto(
                    "ScopeHashMismatch: witness scope_hash does not match request deleted_keys_hash".to_string(),
                ));
            }
            if witness.footprint_hash() != &req.footprint_hash {
                return Err(CryptoError::Crypto(
                    "FootprintHashMismatch: witness footprint_hash does not match request footprint_hash".to_string(),
                ));
            }
        }

        // (d) scanned_at_tx >= as_of_tx and scanned_at_tx >= deleted_after_tx
        for witness in &req.witnesses {
            if witness.scanned_at_tx().0 < witness.as_of_tx().0 {
                return Err(CryptoError::Crypto(
                    "InvalidTxOrder: witness scanned_at_tx earlier than as_of_tx".to_string(),
                ));
            }
            if witness.scanned_at_tx().0 < req.deleted_after_tx {
                return Err(CryptoError::Crypto(
                    "InvalidTxOrder: witness scanned_at_tx earlier than request deleted_after_tx"
                        .to_string(),
                ));
            }
        }

        // (e) SsTables manifest_file_count check
        for witness in &req.witnesses {
            if witness.layer() == &DeletionLayer::SsTableAllLevels {
                if let Some(WitLayerEvidence::SsTables {
                    files_in_manifest, ..
                }) = witness.layer_evidence()
                {
                    for probe in probes {
                        if let Some(probe_count) = probe.manifest_file_count() {
                            if *files_in_manifest != probe_count {
                                return Err(CryptoError::Crypto(
                                    "ManifestFileCountMismatch: SsTables files_in_manifest does not match probe manifest_file_count".to_string(),
                                ));
                            }
                        }
                    }
                }
            }
        }

        // (f) Missing mandatory layer for requested assurance level
        let present_layers: HashSet<u16> =
            req.witnesses.iter().map(|w| w.layer().layer_id()).collect();
        let required_layers = required_layers_for_assurance(req.assurance);
        for req_layer in &required_layers {
            if !present_layers.contains(req_layer) {
                return Err(CryptoError::Crypto(format!(
                    "MissingRequiredLayer: required layer {} missing for assurance level {:?}",
                    req_layer, req.assurance
                )));
            }
        }

        // (g) Empty footprint without observed_empty check
        if req.footprint_hash == [0u8; 32] && !req.observed_empty {
            return Err(CryptoError::Crypto(
                "EmptyFootprintWithoutObservedEmpty: zero footprint_hash requires observed_empty == true".to_string(),
            ));
        }
        if req.deleted_keys_hash == [0u8; 32] && !req.observed_empty {
            return Err(CryptoError::Crypto(
                "EmptyDeletedKeysWithoutObservedEmpty: zero deleted_keys_hash requires observed_empty == true".to_string(),
            ));
        }

        // ADR-14 Two-stage supersedes check
        if req.stage == 2 {
            let sup_id = req.supersedes.ok_or_else(|| {
                CryptoError::Crypto(
                    "MissingSupersedesForPhysicalStage: stage 2 Physical proof requires supersedes proof ID reference".to_string(),
                )
            })?;

            let sup_hex = hex_encode(&sup_id);
            let mut found_valid_logical = false;
            if proofs_dir.exists() {
                if let Ok(entries) = fs::read_dir(&proofs_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                            if filename.contains(&sup_hex) && filename.ends_with(".json") {
                                if let Ok(json_str) = fs::read_to_string(&path) {
                                    if let Ok(sup_proof) =
                                        serde_json::from_str::<ProofV4>(&json_str)
                                    {
                                        if sup_proof.stage == 1
                                            && sup_proof.deleted_keys_hash == req.deleted_keys_hash
                                        {
                                            found_valid_logical = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !found_valid_logical {
                return Err(CryptoError::Crypto(
                    "InvalidSupersedesReference: superseded proof not found or not in stage 1 Logical".to_string(),
                ));
            }
        } else if req.stage == 1 && req.supersedes.is_some() {
            return Err(CryptoError::Crypto(
                "UnexpectedSupersedesForLogicalStage: stage 1 Logical proof must not specify supersedes".to_string(),
            ));
        }

        // Build ProofV4
        let mut builder = ProofBuilder::new(req.scope.clone(), &self.db_instance_id)
            .with_stage(req.stage)
            .with_storage_scope(&req.storage_scope)
            .with_deleted_keys_hash(req.deleted_keys_hash)
            .with_footprint_hash(req.footprint_hash)
            .with_observed_empty(req.observed_empty)
            .with_deleted_after_tx(req.deleted_after_tx)
            .with_environment_tlv(req.environment_tlv.clone());

        if let Some(sup) = req.supersedes {
            builder = builder.with_supersedes(sup);
        }

        if let Some(receipt) = req.wal_chain_receipt {
            builder = builder.with_wal_chain_receipt(receipt);
        }

        for w in &req.witnesses {
            let layer_id = w.layer().layer_id();
            let assurance = w.assurance().level();
            let claim_text_id = w.claim().id();
            let scanned_at_tx = w.scanned_at_tx().0;

            let (witness_digest, evidence_tlv) = if let Some(ev) = w.layer_evidence() {
                (ev.witness_digest(layer_id), ev.to_canonical_tlv())
            } else {
                ([0u8; 32], Vec::new())
            };

            builder = builder.add_claim(ProofClaimV4 {
                layer_id,
                assurance,
                claim_text_id,
                witness_digest,
                scanned_at_tx,
                evidence_tlv,
            });
        }

        let mandatory_excl = mandatory_exclusions_for(req.assurance);
        let mut all_excl_ids: Vec<u16> = mandatory_excl.iter().map(|e| e.id()).collect();
        for extra in &req.extra_exclusions {
            all_excl_ids.push(extra.id());
        }
        all_excl_ids.sort();
        all_excl_ids.dedup();
        for excl_id in all_excl_ids {
            builder = builder.add_excluded_scope(excl_id);
        }

        let mut store_guard = self
            .store
            .lock()
            .map_err(|_| CryptoError::Crypto("Failed to acquire ProofStore lock".to_string()))?;

        let seq = store_guard.last_seq() + 1;
        let prev_hash = store_guard.head_chain_hash();
        let wall_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_millis()).unwrap_or(1))
            .unwrap_or(1);

        builder = builder
            .with_chain_link(seq, prev_hash)
            .with_timestamps(wall_ms, seq);

        let proof = builder.finish_checked(&*self.signer)?;

        // Self-verification with verify_with_policy
        let mut allowed_claims = HashSet::new();
        for id in 1..=8 {
            allowed_claims.insert(id);
        }

        let mut trusted_signers = HashSet::new();
        trusted_signers.insert(self.signer.key_id());

        let policy = ProofPolicy {
            required_layers,
            mandatory_excluded_scopes: mandatory_excl.iter().map(|e| e.id()).collect(),
            allowed_claim_text_ids: allowed_claims,
            trusted_anchor: true,
            expected_prev_chain_hash: Some(prev_hash),
            trusted_signer_key_ids: trusted_signers,
            ..ProofPolicy::default()
        };

        let report = verify_with_policy(&proof, &self.signer.public_key(), &policy);
        if !report.is_valid {
            return Err(CryptoError::Crypto(format!(
                "Self-verification failed for issued proof: {:?}",
                report
            )));
        }

        // Save proof to store
        let chain_hash = store_guard.save_proof(&proof)?;
        drop(store_guard);

        Ok(IssIssuedProof { proof, chain_hash })
    }
}

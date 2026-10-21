//! Deletion runtime and two-stage proof issuance (§C3, §C7, §C9, ADR-14).

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use contextra_crypto::deletion_proof::issuer::{IssIssuedProof, IssRequest, ProofIssuer};
use contextra_crypto::deletion_proof::probe::WitStoreProbe;
use contextra_crypto::deletion_proof::proof_store::ProofStoreWarning;
use contextra_crypto::deletion_proof::signer::{ProofKeyPairSigner, ProofSigner};
use contextra_crypto::deletion_proof::types::{DeletionScope, ExcludedScope};
use contextra_crypto::deletion_witness::LayerWitness;
<<<<<<< HEAD
use contextra_ports::Rng;
=======
use contextra_ports::{OsEntropyRng, SecretBytes};
>>>>>>> 0000a4c4f (feat(ports,sys,engine,mcp): implement CryptoRng, SecretBytes, and secure proof signer key loading (F-01, F-10, F-22))
use contextra_types::{ContextraError, Result, TxId};
use tracing::error;
use zeroize::Zeroizing;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Validates that a key file is not a symlink and has strict 0600 permissions on Unix.
fn check_key_file_security(path: &Path) -> Result<()> {
    let meta = std::fs::symlink_metadata(path).map_err(|e| {
        ContextraError::Crypto(format!("Failed to stat key file '{path:?}': {e}"))
    })?;

    if meta.file_type().is_symlink() {
        return Err(ContextraError::Crypto(format!(
            "Signer key file '{path:?}' cannot be a symlink"
        )));
    }

    #[cfg(unix)]
    {
        let mode = meta.permissions().mode() & 0o777;
        if mode != 0o600 {
            return Err(ContextraError::Crypto(format!(
                "Signer key file '{path:?}' permissions must be 0600, got 0{mode:o}"
            )));
        }
    }

    Ok(())
}

use super::copy_observer::DpiCopyObserverReport;
use super::environment::observe_environment;
use super::forget_options::{FonLogicalHandle, ForgetOptions};

/// Trait for retrieving or generating Ed25519 signing key seed material.
pub trait ProofSignerSource: Send + Sync {
    /// Obtains the 32-byte Ed25519 secret seed wrapped in a [`Zeroizing`] buffer.
    ///
    /// # Errors
    ///
    /// Returns [`ContextraError::Crypto`] if key loading or CSPRNG key generation fails (fail-closed).
    fn get_signer_seed(&self) -> Result<Zeroizing<[u8; 32]>>;
}

/// Default production signer source that loads an existing Ed25519 key file from `key_path`,
/// or generates a new CSPRNG key using the Rng port and persists it with strict 0600 permissions.
pub struct FileOrCsprngSignerSource {
    key_path: PathBuf,
}

impl FileOrCsprngSignerSource {
    /// Creates a new [`FileOrCsprngSignerSource`] targeting `key_path`.
    #[must_use]
    pub fn new(key_path: PathBuf) -> Self {
        Self { key_path }
    }
}

impl ProofSignerSource for FileOrCsprngSignerSource {
    fn get_signer_seed(&self) -> Result<Zeroizing<[u8; 32]>> {
        if self.key_path.exists() {
            check_key_file_security(&self.key_path)?;

            let raw_bytes = std::fs::read(&self.key_path)
                .map_err(|e| ContextraError::Crypto(format!("Failed to read key file: {e}")))?;

            if raw_bytes.len() != 32 {
                return Err(ContextraError::Crypto(format!(
                    "Invalid key file length: expected 32 bytes, got {}",
                    raw_bytes.len()
                )));
            }

            let mut secret = [0u8; 32];
            secret.copy_from_slice(&raw_bytes);
            Ok(Zeroizing::new(secret))
        } else {
            // TODO(WP-S1 / PR-3 Schritt 5): Signer-Verwaltung komplett umschreiben:
            // 1. Genau ein deterministischer Pfad, kein HOME-Fallback.
            // 2. Atomare Erstellung mit `create_new(true)`.
            // 3. Strikte Dateiberechtigungen (0600 auf Unix via std::os::unix::fs::OpenOptionsExt, bzw. Owner-only ACL auf Windows).
            if let Some(parent) = self.key_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    ContextraError::Crypto(format!("Failed to create signer key dir: {e}"))
                })?;
            }

<<<<<<< HEAD
            // TODO(WP-S1 / PR-3 Schritt 4 / F-01): `SystemRng` durch echten CSPRNG (`CryptoRng` / `OsCryptoRng`) ersetzen!
            // Befund F-01: `SystemRng` ist ein deterministischer SplitMix64-Generator und KEIN kryptographischer PRNG.
            // Die Generierung von Ed25519-Schlüsseln aus schwachen Entropiequellen bricht alle Lösch- und Nachweisgarantien.
            let rng = contextra_ports::SystemRng::new();
            let mut secret = [0u8; 32];
            rng.fill_bytes(&mut secret);

            let keypair = ProofKeyPairSigner::from_secret_bytes(secret);
            keypair.save_to_file(&self.key_path).map_err(|e| {
                ContextraError::Crypto(format!("Failed to save Ed25519 signer key: {e}"))
            })?;

            Ok(Zeroizing::new(secret))
=======
            let os_rng = OsEntropyRng::new();
            let secret_bytes = SecretBytes::<32>::generate(&os_rng).map_err(|e| {
                ContextraError::Crypto(format!("Failed to generate CSPRNG secret key: {e}"))
            })?;
            let secret = *secret_bytes.expose();

            let mut options = std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }

            match options.open(&self.key_path) {
                Ok(mut file) => {
                    use std::io::Write;
                    file.write_all(&secret).map_err(|e| {
                        ContextraError::Crypto(format!("Failed to write signer key file: {e}"))
                    })?;
                    file.sync_all().map_err(|e| {
                        ContextraError::Crypto(format!("Failed to fsync signer key file: {e}"))
                    })?;
                    #[cfg(windows)]
                    {
                        let _ = contextra_sys::set_restrictive_file_acl(&self.key_path);
                    }
                    Ok(Zeroizing::new(secret))
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    check_key_file_security(&self.key_path)?;
                    let bytes = std::fs::read(&self.key_path).map_err(|e| {
                        ContextraError::Crypto(format!("Failed to read key file: {e}"))
                    })?;
                    if bytes.len() != 32 {
                        return Err(ContextraError::Crypto(format!(
                            "Invalid key file length: expected 32 bytes, got {}",
                            bytes.len()
                        )));
                    }
                    let mut sec = [0u8; 32];
                    sec.copy_from_slice(&bytes);
                    Ok(Zeroizing::new(sec))
                }
                Err(e) => Err(ContextraError::Crypto(format!(
                    "Failed to create signer key file: {e}"
                ))),
            }
>>>>>>> 0000a4c4f (feat(ports,sys,engine,mcp): implement CryptoRng, SecretBytes, and secure proof signer key loading (F-01, F-10, F-22))
        }
    }
}

/// Warning counters for store anomalies detected during initialization.
#[derive(Debug, Default)]
pub struct DpiRuntimeWarningCounters {
    /// Count of proof chain sequence gaps.
    pub chain_gaps: AtomicU64,
    /// Count of proof chain hash mismatches.
    pub chain_mismatches: AtomicU64,
    /// Count of unchained or corrupted proof files.
    pub unchained_files: AtomicU64,
}

impl DpiRuntimeWarningCounters {
    /// Records a [`ProofStoreWarning`].
    pub fn record(&self, warning: &ProofStoreWarning) {
        match warning {
            ProofStoreWarning::ProofChainGap { .. } => {
                self.chain_gaps.fetch_add(1, Ordering::Relaxed);
            }
            ProofStoreWarning::ChainMismatch { .. } => {
                self.chain_mismatches.fetch_add(1, Ordering::Relaxed);
            }
            ProofStoreWarning::UnchainedProofFile { .. }
            | ProofStoreWarning::MissingProofFile { .. } => {
                self.unchained_files.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

/// Standalone input struct for Stage 1 (Logical Phase-A) proof issuance.
#[derive(Debug)]
pub struct DpiLogicalInput {
    /// Deletion scope target.
    pub scope: DeletionScope,
    /// Storage scope string description.
    pub storage_scope: String,
    /// Length-prefixed BLAKE3 hash of deleted keys.
    pub deleted_keys_hash: [u8; 32],
    /// Footprint hash of affected storage bounds.
    pub footprint_hash: [u8; 32],
    /// Flag indicating whether the scope was confirmed empty without key deletion.
    pub observed_empty: bool,
    /// TxId after which logical deletion was executed.
    pub deleted_after_tx: TxId,
    /// Initial logical layer witnesses.
    pub witnesses: Vec<LayerWitness>,
    /// Additional scope file names affected by deletion (for copy observation).
    pub scope_file_names: Vec<String>,
}

/// Engine deletion runtime managing Ed25519 proof signing and issuance (§C3, §C7, ADR-14).
pub struct DpiDeletionRuntime {
    issuer: Arc<ProofIssuer>,
    db_instance_id: [u8; 16],
    warning_counters: Arc<DpiRuntimeWarningCounters>,
    signer: Arc<dyn ProofSigner>,
    proof_dir: PathBuf,
}

impl DpiDeletionRuntime {
    /// Opens or initializes `DpiDeletionRuntime` at `db_dir` using `signer_cfg_path`.
    ///
    /// The Ed25519 signer key file is loaded with strict 0600 permissions (fail-closed).
    /// Default key file path is `~/.contextra/proof_ed25519.key` if `signer_cfg_path` is None.
    ///
    /// # Errors
    ///
    /// Returns [`ContextraError::Crypto`] if key permissions or store verification fails.
    pub fn open(
        db_dir: &Path,
        signer_cfg_path: Option<&Path>,
        db_instance_id: [u8; 16],
    ) -> Result<Self> {
        let key_path = if let Some(p) = signer_cfg_path {
            p.to_path_buf()
        } else {
            let home = std::env::var("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("."));
            home.join(".contextra").join("proof_ed25519.key")
        };

        let source = FileOrCsprngSignerSource::new(key_path);
        Self::open_with_source(db_dir, &source, db_instance_id)
    }

    /// Opens or initializes `DpiDeletionRuntime` at `db_dir` using an explicit [`ProofSignerSource`].
    ///
    /// # Errors
    ///
    /// Returns [`ContextraError::Crypto`] if key retrieval or store verification fails.
    pub fn open_with_source(
        db_dir: &Path,
        source: &dyn ProofSignerSource,
        db_instance_id: [u8; 16],
    ) -> Result<Self> {
        let secret = source.get_signer_seed()?;
        let keypair = ProofKeyPairSigner::from_secret_bytes(*secret);
        let signer: Arc<dyn ProofSigner> = Arc::new(keypair);

        let proof_dir = db_dir.join("proofs_v4");
        std::fs::create_dir_all(&proof_dir).map_err(|e| {
            ContextraError::Crypto(format!("Failed to create proof directory: {e}"))
        })?;

        let (issuer, warnings) = ProofIssuer::open(&proof_dir, Arc::clone(&signer), db_instance_id)
            .map_err(|e| ContextraError::Crypto(format!("Failed to open ProofIssuer: {e}")))?;

        let counters = Arc::new(DpiRuntimeWarningCounters::default());
        for warn in &warnings {
            counters.record(warn);
            error!(warning = ?warn, "ProofStore warning detected on startup (fail-closed requirement)");
        }

        if !warnings.is_empty() {
            return Err(ContextraError::Crypto(format!(
                "ProofStore corrupted or contains gaps/mismatches ({} warnings)",
                warnings.len()
            )));
        }

        Ok(Self {
            issuer: Arc::new(issuer),
            db_instance_id,
            warning_counters: counters,
            signer,
            proof_dir,
        })
    }

    /// Accesses the underlying [`ProofIssuer`].
    #[must_use]
    pub fn issuer(&self) -> &Arc<ProofIssuer> {
        &self.issuer
    }

    /// Accesses the proof directory path.
    #[must_use]
    pub fn proof_dir(&self) -> &Path {
        &self.proof_dir
    }

    /// Accesses the active signer.
    #[must_use]
    pub fn signer(&self) -> &Arc<dyn ProofSigner> {
        &self.signer
    }

    /// Returns the database instance ID.
    #[must_use]
    pub fn db_instance_id(&self) -> [u8; 16] {
        self.db_instance_id
    }

    /// Returns warning counters for ProofStore anomalies.
    #[must_use]
    pub fn warning_counters(&self) -> &Arc<DpiRuntimeWarningCounters> {
        &self.warning_counters
    }

    /// Issues Stage 1 (Logical Phase-A) proof and returns [`FonLogicalHandle`] and [`IssIssuedProof`].
    ///
    /// Counter-checks mandatory and extra exclusions (e.g., `RegisteredCopies` if copies hold scope data).
    ///
    /// # Errors
    ///
    /// Returns [`ContextraError::Crypto`] or invalid input errors if options validation or proof issuance fails.
    pub fn finalize_logical(
        &self,
        input: DpiLogicalInput,
        opts: &ForgetOptions,
        copy_observer_report: Option<&DpiCopyObserverReport>,
        probes: &[&dyn WitStoreProbe],
    ) -> Result<(FonLogicalHandle, IssIssuedProof)> {
        opts.validate()?;

        let mut extra_exclusions = Vec::new();

        if let Some(rep) = copy_observer_report {
            if rep.copies_holding_scope > 0 {
                extra_exclusions.push(ExcludedScope::RegisteredCopies {
                    ids: rep.ids.iter().map(hex_encode_16).collect(),
                });
            }
        }

        let environment_tlv = observe_environment(&self.proof_dir, false, false);

        let req = IssRequest {
            scope: input.scope,
            storage_scope: input.storage_scope,
            deleted_keys_hash: input.deleted_keys_hash,
            footprint_hash: input.footprint_hash,
            observed_empty: input.observed_empty,
            deleted_after_tx: input.deleted_after_tx.inner(),
            stage: 1, // Stage 1 = Logical
            supersedes: None,
            assurance: opts.assurance,
            witnesses: input.witnesses,
            extra_exclusions,
            environment_tlv,
            wal_chain_receipt: None,
        };

        let issued = self
            .issuer
            .issue(req, probes)
            .map_err(|e| ContextraError::Crypto(format!("ProofIssuer failed stage 1: {e}")))?;

        let proof_id = issued.proof.proof_id;
        let proof_path = self
            .proof_dir
            .join("proofs")
            .join(format!("{}.json", hex_encode_32(&proof_id)));

        let handle = FonLogicalHandle {
            proof_id: proof_id_32_to_16(&proof_id),
            proof_path,
            full_proof_id: proof_id,
        };

        Ok((handle, issued))
    }
}

/// Legacy ID-derivation helper retained strictly for negative security testing.
#[doc(hidden)]
#[must_use]
pub fn db_instance_id_to_secret_legacy_test_only(db_id: &[u8; 16]) -> [u8; 32] {
    let hash = blake3::Hasher::new_derive_key("contextra/proof-signer-secret/v1")
        .update(db_id)
        .finalize();
    *hash.as_bytes()
}

fn hex_encode_32(bytes: &[u8; 32]) -> String {
    let mut s = String::with_capacity(64);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn hex_encode_16(bytes: &[u8; 16]) -> String {
    let mut s = String::with_capacity(32);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn proof_id_32_to_16(bytes: &[u8; 32]) -> [u8; 16] {
    let mut out = [0u8; 16];
    out.copy_from_slice(&bytes[..16]);
    out
}

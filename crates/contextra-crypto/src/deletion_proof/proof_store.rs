//! Persisted proof store and chain log management (ADR-13).

use super::proof_v4::ProofV4;
use crate::error::{CryptoError, Result};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    let mut bytes = Vec::with_capacity(s.len() / 2);
    for i in (0..s.len()).step_by(2) {
        let b = u8::from_str_radix(&s[i..i + 2], 16).ok()?;
        bytes.push(b);
    }
    Some(bytes)
}

/// Failpoint hook for testing crash recovery during save_proof.
#[cfg(any(test, feature = "test-utils"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreCrashFailpoint {
    /// Crash after writing .bin file before .json write.
    PostBinWrite,
    /// Crash after writing .json file before directory fsync.
    PostJsonWrite,
    /// Crash after directory fsync before chain.log append.
    PostDirFsync,
    /// Crash after appending to chain.log before final directory fsync.
    PostChainLogAppend,
    /// Crash after final directory fsync.
    PostFinalFsync,
}

/// Warning emitted during store opening when a chain log or proof file gap is detected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofStoreWarning {
    /// A sequence gap was found in chain.log (expected seq, found seq).
    ProofChainGap {
        /// Expected monotonic sequence number.
        expected_seq: u64,
        /// Actual sequence number found in log line.
        found_seq: u64,
    },
    /// A proof file referenced in chain.log was missing from disk.
    MissingProofFile {
        /// Sequence number of missing proof.
        seq: u64,
        /// Expected file path.
        path: PathBuf,
    },
    /// A proof file (.bin or .json) exists on disk but is not recorded in chain.log.
    UnchainedProofFile {
        /// Sequence number of unchained proof.
        seq: u64,
        /// File path of unchained proof file.
        path: PathBuf,
    },
    /// Chain hash mismatch detected between chain.log and recalculated proof hash.
    ChainMismatch {
        /// Sequence number of mismatch.
        seq: u64,
        /// Expected chain hash from log.
        expected_hash: [u8; 32],
        /// Recomputed chain hash from canonical .bin proof file.
        recomputed_hash: [u8; 32],
    },
}

/// Durable store for saving ProofV4 instances as .bin (canonical TLV) and .json with chain log appending.
pub struct ProofStore {
    dir: PathBuf,
    last_seq: u64,
    last_chain_hash: [u8; 32],
    #[cfg(any(test, feature = "test-utils"))]
    failpoint: Option<StoreCrashFailpoint>,
}

impl ProofStore {
    /// Sets an active failpoint for testing crash simulation.
    #[cfg(any(test, feature = "test-utils"))]
    pub fn set_failpoint(&mut self, fp: Option<StoreCrashFailpoint>) {
        self.failpoint = fp;
    }
    /// Opens or initializes a ProofStore at `dir`.
    pub fn open(dir: impl AsRef<Path>) -> Result<(Self, Vec<ProofStoreWarning>)> {
        let dir_path = dir.as_ref().to_path_buf();
        let proofs_dir = dir_path.join("proofs");
        std::fs::create_dir_all(&proofs_dir)
            .map_err(|e| CryptoError::Crypto(format!("Failed to create proofs directory: {e}")))?;

        let chain_log_path = dir_path.join("chain.log");
        let mut warnings = Vec::new();

        let mut last_seq = 0u64;
        let mut last_chain_hash = [0u8; 32];

        let mut log_chained_seqs = std::collections::HashSet::new();

        if chain_log_path.exists() {
            let file = File::open(&chain_log_path)
                .map_err(|e| CryptoError::Crypto(format!("Failed to open chain.log: {e}")))?;
            let reader = BufReader::new(file);

            let mut expected_seq = 1u64;
            let mut current_prev_hash = [0u8; 32];

            for line in reader.lines() {
                let line_str =
                    line.map_err(|e| CryptoError::Crypto(format!("Failed to read line: {e}")))?;
                if line_str.trim().is_empty() {
                    continue;
                }

                let parts: Vec<&str> = line_str.split('|').map(|s| s.trim()).collect();
                if parts.len() >= 3 {
                    if let Ok(seq) = parts[0].parse::<u64>() {
                        log_chained_seqs.insert(seq);
                        if seq != expected_seq {
                            warnings.push(ProofStoreWarning::ProofChainGap {
                                expected_seq,
                                found_seq: seq,
                            });
                            expected_seq = seq;
                        }

                        let proof_id_hex = parts[1];
                        let bin_filename = format!("{:016}-{}.bin", seq, proof_id_hex);
                        let bin_path = proofs_dir.join(&bin_filename);
                        if !bin_path.exists() {
                            warnings.push(ProofStoreWarning::MissingProofFile {
                                seq,
                                path: bin_path.clone(),
                            });
                        } else if let Ok(bin_data) = std::fs::read(&bin_path) {
                            if let Ok(fields) =
                                super::canon::TlvDecoder::new(&bin_data).decode_all()
                            {
                                // Reconstruct ProofV4 and recompute chain hash to verify integrity
                                if let (Some(expected_hash_bytes), true) =
                                    (hex_decode(parts[2]), !fields.is_empty())
                                {
                                    if expected_hash_bytes.len() == 32 {
                                        let mut expected_hash = [0u8; 32];
                                        expected_hash.copy_from_slice(&expected_hash_bytes);
                                        // Compute chain hash over raw binary proof file
                                        let mut inner_hasher = blake3::Hasher::new();
                                        inner_hasher.update(super::proof_v4::PROOF_V4_DOMAIN_TAG);
                                        inner_hasher.update(&bin_data);
                                        // Extract signature from TLV or JSON if present
                                        let json_path = proofs_dir
                                            .join(format!("{:016}-{}.json", seq, proof_id_hex));
                                        if let Ok(json_str) = std::fs::read_to_string(&json_path) {
                                            if let Ok(proof) =
                                                serde_json::from_str::<ProofV4>(&json_str)
                                            {
                                                let recomputed_hash =
                                                    proof.compute_chain_hash(&current_prev_hash);
                                                if recomputed_hash != expected_hash {
                                                    warnings.push(
                                                        ProofStoreWarning::ChainMismatch {
                                                            seq,
                                                            expected_hash,
                                                            recomputed_hash,
                                                        },
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if let Some(hash_bytes) = hex_decode(parts[2]) {
                            if hash_bytes.len() == 32 {
                                last_chain_hash.copy_from_slice(&hash_bytes);
                                current_prev_hash = last_chain_hash;
                            }
                        }

                        last_seq = seq;
                        expected_seq += 1;
                    }
                }
            }
        }

        // Detect unchained proof files (.bin or .json files without chain.log line)
        if let Ok(entries) = std::fs::read_dir(&proofs_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Some(filename) = p.file_name().and_then(|f| f.to_str()) {
                    if filename.len() >= 16 {
                        if let Ok(seq) = filename[..16].parse::<u64>() {
                            if !log_chained_seqs.contains(&seq) {
                                warnings.push(ProofStoreWarning::UnchainedProofFile {
                                    seq,
                                    path: p.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok((
            Self {
                dir: dir_path,
                last_seq,
                last_chain_hash,
                #[cfg(any(test, feature = "test-utils"))]
                failpoint: None,
            },
            warnings,
        ))
    }

    /// Verifies the full chain integrity on demand, returning warnings.
    pub fn verify_chain(&self) -> Result<Vec<ProofStoreWarning>> {
        let (_, warnings) = Self::open(&self.dir)?;
        Ok(warnings)
    }

    /// Saves a ProofV4 instance to `proofs/{seq:016}-{proof_id}.bin` and `.json`, and appends to `chain.log`.
    pub fn save_proof(&mut self, proof: &ProofV4) -> Result<[u8; 32]> {
        // Fail-closed check: verify no chain mismatch or unchained proof file before proceeding
        let warnings = self.verify_chain()?;
        if warnings.iter().any(|w| {
            matches!(
                w,
                ProofStoreWarning::ChainMismatch { .. }
                    | ProofStoreWarning::UnchainedProofFile { .. }
            )
        }) {
            return Err(CryptoError::Crypto(format!(
                "Refusing save_proof: store contains chain mismatch or unchained proof files: {:?}",
                warnings
            )));
        }

        let seq = self.last_seq + 1;
        let proof_id_hex = hex_encode(&proof.proof_id);

        let proofs_dir = self.dir.join("proofs");
        std::fs::create_dir_all(&proofs_dir)
            .map_err(|e| CryptoError::Crypto(format!("Failed to create proofs dir: {e}")))?;

        let bin_path = proofs_dir.join(format!("{:016}-{}.bin", seq, proof_id_hex));
        let json_path = proofs_dir.join(format!("{:016}-{}.json", seq, proof_id_hex));

        // 1. Save canonical TLV to .bin with O_EXCL
        let canonical_tlv = proof.construct_canonical_payload();
        {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&bin_path)
                .map_err(|e| CryptoError::Crypto(format!("Failed O_EXCL bin file create: {e}")))?;
            file.write_all(&canonical_tlv)
                .map_err(|e| CryptoError::Crypto(format!("Failed write bin file: {e}")))?;
            file.sync_all()
                .map_err(|e| CryptoError::Crypto(format!("Failed fsync bin file: {e}")))?;
        }

        #[cfg(any(test, feature = "test-utils"))]
        if self.failpoint == Some(StoreCrashFailpoint::PostBinWrite) {
            return Err(CryptoError::Crypto(
                "Simulated crash post .bin write".into(),
            ));
        }

        // 2. Save JSON to .json with O_EXCL
        let json_bytes = serde_json::to_vec_pretty(proof)
            .map_err(|e| CryptoError::Crypto(format!("Failed JSON serialize: {e}")))?;
        {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&json_path)
                .map_err(|e| CryptoError::Crypto(format!("Failed O_EXCL json file create: {e}")))?;
            file.write_all(&json_bytes)
                .map_err(|e| CryptoError::Crypto(format!("Failed write json file: {e}")))?;
            file.sync_all()
                .map_err(|e| CryptoError::Crypto(format!("Failed fsync json file: {e}")))?;
        }

        #[cfg(any(test, feature = "test-utils"))]
        if self.failpoint == Some(StoreCrashFailpoint::PostJsonWrite) {
            return Err(CryptoError::Crypto(
                "Simulated crash post .json write".into(),
            ));
        }

        contextra_durable_fs::sync_dir(&proofs_dir)
            .map_err(|e| CryptoError::Crypto(format!("Failed fsync proofs dir: {e}")))?;

        #[cfg(any(test, feature = "test-utils"))]
        if self.failpoint == Some(StoreCrashFailpoint::PostDirFsync) {
            return Err(CryptoError::Crypto(
                "Simulated crash post proofs dir fsync".into(),
            ));
        }

        // 3. Compute new chain hash
        let new_chain_hash = proof.compute_chain_hash(&self.last_chain_hash);
        let chain_hash_hex = hex_encode(&new_chain_hash);

        // 4. Append to chain.log
        let chain_log_path = self.dir.join("chain.log");
        let log_line = format!("{:016} | {} | {}\n", seq, proof_id_hex, chain_hash_hex);
        {
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&chain_log_path)
                .map_err(|e| CryptoError::Crypto(format!("Failed open chain.log: {e}")))?;
            file.write_all(log_line.as_bytes())
                .map_err(|e| CryptoError::Crypto(format!("Failed write chain.log: {e}")))?;
            file.sync_all()
                .map_err(|e| CryptoError::Crypto(format!("Failed fsync chain.log: {e}")))?;
        }

        #[cfg(any(test, feature = "test-utils"))]
        if self.failpoint == Some(StoreCrashFailpoint::PostChainLogAppend) {
            return Err(CryptoError::Crypto(
                "Simulated crash post chain.log append".into(),
            ));
        }

        contextra_durable_fs::sync_dir(&self.dir)
            .map_err(|e| CryptoError::Crypto(format!("Failed fsync base dir: {e}")))?;

        #[cfg(any(test, feature = "test-utils"))]
        if self.failpoint == Some(StoreCrashFailpoint::PostFinalFsync) {
            return Err(CryptoError::Crypto(
                "Simulated crash post final dir fsync".into(),
            ));
        }

        self.last_seq = seq;
        self.last_chain_hash = new_chain_hash;

        Ok(new_chain_hash)
    }

    /// Returns the last sequence number saved.
    pub fn last_seq(&self) -> u64 {
        self.last_seq
    }

    /// Returns the current head chain hash.
    pub fn head_chain_hash(&self) -> [u8; 32] {
        self.last_chain_hash
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::super::types::DeletionScope;
    use super::*;
    use crate::deletion_proof::builder::ProofBuilder;
    use crate::deletion_proof::signer::ProofKeyPairSigner;
    use contextra_types::{DocId, TenantId};
    use rand::rngs::OsRng;
    use tempfile::tempdir;

    #[test]
    fn test_proof_store_save_and_open_roundtrip() {
        let dir = tempdir().expect("tempdir failed");
        let signer = ProofKeyPairSigner::generate(&mut OsRng);

        let (mut store, warnings) = ProofStore::open(dir.path()).expect("open store failed");
        assert!(warnings.is_empty());

        let scope = DeletionScope::Document {
            doc_id: DocId(1),
            tenant_id: TenantId::try_new(1).expect("tenant"),
        };
        let proof = ProofBuilder::with_required(scope, "db-store-test", [0x11; 32], [0u8; 32], 10, 1)
            .finish(&signer)
            .expect("builder finish failed");

        let chain_hash = store.save_proof(&proof).expect("save proof failed");

        assert_eq!(store.last_seq(), 1);
        assert_eq!(store.head_chain_hash(), chain_hash);

        // Reopen store
        let (reopened, warnings2) = ProofStore::open(dir.path()).expect("reopen store failed");
        assert!(warnings2.is_empty());
        assert_eq!(reopened.last_seq(), 1);
        assert_eq!(reopened.head_chain_hash(), chain_hash);
    }

    #[test]
    fn test_proof_store_detects_gap_warning() {
        let dir = tempdir().expect("tempdir failed");
        let chain_log_path = dir.path().join("chain.log");

        // Manually write gap in chain.log (seq 1, then seq 3)
        let log_content = "0000000000000001 | 0101010101010101010101010101010101010101010101010101010101010101 | 0000000000000000000000000000000000000000000000000000000000000000\n\
                           0000000000000003 | 0202020202020202020202020202020202020202020202020202020202020202 | 0000000000000000000000000000000000000000000000000000000000000000\n";
        std::fs::write(&chain_log_path, log_content).expect("write chain.log failed");

        let (_store, warnings) = ProofStore::open(dir.path()).expect("open failed");
        assert!(!warnings.is_empty());
        assert!(warnings.iter().any(|w| matches!(
            w,
            ProofStoreWarning::ProofChainGap {
                expected_seq: 2,
                found_seq: 3
            }
        )));
    }
}

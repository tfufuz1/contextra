//! Integrity verifier for WAL HMAC chains.

use contextra_types::{ContextraError, Result};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Stateful wrapper around HMAC-SHA256 initialized with WAL domain separation.
pub struct WalHmac {
    mac: Hmac<Sha256>,
}

impl WalHmac {
    pub fn new(integrity_key: &[u8]) -> Result<Self> {
        if integrity_key.is_empty() {
            return Err(ContextraError::InvalidInput(
                "Key cannot be empty".to_string(),
            ));
        }
        if integrity_key.len() > 10_000 {
            return Err(ContextraError::InvalidInput(format!(
                "integrity_key length {} exceeds maximum allowed bound of 10000 bytes",
                integrity_key.len()
            )));
        }
        let mut mac = Hmac::<Sha256>::new_from_slice(integrity_key)
            .map_err(|e| ContextraError::Crypto(format!("HMAC key error: {}", e)))?;
        mac.update(b"contextra-wal-v1");
        Ok(Self { mac })
    }

    /// Feeds new data into the ongoing HMAC calculation.
    pub fn update(&mut self, data: &[u8]) {
        self.mac.update(data);
    }

    /// Finalizes the HMAC calculation and returns the 32-byte digest.
    pub fn finalize(self) -> [u8; 32] {
        self.mac.finalize().into_bytes().into()
    }
}

/// Snapshot of a WAL entry used for cryptographic integrity verification.
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct WalEntrySnapshot {
    pub tx_id: u64,
    pub seq_no: u64,
    pub op_type: u8,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub checksum: [u8; 32],
    pub prev_hmac: [u8; 32],
}

/// Stateful verifier for WAL HMAC-SHA256 hash chains.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct IntegrityVerifier {
    last_hmac: [u8; 32],
    integrity_key: Vec<u8>,
    last_seq_no: Option<u64>,
}

impl IntegrityVerifier {
    pub fn new(integrity_key: &[u8]) -> Self {
        Self {
            last_hmac: [0u8; 32],
            integrity_key: integrity_key.to_vec(),
            last_seq_no: None,
        }
    }

    pub fn set_last_hmac(&mut self, hmac: [u8; 32]) {
        self.last_hmac = hmac;
    }

    pub fn last_hmac_snapshot(&self) -> [u8; 32] {
        self.last_hmac
    }

    pub fn set_last_seq_no(&mut self, seq_no: Option<u64>) {
        self.last_seq_no = seq_no;
    }

    pub fn last_seq_no_snapshot(&self) -> Option<u64> {
        self.last_seq_no
    }

    /// Verifies a V3 entry and updates the chain state.
    pub fn verify_and_update_v3(&mut self, entry: &WalEntrySnapshot, offset: u64) -> Result<()> {
        if entry.op_type > 2 {
            return Err(ContextraError::wal_corruption(
                offset,
                format!("Unsupported op_type {}", entry.op_type),
            ));
        }

        if let Some(last_seq) = self.last_seq_no {
            if entry.seq_no < last_seq {
                return Err(ContextraError::wal_corruption(
                    offset,
                    format!(
                        "Non-monotonic sequence number {} (last: {})",
                        entry.seq_no, last_seq
                    ),
                ));
            }
            if entry.seq_no > last_seq + 1 {
                return Err(ContextraError::wal_corruption(
                    offset,
                    format!(
                        "Sequence gap detected: expected {}, got {}",
                        last_seq + 1,
                        entry.seq_no
                    ),
                ));
            }
        }

        let mut mac = WalHmac::new(&self.integrity_key)?;
        mac.update(&self.last_hmac);
        mac.update(&entry.seq_no.to_le_bytes());

        let tx_id_bytes = entry.tx_id.to_le_bytes();
        mac.update(&tx_id_bytes);

        if entry.op_type == 0 {
            mac.update(&[0u8]);
            mac.update(&(entry.key.len() as u32).to_le_bytes());
            mac.update(&entry.key);
            mac.update(&(entry.value.len() as u32).to_le_bytes());
            mac.update(&entry.value);
        } else if entry.op_type == 1 {
            mac.update(&[1u8]);
            mac.update(&(entry.key.len() as u32).to_le_bytes());
            mac.update(&entry.key);
        } else if entry.op_type == 2 {
            mac.update(&[2u8]);
            let committed = if entry.value.first().copied().unwrap_or(0) != 0 {
                1u8
            } else {
                0u8
            };
            mac.update(&[committed]);
        }

        let computed = mac.finalize();
        if contextra_crypto::verify_chain_complete(&computed, &entry.checksum).is_err()
            || contextra_crypto::verify_chain_complete(&entry.prev_hmac, &self.last_hmac).is_err()
        {
            return Err(ContextraError::wal_corruption(
                offset,
                format!("HMAC mismatch for seq {}", entry.seq_no),
            ));
        }

        self.last_hmac = computed;
        self.last_seq_no = Some(entry.seq_no);
        Ok(())
    }

    /// Verifies a V2 entry (legacy without tx_id and without length prefixes) and updates the chain state.
    #[deprecated(note = "Only for legacy replay of WalVersion::V2 files")]
    pub fn verify_and_update_v2(&mut self, entry: &WalEntrySnapshot, offset: u64) -> Result<()> {
        if entry.op_type > 2 {
            return Err(ContextraError::wal_corruption(
                offset,
                format!("Unsupported op_type {}", entry.op_type),
            ));
        }

        if let Some(last_seq) = self.last_seq_no {
            if entry.seq_no < last_seq {
                return Err(ContextraError::wal_corruption(
                    offset,
                    format!(
                        "Non-monotonic sequence number {} (last: {})",
                        entry.seq_no, last_seq
                    ),
                ));
            }
            if entry.seq_no > last_seq + 1 {
                return Err(ContextraError::wal_corruption(
                    offset,
                    format!(
                        "Sequence gap detected: expected {}, got {}",
                        last_seq + 1,
                        entry.seq_no
                    ),
                ));
            }
        }

        let mut mac = WalHmac::new(&self.integrity_key)?;
        mac.update(&self.last_hmac);
        mac.update(&entry.seq_no.to_le_bytes());

        mac.update(&[entry.op_type]);
        if entry.op_type == 0 {
            mac.update(&entry.key);
            mac.update(&entry.value);
        } else if entry.op_type == 1 {
            mac.update(&entry.key);
        } else if entry.op_type == 2 {
            let committed = if entry.value.first().copied().unwrap_or(0) != 0 {
                1u8
            } else {
                0u8
            };
            mac.update(&[committed]);
        }

        let computed = mac.finalize();
        if contextra_crypto::verify_chain_complete(&computed, &entry.checksum).is_err()
            || contextra_crypto::verify_chain_complete(&entry.prev_hmac, &self.last_hmac).is_err()
        {
            return Err(ContextraError::wal_corruption(
                offset,
                format!("HMAC mismatch for seq {}", entry.seq_no),
            ));
        }

        self.last_hmac = computed;
        self.last_seq_no = Some(entry.seq_no);
        Ok(())
    }

    /// Updates the chain state for legacy V1 entries without HMAC verification.
    pub fn skip_hmac_verify_legacy(&mut self, entry: &WalEntrySnapshot) {
        self.last_hmac = entry.checksum;
        self.last_seq_no = Some(entry.seq_no);
    }

    /// Default verification delegating to V3 verification.
    pub fn verify_and_update(&mut self, entry: &WalEntrySnapshot, offset: u64) -> Result<()> {
        self.verify_and_update_v3(entry, offset)
    }
}

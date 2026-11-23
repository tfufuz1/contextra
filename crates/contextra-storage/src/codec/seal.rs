//! # Seal-Rahmen und Versiegelungsbudget (SPEC §10.10, §10.1, §10.4, §10.5, §10.7, §16.3)
//!
//! Ein versiegelter Rahmen hat die Form `nonce[24] ‖ ciphertext ‖ tag[16]`: der Tag ersetzt
//! das `chk`-Feld (§10.1), die Nonce steht vor dem Content (§10.7). Die AAD bindet Art,
//! Space, Cluster und Ort der Einheit, damit ein vertauschter Block nicht öffnet (§10.4).
//!
//! Lokale Adapter (Abweichungen vom Port, siehe PR-Beschreibung):
//! - [`SealCipher`]: AEAD mit 24-Byte-Nonce (UE-10). Der Port `Cipher` führt noch 12 Byte
//!   (WP-2.4).
//! - [`SealHighWater`]: dauerhafte Hochwassermarke `seal_hw` im Catalog (D-28, §10.8).
//!
//! Der Kern bindet keine Krypto-Crate (§3.5). Die AEAD-Implementierung kommt von außen.

use std::fmt;
use std::io;

use contextra_ports::persistence::{Entropy, Kdf};

/// Länge der Nonce je Versiegelung (SD-14).
pub const SEAL_NONCE_LEN: usize = 24;
/// Länge des AEAD-Tags, das das `chk`-Feld ersetzt (§10.1).
pub const SEAL_TAG_LEN: usize = 16;
/// Schranke je Schlüssel: höchstens 2^32 Versiegelungen (D-28, §16.3).
pub const SEAL_NONCE_LIMIT: u64 = 1 << 32;
/// Größe einer Reservierung von `seal_hw` (Annahme, siehe PR; §10.8 nennt keine Zahl).
pub const SEAL_RESERVE_BLOCK: u64 = 1 << 16;
/// Längste AAD (Blob-Record, 43 Byte).
pub const SEAL_AAD_MAX_LEN: usize = 43;

const SEAL_LABEL: &[u8] = b"ctxs seal v1";
const KIND_REDO_OP: u8 = 1;
const KIND_BLOCK: u8 = 2;
const KIND_BLOB: u8 = 3;

/// Nonce einer Versiegelung.
pub type SealNonce = [u8; SEAL_NONCE_LEN];

/// AEAD mit 32-Byte-Schlüssel und 24-Byte-Nonce (XChaCha20-Poly1305, UE-10).
pub trait SealCipher: Send + Sync {
    /// Liefert `ciphertext ‖ tag`.
    fn seal(&self, key: &[u8; 32], nonce: &SealNonce, aad: &[u8], plain: &[u8]) -> Vec<u8>;
    /// Prüft Tag und entschlüsselt; `None` bei jeder Abweichung.
    fn open(&self, key: &[u8; 32], nonce: &SealNonce, aad: &[u8], sealed: &[u8])
        -> Option<Vec<u8>>;
}

/// Dauerhafte Hochwassermarke `seal_hw` je Schlüsselepoche (catalog `reserve`, §10.8).
pub trait SealHighWater: Send + Sync {
    /// Kehrt erst zurück, wenn `new_hw` dauerhaft ist.
    fn persist(&self, new_hw: u64) -> io::Result<()>;
}

/// Fehler des Seal-Rahmens. Ohne Inhaltsfelder, damit keine Schlüssel oder Klartexte in Fehlern landen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SealError {
    /// Rahmen ungültig oder Tag falsch; fail-closed (§10.11).
    Corrupt,
    /// Schranke der Schlüsselepoche erreicht; neue Epoche nötig (§16.3).
    Exhausted,
    /// Die Hochwassermarke konnte nicht dauerhaft gemacht werden.
    Io,
}

impl fmt::Display for SealError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Corrupt => write!(f, "Versiegelter Rahmen ungültig (fail-closed)"),
            Self::Exhausted => write!(f, "Versiegelungsschranke der Schlüsselepoche erreicht"),
            Self::Io => write!(f, "Hochwassermarke der Versiegelung nicht dauerhaft"),
        }
    }
}

impl std::error::Error for SealError {}

/// Siegelschlüssel: `Kdf(cluster_key, "ctxs seal v1" ‖ key_epoch u32)` (§10.10).
pub fn derive_seal_key(kdf: &dyn Kdf, cluster_key: &[u8; 32], key_epoch: u32) -> [u8; 32] {
    let epoch = key_epoch.to_le_bytes();
    kdf.derive(cluster_key, &[SEAL_LABEL, epoch.as_slice()])
}

/// Typisierte AAD je Einheit. Kodierung: Art-Byte ‖ feste Felder, little-endian.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SealAad {
    /// Redo-Op-Nutzlast (§10.4): (space, cluster, op_uid, op_index). 37 Byte.
    RedoOp {
        /// Space der Op.
        space: u16,
        /// Cluster der Op (opak).
        cluster: [u8; 16],
        /// Eindeutige Op-Kennung.
        op_uid: [u8; 16],
        /// Index der Op im Frame.
        op_index: u16,
    },
    /// Block (§10.5): (space, cluster, block_uid). 35 Byte.
    Block {
        /// Space des Blocks.
        space: u16,
        /// Cluster des Blocks (opak).
        cluster: [u8; 16],
        /// Eindeutige Block-Kennung.
        block_uid: [u8; 16],
    },
    /// Blob-Record (§10.7): (space, cluster, blob_id, record_uid). 43 Byte.
    Blob {
        /// Space des Records.
        space: u16,
        /// Cluster des Records (opak).
        cluster: [u8; 16],
        /// Blob-Kennung.
        blob_id: u64,
        /// Eindeutige Record-Kennung.
        record_uid: [u8; 16],
    },
}

impl SealAad {
    /// Kodiert die AAD. Das Art-Byte verhindert, dass eine Einheit als andere Art öffnet.
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(SEAL_AAD_MAX_LEN);
        match *self {
            Self::RedoOp {
                space,
                cluster,
                op_uid,
                op_index,
            } => {
                out.push(KIND_REDO_OP);
                out.extend_from_slice(&space.to_le_bytes());
                out.extend_from_slice(&cluster);
                out.extend_from_slice(&op_uid);
                out.extend_from_slice(&op_index.to_le_bytes());
            }
            Self::Block {
                space,
                cluster,
                block_uid,
            } => {
                out.push(KIND_BLOCK);
                out.extend_from_slice(&space.to_le_bytes());
                out.extend_from_slice(&cluster);
                out.extend_from_slice(&block_uid);
            }
            Self::Blob {
                space,
                cluster,
                blob_id,
                record_uid,
            } => {
                out.push(KIND_BLOB);
                out.extend_from_slice(&space.to_le_bytes());
                out.extend_from_slice(&cluster);
                out.extend_from_slice(&blob_id.to_le_bytes());
                out.extend_from_slice(&record_uid);
            }
        }
        out
    }
}

/// Zieht eine frische Nonce aus `Entropy` (§10.10: je Versiegelung neu).
pub fn fresh_nonce(entropy: &dyn Entropy) -> io::Result<SealNonce> {
    let mut nonce = [0u8; SEAL_NONCE_LEN];
    entropy.fill(&mut nonce)?;
    Ok(nonce)
}

/// Versiegelt `plain` zum Rahmen `nonce ‖ ciphertext ‖ tag`.
pub fn seal_frame(
    cipher: &dyn SealCipher,
    key: &[u8; 32],
    nonce: &SealNonce,
    aad: &[u8],
    plain: &[u8],
) -> Vec<u8> {
    let sealed = cipher.seal(key, nonce, aad, plain);
    let mut frame = Vec::with_capacity(SEAL_NONCE_LEN + sealed.len());
    frame.extend_from_slice(nonce);
    frame.extend_from_slice(&sealed);
    frame
}

/// Öffnet einen Rahmen. Jede Abweichung (Länge, Tag, AAD, Schlüssel) ist `Corrupt`.
pub fn open_frame(
    cipher: &dyn SealCipher,
    key: &[u8; 32],
    aad: &[u8],
    frame: &[u8],
) -> Result<Vec<u8>, SealError> {
    let (nonce_bytes, sealed) = frame
        .split_at_checked(SEAL_NONCE_LEN)
        .ok_or(SealError::Corrupt)?;
    if sealed.len() < SEAL_TAG_LEN {
        return Err(SealError::Corrupt);
    }
    let nonce: SealNonce = nonce_bytes.try_into().map_err(|_| SealError::Corrupt)?;
    cipher
        .open(key, &nonce, aad, sealed)
        .ok_or(SealError::Corrupt)
}

/// Zählt Versiegelungen je Schlüsselepoche (D-28).
///
/// Nur reservierte Werte werden benutzt. Ist der reservierte Block verbraucht, macht
/// [`SealBudget::take`] den nächsten Hochwert dauerhaft, bevor die Versiegelung läuft.
/// Nach einem Absturz gilt der ganze dauerhafte Hochwert als verbraucht: der Rest des
/// Blocks verfällt (§16.3). Invariante: `used <= reserved <= limit`.
#[derive(Debug)]
pub struct SealBudget {
    limit: u64,
    block: u64,
    reserved: u64,
    used: u64,
}

impl SealBudget {
    /// Öffnet das Budget einer Epoche aus dem dauerhaften Hochwert.
    pub fn resume(limit: u64, block: u64, durable_hw: u64) -> Result<Self, SealError> {
        if block == 0 || durable_hw > limit {
            return Err(SealError::Corrupt);
        }
        Ok(Self {
            limit,
            block,
            reserved: durable_hw,
            used: durable_hw,
        })
    }

    /// Budget mit den Standardwerten der Spec (2^32 je Epoche).
    pub fn for_epoch(durable_hw: u64) -> Result<Self, SealError> {
        Self::resume(SEAL_NONCE_LIMIT, SEAL_RESERVE_BLOCK, durable_hw)
    }

    /// Anzahl bisher belegter Versiegelungen dieser Epoche.
    pub fn used(&self) -> u64 {
        self.used
    }

    /// Dauerhaft reservierte Obergrenze dieser Epoche.
    pub fn reserved(&self) -> u64 {
        self.reserved
    }

    /// Belegt eine Versiegelung. Schlägt die Reservierung fehl, bleibt der Zustand unverändert.
    pub fn take(&mut self, hw: &dyn SealHighWater) -> Result<(), SealError> {
        if self.used >= self.limit {
            return Err(SealError::Exhausted);
        }
        if self.used == self.reserved {
            let next = self.reserved.saturating_add(self.block).min(self.limit);
            hw.persist(next).map_err(|_| SealError::Io)?;
            self.reserved = next;
        }
        self.used = self.used.checked_add(1).ok_or(SealError::Exhausted)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/format/seal_cipher.rs"]
mod seal_cipher;

#[cfg(test)]
#[path = "../../tests/format/seal_tests.rs"]
mod seal_tests;

// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: CopyRegEntry, CopyKind, CopyStatus und CpyCopyRegistry Trait Definition für Checkpoint Kopienregister.
// INVARIANTEN: Zero-Panic, Fail-Closed, Identisches Format zu contextra-store CopyRegEntry.

use contextra_ports::BoxFuture;
use contextra_types::{ContextraError, Result};

/// Computes a deterministic 16-byte file identifier for a file name using BLAKE3 derive_key.
pub fn file_id_for_name(name: &str) -> [u8; 16] {
    let hash = blake3::Hasher::new_derive_key("contextra/copy-file-id/v1")
        .update(name.as_bytes())
        .finalize();
    let mut file_id = [0u8; 16];
    file_id.copy_from_slice(&hash.as_bytes()[..16]);
    file_id
}

/// Kind of copy recorded in the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyKind {
    Hardlink = 1,
    Reflink = 2,
    FullCopy = 3,
}

/// Detailed status of a registered copy for scope / Article 17/19 verification.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CopyStatus {
    /// The copy has been physically removed / unlinked.
    Deleted,
    /// The copy destination or medium is unreachable.
    Unreachable,
    /// The copy is out of system scope (e.g. Art. 19 external recipients, Art. 28 processors).
    OutOfScope { reason: String },
    /// The copy is legally retained under statutory retention obligations.
    Retained { legal_basis: String },
    /// Default status for unknown or unverified copies. Blocks full proof assertions.
    #[default]
    Unknown,
}

impl CopyStatus {
    /// Creates an `OutOfScope` status for external recipients under Art. 19 GDPR.
    #[must_use]
    pub fn art19_recipient(recipient_notice: &str) -> Self {
        CopyStatus::OutOfScope {
            reason: format!(
                "Art. 19 DSGVO: Empfänger-Benachrichtigung erforderlich ({recipient_notice})"
            ),
        }
    }

    /// Creates an `OutOfScope` status for data processors under Art. 28 GDPR.
    #[must_use]
    pub fn art28_processor(processor_notice: &str) -> Self {
        CopyStatus::OutOfScope {
            reason: format!(
                "Art. 28 DSGVO: Auftragsverarbeiter-Weisung erforderlich ({processor_notice})"
            ),
        }
    }

    /// Returns `true` if the status represents verified deletion.
    #[must_use]
    pub fn is_deleted(&self) -> bool {
        matches!(self, CopyStatus::Deleted)
    }

    /// Returns `true` if the status is unknown and blocks full deletion proof assertions.
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        matches!(self, CopyStatus::Unknown)
    }
}

/// An entry in the copy registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyRegEntry {
    pub copy_id: [u8; 16],
    pub kind: CopyKind,
    pub file_ids: Vec<[u8; 16]>,
    pub created_tx: u64,
}

impl CopyRegEntry {
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.copy_id);
        let kind_u8 = match self.kind {
            CopyKind::Hardlink => 1u8,
            CopyKind::Reflink => 2u8,
            CopyKind::FullCopy => 3u8,
        };
        buf.push(kind_u8);
        #[allow(clippy::cast_possible_truncation)]
        buf.extend_from_slice(&(self.file_ids.len() as u32).to_be_bytes());
        for fid in &self.file_ids {
            buf.extend_from_slice(fid);
        }
        buf.extend_from_slice(&self.created_tx.to_be_bytes());
        Ok(buf)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 16 + 1 + 4 + 8 {
            return Err(ContextraError::Serialization(
                "CopyRegEntry too short".into(),
            ));
        }

        let mut copy_id = [0u8; 16];
        copy_id.copy_from_slice(&bytes[0..16]);

        let kind = match bytes[16] {
            1 => CopyKind::Hardlink,
            2 => CopyKind::Reflink,
            3 => CopyKind::FullCopy,
            k => {
                return Err(ContextraError::Serialization(format!(
                    "Unknown CopyKind {k}"
                )))
            }
        };

        let file_count = u32::from_be_bytes(bytes[17..21].try_into().unwrap_or([0; 4])) as usize;
        let expected_len = 16 + 1 + 4 + (file_count * 16) + 8;
        if bytes.len() < expected_len {
            return Err(ContextraError::Serialization(
                "CopyRegEntry truncated file_ids".into(),
            ));
        }

        let mut offset = 21;
        let mut file_ids = Vec::with_capacity(file_count);
        for _ in 0..file_count {
            let mut fid = [0u8; 16];
            fid.copy_from_slice(&bytes[offset..offset + 16]);
            file_ids.push(fid);
            offset += 16;
        }

        let created_tx = u64::from_be_bytes(bytes[offset..offset + 8].try_into().unwrap_or([0; 8]));

        Ok(Self {
            copy_id,
            kind,
            file_ids,
            created_tx,
        })
    }
}

/// Trait definition for copy registry implementations.
pub trait CpyCopyRegistry: Send + Sync {
    fn register<'a>(&'a mut self, entry: CopyRegEntry) -> BoxFuture<'a, Result<()>>;
    fn remove<'a>(&'a mut self, copy_id: &'a [u8; 16]) -> BoxFuture<'a, Result<()>>;
    fn list(&self) -> Vec<CopyRegEntry>;
    fn holds_files(&self, file_ids: &[[u8; 16]]) -> Vec<[u8; 16]>;

    /// Queries the status of a registered copy.
    /// Default implementation returns [`CopyStatus::Unknown`] if not overridden.
    fn status<'a>(&'a self, _copy_id: &'a [u8; 16]) -> CopyStatus {
        CopyStatus::Unknown
    }

    /// Updates the status of a registered copy.
    /// Default implementation is no-op returning `Ok(())`.
    fn update_status<'a>(
        &'a mut self,
        _copy_id: &'a [u8; 16],
        _status: CopyStatus,
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    /// Flushes / fsyncs copy registry entries to durable persistent storage.
    /// Default implementation returns `Ok(())`.
    fn sync_registry<'a>(&'a mut self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }
}

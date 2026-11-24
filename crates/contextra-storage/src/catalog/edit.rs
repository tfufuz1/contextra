//! Typed Catalog edit patterns for the E-67 purge lifecycle.

use super::ops::{CatalogOp, FileEntry};
use contextra_storage_ports::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PurgeBatchEdit { ticket: [u8; 16], remove_file_id: u64, replacement: Option<FileEntry> }

impl PurgeBatchEdit {
    pub fn replace(ticket: [u8; 16], old_file_id: u64, replacement: FileEntry) -> Result<Self, Error> {
        if old_file_id == replacement.file_id { return Err(Error::Limit { limit: "purge replacement must use a new file id" }); }
        Ok(Self { ticket, remove_file_id: old_file_id, replacement: Some(replacement) })
    }
    pub fn remove(ticket: [u8; 16], file_id: u64) -> Self {
        Self { ticket, remove_file_id: file_id, replacement: None }
    }
    pub fn ticket(&self) -> &[u8; 16] { &self.ticket }
    pub fn into_op(self) -> CatalogOp {
        CatalogOp::PurgeBatch { ticket: self.ticket, remove_file_id: self.remove_file_id, replacement: self.replacement }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnlinkAckEdit { ticket: [u8; 16], file_id: u64 }

impl UnlinkAckEdit {
    /// Construct only after `unlink` and the containing `sync_dir` succeeded.
    pub fn after_durable_unlink(ticket: [u8; 16], file_id: u64) -> Self { Self { ticket, file_id } }
    pub fn into_op(self) -> CatalogOp { CatalogOp::UnlinkAck { ticket: self.ticket, file_id: self.file_id } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PurgeCompleteEdit { ticket: [u8; 16] }

impl PurgeCompleteEdit {
    /// The state fold additionally rejects completion while any source file
    /// remains in the ticket's persistent graveyard.
    pub fn after_owner_attestations(ticket: [u8; 16]) -> Self { Self { ticket } }
    pub fn into_op(self) -> CatalogOp { CatalogOp::PurgeComplete { ticket: self.ticket } }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogEdit {
    PurgeBatch(PurgeBatchEdit),
    UnlinkAck(UnlinkAckEdit),
    PurgeComplete(PurgeCompleteEdit),
}

impl CatalogEdit {
    pub fn into_op(self) -> CatalogOp {
        match self {
            Self::PurgeBatch(edit) => edit.into_op(),
            Self::UnlinkAck(edit) => edit.into_op(),
            Self::PurgeComplete(edit) => edit.into_op(),
        }
    }
}

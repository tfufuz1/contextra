//! Reconstructed live-reference index for immutable blob incarnations.
//!
//! This is a candidate generator only. Callers must revalidate candidates
//! against the durable catalog immediately before removing a file.

use contextra_storage_ports::Error;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlobId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlobIncarnation(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlobRef {
    pub blob_id: BlobId,
    pub incarnation: BlobIncarnation,
    pub offset: u64,
    pub len: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct FileIdentity(BlobId, BlobIncarnation);

/// Tracks references discovered in live Runs. Rebuild this index from Runs
/// after recovery; it is deliberately not an authority for deletion.
#[derive(Debug, Default, Clone)]
pub struct BlobGcIndex {
    live: BTreeMap<FileIdentity, u64>,
}

impl BlobGcIndex {
    pub fn new() -> Self { Self::default() }

    pub fn add_reference(&mut self, reference: BlobRef) -> Result<(), Error> {
        if reference.len == 0 || reference.offset.checked_add(reference.len).is_none() {
            return Err(Error::Corrupt { place: "blob reference bounds" });
        }
        let id = FileIdentity(reference.blob_id, reference.incarnation);
        let count = self.live.entry(id).or_insert(0);
        *count = count.checked_add(1).ok_or(Error::Limit { limit: "blob live reference count" })?;
        Ok(())
    }

    pub fn remove_reference(&mut self, reference: BlobRef) -> Result<(), Error> {
        let id = FileIdentity(reference.blob_id, reference.incarnation);
        let count = self.live.get_mut(&id).ok_or(Error::Corrupt { place: "unknown live blob reference" })?;
        *count = count.checked_sub(1).ok_or(Error::Corrupt { place: "blob reference count underflow" })?;
        if *count == 0 { self.live.remove(&id); }
        Ok(())
    }

    /// Lists file incarnations with no references in the currently indexed
    /// live Runs. It does not unlink them or assert that they are purgeable.
    pub fn remove_candidates(&self, known_files: impl IntoIterator<Item = (BlobId, BlobIncarnation)>) -> Vec<(BlobId, BlobIncarnation)> {
        known_files.into_iter()
            .filter(|(blob, incarnation)| !self.live.contains_key(&FileIdentity(*blob, *incarnation)))
            .collect()
    }

    pub fn live_reference_count(&self, blob: BlobId, incarnation: BlobIncarnation) -> u64 {
        self.live.get(&FileIdentity(blob, incarnation)).copied().unwrap_or(0)
    }
}

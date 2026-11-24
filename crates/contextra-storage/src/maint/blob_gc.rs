//! Blob GC orchestration using BlobGcIndex and live run references (SPEC §5.7, SP-MAINT, INV-BLOB-1).
#![forbid(unsafe_code)]

use crate::store::blob::gc_index::{BlobGcIndex, BlobId, BlobIncarnation};
use contextra_storage_ports::traits::{FileRole, Fs, IoCtx};
use contextra_storage_ports::Error;
use std::path::{Path, PathBuf};

fn blobgc_ctx(purpose: &'static str) -> IoCtx {
    IoCtx {
        role: FileRole::Blob,
        purpose,
    }
}

pub struct BlobGcJob {
    pub unreferenced_files: Vec<(BlobId, BlobIncarnation)>,
}

pub struct BlobGcRunner;

impl BlobGcRunner {
    /// Identifies unreferenced blob candidate files ready for deletion from the index.
    pub fn plan_gc(
        gc_index: &BlobGcIndex,
        known_files: impl IntoIterator<Item = (BlobId, BlobIncarnation)>,
    ) -> BlobGcJob {
        let unreferenced = gc_index.remove_candidates(known_files);
        BlobGcJob {
            unreferenced_files: unreferenced,
        }
    }

    /// Safely unlinks fully dead blob files and syncs the parent directory.
    pub fn execute_unlinks(
        fs: &dyn Fs,
        blobs_dir: &Path,
        job: &BlobGcJob,
    ) -> Result<Vec<PathBuf>, Error> {
        let mut unlinked = Vec::new();
        for (blob_id, inc) in &job.unreferenced_files {
            let path = blobs_dir.join(format!("{:016x}_{:016x}.blob", blob_id.0, inc.0));
            match fs.unlink(&path, blobgc_ctx("blob_unlink_dead")) {
                Ok(()) => unlinked.push(path),
                Err(Error::Io { kind: std::io::ErrorKind::NotFound, .. }) => {}
                Err(e) => return Err(e),
            }
        }
        if !unlinked.is_empty() {
            fs.sync_dir(blobs_dir, blobgc_ctx("blob_sync_dir"))?;
        }
        Ok(unlinked)
    }
}

//! Blob GC orchestration using BlobGcIndex and live run references (SPEC §5.7, SP-MAINT, INV-BLOB-1).
#![forbid(unsafe_code)]

use crate::store::blob::gc_index::BlobGcIndex;
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
    pub unreferenced_file_ids: Vec<u64>,
}

pub struct BlobGcRunner;

impl BlobGcRunner {
    /// Identifies unreferenced blob candidate files ready for deletion from the index.
    pub fn plan_gc(gc_index: &BlobGcIndex) -> BlobGcJob {
        let unreferenced = gc_index.ready_to_remove();
        BlobGcJob {
            unreferenced_file_ids: unreferenced,
        }
    }

    /// Safely unlinks fully dead blob files and syncs the parent directory.
    pub fn execute_unlinks(
        fs: &dyn Fs,
        blobs_dir: &Path,
        job: &BlobGcJob,
    ) -> Result<Vec<PathBuf>, Error> {
        let mut unlinked = Vec::new();
        for file_id in &job.unreferenced_file_ids {
            let path = blobs_dir.join(format!("{:016x}.blob", file_id));
            if fs.exists(&path)? {
                fs.unlink(&path, blobgc_ctx("blob_unlink_dead"))?;
                unlinked.push(path);
            }
        }
        if !unlinked.is_empty() {
            fs.sync_dir(blobs_dir, blobgc_ctx("blob_sync_dir"))?;
        }
        Ok(unlinked)
    }
}

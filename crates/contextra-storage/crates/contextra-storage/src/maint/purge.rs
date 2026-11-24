//! Physical file purge and section relocation (SPEC §9.5, SP-PURGE, INV-PL-2/4, INV-ERA-11).
#![forbid(unsafe_code)]

use contextra_storage_ports::traits::{FileRole, Fs, IoCtx};
use contextra_storage_ports::{ClusterId, Error};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn purge_ctx(purpose: &'static str) -> IoCtx {
    IoCtx {
        role: FileRole::Run,
        purpose,
    }
}

pub struct PurgePlan {
    pub purged_clusters: BTreeSet<ClusterId>,
    pub dead_files: Vec<u64>,
    pub survivor_files: Vec<u64>,
}

pub struct PurgeRunner;

impl PurgeRunner {
    /// Classifies run files affected by purged clusters into completely dead files
    /// (ready for direct unlink) and survivor files (requiring section relocation).
    pub fn plan_purge(
        purged: &[ClusterId],
        file_clusters: &[(u64, Vec<ClusterId>)],
    ) -> PurgePlan {
        let purged_set: BTreeSet<ClusterId> = purged.iter().copied().collect();
        let mut dead_files = Vec::new();
        let mut survivor_files = Vec::new();

        for (file_id, clusters) in file_clusters {
            let total = clusters.len();
            let purged_count = clusters.iter().filter(|c| purged_set.contains(c)).count();
            if purged_count == 0 {
                continue;
            }
            if purged_count == total {
                dead_files.push(*file_id);
            } else {
                survivor_files.push(*file_id);
            }
        }

        PurgePlan {
            purged_clusters: purged_set,
            dead_files,
            survivor_files,
        }
    }

    /// Directly unlinks completely dead files after reader drain and syncs the directory.
    pub fn unlink_dead_files(
        fs: &dyn Fs,
        runs_dir: &Path,
        dead_files: &[u64],
    ) -> Result<Vec<PathBuf>, Error> {
        let mut unlinked = Vec::new();
        for file_id in dead_files {
            let path = runs_dir.join(format!("{:016x}.run", file_id));
            match fs.unlink(&path, purge_ctx("purge_unlink_dead_run")) {
                Ok(()) => unlinked.push(path),
                Err(Error::Io { kind: std::io::ErrorKind::NotFound, .. }) => {}
                Err(e) => return Err(e),
            }
        }
        if !unlinked.is_empty() {
            fs.sync_dir(runs_dir, purge_ctx("purge_sync_dir"))?;
        }
        Ok(unlinked)
    }
}

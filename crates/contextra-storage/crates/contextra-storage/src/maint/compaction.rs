//! Run compaction across maintenance groups (SPEC §5, §16, SP-MAINT, INV-PL-1..5).
#![forbid(unsafe_code)]

use contextra_storage_ports::traits::{FileRole, Fs, IoCtx};
use contextra_storage_ports::{Error, Stamp};
use std::path::{Path, PathBuf};

fn compact_ctx(purpose: &'static str) -> IoCtx {
    IoCtx {
        role: FileRole::Run,
        purpose,
    }
}

#[derive(Debug, Clone)]
pub struct CompactionPlan {
    pub input_file_ids: Vec<u64>,
    pub output_file_id: u64,
    pub floor_stamp: Stamp,
}

pub struct Compactor;

impl Compactor {
    /// Installs a compacted output run file following the atomic two-phase write pattern.
    pub fn install_compacted_run(
        fs: &dyn Fs,
        runs_dir: &Path,
        output_file_id: u64,
        compacted_bytes: &[u8],
    ) -> Result<PathBuf, Error> {
        let temp_name = format!(".building.{:016x}", output_file_id);
        let final_name = format!("{:016x}.run", output_file_id);
        let temp_path = runs_dir.join(&temp_name);
        let final_path = runs_dir.join(&final_name);

        let file = fs.create_new(&temp_path, compact_ctx("compact_create_temp"))?;
        if let Err(e) = file
            .preallocate(compacted_bytes.len() as u64, compact_ctx("compact_prealloc"))
            .and_then(|_| file.append_prealloc(compacted_bytes, compact_ctx("compact_write_data")))
            .and_then(|_| file.sync_all(compact_ctx("compact_sync_file")))
        {
            let _ = fs.unlink(&temp_path, compact_ctx("compact_cleanup_temp"));
            return Err(e);
        }
        drop(file);

        if let Err(e) = fs.rename(&temp_path, &final_path, compact_ctx("compact_rename")) {
            let _ = fs.unlink(&temp_path, compact_ctx("compact_cleanup_temp"));
            return Err(e);
        }

        if let Err(e) = fs.sync_dir(runs_dir, compact_ctx("compact_sync_dir")) {
            return Err(e);
        }

        Ok(final_path)
    }
}

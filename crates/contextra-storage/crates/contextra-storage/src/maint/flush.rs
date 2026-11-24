//! Memtable to Run flush orchestration (SPEC §5, §16, SP-MAINT, INV-CAT-1).
#![forbid(unsafe_code)]

use contextra_storage_ports::traits::{FileRole, Fs, IoCtx};
use contextra_storage_ports::{Error, Stamp};
use std::path::{Path, PathBuf};

fn flush_ctx(purpose: &'static str) -> IoCtx {
    IoCtx {
        role: FileRole::Run,
        purpose,
    }
}

pub struct FlushJob {
    pub file_id: u64,
    pub min_stamp: Stamp,
    pub max_stamp: Stamp,
    pub raw_data: Vec<u8>,
}

pub struct FlushRunner;

impl FlushRunner {
    /// Executes durable file installation protocol:
    /// 1. Write to `.building.<id>`
    /// 2. `sync_data` + `sync_all` on the temporary file
    /// 3. Atomic rename to target run filename `<id>.run`
    /// 4. `sync_dir` on parent directory
    pub fn install_run(
        fs: &dyn Fs,
        runs_dir: &Path,
        job: &FlushJob,
    ) -> Result<PathBuf, Error> {
        let temp_name = format!(".building.{:016x}", job.file_id);
        let final_name = format!("{:016x}.run", job.file_id);
        let temp_path = runs_dir.join(&temp_name);
        let final_path = runs_dir.join(&final_name);

        let file = fs.create_new(&temp_path, flush_ctx("flush_create_temp"))?;
        if let Err(e) = file
            .preallocate(job.raw_data.len() as u64, flush_ctx("flush_prealloc"))
            .and_then(|_| file.append_prealloc(&job.raw_data, flush_ctx("flush_write_data")))
            .and_then(|_| file.sync_all(flush_ctx("flush_sync_file")))
        {
            let _ = fs.unlink(&temp_path, flush_ctx("flush_cleanup_temp"));
            return Err(e);
        }
        drop(file);

        if let Err(e) = fs.rename(&temp_path, &final_path, flush_ctx("flush_rename")) {
            let _ = fs.unlink(&temp_path, flush_ctx("flush_cleanup_temp"));
            return Err(e);
        }

        if let Err(e) = fs.sync_dir(runs_dir, flush_ctx("flush_sync_dir")) {
            return Err(e);
        }

        Ok(final_path)
    }
}

//! Startsequenz nach SPEC §14.1 (11 Phasen).

use std::path::PathBuf;
use std::sync::Arc;
use contextra_ports::{Clock, Rng, Vfs};
use contextra_types::Result;
use crate::hardening::apply_process_hardening;

/// Führt die Startsequenz durch.
pub async fn execute_startup_sequence(
    data_dir: PathBuf,
    vfs: Arc<dyn Vfs>,
    _clock: Arc<dyn Clock>,
    _rng: Arc<dyn Rng>,
) -> Result<()> {
    // Phase 1: Lock
    let _lock_path = data_dir.join("LOCK");
    if !vfs.exists(&data_dir)? {
        vfs.create_dir_all(&data_dir)?;
    }
    // Phase 4: Prozesshärtung
    apply_process_hardening()?;
    // Phase 5: tmp/ cleanup
    let tmp_dir = data_dir.join("tmp");
    if vfs.exists(&tmp_dir)? {
        let entries = vfs.read_dir(&tmp_dir)?;
        for entry in entries {
            let _ = vfs.remove_file(&entry);
        }
    }
    Ok(())
}

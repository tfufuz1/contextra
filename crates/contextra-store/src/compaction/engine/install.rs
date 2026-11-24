//! # SSTable Installation & Cleanup (Ring 1)
//!
//! Pfadgenerierung und Installation neu erstellter SSTables.

use crate::error::StoreResult as Result;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Generiert einen eindeutigen Pfad für eine neue SSTable-Datei unter Verwendung monotoner Zähler.
pub fn generate_sst_path(
    data_path: &Path,
    global_counter: &AtomicU64,
    instance_counter: &AtomicU64,
) -> Result<PathBuf> {
    let seq = global_counter.fetch_add(1, Ordering::Relaxed);
    let count = instance_counter.fetch_add(1, Ordering::Relaxed);
    Ok(data_path.join(format!("sst-compact-{:020}-{:04}.sst", seq, count % 10000)))
}

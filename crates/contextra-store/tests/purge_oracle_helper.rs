// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Orakel-Hilfsfunktion für Purge-Compaction Testverifikation.

use contextra_store::sstable::witness_scan::scan_sstables;
use contextra_store::sstable::witness_types::{SstScanInputs, SstScopeSpec};
use std::path::Path;

pub fn verify_purge_oracle(dir_path: &Path, scope: &SstScopeSpec, as_of_tx: u64) {
    let mut live_files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "sst" || ext == "sstable") {
                live_files.push(path);
            }
        }
    }

    let inputs = SstScanInputs {
        data_dirs: vec![dir_path.to_path_buf()],
        live_files,
        pending_delete_files: Vec::new(),
        manifest_generation: 1,
        as_of_tx,
        depth: 0,
    };

    let evidence = scan_sstables(scope, &inputs, None, || 1)
        .expect("Witness scan oracle failed to execute");

    assert_eq!(
        evidence.versions_found, 0,
        "Oracle check failed: versions_found == {} (expected 0)",
        evidence.versions_found
    );
    assert_eq!(
        evidence.tombstones_remaining, 0,
        "Oracle check failed: tombstones_remaining == {} (expected 0)",
        evidence.tombstones_remaining
    );
}

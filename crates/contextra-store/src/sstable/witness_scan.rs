// FILE-CONTEXT
// STAND: 2026-10-08 (T-2026-9104)
// ZWECK: Multi-Step Witness Scanner für SSTables und Evidenzerstellung (C4.2).
// INVARIANTEN: 7-Schritt-Scan; Pinning & Generatons-Callback; Fail-closed; Evidenz V1 Contract.

use super::witness_inventory::SstInventory;
use super::witness_reader::WitnessReader;
use super::witness_types::{SstScanEvidence, SstScanInputs, SstScopeSpec, WitnessReadError};
use crate::wal::KeyManager;
use std::fs;
use std::sync::Arc;

/// Executes the 7-step witness scan pipeline and generates an SstScanEvidence report.
pub fn scan_sstables<F>(
    scope: &SstScopeSpec,
    inputs: &SstScanInputs,
    key_manager: Option<Arc<KeyManager>>,
    generation_callback: F,
) -> Result<SstScanEvidence, WitnessReadError>
where
    F: Fn() -> u64,
{
    let mut evidence = SstScanEvidence {
        manifest_gen: inputs.manifest_generation,
        files_in_manifest: inputs.live_files.len() as u64,
        ..Default::default()
    };

    // Step 1: Inventory
    let inventory = SstInventory::build(inputs)?;
    evidence.orphan_files_found = inventory.orphan.len() as u64;
    evidence.unclassified_files = inventory.unknown.len() as u64;

    // Collect all candidate data files to scan: Live, ObsoletePending, Orphan, Temp
    let mut candidate_files = Vec::new();
    candidate_files.extend(inventory.live.clone());
    candidate_files.extend(inventory.obsolete_pending.clone());
    candidate_files.extend(inventory.orphan.clone());
    candidate_files.extend(inventory.temp.clone());

    // Step 2 & 3 & 4 & 5 & 6: File validation, Pruning, Sweep, Index Separators, Tombstones
    for file_path in &candidate_files {
        if !file_path.exists() {
            continue;
        }

        let mut reader = match WitnessReader::open(file_path, key_manager.clone()) {
            Ok(r) => r,
            Err(WitnessReadError::NotWitnessable(msg)) => {
                return Err(WitnessReadError::NotWitnessable(msg));
            }
            Err(_) => {
                // Unparseable SSTable candidate file -> unclassified
                evidence.unclassified_files += 1;
                continue;
            }
        };

        // Complete first_key read for metadata AP-08
        let _ = reader.read_first_key();
        let metadata = reader.metadata();

        // Step 3: Range Pruning
        if !key_range_overlaps_scope(&metadata.first_key, &metadata.last_key, scope) {
            evidence.files_pruned_with_verified_range += 1;
            continue;
        }

        // Step 4: Sweep over non-pruned file
        evidence.files_scanned += 1;
        evidence.blocks_scanned += reader.index().len() as u64;
        evidence.bytes_scanned += reader.file_size();

        // Step 5: Index separators and fence keys scan
        for (index_key, _) in reader.index() {
            if key_matches_scope(index_key.as_ref(), scope) {
                evidence.identifier_hits_index += 1;
            }
        }

        // Read all entries in block
        let entries = match reader.read_all_entries() {
            Ok(e) => e,
            Err(WitnessReadError::NotWitnessable(msg)) => {
                return Err(WitnessReadError::NotWitnessable(msg));
            }
            Err(_) => {
                evidence.unclassified_files += 1;
                continue;
            }
        };

        for entry in &entries {
            if entry.raw_seq() <= inputs.as_of_tx && key_matches_scope(entry.key.as_ref(), scope) {
                if entry.is_tombstone {
                    evidence.tombstones_remaining += 1;
                } else {
                    evidence.versions_found += 1;
                }
            }
        }
    }

    // Step 7: Historical Manifest files raw byte prefix search
    for manifest_path in &inventory.old_manifest {
        evidence.manifests_scanned += 1;
        let content = fs::read(manifest_path).map_err(|e| {
            WitnessReadError::Io(format!(
                "Failed to read historical manifest file {}: {}",
                manifest_path.display(),
                e
            ))
        })?;

        for prefix in &scope.prefixes {
            if contains_subslice(&content, prefix) {
                evidence.range_hits += 1;
            }
        }
        for explicit_key in &scope.explicit {
            if contains_subslice(&content, explicit_key) {
                evidence.range_hits += 1;
            }
        }
    }

    // Generational Pin Check (up to 3 retries)
    let mut retry_count = 0;
    while retry_count < 3 {
        let current_gen = generation_callback();
        if current_gen == inputs.manifest_generation {
            break;
        }
        retry_count += 1;
        if retry_count >= 3 {
            return Err(WitnessReadError::ScanTimeout(
                "Manifest generation changed during scan across 3 validation retries".into(),
            ));
        }
    }

    Ok(evidence)
}

fn key_matches_scope(key: &[u8], scope: &SstScopeSpec) -> bool {
    for explicit_key in &scope.explicit {
        if key == explicit_key.as_slice() {
            return true;
        }
    }
    for prefix in &scope.prefixes {
        if key.starts_with(prefix) {
            return true;
        }
    }
    false
}

fn key_range_overlaps_scope(first_key: &[u8], last_key: &[u8], scope: &SstScopeSpec) -> bool {
    if scope.explicit.is_empty() && scope.prefixes.is_empty() {
        return true; // Empty scope means full scan
    }

    if first_key.is_empty() || last_key.is_empty() {
        return true; // Cannot prune without valid first/last key
    }

    for explicit in &scope.explicit {
        let exp_slice = explicit.as_slice();
        if exp_slice >= first_key && exp_slice <= last_key {
            return true;
        }
    }

    for prefix in &scope.prefixes {
        let p_slice = prefix.as_slice();
        if last_key >= p_slice && (first_key.starts_with(p_slice) || first_key <= p_slice) {
            return true;
        }
    }

    false
}

fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || haystack.len() < needle.len() {
        return false;
    }
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

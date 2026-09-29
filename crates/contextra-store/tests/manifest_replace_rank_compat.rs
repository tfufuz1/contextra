use contextra_core::Result;
use contextra_store::manifest::{Manifest, ManifestEntry};
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_manifest_replace_rank_backward_compatibility() -> Result<()> {
    let tmp = TempDir::new().expect("temp dir");
    let manifest_path = tmp.path().join("MANIFEST");

    let manifest = Manifest::open(&manifest_path).await?;

    let sst1 = PathBuf::from("sst-001.sst");
    let sst2 = PathBuf::from("sst-002.sst");
    let sst3_compacted = PathBuf::from("sst-compact-003.sst");

    // 1. Append Add entries
    manifest
        .append(&ManifestEntry::Add {
            path: sst1.clone(),
            max_tx: 10,
        })
        .await?;
    manifest
        .append(&ManifestEntry::Add {
            path: sst2.clone(),
            max_tx: 20,
        })
        .await?;

    // 2. Append legacy Replace entry with explicit non-zero rank (e.g. rank = 42)
    let legacy_replace = ManifestEntry::Replace {
        removed: vec![sst1.clone(), sst2.clone()],
        added: sst3_compacted.clone(),
        added_max_tx: 20,
        rank: 42,
    };
    manifest.append(&legacy_replace).await?;

    // 3. Load manifest back from disk and verify reconstruction
    let entries = Manifest::load(&manifest_path).await?;
    assert_eq!(entries.len(), 3);

    let valid = Manifest::reconstruct_valid_sstables(&entries);
    assert_eq!(valid.len(), 1);
    assert_eq!(valid[0].0, sst3_compacted);

    // 4. Append a new Replace entry (writes rank = 0 internally)
    let sst4 = PathBuf::from("sst-004.sst");
    let sst5_compacted = PathBuf::from("sst-compact-005.sst");

    manifest
        .append(&ManifestEntry::Add {
            path: sst4.clone(),
            max_tx: 30,
        })
        .await?;

    let new_replace = ManifestEntry::Replace {
        removed: vec![sst3_compacted.clone(), sst4.clone()],
        added: sst5_compacted.clone(),
        added_max_tx: 30,
        rank: 0,
    };
    manifest.append(&new_replace).await?;

    // 5. Reload and verify final set of valid SSTables
    let reloaded_entries = Manifest::load(&manifest_path).await?;
    let final_valid = Manifest::reconstruct_valid_sstables(&reloaded_entries);
    assert_eq!(final_valid.len(), 1);
    assert_eq!(final_valid[0].0, sst5_compacted);

    let dead = Manifest::reconstruct_dead_sstables(&reloaded_entries);
    assert!(dead.contains(&sst1));
    assert!(dead.contains(&sst2));
    assert!(dead.contains(&sst3_compacted));
    assert!(dead.contains(&sst4));
    assert!(!dead.contains(&sst5_compacted));

    Ok(())
}

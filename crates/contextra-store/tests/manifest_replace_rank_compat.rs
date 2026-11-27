use contextra_store::manifest::{Manifest, ManifestEntry};
use contextra_types::Result;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_manifest_replace_has_no_persisted_rank_field() -> Result<()> {
    let tmp = TempDir::new().expect("temp dir");
    let manifest_path = tmp.path().join("MANIFEST");

    let manifest = Manifest::open(&manifest_path).await?;

    let sst1 = PathBuf::from("sst-001.sst");
    let sst2 = PathBuf::from("sst-002.sst");
    let sst3_compacted = PathBuf::from("sst-compact-003.sst");

    // Legacy v1 payloads contain a reserved rank word. The reader keeps format compatibility
    // while the public Replace variant no longer exposes that obsolete field.
    let legacy_entry = ManifestEntry::Replace {
        removed: vec![sst1.clone()],
        added: sst3_compacted.clone(),
        added_max_tx: 20,
    }
    .to_bytes()?;
    let mut legacy_entry = legacy_entry;
    legacy_entry
        .get_mut(17..25)
        .ok_or_else(|| {
            contextra_types::ContextraError::Serialization("missing v1 rank field".into())
        })?
        .copy_from_slice(&42u64.to_le_bytes());
    let legacy_payload = legacy_entry.get(8..).ok_or_else(|| {
        contextra_types::ContextraError::Serialization("missing v1 payload".into())
    })?;
    let legacy_crc = crc32fast::hash(legacy_payload);
    legacy_entry
        .get_mut(4..8)
        .ok_or_else(|| contextra_types::ContextraError::Serialization("missing v1 CRC".into()))?
        .copy_from_slice(&legacy_crc.to_le_bytes());
    assert_eq!(
        ManifestEntry::from_bytes(legacy_entry.get(4..).ok_or_else(|| {
            contextra_types::ContextraError::Serialization("missing entry frame".into())
        })?)?,
        ManifestEntry::Replace {
            removed: vec![sst1.clone()],
            added: sst3_compacted.clone(),
            added_max_tx: 20,
        }
    );

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

    // 2. Append a Replace entry. The current model carries no persisted rank field.
    let legacy_replace = ManifestEntry::Replace {
        removed: vec![sst1.clone(), sst2.clone()],
        added: sst3_compacted.clone(),
        added_max_tx: 20,
    };
    manifest.append(&legacy_replace).await?;

    // 3. Load manifest back from disk and verify reconstruction
    let entries = Manifest::load(&manifest_path).await?;
    assert_eq!(entries.len(), 3);

    let valid = Manifest::reconstruct_valid_sstables(&entries);
    assert_eq!(valid.len(), 1);
    assert_eq!(valid[0].0, sst3_compacted);

    // 4. Append another Replace entry.
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

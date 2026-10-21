// FILE-CONTEXT
// STAND: 2026-10-20
// ZWECK: WP-5a Regressionstest für SSTable Swap Iteratorparitäts-Vertauschung (Befund S3 / Swap).
// INVARIANTE:
// Purge von zwei SSTables, wobei die ältere komplett leer wird und die neuere überlebende Schlüssel enthält,
// darf die Zuordnung der neuen Reader nicht vertauschen oder lebende Schlüssel verdecken/entfernen.

use contextra_ports::StorageEngine;
use contextra_store::compaction::purge_filter::{FreeSpaceProbe, PurgeScope};
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::LsmStorage;
use contextra_types::{Result, TxId};
use tempfile::tempdir;

struct TestProbe;
impl FreeSpaceProbe for TestProbe {
    fn available_bytes(&self) -> std::result::Result<Option<u64>, contextra_ports::ProbeError> {
        Ok(Some(10 * 1024 * 1024 * 1024))
    }
    fn free_space_bytes(&self) -> Option<u64> {
        Some(10 * 1024 * 1024 * 1024)
    }
}

#[tokio::test]
async fn test_purge_two_ssts_first_becomes_empty() -> Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config.clone()).await?;

    // SSTable 1: Enthält NUR key_to_purge
    let tx1 = TxId::try_new(10).unwrap();
    storage.put(tx1, b"key_to_purge", b"val_old").await?;
    storage.commit(tx1).await?;
    StorageEngine::flush(&storage).await?;

    // SSTable 2: Enthält key_to_purge (überschrieben) und key_preserved
    let tx2 = TxId::try_new(20).unwrap();
    storage.put(tx2, b"key_to_purge", b"val_new").await?;
    storage.put(tx2, b"key_preserved", b"val_preserved").await?;
    storage.commit(tx2).await?;
    StorageEngine::flush(&storage).await?;

    // Vor dem Purge: Beide Schlüssel lesbar
    assert!(storage
        .get_at_seq(b"key_to_purge", u64::MAX)
        .await?
        .is_some());
    assert_eq!(
        storage
            .get_at_seq(b"key_preserved", u64::MAX)
            .await?
            .as_deref(),
        Some(&b"val_preserved"[..])
    );

    // Purge ausführen: key_to_purge löschen
    // SSTable 1 wird dadurch komplett leer (0 verbleibende Keys -> Remove)
    // SSTable 2 behält key_preserved (-> Replace)
    let probe = TestProbe;
    let scope = PurgeScope::Keys(vec![b"key_to_purge".to_vec()]);
    let report = storage
        .purge_keys(scope, 100, Some(&probe))
        .await
        .map_err(|e| contextra_types::ContextraError::Storage(format!("Purge failed: {:?}", e)))?;

    assert_eq!(
        report.records_purged, 2,
        "Es müssen 2 Versionen von key_to_purge bereinigt worden sein"
    );
    assert_eq!(
        report.sstables_replaced, 2,
        "2 SSTables mussten angefasst werden"
    );

    // Nach dem Purge:
    // 1. key_to_purge ist restlos verschwunden
    let purged_val = storage.get_at_seq(b"key_to_purge", u64::MAX).await?;
    assert!(
        purged_val.is_none(),
        "key_to_purge darf nach Purge nicht mehr sichtbar sein"
    );

    // 2. key_preserved MUSS exakt seinen Wert behalten haben (darf nicht durch falsches Mapping gelöscht/verschoben sein)
    let preserved_val = storage.get_at_seq(b"key_preserved", u64::MAX).await?;
    assert_eq!(
        preserved_val.as_deref(),
        Some(&b"val_preserved"[..]),
        "key_preserved muss nach dem Purge intakt und korrekt lesbar bleiben"
    );

    Ok(())
}

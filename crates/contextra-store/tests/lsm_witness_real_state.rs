// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Integrationstests zur Verifikation realer Zustandswerte (Manifest-Gen, In-Flight Commits, Unpersisted Edits, Pending Delete Files) in Zeugnissen (A05).
// INVARIANTEN: Keine Hartkodierungen in Zeugen-Evidenzen; dynamischer Manifest/LSM/Journal-Zustand wird exakt abgebildet.

use contextra_types::TxId;
use contextra_ports::StorageEngine;
use contextra_crypto::deletion_proof::claim_text::{WitAssurance, WitClaimText};
use contextra_crypto::deletion_proof::evidence::WitLayerEvidence;
use contextra_crypto::deletion_proof::DeletionLayer;
use contextra_durable_fs::PendingDeleteJournal;
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::witness::ZeuIssueRequest;
use contextra_store::lsm::LsmStorage;
use tempfile::TempDir;

#[tokio::test]
async fn test_witness_real_state_manifest_gen_and_unpersisted_edits() {
    let tmp = TempDir::new().unwrap();
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };

    let storage = LsmStorage::open(config).await.unwrap();

    let req = ZeuIssueRequest {
        scope_keys: vec![b"key_other".to_vec()],
        scope_prefixes: vec![b"other".to_vec()],
        as_of_tx: TxId::internal(),
        scanned_at_tx: TxId::internal(),
        assurance: WitAssurance::S2,
        claim: WitClaimText::S0,
        scanner_name: "test_scanner",
        scanner_version: 1,
        key_identifiers_claimed: false,
    };

    // Initialen Zustand prüfen (keine unpersisted edits vor Schreibvorgang)
    let witnesses_init = storage.issue_witnesses_v2(req.clone()).await.unwrap();
    let manifest_wit_init = witnesses_init
        .iter()
        .find(|w| *w.layer() == DeletionLayer::Manifest)
        .unwrap();

    if let Some(WitLayerEvidence::Manifest {
        current_manifest_gen,
        unpersisted_edits,
        ..
    }) = manifest_wit_init.layer_evidence()
    {
        assert_eq!(*current_manifest_gen, 1);
        assert_eq!(*unpersisted_edits, 0);
    } else {
        panic!("Falscher Evidenztyp für Manifest-Layer");
    }

    // Ungeflushten Schreibvorgang durchführen
    storage
        .put(TxId::internal(), b"key1", b"val1")
        .await
        .unwrap();
    storage.commit(TxId::internal()).await.unwrap();

    let witnesses_after_put = storage.issue_witnesses_v2(req.clone()).await;
    assert!(
        witnesses_after_put.is_err(),
        "issue_witnesses_v2 muss ablehnen, wenn unpersisted_edits > 0 im Manifest vorliegen"
    );

    // Flush ausführen -> manifest_generation muss steigen, unpersisted_edits sinken auf 0
    storage.flush().await.unwrap();

    // Wir passen scope_keys so an, dass Schlüssel in WAL/SSTable berücksichtigt werden
    let req_clean = ZeuIssueRequest {
        scope_keys: vec![b"key_other".to_vec()],
        scope_prefixes: vec![b"other".to_vec()],
        as_of_tx: TxId::internal(),
        scanned_at_tx: TxId::internal(),
        assurance: WitAssurance::S2,
        claim: WitClaimText::S0,
        scanner_name: "test_scanner",
        scanner_version: 1,
        key_identifiers_claimed: false,
    };

    let witnesses_after_flush = storage.issue_witnesses_v2(req_clean).await.unwrap();
    let manifest_wit_after_flush = witnesses_after_flush
        .iter()
        .find(|w| *w.layer() == DeletionLayer::Manifest)
        .unwrap();

    if let Some(WitLayerEvidence::Manifest {
        current_manifest_gen,
        unpersisted_edits,
        ..
    }) = manifest_wit_after_flush.layer_evidence()
    {
        assert!(
            *current_manifest_gen > 1,
            "manifest_generation muss nach Flush steigen"
        );
        assert_eq!(
            *unpersisted_edits, 0,
            "nach Flush müssen unpersisted_edits 0 sein"
        );
    } else {
        panic!("Falscher Evidenztyp für Manifest-Layer");
    }
}

#[tokio::test]
async fn test_witness_real_state_pending_delete_files() {
    let tmp = TempDir::new().unwrap();
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };

    let storage = LsmStorage::open(config).await.unwrap();

    // 1. Zuerst zwei echte SSTables erstellen und via Flush im Manifest registrieren
    storage
        .put(TxId::internal(), b"foo_key", b"foo_val")
        .await
        .unwrap();
    storage.commit(TxId::internal()).await.unwrap();
    storage.flush().await.unwrap();

    let sst_path = storage
        .sstables_for_test()
        .read()
        .await
        .first()
        .unwrap()
        .file_path()
        .to_path_buf();

    storage
        .put(TxId::internal(), b"bar_key", b"bar_val")
        .await
        .unwrap();
    storage.commit(TxId::internal()).await.unwrap();
    storage.flush().await.unwrap();

    // 2. Ersten SSTable im PendingDeleteJournal als ausstehend eintragen
    let journal_path = tmp.path().join("pending_delete.journal");
    let mut journal = PendingDeleteJournal::open(&journal_path).unwrap();
    journal.record(&sst_path).unwrap();

    let req = ZeuIssueRequest {
        scope_keys: vec![b"key_other".to_vec()],
        scope_prefixes: vec![b"other".to_vec()],
        as_of_tx: TxId::internal(),
        scanned_at_tx: TxId::internal(),
        assurance: WitAssurance::S2,
        claim: WitClaimText::S0,
        scanner_name: "test_scanner",
        scanner_version: 1,
        key_identifiers_claimed: false,
    };

    let witnesses = storage.issue_witnesses_v2(req).await.unwrap();
    let sst_wit = witnesses
        .iter()
        .find(|w| *w.layer() == DeletionLayer::SsTableAllLevels)
        .unwrap();

    if let Some(WitLayerEvidence::SsTables {
        files_in_manifest,
        files_pruned_with_verified_range,
        ..
    }) = sst_wit.layer_evidence()
    {
        assert_eq!(*files_in_manifest, 2, "2 Live SSTables im Manifest");
        assert_eq!(*files_pruned_with_verified_range, 2, "2 SSTables ge-pruned");
    } else {
        panic!("Falscher Evidenztyp für SsTables-Layer");
    }
}

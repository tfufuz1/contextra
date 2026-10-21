use contextra_ports::{LayerScan, PhysicalScope, TextIndex};
use contextra_text::inverted::InvertedIndex;
use contextra_types::{DocId, TenantId, TxId};
use std::sync::Arc;

#[path = "../src/inverted/tests/mock.rs"]
mod mock;

use mock::MockStorage;

#[tokio::test]
async fn text_scanner_detects_planted_marker() -> contextra_types::Result<()> {
    let storage = Arc::new(MockStorage::new());
    let index = InvertedIndex::new(storage.clone(), "scan_ns");

    let tx = TxId::internal();
    let doc_planted = DocId::new(42);
    index.insert(tx, doc_planted, "apple banana cherry").await?;
    index.commit(tx).await?;

    let key_planted = format!("t:0:{}", doc_planted.inner()).into_bytes();
    let scope_planted = PhysicalScope::try_new(TenantId::SYSTEM, vec![key_planted], vec![]).unwrap();

    let obs = index.scan(&scope_planted).unwrap();
    assert_eq!(obs.layer_id, 20);
    assert!(!obs.clean, "Layer scan must detect planted marker in text index");
    assert_eq!(obs.item_count, 1);

    // Unrelated scope
    let key_other = format!("t:0:99999").into_bytes();
    let scope_other = PhysicalScope::try_new(TenantId::SYSTEM, vec![key_other], vec![]).unwrap();

    let obs_other = index.scan(&scope_other).unwrap();
    assert_eq!(obs_other.layer_id, 20);
    assert!(obs_other.clean, "Layer scan must be clean for unplanted scope in text index");
    assert_eq!(obs_other.item_count, 1);

    Ok(())
}

#[tokio::test]
async fn text_scanner_empty_layer_requires_declaration() -> contextra_types::Result<()> {
    let storage = Arc::new(MockStorage::new());
    let index = InvertedIndex::new(storage.clone(), "scan_empty_ns");

    let dummy_key = b"t:0:1".to_vec();
    let scope = PhysicalScope::try_new(TenantId::SYSTEM, vec![dummy_key], vec![]).unwrap();

    let obs = index.scan(&scope).unwrap();
    assert_eq!(obs.layer_id, 20);
    assert!(obs.clean, "Empty text index must declare clean = true");
    assert_eq!(obs.item_count, 0, "Empty text index must explicitly declare item_count = 0");

    Ok(())
}

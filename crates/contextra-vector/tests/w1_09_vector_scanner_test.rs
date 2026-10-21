use contextra_ports::{LayerScan, PhysicalScope, VectorIndex};
use contextra_types::{DocId, TenantId, TxId};
use contextra_vector::hnsw::{HnswConfigBuilder, HnswIndex};

#[tokio::test]
async fn vector_scanner_detects_planted_marker() -> contextra_types::Result<()> {
    let config = HnswConfigBuilder::new(4).build().unwrap();
    let index = HnswIndex::try_new(config)?;

    let tx = TxId::internal();
    let doc_planted = DocId::new(42);
    index.insert(tx, doc_planted, &[1.0, 0.0, 0.0, 0.0]).await?;
    index.commit(tx).await?;

    let key_planted = format!("t:1:{}", doc_planted.inner()).into_bytes();
    let scope_planted = PhysicalScope::try_new(TenantId::new(1), vec![key_planted], vec![]).unwrap();

    let obs = index.scan(&scope_planted).unwrap();
    assert_eq!(obs.layer_id, 3);
    assert!(!obs.clean, "Layer scan must detect planted marker (clean = false)");
    assert_eq!(obs.item_count, 1);

    // Unrelated scope
    let key_other = format!("t:1:99999").into_bytes();
    let scope_other = PhysicalScope::try_new(TenantId::new(1), vec![key_other], vec![]).unwrap();

    let obs_other = index.scan(&scope_other).unwrap();
    assert_eq!(obs_other.layer_id, 3);
    assert!(obs_other.clean, "Layer scan must be clean for unplanted scope");
    assert_eq!(obs_other.item_count, 1);

    Ok(())
}

#[tokio::test]
async fn scanner_empty_layer_requires_declaration() -> contextra_types::Result<()> {
    let config = HnswConfigBuilder::new(4).build().unwrap();
    let index = HnswIndex::try_new(config)?;

    let dummy_key = b"t:1:1".to_vec();
    let scope = PhysicalScope::try_new(TenantId::new(1), vec![dummy_key], vec![]).unwrap();

    let obs = index.scan(&scope).unwrap();
    assert_eq!(obs.layer_id, 3);
    assert!(obs.clean, "Empty index must declare clean = true");
    assert_eq!(obs.item_count, 0, "Empty index must explicitly declare item_count = 0");

    Ok(())
}

//! Core Smoke Test for contextra-store (Gate G-A)

use contextra_store::lsm::{LsmConfig, LsmStorage};

#[tokio::test]
async fn test_core_smoke() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("new storage");
    let seq = storage.last_applied_seq();
    assert_eq!(seq, 0);
}

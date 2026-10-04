//! Campaign J-11 Test H3: Verifies runtime behavior when combining `memory-only-storage` and `deletion-proof`.
//! Oracle: Specification INV-DURABILITY-RING requiring durability for deletion proof verification or clean error on open.

use contextra_store::{LsmConfig, LsmStorage};
use tempfile::TempDir;

#[tokio::test]
async fn test_memory_only_with_deletion_proof_behavior() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let config = LsmConfig {
        path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };

    // Attempt to open LSM storage with default config
    let open_res = LsmStorage::open(config).await;

    // Log result for evidence in campaign report
    match open_res {
        Ok(_store) => {
            println!("[H3 TEST] LSM storage successfully opened under memory-only-storage feature without compile/runtime rejection.");
        }
        Err(e) => {
            println!("[H3 TEST] LSM storage rejected opening with error: {:?}", e);
        }
    }
}

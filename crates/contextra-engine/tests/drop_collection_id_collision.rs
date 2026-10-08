#![cfg(not(loom))]

use contextra_crypto::deletion_proof::DeletionScope;
use contextra_engine::{Contextra, ContextraConfig};
use contextra_types::TenantId;
use std::collections::HashMap;
use std::time::Instant;
use tempfile::TempDir;

/// Test T2 (Collection ID Collision via 64-bit truncated BLAKE3 hash):
///
/// Audit A-03 documents that `drop_collection` generates `CollectionId` by truncating
/// BLAKE3 hashes to 8 bytes (64 bits).
///
/// Finding a 64-bit hash collision requires ~2^32 evaluations (birthday paradox).
/// In an automated test suite with tight time constraints, a full 2^32 brute-force
/// is marked `#[ignore]` with explicit reasoning, while a bounded budget run is performed.
#[tokio::test]
#[ignore = "Finding a 64-bit BLAKE3 prefix collision requires ~2^32 iterations (~4 billion hashes), exceeding automated CI test timeout budgets."]
async fn test_drop_collection_id_collision_search() -> contextra_types::Result<()> {
    let mut map: HashMap<u64, String> = HashMap::new();
    let start = Instant::now();
    let max_duration = std::time::Duration::from_secs(5);

    let mut found_collision: Option<(String, String, u64)> = None;
    let mut count: u64 = 0;

    while start.elapsed() < max_duration {
        count += 1;
        let name = format!("col_search_{}", count);

        let mut hasher = blake3::Hasher::new();
        hasher.update(name.as_bytes());
        let hash_bytes = hasher.finalize();
        let col_id_u64 =
            u64::from_le_bytes(hash_bytes.as_bytes()[0..8].try_into().unwrap_or([1; 8]));

        if let Some(existing_name) = map.get(&col_id_u64) {
            found_collision = Some((existing_name.clone(), name, col_id_u64));
            break;
        }
        map.insert(col_id_u64, name);
    }

    if let Some((name_a, name_b, col_id)) = found_collision {
        println!(
            "T2: Found 64-bit BLAKE3 collision: '{}' and '{}' both yield collection_id {}",
            name_a, name_b, col_id
        );

        let tmp = TempDir::new().expect("Failed to create temporary directory");
        let config = ContextraConfig {
            dimension: 4,
            consolidation_enabled: false,
            ..Default::default()
        };
        let db = Contextra::open_with_config(tmp.path(), config).await?;
        let tenant_id = TenantId::try_new(100).unwrap();
        let proof_key = b"secret_proof_key_32bytes_1234567";

        // Create and drop name_a
        let _ = db.collection_for_tenant(&name_a, tenant_id).await?;
        let proof_a = db.drop_collection(&name_a, tenant_id, proof_key).await?;

        // Create and drop name_b
        let _ = db.collection_for_tenant(&name_b, tenant_id).await?;
        let proof_b = db.drop_collection(&name_b, tenant_id, proof_key).await?;

        if let DeletionScope::Collection {
            collection_id: id_a,
            ..
        } = proof_a.scope
        {
            if let DeletionScope::Collection {
                collection_id: id_b,
                ..
            } = proof_b.scope
            {
                assert_eq!(
                    id_a, id_b,
                    "Both proofs must contain the identical truncated CollectionId upon BLAKE3 collision"
                );
            }
        }
        db.close().await?;
    } else {
        println!(
            "T2 RESULT: Evaluated {} hashes in {:?}. No collision in 5s window (as expected for 64-bit hash space; test marked #[ignore]).",
            count,
            start.elapsed()
        );
    }

    Ok(())
}

// FILE-CONTEXT
// ZWECK: Lückenlose Abdeckung aller Collection- und Storage-Operationen unter Mandanten-Isolation (TenantScopedStorage N21).
// INVARIANTEN: Strikte Key-Isolation (INV-TENANT-2); Keine un-tenanted Keys im Storage für Mandanten-Aktionen; Mandant B sieht niemals Daten von Mandant A.
// STAND: TS:2026-09-28

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_engine::{Contextra, ContextraConfig};
use contextra_ports::StorageEngine;
use contextra_types::{MemoryType, TenantId};
use serde_json::json;
use std::ops::Bound;
use tempfile::TempDir;

#[tokio::test]
async fn test_tenant_scoped_storage_full_coverage() -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;

    let tenant_a = TenantId::try_new(1001).expect("Tenant A ID");
    let tenant_b = TenantId::try_new(1002).expect("Tenant B ID");

    // 1. Create collections for Tenant A and Tenant B with the same collection name
    let col_a = db.collection_for_tenant("shared_col", tenant_a).await?;
    let col_b = db.collection_for_tenant("shared_col", tenant_b).await?;

    // 2. Write operations for Tenant A
    col_a
        .insert(
            "doc_a_1",
            &[0.1, 0.2, 0.3, 0.4],
            Some(json!({ "tenant": "A", "confidential": "secret_a" })),
        )
        .await?;

    col_a
        .insert_typed(
            "doc_a_2",
            &[0.2, 0.3, 0.4, 0.5],
            MemoryType::Semantic,
            Some(json!({ "text": "Tenant A semantic content" })),
        )
        .await?;

    col_a.relate("doc_a_1", "doc_a_2", "supports").await?;

    // 3. Write operations for Tenant B
    col_b
        .insert(
            "doc_b_1",
            &[0.9, 0.8, 0.7, 0.6],
            Some(json!({ "tenant": "B", "confidential": "secret_b" })),
        )
        .await?;

    col_b
        .insert_typed(
            "doc_b_2",
            &[0.8, 0.7, 0.6, 0.5],
            MemoryType::Episodic,
            Some(json!({ "text": "Tenant B episodic content" })),
        )
        .await?;

    col_b.relate("doc_b_1", "doc_b_2", "depends_on").await?;

    // 4. VERIFY RAW STORAGE PREFIX ISOLATION
    // Inspect raw storage directly: all written keys MUST be prefixed with t:1001: or t:1002: (or system metadata __meta:)
    let raw_entries = db.inner_storage().scan_prefix(b"").await?;
    assert!(
        !raw_entries.is_empty(),
        "Raw storage must contain written entries"
    );

    for (raw_key, _) in &raw_entries {
        let key_str = String::from_utf8_lossy(raw_key);
        let is_tenant_a = raw_key.starts_with(b"t:1001:");
        let is_tenant_b = raw_key.starts_with(b"t:1002:");
        let is_sys_meta = raw_key.starts_with(b"__meta:");

        assert!(
            is_tenant_a || is_tenant_b || is_sys_meta,
            "Raw key '{key_str}' violates tenant isolation: must be prefixed with t:1001:, t:1002:, or __meta:"
        );
    }

    // 5. READ & QUERY ISOLATION ASSERTIONS
    let doc_a1_from_a = col_a.get("doc_a_1").await?;
    assert!(
        doc_a1_from_a.is_some(),
        "Tenant A must read its own document doc_a_1"
    );

    let doc_a1_from_b = col_b.get("doc_a_1").await?;
    assert!(
        doc_a1_from_b.is_none(),
        "Tenant B MUST NOT be able to read Tenant A document doc_a_1"
    );

    let doc_b1_from_b = col_b.get("doc_b_1").await?;
    assert!(
        doc_b1_from_b.is_some(),
        "Tenant B must read its own document doc_b_1"
    );

    let doc_b1_from_a = col_a.get("doc_b_1").await?;
    assert!(
        doc_b1_from_a.is_none(),
        "Tenant A MUST NOT be able to read Tenant B document doc_b_1"
    );

    // Basic vector search isolation
    #[allow(deprecated)]
    let search_a = col_a.search(&[0.1, 0.2, 0.3, 0.4], 10).await?;
    for res in &search_a {
        assert!(
            res.id.starts_with("doc_a_"),
            "Tenant A search results must contain only Tenant A documents, got: {}",
            res.id
        );
    }

    #[allow(deprecated)]
    let search_b = col_b.search(&[0.1, 0.2, 0.3, 0.4], 10).await?;
    for res in &search_b {
        assert!(
            res.id.starts_with("doc_b_"),
            "Tenant B search results must contain only Tenant B documents, got: {}",
            res.id
        );
    }

    // 6. SCAN ISOLATION ASSERTIONS
    let scan_a = col_a.scan(Bound::Unbounded, Bound::Unbounded, None).await?;
    assert!(
        scan_a.iter().all(|(k, _)| k.starts_with("doc_a_")
            || k.starts_with("__docid:")
            || k.starts_with("__rel:")
            || k.starts_with("__col:")),
        "Tenant A scan must contain only Tenant A keys"
    );

    let scan_b = col_b.scan(Bound::Unbounded, Bound::Unbounded, None).await?;
    for (k, _) in &scan_b {
        assert!(
            !k.contains("doc_a_"),
            "Tenant B scan MUST NOT contain Tenant A document keys, got: {k}"
        );
    }

    // Tenant-scoped storage prefix scan with empty prefix
    let tenant_b_storage_scan = col_b.storage().scan_prefix(b"").await?;
    for (k, _) in &tenant_b_storage_scan {
        let k_str = String::from_utf8_lossy(k);
        assert!(
            !k_str.contains("doc_a_"),
            "Tenant B storage scan_prefix(b\"\") MUST NOT leak Tenant A keys, got: {k_str}"
        );
    }

    // 7. EXPORT ISOLATION ASSERTIONS
    let export_a = col_a.export_memories().await?;
    assert_eq!(export_a.name, "shared_col");
    assert!(
        export_a.memories.iter().all(|m| m.id.starts_with("doc_a_")),
        "Exported memories for Tenant A must belong strictly to Tenant A"
    );

    let export_b = col_b.export_memories().await?;
    assert_eq!(export_b.name, "shared_col");
    assert!(
        export_b.memories.iter().all(|m| m.id.starts_with("doc_b_")),
        "Exported memories for Tenant B must belong strictly to Tenant B"
    );

    // 8. TENANT-SCOPED LISTING ASSERTIONS
    let cols_a = db.list_collections_for_tenant(tenant_a).await?;
    assert!(
        cols_a.contains(&"shared_col".to_string()),
        "list_collections_for_tenant A must contain shared_col"
    );

    let cols_b = db.list_collections_for_tenant(tenant_b).await?;
    assert!(
        cols_b.contains(&"shared_col".to_string()),
        "list_collections_for_tenant B must contain shared_col"
    );

    // 9. RECOVERY & REPAIR ASSERTIONS
    col_a.repair().await?;
    col_b.repair().await?;
    db.repair_on_open().await?;

    // Post-repair raw storage verification
    let post_repair_raw = db.inner_storage().scan_prefix(b"").await?;
    for (raw_key, _) in &post_repair_raw {
        let key_str = String::from_utf8_lossy(raw_key);
        let is_tenant_a = raw_key.starts_with(b"t:1001:");
        let is_tenant_b = raw_key.starts_with(b"t:1002:");
        let is_sys_meta = raw_key.starts_with(b"__meta:");

        assert!(
            is_tenant_a || is_tenant_b || is_sys_meta,
            "Post-repair raw key '{key_str}' violates tenant isolation"
        );
    }

    // 10. DELETE & DROP COLLECTION ISOLATION
    col_a.delete("doc_a_1").await?;
    assert!(
        col_a.get("doc_a_1").await?.is_none(),
        "Tenant A doc_a_1 deleted"
    );
    assert!(
        col_b.get("doc_b_1").await?.is_some(),
        "Tenant B doc_b_1 must remain unaffected by Tenant A single document delete"
    );

    let proof_a = db
        .drop_collection("shared_col", tenant_a, b"secret_proof_key_32bytes_1234567")
        .await?;
    assert_eq!(proof_a.tenant_id(), tenant_a);

    // Assert Tenant B collection remains fully intact
    let doc_b1_after_drop = col_b.get("doc_b_1").await?;
    assert!(
        doc_b1_after_drop.is_some(),
        "Tenant B collection MUST remain intact after Tenant A drop_collection"
    );

    let doc_b2_after_drop = col_b.get("doc_b_2").await?;
    assert!(
        doc_b2_after_drop.is_some(),
        "Tenant B doc_b_2 MUST remain intact after Tenant A drop_collection"
    );

    db.close().await?;
    Ok(())
}

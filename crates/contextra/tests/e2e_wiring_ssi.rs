#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra::{open_with_config, ContextraConfig};
use contextra_core::{ContextraError, TenantId};
use contextra_ports::clock::ManualClock;
use contextra_ports::rng::SeededRng;
use contextra_ports::Rng;
use serde_json::json;

#[tokio::test]
async fn test_two_tenant_isolation_facade() -> Result<(), ContextraError> {
    let seed: u64 = 20261013_02;
    let rng = SeededRng::new(seed);
    let clock = ManualClock::new(1760390400_000_000_000);
    println!("e2e_wiring_ssi tenant seed: {}, clock_nanos: {}", seed, clock.now_nanos());

    let tmp_dir = std::env::temp_dir().join(format!("e2e_wiring_ssi_tenant_{}", rng.next_u64()));
    let _ = std::fs::remove_dir_all(&tmp_dir);

    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = open_with_config(&tmp_dir, config).await?;

    let tenant_a = TenantId::try_new(1001)?;
    let tenant_b = TenantId::try_new(1002)?;

    let col_a = db.collection_for_tenant("shared_catalog", tenant_a).await?;
    let col_b = db.collection_for_tenant("shared_catalog", tenant_b).await?;

    // Tenant A inserts doc1
    col_a
        .insert(
            "doc1",
            &[0.1, 0.2, 0.3, 0.4],
            Some(json!({"tenant": "A", "val": 100})),
        )
        .await?;

    // Tenant B cannot see Tenant A's doc1
    let doc_b = col_b.get("doc1").await?;
    assert!(
        doc_b.is_none(),
        "Tenant B must NOT see Tenant A's document"
    );

    // Tenant B inserts own doc1
    col_b
        .insert(
            "doc1",
            &[0.5, 0.6, 0.7, 0.8],
            Some(json!({"tenant": "B", "val": 200})),
        )
        .await?;

    // Both get their own doc1
    let res_a = col_a.get("doc1").await?.expect("Tenant A doc1");
    let res_b = col_b.get("doc1").await?.expect("Tenant B doc1");

    assert_eq!(
        res_a.metadata.as_ref().and_then(|m| m.get("tenant")),
        Some(&json!("A"))
    );
    assert_eq!(
        res_b.metadata.as_ref().and_then(|m| m.get("tenant")),
        Some(&json!("B"))
    );

    db.close().await?;
    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(())
}

#[tokio::test]
#[ignore = "blockiert durch PROMPT-FACADE-01: DB Transaction / SSI Write-Skew API is not re-exported or wired through Contextra primary facade"]
async fn test_write_skew_prevention_facade() -> Result<(), ContextraError> {
    let seed: u64 = 20261013_03;
    let rng = SeededRng::new(seed);
    let clock = ManualClock::new(1760390400_000_000_000);
    println!("e2e_wiring_ssi skew seed: {}, clock_nanos: {}", seed, clock.now_nanos());

    let tmp_dir = std::env::temp_dir().join(format!("e2e_wiring_ssi_skew_{}", rng.next_u64()));
    let _ = std::fs::remove_dir_all(&tmp_dir);

    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = open_with_config(&tmp_dir, config).await?;
    let col = db.collection("ssi_col").await?;

    // Initial setup: x = 100, y = 100
    col.insert("x", &[1.0, 0.0, 0.0, 0.0], Some(json!({"val": 100}))).await?;
    col.insert("y", &[0.0, 1.0, 0.0, 0.0], Some(json!({"val": 100}))).await?;

    // Simulation of transaction write-skew over facade:
    // Tx1 reads x and y, deducts 100 from x if x + y >= 100 -> writes x = 0
    // Tx2 reads x and y, deducts 100 from y if x + y >= 100 -> writes y = 0
    // Under SSI, one of them MUST abort with serialization failure / SSI conflict.
    // However, explicit `begin_transaction` or interactive transaction context
    // is not re-exported/wired on `Contextra` or `ContextraBuilder` facade.

    db.close().await?;
    let _ = std::fs::remove_dir_all(&tmp_dir);
    Ok(())
}

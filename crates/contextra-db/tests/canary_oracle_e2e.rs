#![allow(deprecated)]

//! End-to-End Integration Tests for Canary Oracle and Physical Deletion Sweep (LN-6.1).

use contextra_db::{Contextra, ContextraConfig};
use contextra_testkit::erasure_harness::{raw_sweep, CanarySet};
use contextra_types::{DistanceMetric, TenantId};
use serde_json::json;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

/// Oracle Self-Test / Negativkontrolle:
/// Confirms that `raw_sweep` correctly detects an intentionally injected canary (negative control against blind witness).
#[test]
fn test_oracle_self_test_negative_control() {
    let tmp = TempDir::new().expect("create temp dir");
    let canary_set = CanarySet::generate(0x1234_5678);

    let dummy_path = tmp.path().join("dummy_storage.dat");
    let mut f = File::create(&dummy_path).expect("create dummy file");
    f.write_all(b"HEADER_PADDING_BYTES_").expect("write header");
    f.write_all(canary_set.text.as_bytes())
        .expect("write canary bytes");
    f.write_all(b"_FOOTER_PADDING_BYTES").expect("write footer");
    f.flush().expect("flush dummy file");

    let report = raw_sweep(tmp.path(), canary_set.needles()).expect("raw_sweep");

    assert!(
        !report.is_clean(),
        "Oracle self-test failed: injected canary was not detected by raw_sweep!"
    );
    assert!(
        report.hit_count() >= 1,
        "Oracle self-test expected at least 1 hit, got {}",
        report.hit_count()
    );

    let hit_desc = canary_set.describe(&report.hits[0]);
    assert!(
        hit_desc.contains("text") || hit_desc.contains("CanaryHit"),
        "Hit description should contain valid canary information: {}",
        hit_desc
    );

    // Gegenprobe: Clean dir yields zero hits
    let clean_tmp = TempDir::new().expect("create clean temp dir");
    let clean_file = clean_tmp.path().join("clean.dat");
    std::fs::write(&clean_file, b"COMPLETELY_CLEAN_DATA_WITHOUT_CANARIES").expect("write clean");

    let clean_report = raw_sweep(clean_tmp.path(), canary_set.needles()).expect("raw_sweep clean");
    assert!(
        clean_report.is_clean(),
        "Oracle self-test failed: clean directory yielded false positive hits!"
    );
}

/// Current State Test (NOT IGNORED):
/// Documents the current storage state: after inserting a document containing canary bytes and flushing to disk,
/// `raw_sweep` MUST detect canary hits in the DB storage directory (SSTables / WAL).
#[tokio::test]
async fn test_canary_oracle_current_state_found_in_sst_after_flush() {
    let tmp = TempDir::new().expect("create temp dir");
    let config = ContextraConfig {
        dimension: 4,
        max_elements: 1000,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config)
        .await
        .expect("open db");

    let tenant_id = TenantId::try_new(1001).expect("tenant id");
    let canary_set = CanarySet::generate(0xCAFE_BABE);

    let col = db
        .collection_for_tenant("canary_col", tenant_id)
        .await
        .expect("collection");

    let payload = json!({
        &canary_set.meta_key: canary_set.meta_val,
        "text": canary_set.text,
        "relation": canary_set.relation,
    });

    col.insert(&canary_set.doc_id, &canary_set.vector, Some(payload))
        .await
        .expect("insert document with canaries");

    // Flush storage to persist data to disk SSTables / WAL
    db.inner_storage().flush().await.expect("flush storage");

    let report = raw_sweep(tmp.path(), canary_set.needles()).expect("raw_sweep on DB dir");

    assert!(
        !report.is_clean(),
        "CURRENT STATE TEST FAILED: Expected canary bytes to be present on disk after insert & flush, but raw_sweep was clean!"
    );

    assert!(
        report.hit_count() >= 1,
        "Expected at least 1 hit in DB directory after insert, found {}",
        report.hit_count()
    );
}

/// Target State Test (IGNORED):
/// Target specification for S0/S1/S2 complete physical erasure via public deletion API (`forget`).
/// Currently ignored as public deletion wiring is pending (LN-1.x).
#[tokio::test]
#[ignore = "LN-1.x: Facade-level public forget/PurgeDriver API is pending wiring on Contextra struct"]
async fn test_canary_oracle_target_state_s0_s1_s2_erased() {
    let tmp = TempDir::new().expect("create temp dir");
    let config = ContextraConfig {
        dimension: 4,
        max_elements: 1000,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config)
        .await
        .expect("open db");

    let tenant_id = TenantId::try_new(2002).expect("tenant id");
    let canary_set = CanarySet::generate(0xDEAD_BEEF);

    let col = db
        .collection_for_tenant("canary_col_purge", tenant_id)
        .await
        .expect("collection");

    let payload = json!({
        &canary_set.meta_key: canary_set.meta_val,
        "text": canary_set.text,
    });

    col.insert(&canary_set.doc_id, &canary_set.vector, Some(payload))
        .await
        .expect("insert doc");

    db.inner_storage().flush().await.expect("flush storage");

    // Verify canary present before deletion
    let report_before = raw_sweep(tmp.path(), canary_set.needles()).expect("raw_sweep before");
    assert!(!report_before.is_clean());

    // Public deletion call (to be wired in LN-1.x)
    // db.forget(&canary_set.doc_id, tenant_id).await.expect("forget");
    // db.inner_storage().flush().await.expect("flush post-forget");

    let report_after = raw_sweep(tmp.path(), canary_set.needles()).expect("raw_sweep after");
    assert!(
        report_after.is_clean(),
        "TARGET STATE TEST FAILED: Residual canary bytes found on disk after forget! Hits: {:?}",
        report_after.hits
    );
}

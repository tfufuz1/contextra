//! Integration tests verifying witness honesty, fail-closed semantics, and absence of fake/zeroed witness constants
//! in `Contextra::forget` and physical purge execution (§C5, INV-D1/D3).

#![forbid(unsafe_code)]

use std::fs;
use contextra_engine::deletion::ForgetOptions;
use contextra_types::TenantId;

#[test]
fn test_ratchet_no_fake_witness_constants_in_forget_pipeline() {
    let forget_code = fs::read_to_string("src/contextra_impl/forget.rs")
        .or_else(|_| fs::read_to_string("crates/contextra-engine/src/contextra_impl/forget.rs"))
        .expect("Failed to read forget.rs");

    let purge_driver_code = fs::read_to_string("src/deletion/purge_driver.rs")
        .or_else(|_| fs::read_to_string("crates/contextra-engine/src/deletion/purge_driver.rs"))
        .expect("Failed to read purge_driver.rs");

    // Rule 1: No hardcoded `marked_zeroed: true` claiming zeroed status without measurement
    assert!(
        !forget_code.contains("marked_zeroed: true"),
        "Ratchet violation in forget.rs: hardcoded 'marked_zeroed: true' found!"
    );
    assert!(
        !purge_driver_code.contains("marked_zeroed: true"),
        "Ratchet violation in purge_driver.rs: hardcoded 'marked_zeroed: true' found!"
    );

    // Rule 2: No hardcoded fake Memtable evidence pretending to be ErasureLedger (`entries_found: 0`)
    assert!(
        !forget_code.contains("entries_found: 0"),
        "Ratchet violation in forget.rs: fake Memtable evidence with 'entries_found: 0' found!"
    );

    // Rule 3: No `Some(0)` passed as witness measurement override in purge_driver or forget
    assert!(
        !purge_driver_code.contains("Some(0)"),
        "Ratchet violation in purge_driver.rs: fake 'Some(0)' witness override found!"
    );

    // Rule 4: No `[0u8; 16]` / `[0; 16]` zero instance IDs
    assert!(
        !forget_code.contains("[0u8; 16]") && !forget_code.contains("[0; 16]"),
        "Ratchet violation in forget.rs: hardcoded '[0; 16]' zero instance/proof ID found!"
    );
}

#[tokio::test]
async fn test_forget_fails_closed_when_layer_witness_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let db = contextra_engine::Contextra::open(tmp.path()).await.unwrap();

    let col = db.collection("default").await.unwrap();
    col.insert("doc-1", &[0.1f32; 768], None).await.unwrap();

    // Call forget with default options
    let opts = ForgetOptions::default();
    let receipt = db.forget("doc-1", opts).await.unwrap();

    // Verification:
    // 1. proof MUST be None because ErasureLedger scan is pending and not fake-verified
    assert!(
        receipt.proof.is_none(),
        "Proof must NOT be issued when any layer is Pending/Failed"
    );
    assert_eq!(
        receipt.assurance_reached,
        contextra_crypto::deletion_proof::claim_text::WitAssurance::S0,
        "Assurance reached must be S0 when proof is not issued"
    );
}

#[tokio::test]
async fn test_forget_tenant_scope_honesty() {
    let tmp = tempfile::tempdir().unwrap();
    let db = contextra_engine::Contextra::open(tmp.path()).await.unwrap();

    let tenant_b = TenantId::try_new(42).unwrap();
    let opts = ForgetOptions::default();

    let receipt = db.forget_tenant(tenant_b, opts).await.unwrap();

    match receipt.scope {
        contextra_crypto::deletion_proof::types::DeletionScope::Tenant { tenant_id } => {
            assert_eq!(tenant_id, tenant_b, "Scope must match target tenant_id");
        }
        _ => panic!("Expected Tenant deletion scope"),
    }
}

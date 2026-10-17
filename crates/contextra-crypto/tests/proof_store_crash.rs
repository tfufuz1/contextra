//! ProofStore crash resilience and recovery test suite covering all 5 crash failpoints.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::too_many_lines
)]

use contextra_crypto::deletion_proof::builder::ProofBuilder;
use contextra_crypto::deletion_proof::proof_store::{
    ProofStore, ProofStoreWarning, StoreCrashFailpoint,
};
use contextra_crypto::deletion_proof::signer::ProofKeyPairSigner;
use contextra_crypto::deletion_proof::types::DeletionScope;
use contextra_types::{DocId, TenantId};
use tempfile::tempdir;

fn helper_create_proof(
    signer: &ProofKeyPairSigner,
    seq: u64,
    prev_hash: [u8; 32],
) -> contextra_crypto::deletion_proof::proof_v4::ProofV4 {
    let scope = DeletionScope::Document {
        doc_id: DocId(100),
        tenant_id: TenantId::try_new(1).expect("tenant"),
    };

    ProofBuilder::new(scope, "db-crash-test")
        .with_deleted_after_tx(100)
        .with_deleted_keys_hash([0x11; 32])
        .with_chain_link(seq, prev_hash)
        .finish(signer)
        .expect("builder finish failed")
}

#[test]
fn test_crash_post_bin_write_yields_unchained_warning() {
    let dir = tempdir().expect("tempdir failed");
    let signer = ProofKeyPairSigner::from_secret_bytes([1u8; 32]);

    {
        let (mut store, _) = ProofStore::open(dir.path()).expect("open store");
        store.set_failpoint(Some(StoreCrashFailpoint::PostBinWrite));
        let proof = helper_create_proof(&signer, 1, [0u8; 32]);
        assert!(store.save_proof(&proof).is_err());
    }

    // Reopen store after simulated crash
    let (reopened, warnings) = ProofStore::open(dir.path()).expect("reopen store");
    assert_eq!(reopened.last_seq(), 0);
    assert!(
        warnings
            .iter()
            .any(|w| matches!(w, ProofStoreWarning::UnchainedProofFile { seq: 1, .. })),
        "Must detect unchained .bin proof file"
    );
}

#[test]
fn test_crash_post_json_write_yields_unchained_warning() {
    let dir = tempdir().expect("tempdir failed");
    let signer = ProofKeyPairSigner::from_secret_bytes([1u8; 32]);

    {
        let (mut store, _) = ProofStore::open(dir.path()).expect("open store");
        store.set_failpoint(Some(StoreCrashFailpoint::PostJsonWrite));
        let proof = helper_create_proof(&signer, 1, [0u8; 32]);
        assert!(store.save_proof(&proof).is_err());
    }

    let (reopened, warnings) = ProofStore::open(dir.path()).expect("reopen store");
    assert_eq!(reopened.last_seq(), 0);
    assert!(
        warnings
            .iter()
            .any(|w| matches!(w, ProofStoreWarning::UnchainedProofFile { seq: 1, .. })),
        "Must detect unchained proof files (.bin/.json)"
    );
}

#[test]
fn test_crash_post_dir_fsync_yields_unchained_warning() {
    let dir = tempdir().expect("tempdir failed");
    let signer = ProofKeyPairSigner::from_secret_bytes([1u8; 32]);

    {
        let (mut store, _) = ProofStore::open(dir.path()).expect("open store");
        store.set_failpoint(Some(StoreCrashFailpoint::PostDirFsync));
        let proof = helper_create_proof(&signer, 1, [0u8; 32]);
        assert!(store.save_proof(&proof).is_err());
    }

    let (reopened, warnings) = ProofStore::open(dir.path()).expect("reopen store");
    assert_eq!(reopened.last_seq(), 0);
    assert!(
        warnings
            .iter()
            .any(|w| matches!(w, ProofStoreWarning::UnchainedProofFile { seq: 1, .. })),
        "Must detect unchained proof files prior to chain log append"
    );
}

#[test]
fn test_crash_post_chain_log_append_recovered_successfully() {
    let dir = tempdir().expect("tempdir failed");
    let signer = ProofKeyPairSigner::from_secret_bytes([1u8; 32]);

    {
        let (mut store, _) = ProofStore::open(dir.path()).expect("open store");
        store.set_failpoint(Some(StoreCrashFailpoint::PostChainLogAppend));
        let proof = helper_create_proof(&signer, 1, [0u8; 32]);
        assert!(store.save_proof(&proof).is_err());
    }

    // Reopen store: line was appended to chain.log and fsynced, so proof is fully chained
    let (reopened, warnings) = ProofStore::open(dir.path()).expect("reopen store");
    assert_eq!(reopened.last_seq(), 1);
    assert!(
        warnings.is_empty(),
        "No warnings expected when chain.log was appended"
    );
}

#[test]
fn test_crash_post_final_fsync_recovered_successfully() {
    let dir = tempdir().expect("tempdir failed");
    let signer = ProofKeyPairSigner::from_secret_bytes([1u8; 32]);

    {
        let (mut store, _) = ProofStore::open(dir.path()).expect("open store");
        store.set_failpoint(Some(StoreCrashFailpoint::PostFinalFsync));
        let proof = helper_create_proof(&signer, 1, [0u8; 32]);
        assert!(store.save_proof(&proof).is_err());
    }

    let (reopened, warnings) = ProofStore::open(dir.path()).expect("reopen store");
    assert_eq!(reopened.last_seq(), 1);
    assert!(warnings.is_empty());
}

#[test]
fn test_fail_closed_refuses_save_on_chain_mismatch_or_unchained_files() {
    let dir = tempdir().expect("tempdir failed");
    let signer = ProofKeyPairSigner::from_secret_bytes([1u8; 32]);

    {
        let (mut store, _) = ProofStore::open(dir.path()).expect("open store");
        store.set_failpoint(Some(StoreCrashFailpoint::PostBinWrite));
        let proof = helper_create_proof(&signer, 1, [0u8; 32]);
        assert!(store.save_proof(&proof).is_err());
    }

    let (mut store, warnings) = ProofStore::open(dir.path()).expect("reopen store");
    assert!(!warnings.is_empty());

    // Attempting save_proof on unchained store MUST fail closed
    let proof2 = helper_create_proof(&signer, 1, [0u8; 32]);
    assert!(store.save_proof(&proof2).is_err());
}

// FILE-CONTEXT
// STAND: 2026-10-14
// ZWECK: Integrationstests fuer DpiRestoreFilter mit RevocationLog und ErasureLedger (Task B10).

#![forbid(unsafe_code)]

use contextra_checkpoint::restore_filter::CpyRestoreFilter;
use contextra_crypto::revocation_log::{RevocationLog, RevocationTarget};
use contextra_crypto::{SigningKey, VerifyingKey};
use contextra_engine::deletion::DpiRestoreFilter;
use contextra_ports::{Rng, SystemClock, SystemRng};
use contextra_store::erasure_ledger::{ErasureLedger, LedgerKey, LedgerOpenStatus};
use std::sync::Arc;
use tempfile::tempdir;

fn test_keypair() -> (SigningKey, VerifyingKey) {
    let mut rng = SystemRng::new();
    let mut seed = [0u8; 32];
    rng.fill_bytes(&mut seed);
    let sk = SigningKey::from_bytes(&seed);
    let vk = sk.verifying_key();
    (sk, vk)
}

fn create_test_ledger() -> ErasureLedger {
    let tmp = tempdir().expect("tempdir");
    let key = LedgerKey::new([1u8; 32]);
    let handle = tokio::runtime::Handle::current();
    let res = handle.block_on(async {
        ErasureLedger::open_or_fail_closed(tmp.path(), key, false, None).await
    });
    match res {
        Ok((Some(ledger), LedgerOpenStatus::Fresh)) => ledger,
        _ => panic!("Failed to open test ledger"),
    }
}

#[test]
fn test_restore_filter_revoked_group_drops() -> Result<(), Box<dyn std::error::Error>> {
    let (sk, vk) = test_keypair();
    let clock = Arc::new(SystemClock::new());
    let log = Arc::new(RevocationLog::new_in_memory(clock, Some(sk), vk));
    let ledger = Arc::new(create_test_ledger());

    log.append(RevocationTarget::Group(42))?;

    let filter = DpiRestoreFilter::from_runtime(ledger, Some(log), None);

    let key = b"group:42:item1";
    assert!(
        filter.drop_record(key, 100),
        "Record for revoked group 42 MUST be dropped"
    );

    Ok(())
}

#[test]
fn test_restore_filter_non_revoked_key_keeps() -> Result<(), Box<dyn std::error::Error>> {
    let (sk, vk) = test_keypair();
    let clock = Arc::new(SystemClock::new());
    let log = Arc::new(RevocationLog::new_in_memory(clock, Some(sk), vk));
    let ledger = Arc::new(create_test_ledger());

    log.append(RevocationTarget::Group(99))?;

    let filter = DpiRestoreFilter::from_runtime(ledger, Some(log), None);

    let key = b"group:42:item1";
    assert!(
        !filter.drop_record(key, 100),
        "Record for active group 42 MUST be kept"
    );

    Ok(())
}

#[test]
fn test_restore_filter_trait_compatibility() {
    let ledger = Arc::new(create_test_ledger());
    let filter: Box<dyn CpyRestoreFilter> = Box::new(DpiRestoreFilter::for_lsm(ledger));
    assert!(!filter.drop_record(b"any_key", 10));
}

//! Tests for Stufe 7: F-16 Honest Integrity Guarantee.

use contextra_store::wal::{Wal, WalIntegrityGuarantee};
use tempfile::tempdir;

#[tokio::test]
async fn wal_integrity_guarantee_bitrot_only_without_key_manager() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("wal_guarantee.wal");

    let wal = Wal::open(&wal_path).await.expect("open wal");
    assert_eq!(
        wal.integrity_guarantee(),
        WalIntegrityGuarantee::BitrotOnly,
        "WAL opened without KeyManager must report BitrotOnly integrity guarantee"
    );
    wal.close().await.expect("close wal");
}

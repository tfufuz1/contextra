//! Contract tests for PO-04, PO-06, PO-08: RecordSealer, FileKeyring, FileKey.

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "../src/gate.rs"]
pub mod gate;
#[path = "../src/keys.rs"]
pub mod keys;

use contextra_ports::BoxFuture;
use gate::{KeyBuf, KeyRange};
use keys::{Envelope, FileId, FileKey, FileKeyring, FileKind, RecordSealer, SealCtx};

struct MockRecordSealer;

impl RecordSealer for MockRecordSealer {
    fn seal<'a>(
        &'a self,
        _ctx: &'a SealCtx,
        _key: &'a KeyBuf,
        plain: &'a [u8],
    ) -> BoxFuture<'a, Result<Envelope, String>> {
        Box::pin(async move { Ok(Envelope::new(plain.to_vec())) })
    }

    fn open<'a>(
        &'a self,
        _key: &'a KeyBuf,
        env: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<u8>, String>> {
        Box::pin(async move { Ok(env.to_vec()) })
    }

    fn evict(&self, _scope: &KeyRange) {}

    fn make_slots_durable(&self) -> BoxFuture<'_, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }
}

struct MockFileKeyring;

impl FileKeyring for MockFileKeyring {
    fn create(&self, _kind: FileKind, id: FileId) -> BoxFuture<'_, Result<FileKey, String>> {
        Box::pin(async move { Ok(FileKey::new(vec![0xAA; 32], id)) })
    }

    fn open(&self, id: FileId) -> BoxFuture<'_, Result<FileKey, String>> {
        Box::pin(async move { Ok(FileKey::new(vec![0xAA; 32], id)) })
    }

    fn destroy(&self, _id: FileId) -> BoxFuture<'_, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }
}

#[test]
fn po_04_record_sealer_and_file_keyring_dyn_compatible() {
    let sealer: Box<dyn RecordSealer> = Box::new(MockRecordSealer);
    let keyring: Box<dyn FileKeyring> = Box::new(MockFileKeyring);

    fn assert_send_sync<T: Send + Sync + ?Sized>() {}
    assert_send_sync::<dyn RecordSealer>();
    assert_send_sync::<dyn FileKeyring>();

    let _ = sealer;
    let _ = keyring;
}

#[tokio::test]
async fn po_06_file_keyring_destroy_contract() {
    let keyring = MockFileKeyring;
    let file_id = FileId::new(42);
    let res = keyring.destroy(file_id).await;
    assert!(res.is_ok());
}

#[test]
fn po_08_file_key_no_debug_leak() {
    let secret_bytes = vec![0xDE, 0xAD, 0xBE, 0xEF];
    let file_key = FileKey::new(secret_bytes, FileId::new(1));
    let debug_output = format!("{:?}", file_key);

    assert!(!debug_output.contains("DEADBEEF"));
    assert!(!debug_output.contains("222"));
    assert!(debug_output.contains("[REDACTED]"));
}

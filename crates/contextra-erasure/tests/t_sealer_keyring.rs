//! Tests für Sealer und FileKeyring (T-VAULT-2, T-VAULT-4, T-FK-1..3).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_erasure::vault::keyring::InMemoryKeyring;
use contextra_erasure::vault::sealer::VaultSealer;
use contextra_ports::gate::{KeyBuf, KeyRange};
use contextra_ports::keys::{FileId, FileKeyring, FileKind, RecordSealer, SealCtx};
use std::any::TypeId;

/// SPEC: T-FK-1 (Kanonischer FileId-Typ aus contextra-types über ports und erasure identisch)
#[test]
fn t_fk_1_canonical_file_id_identity() {
    // Static assertion: TypeId must match exactly
    assert_eq!(
        TypeId::of::<contextra_types::FileId>(),
        TypeId::of::<contextra_ports::FileId>(),
        "contextra_types::FileId and contextra_ports::FileId must be identical types"
    );
    assert_eq!(
        TypeId::of::<contextra_types::FileId>(),
        TypeId::of::<FileId>(),
        "contextra_types::FileId and FileId in ports::keys must be identical types"
    );

    // Direct assignment without any conversion (compile-time type equivalence check)
    let raw_bytes = [0x5Au8; 16];
    let types_fid: contextra_types::FileId = contextra_types::FileId::new(raw_bytes);
    let ports_fid: contextra_ports::FileId = types_fid;
    let local_fid: FileId = ports_fid;

    assert_eq!(types_fid, ports_fid);
    assert_eq!(ports_fid, local_fid);
    assert_eq!(local_fid.as_bytes(), &raw_bytes);
}

#[tokio::test]
async fn t_fk_1_keyring_create_open_destroy() {
    let keyring = InMemoryKeyring::new();
    let file_id = FileId::new([42u8; 16]);

    let key = keyring.open(file_id).await;
    assert!(key.is_err());

    let created = keyring
        .create(FileKind::SSTable, file_id)
        .await
        .expect("create file key failed");
    assert_eq!(created.file_id(), file_id);
    assert_eq!(created.as_bytes().len(), 32);

    let key_opened = keyring.open(file_id).await.expect("open file key failed");
    assert_eq!(key_opened.as_bytes(), created.as_bytes());

    keyring
        .destroy(file_id)
        .await
        .expect("destroy file key failed");

    let open_after = keyring.open(file_id).await;
    assert!(open_after.is_err());
}

#[tokio::test]
async fn t_vault_2_and_4_sealer_seal_open_evict() {
    let sealer = VaultSealer::new_in_memory();
    let ctx = SealCtx::new(1);
    let key = KeyBuf::new([0x01; 16]);
    let plain = b"hello secret contextra document";

    let env = sealer.seal(&ctx, &key, plain).await.expect("seal failed");

    let decrypted = sealer.open(&key, env.payload()).await.expect("open failed");
    assert_eq!(&decrypted[..], plain);

    let range = KeyRange::new(KeyBuf::new([0x00; 16]), KeyBuf::new([0x02; 16]));
    sealer.evict(&range);

    sealer
        .make_slots_durable()
        .await
        .expect("make_slots_durable failed");
}

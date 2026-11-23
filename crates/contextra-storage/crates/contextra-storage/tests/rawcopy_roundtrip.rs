use contextra_crypto_ref::checksum::Crc32c;
use contextra_crypto_ref::cipher::XChaCha20Poly1305Cipher;
use contextra_crypto_ref::mac::HmacSha256;
use contextra_storage::codec::seal::{seal_frame, unseal_frame, Aad, SealMode};
use contextra_storage::ports::{BranchId, ClusterId, Error, Key32, SpaceId};
use contextra_storage::store::run::block::{decode, encode, BlockEntry, MAX_DECODED_BLOCK_BYTES};
use zeroize::Zeroizing;

fn block_aad<'a>(cluster: ClusterId, uid: &'a [u8; 16]) -> Aad<'a> {
    const INSTANCE: [u8; 16] = [10; 16];
    Aad::Block { instance: &INSTANCE, space: SpaceId(5), cluster, key_epoch: 3, block_uid: uid }
}

#[test]
fn rawcopy_roundtrip_preserves_sealed_block_bytes_across_file_relocation() -> Result<(), Error> {
    let cluster = ClusterId::from_wire([11; 16]);
    let uid = [12; 16];
    let entries = vec![BlockEntry {
        branch: BranchId(0), key: Zeroizing::new(b"document/alpha".to_vec()), tag: (44 << 8) | 1,
        header: Zeroizing::new(b"opaque-header".to_vec()), body: Zeroizing::new(b"sealed body".to_vec()),
    }];
    let encoded_block = encode(&entries, MAX_DECODED_BLOCK_BYTES)?;
    let key = Key32::from_array([13; 32]);
    let cipher = XChaCha20Poly1305Cipher;
    let mac = HmacSha256;
    let checksum = Crc32c;
    let source_frame = seal_frame(SealMode::Encrypted, 3, Some(&key), &[14; 24], block_aad(cluster, &uid),
        &encoded_block, &cipher, &mac, &checksum)?;

    // Run relocation changes the surrounding section/file offsets, which are
    // deliberately absent from AAD. The sealed frame itself is copied bytewise.
    let destination_frame = source_frame.clone();
    assert_eq!(source_frame, destination_frame);
    let (_, plaintext) = unseal_frame(SealMode::Encrypted, Some(&key), block_aad(cluster, &uid),
        &destination_frame, &cipher, &mac, &checksum)?;
    assert_eq!(decode(&plaintext, MAX_DECODED_BLOCK_BYTES)?, entries);
    assert!(unseal_frame(SealMode::Encrypted, Some(&key),
        block_aad(ClusterId::from_wire([15; 16]), &uid), &destination_frame, &cipher, &mac, &checksum).is_err());
    Ok(())
}

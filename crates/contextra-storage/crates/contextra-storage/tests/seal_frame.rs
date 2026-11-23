use contextra_crypto_ref::checksum::Crc32c;
use contextra_crypto_ref::cipher::XChaCha20Poly1305Cipher;
use contextra_crypto_ref::mac::HmacSha256;
use contextra_storage::codec::seal::{seal_frame, unseal_frame, Aad, SealMode};
use contextra_storage::ports::{ClusterId, Key32, SpaceId};

fn aad<'a>(cluster: ClusterId, block_uid: &'a [u8; 16]) -> Aad<'a> {
    // The arrays are promoted constants so the test uses the exact logical
    // identity across a simulated file/offset relocation.
    const INSTANCE: [u8; 16] = [2; 16];
    Aad::Block { instance: &INSTANCE, space: SpaceId(3), cluster, key_epoch: 7, block_uid }
}

#[test]
fn encrypted_and_authenticated_seals_round_trip_with_direct_source_key() {
    let key = Key32::from_array([9; 32]);
    let cipher = XChaCha20Poly1305Cipher;
    let mac = HmacSha256;
    let checksum = Crc32c;
    let cluster = ClusterId::from_wire([4; 16]);
    let uid = [6; 16];
    let nonce = [8; 24];
    let plaintext = b"cluster-local body";
    for mode in [SealMode::Authenticated, SealMode::Encrypted] {
        let frame = seal_frame(mode, 7, Some(&key), &nonce, aad(cluster, &uid), plaintext,
            &cipher, &mac, &checksum).expect("seal");
        let (_, opened) = unseal_frame(mode, Some(&key), aad(cluster, &uid), &frame,
            &cipher, &mac, &checksum).expect("unseal");
        assert_eq!(&opened[..], plaintext);
    }
}

#[test]
fn authenticated_aad_and_payload_are_bound_and_plain_crc_is_checked() {
    let key = Key32::from_array([9; 32]);
    let cipher = XChaCha20Poly1305Cipher;
    let mac = HmacSha256;
    let checksum = Crc32c;
    let cluster = ClusterId::from_wire([4; 16]);
    let other = ClusterId::from_wire([5; 16]);
    let uid = [6; 16];
    let nonce = [8; 24];
    let frame = seal_frame(SealMode::Authenticated, 7, Some(&key), &nonce, aad(cluster, &uid), b"payload",
        &cipher, &mac, &checksum).expect("authenticated seal");
    assert!(unseal_frame(SealMode::Authenticated, Some(&key), aad(other, &uid), &frame,
        &cipher, &mac, &checksum).is_err());
    let mut changed = frame.clone();
    if let Some(byte) = changed.get_mut(29) { *byte ^= 1; }
    assert!(unseal_frame(SealMode::Authenticated, Some(&key), aad(cluster, &uid), &changed,
        &cipher, &mac, &checksum).is_err());

    let plain = seal_frame(SealMode::Plain, 7, None, &nonce, aad(cluster, &uid), b"payload",
        &cipher, &mac, &checksum).expect("plain seal");
    let mut changed_plain = plain;
    if let Some(byte) = changed_plain.get_mut(29) { *byte ^= 1; }
    assert!(unseal_frame(SealMode::Plain, None, aad(cluster, &uid), &changed_plain,
        &cipher, &mac, &checksum).is_err());
}

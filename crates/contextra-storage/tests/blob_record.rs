use contextra_storage::store::blob::record::BlobRecord;
use zeroize::Zeroizing;

#[test]
fn blob_record_round_trip_preserves_stable_uid_and_seal_frame() {
    let record = BlobRecord { record_uid: [7; 16], seal_frame: Zeroizing::new(vec![1, 2, 3, 4, 5]) };
    let bytes = record.encode(1024).unwrap();
    assert_eq!(BlobRecord::decode(&bytes, 1024).unwrap(), record);
    // Framing contains no blob identity or offset, so the same bytes remain
    // valid when copied to another file location.
    assert_eq!(BlobRecord::decode(&bytes, 1024).unwrap().seal_frame, record.seal_frame);
}

#[test]
fn blob_record_rejects_truncation_length_mismatch_and_bit_flips() {
    let record = BlobRecord { record_uid: [9; 16], seal_frame: Zeroizing::new(vec![0xa5; 24]) };
    let bytes = record.encode(1024).unwrap();
    assert!(BlobRecord::decode(&bytes[..bytes.len() - 1], 1024).is_err());
    assert!(BlobRecord::decode(&bytes, 8).is_err());
    let mut corrupt = bytes;
    *corrupt.get_mut(19).unwrap() ^= 1;
    assert!(BlobRecord::decode(&corrupt, 1024).is_err());
}

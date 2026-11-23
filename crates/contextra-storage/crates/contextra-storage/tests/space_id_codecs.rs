use contextra_crypto_ref::mac::Blake3Mac;
use contextra_storage::catalog::ops::CatalogOp;
use contextra_storage::codec::keys_codec::{self, KeysRecord};
use contextra_storage::codec::seal::Aad;
use contextra_storage::store::redo::frame::{decode_frame, encode_frame, EncodedFrame, FrameHead, FrameKind, OpHead};
use contextra_storage::ports::{BranchId, CallerMeta, ClusterId, Key32, Origin, Protection, SpaceId};
use std::collections::BTreeMap;

const SPACE: SpaceId = SpaceId(u64::MAX - 7);
const INSTANCE: [u8; 16] = [0x42; 16];

#[test]
fn aad_catalog_redo_and_keys_encode_space_id_as_canonical_big_endian_u64() {
    let aad = Aad::Block { instance: &INSTANCE, space: SPACE, cluster: ClusterId::from_wire([8; 16]),
        key_epoch: 4, block_uid: &[9; 16] }.encode();
    assert_eq!(aad.get(21..29), Some(SPACE.0.to_be_bytes().as_slice()));

    let catalog = CatalogOp::SpaceAdd { space: SPACE, protection: Protection::Sealed };
    let mut catalog_bytes = Vec::new();
    catalog.encode(&mut catalog_bytes).unwrap();
    assert_eq!(catalog_bytes.get(6..14), Some(SPACE.0.to_be_bytes().as_slice()));
    assert_eq!(CatalogOp::decode(&catalog_bytes).unwrap().0, catalog);

    let payload = vec![0xAA];
    let frame = EncodedFrame {
        head: FrameHead { stamp: 1, kind: FrameKind::Change, idem: [0; 16], epoch: 0, flags: 0,
            caller: CallerMeta { origin: Origin::User, caller_seq: 3 }, ttl_bucket: 0,
            ops: vec![OpHead { space: SPACE, cluster: ClusterId::from_wire([8; 16]), branch: BranchId(0),
                kind: 1, key_epoch: 4, op_uid: [7; 16], payload_len: 1 }] },
        payloads: vec![payload],
    };
    let root = Key32::from_array([0x19; 32]);
    let mac = Blake3Mac;
    let encoded = encode_frame(&frame, &[0; 32], &root, &mac).unwrap();
    let op_start = 4 + 53;
    assert_eq!(encoded.get(op_start..op_start + 8), Some(SPACE.0.to_be_bytes().as_slice()));
    assert_eq!(decode_frame(&encoded, &[0; 32], &root, &mac).unwrap().head.ops.first().unwrap().space, SPACE);

    let record = KeysRecord { wrapped_root: vec![1], wrapped_space_deks: BTreeMap::from([(SPACE, vec![2, 3])]) };
    let keys = keys_codec::encode(&record, &root, &mac).unwrap();
    assert_eq!(keys.get(19..27), Some(SPACE.0.to_be_bytes().as_slice()));
    assert_eq!(keys_codec::decode(&keys, &root, &mac).unwrap(), record);
}

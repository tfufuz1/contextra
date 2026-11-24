use contextra_storage::engine::memtable::MemKey;
use contextra_storage::ports::{BranchId, Error, Key32};
use contextra_storage::store::blob::record::BlobRecord;
use contextra_storage::store::run::block::BlockEntry;
use contextra_storage::store::run::extension::ExtensionBlock;
use contextra_storage::store::run::rangedel::RangeMarker;
use contextra_storage::store::run::subindex::SubIndexEntry;
use zeroize::Zeroizing;

#[test]
fn secret_canaries_do_not_escape_debug_or_error_text() {
    let canary = "CANARY_SECRET_7d5e10";
    let key = MemKey::new(BranchId(0), canary.as_bytes().to_vec(), 1);
    let wrapped = Key32::from_array([0xA7; 32]);
    let error = Error::Corrupt { place: "sealed block" };
    let rendered = format!("{key:?} {wrapped:?} {error} {error:?}");
    assert!(!rendered.contains(canary));
    assert!(!rendered.contains("A7A7A7"));

    let payload_canary = "EXTENSION_PAYLOAD_CANARY";
    let range = RangeMarker { branch: BranchId(0), start: Zeroizing::new(payload_canary.as_bytes().to_vec()),
        end: Zeroizing::new(b"z".to_vec()), tag: 0x103 };
    let extension = ExtensionBlock::new(1, 1, payload_canary.as_bytes().to_vec()).unwrap();
    let blob = BlobRecord { record_uid: [0xCC; 16], seal_frame: Zeroizing::new(payload_canary.as_bytes().to_vec()) };
    let block = BlockEntry { branch: BranchId(0), key: Zeroizing::new(payload_canary.as_bytes().to_vec()),
        tag: 0x101, header: Zeroizing::new(payload_canary.as_bytes().to_vec()), body: Zeroizing::new(payload_canary.as_bytes().to_vec()) };
    let subindex = SubIndexEntry { last_key: Zeroizing::new(payload_canary.as_bytes().to_vec()), offset: 0,
        len: 1, entries: 1, weight_sum: [99; 4], min_stamp: 1, max_stamp: 1, block_uid: [0xCC; 16] };
    let rendered = format!("{range:?} {extension:?} {blob:?} {block:?} {subindex:?}");
    assert!(!rendered.contains(payload_canary));
    assert!(!rendered.contains("99"));
}

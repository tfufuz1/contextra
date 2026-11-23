use contextra_storage::engine::memtable::MemKey;
use contextra_storage::ports::{BranchId, Error, Key32};

#[test]
fn secret_canaries_do_not_escape_debug_or_error_text() {
    let canary = "CANARY_SECRET_7d5e10";
    let key = MemKey::new(BranchId(0), canary.as_bytes().to_vec(), 1);
    let wrapped = Key32::from_array([0xA7; 32]);
    let error = Error::Corrupt { place: "sealed block" };
    let rendered = format!("{key:?} {wrapped:?} {error} {error:?}");
    assert!(!rendered.contains(canary));
    assert!(!rendered.contains("A7A7A7"));
}

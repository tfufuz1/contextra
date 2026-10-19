// FILE-CONTEXT
// ZWECK: Test for ContextraBuilder KvDeleteMode feature derivation (§4.7, M1.5).

use contextra::ContextraBuilder;
use contextra_store::kv::KvDeleteMode;

#[test]
fn test_builder_kv_delete_mode_derived_from_feature() {
    let builder = ContextraBuilder::new(768);
    let expected = if cfg!(feature = "encryption-at-rest") {
        KvDeleteMode::CryptoShred
    } else {
        KvDeleteMode::TombstoneOnly
    };
    assert_eq!(
        builder.kv_delete_mode(),
        expected,
        "Builder kv_delete_mode must match feature 'encryption-at-rest' state"
    );
}

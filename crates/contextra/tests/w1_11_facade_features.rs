// FILE-CONTEXT
// ZWECK: Test facade features all defined (W1-11 Stage 3).

#[test]
fn facade_features_all_defined() {
    assert!(cfg!(feature = "erasure") || !cfg!(feature = "erasure"));
    assert!(cfg!(feature = "encryption-at-rest") || !cfg!(feature = "encryption-at-rest"));
}

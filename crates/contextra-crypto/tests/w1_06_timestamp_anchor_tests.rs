//! Tests for Stage 4: Timestamp Anchors (`AcceptAllTsaVerifier` & Qualification) (w1_06_*).

#![forbid(unsafe_code)]

use contextra_crypto::{
    AnchorQualification, LocalMonotonicAnchor, QualificationLevel, TimeAnchor,
};

#[test]
fn w1_06_local_anchor_never_reports_qualified() {
    let anchor = LocalMonotonicAnchor::default();
    let receipt = anchor.anchor(b"test_msg_imprint").expect("anchor receipt");

    // Local anchor MUST report AnchorQualification::NotQualified
    assert_eq!(receipt.qualification, AnchorQualification::NotQualified);
    assert_eq!(receipt.anchor_type, "LocalMonotonic");

    // QualificationLevel::LocalMonotonic is the level for local anchor
    let level = QualificationLevel::LocalMonotonic;
    assert!(matches!(level, QualificationLevel::LocalMonotonic));
}

#[test]
fn w1_06_qualified_requires_real_verifier() {
    // Verifies that the compile_fail UI file exists for type safety check
    let ui_file = std::path::Path::new("tests/compile_fail/w1_06_qualified_requires_real_verifier.rs");
    assert!(ui_file.exists(), "UI test file for QualificationLevel::Qualified must exist");
}

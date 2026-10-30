//! Error DTO, schema versioning, and error conversion test suite for `contextra-types`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use contextra_types::*;

#[test]
fn test_error_dto_new_and_with_details() {
    let dto = ContextraErrorDto::new("NotFound", "Document doc_100 not found");
    assert_eq!(dto.kind, "NotFound");
    assert_eq!(dto.message, "Document doc_100 not found");
    assert!(dto.details.is_none());

    let dto_details = ContextraErrorDto::with_details(
        "Storage",
        "Disk I/O failed",
        serde_json::json!({"path": "/var/data/seg_1.sst", "errno": 5}),
    );

    assert_eq!(dto_details.kind, "Storage");
    assert_eq!(dto_details.message, "Disk I/O failed");
    assert!(dto_details.details.is_some());

    let details_val = dto_details.details.unwrap();
    assert_eq!(
        details_val.get("path"),
        Some(&serde_json::json!("/var/data/seg_1.sst"))
    );
    assert_eq!(details_val.get("errno"), Some(&serde_json::json!(5)));
}

#[test]
fn test_error_dto_display_and_formatting() {
    let dto = ContextraErrorDto::new("InvalidInput", "Key cannot be empty");
    let disp = format!("{dto}");
    assert!(disp.contains("InvalidInput"));
    assert!(disp.contains("Key cannot be empty"));

    let debug = format!("{dto:?}");
    assert!(debug.contains("ContextraErrorDto"));
}

#[test]
fn test_hnsw_deletion_error_variants() {
    let err_node = HnswDeletionError::NodeNotFound(DocId::new(42));
    assert_eq!(
        err_node.to_string(),
        "Node for doc_id DocId(42) not found in HNSW index"
    );

    let err_disc = HnswDeletionError::DisconnectedComponent(DocId::new(42));
    assert_eq!(
        err_disc.to_string(),
        "Graph repair resulted in disconnected component for node DocId(42)"
    );

    let err_verif = HnswDeletionError::VerificationFailed {
        remaining_pointers: 5,
    };
    assert_eq!(
        err_verif.to_string(),
        "Verification failed: 5 remaining ghost pointers found"
    );

    let domain_err: ContextraError = err_node.into();
    assert!(matches!(
        domain_err,
        ContextraError::GraphRepairFailed(_)
    ));
}

#[test]
fn test_manifest_schema_version_and_doc_id_width_properties() {
    let v1 = ManifestSchemaVersion::V1;
    assert_eq!(v1.as_u8(), 1);
    assert_eq!(v1.doc_id_width(), DocIdWidth::Bit64);

    let v2 = ManifestSchemaVersion::V2;
    assert_eq!(v2.as_u8(), 2);
    assert_eq!(v2.doc_id_width(), DocIdWidth::Bit128);

    #[cfg(not(feature = "docid-128"))]
    {
        assert!(v1.is_compatible_with_current_build());
        assert!(!v2.is_compatible_with_current_build());
    }
    #[cfg(feature = "docid-128")]
    {
        assert!(!v1.is_compatible_with_current_build());
        assert!(v2.is_compatible_with_current_build());
    }
}

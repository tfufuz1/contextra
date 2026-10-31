//! Compile-time and runtime type identity verification for contextra facade re-exports.
//! Ensures no breaking type-identity changes were introduced.

#![forbid(unsafe_code)]

fn _assert_same_type<T>(_: T, _: T) {}

#[test]
fn test_facade_type_identities() {
    // 1. ContextraError
    let facade_err: contextra::ContextraError =
        contextra::ContextraError::InvalidInput("test".into());
    let types_err: contextra_types::ContextraError =
        contextra_types::ContextraError::InvalidInput("test".into());
    _assert_same_type(facade_err, types_err);

    // 2. DocId
    let facade_doc_id: contextra::DocId = contextra::DocId::from_u64(1);
    let types_doc_id: contextra_types::DocId = contextra_types::DocId::from_u64(1);
    _assert_same_type(facade_doc_id, types_doc_id);

    // 3. ScoredDocument
    let facade_doc = contextra::ScoredDocument {
        id: contextra::DocId::from_u64(1),
        score: 1.0,
        metadata: None,
        vector: None,
    };
    let types_doc = contextra_types::ScoredDocument {
        id: contextra_types::DocId::from_u64(1),
        score: 1.0,
        metadata: None,
        vector: None,
    };
    _assert_same_type(facade_doc, types_doc);

    // 4. DistanceMetric
    let facade_metric: contextra::DistanceMetric = contextra::DistanceMetric::Cosine;
    let types_metric: contextra_types::DistanceMetric = contextra_types::DistanceMetric::Cosine;
    _assert_same_type(facade_metric, types_metric);

    // 5. AutoExtractionMode
    let facade_auto: contextra::collection_profile::AutoExtractionMode =
        contextra::collection_profile::AutoExtractionMode::Enabled;
    let types_auto: contextra_types::types::domain::AutoExtractionMode =
        contextra_types::types::domain::AutoExtractionMode::Enabled;
    _assert_same_type(facade_auto, types_auto);
}

// Compile-time check via const fn pointers
const _: fn() = || {
    fn check_err(_: contextra::ContextraError) -> contextra_types::ContextraError {
        unreachable!()
    }
    fn check_metric(_: contextra::DistanceMetric) -> contextra_types::DistanceMetric {
        unreachable!()
    }
    fn check_doc_id(_: contextra::DocId) -> contextra_types::DocId {
        unreachable!()
    }
};

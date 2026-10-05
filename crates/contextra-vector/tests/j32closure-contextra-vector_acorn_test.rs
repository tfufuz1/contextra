use contextra_core::DocId;
use contextra_vector::acorn::NaiveReferenceIndex;

#[test]
fn test_naive_reference_index_from_vectors() {
    let doc1 = DocId::from(101u64);
    let doc2 = DocId::from(102u64);
    let vec1 = vec![1.0, 2.0, 3.0];
    let vec2 = vec![4.0, 5.0, 6.0];

    let index = NaiveReferenceIndex::from_vectors(vec![
        (doc1, vec1),
        (doc2, vec2),
    ]);

    // Simple assertion to verify creation via from_vectors
    let _ = format!("{:?}", index);
}

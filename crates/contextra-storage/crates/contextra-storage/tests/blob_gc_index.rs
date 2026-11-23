use contextra_storage::store::blob::gc_index::{BlobGcIndex, BlobId, BlobIncarnation, BlobRef};

fn reference(blob_id: u64, incarnation: u64, offset: u64) -> BlobRef {
    BlobRef { blob_id: BlobId(blob_id), incarnation: BlobIncarnation(incarnation), offset, len: 32 }
}

#[test]
fn candidates_are_per_incarnation_and_wait_for_all_live_references() {
    let mut index = BlobGcIndex::new();
    let a = reference(7, 1, 0);
    let b = reference(7, 1, 64);
    index.add_reference(a).unwrap();
    index.add_reference(b).unwrap();
    let files = [(BlobId(7), BlobIncarnation(1)), (BlobId(7), BlobIncarnation(2))];
    assert_eq!(index.remove_candidates(files), vec![(BlobId(7), BlobIncarnation(2))]);

    index.remove_reference(a).unwrap();
    assert!(index.remove_candidates([(BlobId(7), BlobIncarnation(1))]).is_empty());
    index.remove_reference(b).unwrap();
    assert_eq!(index.remove_candidates([(BlobId(7), BlobIncarnation(1))]), vec![(BlobId(7), BlobIncarnation(1))]);
}

#[test]
fn invalid_or_unbalanced_reference_updates_fail_closed() {
    let mut index = BlobGcIndex::new();
    assert!(index.add_reference(BlobRef { len: 0, ..reference(1, 1, 0) }).is_err());
    assert!(index.add_reference(BlobRef { offset: u64::MAX, ..reference(1, 1, u64::MAX) }).is_err());
    assert!(index.remove_reference(reference(1, 1, 0)).is_err());
}

//! Invariant INV-KV-PREFIX-1 Test: Identical prompt prefixes share physical segments via refcounting

use contextra_kvcache::radix::NamespaceRadixTree;

#[test]
fn test_inv_kv_prefix_1_shared_physical_segments() {
    let mut tree = NamespaceRadixTree::new(100);

    let prefix = vec![1, 2, 3, 4, 5];
    let seg_id = 999;

    let ref1 = tree.insert(&prefix, seg_id).unwrap();
    assert_eq!(ref1.segment_id, seg_id);
    assert_eq!(ref1.current_refs(), 1);

    let ref2 = tree.insert(&prefix, 888).unwrap();
    assert_eq!(
        ref2.segment_id, seg_id,
        "INV-KV-PREFIX-1: Identical prefix MUST share segment"
    );
    assert_eq!(ref1.current_refs(), 2);
    assert_eq!(ref2.current_refs(), 2);

    assert_eq!(tree.segment_registry().len(), 1);

    let remaining = ref1.release();
    assert_eq!(remaining, 1);
    assert_eq!(ref2.current_refs(), 1);

    let removed_id = tree.remove(&prefix);
    assert_eq!(removed_id, Some(seg_id));

    let final_remaining = ref2.release();
    assert_eq!(final_remaining, 0);
}

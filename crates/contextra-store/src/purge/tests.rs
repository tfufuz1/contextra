//! Tests für Purge-Primitive (SPEC §8.8, §11.4).

use crate::memtable::MemTable;
use crate::purge::drain::drain_memtable;
use contextra_types::key::{KeyBound, KeyBuf, Matcher, Region, Family};
use contextra_types::tags::{ClusterTag, CollTag, DocTag, TenantTag};

#[test]
fn test_drain_memtable_matching_range() {
    let memtable = MemTable::new();
    let tenant = TenantTag::new([1u8; 16]);
    let coll = CollTag::new([2u8; 16]);
    let cluster = ClusterTag::new([3u8; 16]);
    let doc1 = DocTag::new([4u8; 16]);
    let doc2 = DocTag::new([5u8; 16]);

    let key1 = KeyBuf::new(Region::Data, tenant, coll, cluster, doc1, Family::Doc, 0, 100);
    let key2 = KeyBuf::new(Region::Data, tenant, coll, cluster, doc2, Family::Doc, 0, 101);

    memtable.put(key1.as_bytes().to_vec(), 100, b"val1".to_vec(), 0);
    memtable.put(key2.as_bytes().to_vec(), 101, b"val2".to_vec(), 0);

    let bound1 = key1.bound();
    let matcher = Matcher::Exact(bound1);

    let stats = drain_memtable(&memtable, &matcher);
    assert_eq!(stats.entries_drained, 1);
    assert!(memtable.get(key1.as_bytes(), 105).is_none());
    assert!(memtable.get(key2.as_bytes(), 105).is_some());
}

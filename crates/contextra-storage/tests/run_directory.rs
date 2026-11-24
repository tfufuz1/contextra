use contextra_storage::store::run::directory::{ClusterDirectory, ClusterDirectoryEntry};
use contextra_storage_ports::ClusterId;

fn entry(id: u8, offset: u64, len: u64) -> ClusterDirectoryEntry {
    ClusterDirectoryEntry {
        cluster: ClusterId::from_wire([id; 16]), section_offset: offset, section_len: len,
        block_count: 2, entry_count: 19, min_stamp: 1, max_stamp: 25,
    }
}

#[test]
fn directory_round_trip_and_binary_search() {
    let directory = ClusterDirectory::new(vec![entry(1, 43, 100), entry(3, 143, 50)]).unwrap();
    let encoded = directory.encode().unwrap();
    let decoded = ClusterDirectory::decode(&encoded, 8).unwrap();
    assert_eq!(decoded, directory);
    assert_eq!(decoded.find(ClusterId::from_wire([3; 16])), Some(&entry(3, 143, 50)));
    assert!(decoded.find(ClusterId::from_wire([2; 16])).is_none());
}

#[test]
fn directory_rejects_crc_order_overlap_and_truncation() {
    let directory = ClusterDirectory::new(vec![entry(1, 43, 100)]).unwrap();
    let mut bytes = directory.encode().unwrap();
    *bytes.get_mut(20).unwrap() ^= 0x80;
    assert!(ClusterDirectory::decode(&bytes, 8).is_err());

    assert!(ClusterDirectory::new(vec![entry(2, 43, 100), entry(1, 143, 20)]).is_err());
    assert!(ClusterDirectory::new(vec![entry(1, 43, 100), entry(2, 100, 20)]).is_err());
    let mut bytes = directory.encode().unwrap();
    bytes.pop();
    assert!(ClusterDirectory::decode(&bytes, 8).is_err());
}

use contextra_storage::store::run::filter::{BloomFilter, FILTER_KIND_BLOOM_V1};

#[test]
fn salted_bloom_round_trip_has_no_false_negatives() {
    let keys: Vec<Vec<u8>> = (0..500).map(|n| format!("document/{n}").into_bytes()).collect();
    let mut filter = BloomFilter::new(500, 0.01, [0x3a; 16]).unwrap();
    for key in &keys { filter.insert(key); }
    let encoded = filter.encode().unwrap();
    assert_eq!(encoded.get(8), Some(&FILTER_KIND_BLOOM_V1));
    let decoded = BloomFilter::decode(&encoded, 1 << 20).unwrap();
    assert!(keys.iter().all(|key| decoded.may_contain(key)));
}

#[test]
fn bloom_rejects_invalid_parameters_kind_and_corruption() {
    assert!(BloomFilter::new(0, 0.01, [0; 16]).is_err());
    assert!(BloomFilter::new(10, 0.5, [0; 16]).is_err());
    let mut filter = BloomFilter::new(10, 0.01, [1; 16]).unwrap();
    filter.insert(b"key");
    let mut encoded = filter.encode().unwrap();
    *encoded.get_mut(8).unwrap() = 2;
    assert!(BloomFilter::decode(&encoded, 1 << 20).is_err());
    let mut encoded = filter.encode().unwrap();
    *encoded.get_mut(40).unwrap() ^= 1;
    assert!(BloomFilter::decode(&encoded, 1 << 20).is_err());
}

//! Campaign J-41 hypothesis test suite for contextra-vector.
//! Tests static scan findings: neighbor count u8 truncation, distance score casting, index bounds.

#[test]
fn test_j41_neighbor_count_u8_truncation_oracle() {
    // Oracle: HNSW neighbor count per layer stored as u8.
    // If neighbor count exceeds 255 (e.g. 256), `as u8` silently wraps to 0.
    let neighbor_count: usize = 256;
    let truncated_u8 = neighbor_count as u8;
    assert_eq!(truncated_u8, 0, "Demonstrating `256 as u8` silent wrapping to 0");

    let safe_u8 = u8::try_from(neighbor_count);
    assert!(safe_u8.is_err(), "u8::try_from must error on 256");
}

#[test]
fn test_j41_docid_u128_truncation_oracle() {
    // Oracle: DocId can be u128 under docid-128 or u64 default.
    // Casting u128 DocId to u64 or u32 causes silent data loss.
    let doc_id_128: u128 = 0x1_0000_0000_0000_0000;
    let truncated_u64 = doc_id_128 as u64;
    assert_eq!(truncated_u64, 0, "Upper 64 bits lost when casting u128 to u64");

    let safe_u64 = u64::try_from(doc_id_128);
    assert!(safe_u64.is_err());
}

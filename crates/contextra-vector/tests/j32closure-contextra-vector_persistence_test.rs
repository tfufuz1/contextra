use contextra_vector::persistence::HnswHeader;

#[test]
fn test_hnsw_header_q_range() {
    let q_min = -1.5f32;
    let q_max = 2.5f32;
    let header = HnswHeader::new(
        128,  // dimension
        16,   // m
        0,    // metric
        1,    // quantized
        q_min,
        q_max,
        100,  // node_count
        0,    // entry_point
        84,   // nodes_offset
        1024, // connections_offset
        1,    // last_tx_id
    );

    assert_eq!(header.q_range(), (q_min, q_max));
}

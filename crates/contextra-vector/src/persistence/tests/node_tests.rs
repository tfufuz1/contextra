// FILE-CONTEXT
// ZWECK: Unit-Tests für NodeRecord-Serialisierung und Längenvalidierung.
// INVARIANTEN: Zero-Panic bei unzureichenden Bytes.
// HOTSPOTS: test_node_record_roundtrip_and_errors
// STAND: TS:2026-11-12T00:00:00Z

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use crate::error::{Result, VectorError};
use crate::persistence::node::NodeRecord;

#[test]
fn test_node_record_roundtrip_and_errors() -> Result<()> {
    let record = NodeRecord {
        doc_id: 42,
        max_layer: 3,
        vector_offset: 1024,
        connections_offset: 2048,
    };

    let bytes = record.to_bytes();
    assert_eq!(bytes.len(), NodeRecord::SIZE);
    let parsed = NodeRecord::from_bytes(&bytes)?;
    assert_eq!(record.doc_id, parsed.doc_id);
    assert_eq!(record.max_layer, parsed.max_layer);
    assert_eq!(record.vector_offset, parsed.vector_offset);
    assert_eq!(record.connections_offset, parsed.connections_offset);

    let small_bytes = vec![0u8; 5];
    let err_small = NodeRecord::from_bytes(&small_bytes);
    assert!(matches!(err_small, Err(VectorError::Storage(_))));

    Ok(())
}

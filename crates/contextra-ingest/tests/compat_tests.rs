//! Tests für die Workspace-Kompatibilitätsbrücke (`contextra-types`).

#![cfg(feature = "workspace")]

use contextra_ingest::chunking::types::{ContextChunk, ContextChunkLink, DocId};

#[test]
fn test_doc_id_roundtrip() {
    let local_id = DocId::new(42);
    let ws_id: contextra_types::DocId = local_id.into();
    assert_eq!(ws_id.as_u64(), 42);

    let back: DocId = ws_id.try_into().expect("TryFrom DocId should succeed");
    assert_eq!(back, local_id);
}

#[test]
fn test_context_chunk_conversion() {
    let local_chunk = ContextChunk {
        doc_id: DocId::new(100),
        content: "Test Inhalt für Ingestion".to_string(),
        relevance: 0.95,
        token_count: 5,
        metadata: Some(serde_json::json!({"source": "manual"})),
        contextual_prefix: Some("Kontext-Präfix".to_string()),
        links: vec![ContextChunkLink {
            kind: "elaborates".to_string(),
            target: "101".to_string(),
        }],
    };

    let ws_chunk: contextra_types::ContextChunk = local_chunk.clone().into();
    assert_eq!(ws_chunk.doc_id.as_u64(), 100);
    assert_eq!(ws_chunk.content, "Test Inhalt für Ingestion");
    assert_eq!(ws_chunk.relevance, 0.95);
    assert_eq!(ws_chunk.token_count, 5);
    assert_eq!(ws_chunk.links.len(), 1);
    assert_eq!(ws_chunk.links[0].target.as_u64(), 101);
    assert_eq!(
        ws_chunk.links[0].relation,
        contextra_types::LinkRelation::Elaborates
    );

    let roundtripped: ContextChunk = ws_chunk.into();
    assert_eq!(roundtripped.doc_id, local_chunk.doc_id);
    assert_eq!(roundtripped.content, local_chunk.content);
    assert_eq!(roundtripped.relevance, local_chunk.relevance);
    assert_eq!(roundtripped.token_count, local_chunk.token_count);
    assert_eq!(roundtripped.metadata, local_chunk.metadata);
    assert_eq!(
        roundtripped.contextual_prefix,
        local_chunk.contextual_prefix
    );
    assert_eq!(roundtripped.links.len(), 1);
    assert_eq!(roundtripped.links[0].kind, "elaborates");
    assert_eq!(roundtripped.links[0].target, "101");
}

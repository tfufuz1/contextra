// FILE-CONTEXT
// STAND: 2026-09-16T00:00:00Z
// ZWECK: Integrationstests, Fault-Injection und Nebenläufigkeitstests für FlatBuffers IPC (memfuse-wire)
// INVARIANTEN: Keine Panics bei beschädigten oder abgeschnittenen IPC-Puffern; thread-sicheres Deserialisieren.

use flatbuffers::FlatBufferBuilder;
use memfuse_wire::{
    root_as_search_response, Embedding, EmbeddingArgs, ScoredDocument, ScoredDocumentArgs,
    SearchResponse, SearchResponseArgs, VectorIndexUpdate, VectorIndexUpdateArgs,
};
use std::sync::Arc;
use std::thread;

#[test]
fn test_embedding_roundtrip_and_boundaries() {
    let mut builder = FlatBufferBuilder::new();

    let data_vec = builder.create_vector(&[0.1f32, -0.5f32, 0.999f32]);
    let quant_vec = builder.create_vector(&[10u8, 20u8, 255u8]);

    let emb_offset = Embedding::create(
        &mut builder,
        &EmbeddingArgs {
            data: Some(data_vec),
            quantized_u8: Some(quant_vec),
            metric: 1, // Euclidean
        },
    );
    builder.finish_minimal(emb_offset);

    let buf = builder.finished_data();
    let emb = flatbuffers::root::<Embedding>(buf).expect("Failed to parse Embedding root");

    assert_eq!(emb.metric(), 1);
    let data = emb.data().expect("Missing data vector");
    assert_eq!(data.len(), 3);
    assert_eq!(data.get(0), 0.1f32);
    assert_eq!(data.get(1), -0.5f32);
    assert_eq!(data.get(2), 0.999f32);

    let quant = emb.quantized_u8().expect("Missing quantized vector");
    assert_eq!(quant.len(), 3);
    assert_eq!(quant.get(0), 10);
    assert_eq!(quant.get(2), 255);
}

#[test]
fn test_scored_document_roundtrip_boundaries() {
    let mut builder = FlatBufferBuilder::new();

    let id_str = builder.create_string("doc-12345-test");
    let meta_str = builder.create_string(r#"{"key":"value","nested":{"count":42}}"#);

    let data_vec = builder.create_vector(&[1.0f32, 2.0f32]);
    let emb_offset = Embedding::create(
        &mut builder,
        &EmbeddingArgs {
            data: Some(data_vec),
            quantized_u8: None,
            metric: 0,
        },
    );

    let doc_offset = ScoredDocument::create(
        &mut builder,
        &ScoredDocumentArgs {
            id: Some(id_str),
            score: 0.95f32,
            metadata: Some(meta_str),
            embedding: Some(emb_offset),
        },
    );
    builder.finish_minimal(doc_offset);

    let buf = builder.finished_data();
    let doc = flatbuffers::root::<ScoredDocument>(buf).expect("Failed to parse ScoredDocument");

    assert_eq!(doc.id(), Some("doc-12345-test"));
    assert_eq!(doc.score(), 0.95f32);
    assert_eq!(
        doc.metadata(),
        Some(r#"{"key":"value","nested":{"count":42}}"#)
    );

    let emb = doc.embedding().expect("Missing embedding");
    assert_eq!(emb.metric(), 0);
    assert_eq!(emb.data().unwrap().len(), 2);
    assert!(emb.quantized_u8().is_none());
}

#[test]
fn test_search_response_full_roundtrip() {
    let mut builder = FlatBufferBuilder::new();

    let id1 = builder.create_string("doc-1");
    let doc1 = ScoredDocument::create(
        &mut builder,
        &ScoredDocumentArgs {
            id: Some(id1),
            score: 0.88,
            metadata: None,
            embedding: None,
        },
    );

    let id2 = builder.create_string("doc-2");
    let doc2 = ScoredDocument::create(
        &mut builder,
        &ScoredDocumentArgs {
            id: Some(id2),
            score: 0.72,
            metadata: None,
            embedding: None,
        },
    );

    let results_vec = builder.create_vector(&[doc1, doc2]);
    let resp_offset = SearchResponse::create(
        &mut builder,
        &SearchResponseArgs {
            results: Some(results_vec),
            total_hits: 1000,
            processing_time_ms: 12.34,
        },
    );
    builder.finish(resp_offset, None);

    let buf = builder.finished_data();
    let resp = root_as_search_response(buf).expect("Root verification failed");

    assert_eq!(resp.total_hits(), 1000);
    assert!((resp.processing_time_ms() - 12.34).abs() < 1e-5);

    let results = resp.results().expect("Results missing");
    assert_eq!(results.len(), 2);
    assert_eq!(results.get(0).id(), Some("doc-1"));
    assert_eq!(results.get(0).score(), 0.88f32);
    assert_eq!(results.get(1).id(), Some("doc-2"));
    assert_eq!(results.get(1).score(), 0.72f32);
}

#[test]
fn test_vector_index_update_roundtrip() {
    let mut builder = FlatBufferBuilder::new();

    let id = builder.create_string("vec-update-99");
    let meta = builder.create_string("index_type:hnsw");
    let quant_vec = builder.create_vector(&[1u8, 2u8, 3u8, 4u8, 5u8]);

    let emb_offset = Embedding::create(
        &mut builder,
        &EmbeddingArgs {
            data: None,
            quantized_u8: Some(quant_vec),
            metric: 2, // DotProduct
        },
    );

    let update_offset = VectorIndexUpdate::create(
        &mut builder,
        &VectorIndexUpdateArgs {
            id: Some(id),
            embedding: Some(emb_offset),
            metadata: Some(meta),
        },
    );
    builder.finish_minimal(update_offset);

    let buf = builder.finished_data();
    let update =
        flatbuffers::root::<VectorIndexUpdate>(buf).expect("VectorIndexUpdate root failed");

    assert_eq!(update.id(), Some("vec-update-99"));
    assert_eq!(update.metadata(), Some("index_type:hnsw"));

    let emb = update.embedding().expect("Embedding missing");
    assert_eq!(emb.metric(), 2);
    assert!(emb.data().is_none());
    assert_eq!(emb.quantized_u8().unwrap().len(), 5);
}

#[test]
fn test_fault_injection_and_corruption_no_panic() {
    // 1. Truncated buffer - verify root_as_search_response never panics
    let mut builder = FlatBufferBuilder::new();
    let id = builder.create_string("doc-truncated");
    let doc = ScoredDocument::create(
        &mut builder,
        &ScoredDocumentArgs {
            id: Some(id),
            score: 0.5,
            metadata: None,
            embedding: None,
        },
    );
    let results = builder.create_vector(&[doc]);
    let resp = SearchResponse::create(
        &mut builder,
        &SearchResponseArgs {
            results: Some(results),
            total_hits: 1,
            processing_time_ms: 1.0,
        },
    );
    builder.finish(resp, None);

    let valid_buf = builder.finished_data().to_vec();
    assert!(root_as_search_response(&valid_buf).is_ok());

    // Truncated to partial length - must not panic
    for len in 0..valid_buf.len() {
        let truncated = &valid_buf[..len];
        let _ = root_as_search_response(truncated);
    }

    // 2. Corrupted random garbage
    let garbage = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0xFF, 0x42, 0x13];
    assert!(root_as_search_response(&garbage).is_err());

    // 3. Single bit flips on valid buffer - must not panic
    for i in 0..valid_buf.len() {
        let mut corrupted = valid_buf.clone();
        corrupted[i] ^= 0x01;
        let _ = root_as_search_response(&corrupted);
    }
}

#[test]
fn test_concurrency_parallel_serialization() {
    let threads: Vec<_> = (0..8)
        .map(|t_id| {
            thread::spawn(move || {
                for i in 0..100 {
                    let mut builder = FlatBufferBuilder::new();
                    let doc_id = format!("thread-{}-doc-{}", t_id, i);
                    let id_offset = builder.create_string(&doc_id);
                    let doc = ScoredDocument::create(
                        &mut builder,
                        &ScoredDocumentArgs {
                            id: Some(id_offset),
                            score: (t_id as f32) * 0.1 + (i as f32) * 0.01,
                            metadata: None,
                            embedding: None,
                        },
                    );
                    let results = builder.create_vector(&[doc]);
                    let resp = SearchResponse::create(
                        &mut builder,
                        &SearchResponseArgs {
                            results: Some(results),
                            total_hits: (i + 1) as u32,
                            processing_time_ms: 0.5,
                        },
                    );
                    builder.finish(resp, None);

                    let buf = builder.finished_data();
                    let parsed =
                        root_as_search_response(buf).expect("Parallel verification failed");
                    assert_eq!(parsed.total_hits(), (i + 1) as u32);
                    let res = parsed.results().unwrap();
                    assert_eq!(res.get(0).id(), Some(doc_id.as_str()));
                }
            })
        })
        .collect();

    for t in threads {
        t.join().expect("Thread panicked");
    }
}

#[test]
fn test_shared_buffer_concurrency() {
    let mut builder = FlatBufferBuilder::new();
    let id = builder.create_string("shared-doc");
    let doc = ScoredDocument::create(
        &mut builder,
        &ScoredDocumentArgs {
            id: Some(id),
            score: 0.99,
            metadata: None,
            embedding: None,
        },
    );
    let results = builder.create_vector(&[doc]);
    let resp = SearchResponse::create(
        &mut builder,
        &SearchResponseArgs {
            results: Some(results),
            total_hits: 42,
            processing_time_ms: 2.5,
        },
    );
    builder.finish(resp, None);

    let shared_buf = Arc::new(builder.finished_data().to_vec());

    let threads: Vec<_> = (0..8)
        .map(|_| {
            let buf = Arc::clone(&shared_buf);
            thread::spawn(move || {
                for _ in 0..200 {
                    let parsed = root_as_search_response(&buf).expect("Shared buffer parse failed");
                    assert_eq!(parsed.total_hits(), 42);
                    assert_eq!(parsed.results().unwrap().get(0).id(), Some("shared-doc"));
                }
            })
        })
        .collect();

    for t in threads {
        t.join().expect("Thread panicked");
    }
}

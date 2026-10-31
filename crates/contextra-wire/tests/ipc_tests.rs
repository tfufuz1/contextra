//! # Integrationstests für contextra-wire
//!
//! **Ring & Zweck:** Ring 0 Testsuite.
//! Prüft Roundtrips, Buffer-Verifikation, Fuzzing-Robustheit, Nebenläufigkeit
//! und JSON-RPC 2.0 Spezifikationskonformität.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
// FILE-CONTEXT
// STAND: 2026-10-27T00:00:00Z
// ZWECK: Integrationstests, Mutation Fuzzing, Edge Cases und Concurrency-Tests für contextra-wire.
// INVARIANTEN: Zero panic in verification path; deterministic PRNG fuzzing; strict compliance with JSON-RPC & FlatBuffers specs.

use bytes::Bytes;
use contextra_wire::{
    parse_jsonrpc_payload, parse_jsonrpc_payload_with_config, verify_and_parse_embedding,
    verify_and_parse_hyper_edge, verify_and_parse_hyper_edge_id, verify_and_parse_role_binding,
    verify_and_parse_role_id, verify_and_parse_scored_document, verify_and_parse_search_response,
    verify_and_parse_vector_index_update, Embedding, EmbeddingArgs, FlatBufferConfig, HyperEdge,
    HyperEdgeArgs, HyperEdgeId, HyperEdgeIdArgs, JsonRpcConfig, JsonRpcError, JsonRpcPayload,
    JsonRpcRequest, JsonRpcResponse, RoleBinding, RoleBindingArgs, RoleId, RoleIdArgs,
    ScoredDocument, ScoredDocumentArgs, SearchResponse, SearchResponseArgs, VectorIndexUpdate,
    VectorIndexUpdateArgs, WireBuffer,
};
use flatbuffers::FlatBufferBuilder;
use std::sync::Arc;
use std::thread;

/// Deterministic Pseudo-Random Number Generator (LCG) for reproducible mutation testing.
struct SimpleLcg {
    state: u64,
}

impl SimpleLcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 32) as u32
    }

    fn next_u8(&mut self) -> u8 {
        self.next_u32() as u8
    }

    fn next_usize(&mut self, upper_bound: usize) -> usize {
        if upper_bound == 0 {
            0
        } else {
            (self.next_u32() as usize) % upper_bound
        }
    }
}

// ============================================================================
// 1. Roundtrip Tests for FlatBuffers Types
// ============================================================================

// ANCHOR[contextra-wire:TST-001]
// Prüft Testfall `test_embedding_roundtrip_basic`.
#[test]
fn test_embedding_roundtrip_basic() {
    let mut builder = FlatBufferBuilder::new();
    let data_vec = builder.create_vector(&[1.0f32, -2.5f32, 3.0f32]);
    let quant_vec = builder.create_vector(&[10u8, 20u8, 30u8]);

    let offset = Embedding::create(
        &mut builder,
        &EmbeddingArgs {
            data: Some(data_vec),
            quantized_u8: Some(quant_vec),
            metric: 1,
        },
    );
    builder.finish_minimal(offset);
    let buf = builder.finished_data();

    let config = FlatBufferConfig::default();
    let emb = verify_and_parse_embedding(buf, &config).expect("Verified Embedding");
    assert_eq!(emb.metric(), 1);
    assert_eq!(emb.data().unwrap().len(), 3);
    assert_eq!(emb.quantized_u8().unwrap().len(), 3);
}

// ANCHOR[contextra-wire:TST-002]
// Prüft Testfall `test_embedding_roundtrip_quantized_only`.
#[test]
fn test_embedding_roundtrip_quantized_only() {
    let mut builder = FlatBufferBuilder::new();
    let quant_vec = builder.create_vector(&[128u8, 255u8]);

    let offset = Embedding::create(
        &mut builder,
        &EmbeddingArgs {
            data: None,
            quantized_u8: Some(quant_vec),
            metric: 2,
        },
    );
    builder.finish_minimal(offset);
    let buf = builder.finished_data();

    let config = FlatBufferConfig::default();
    let emb = verify_and_parse_embedding(buf, &config).expect("Verified Embedding");
    assert_eq!(emb.metric(), 2);
    assert!(emb.data().is_none());
    assert_eq!(emb.quantized_u8().unwrap().len(), 2);
}

// ANCHOR[contextra-wire:TST-003]
// Prüft Testfall `test_embedding_roundtrip_unquantized_only`.
#[test]
fn test_embedding_roundtrip_unquantized_only() {
    let mut builder = FlatBufferBuilder::new();
    let data_vec = builder.create_vector(&[0.5f32]);

    let offset = Embedding::create(
        &mut builder,
        &EmbeddingArgs {
            data: Some(data_vec),
            quantized_u8: None,
            metric: 0,
        },
    );
    builder.finish_minimal(offset);
    let buf = builder.finished_data();

    let config = FlatBufferConfig::default();
    let emb = verify_and_parse_embedding(buf, &config).expect("Verified Embedding");
    assert_eq!(emb.metric(), 0);
    assert_eq!(emb.data().unwrap().len(), 1);
    assert!(emb.quantized_u8().is_none());
}

// ANCHOR[contextra-wire:TST-004]
// Prüft Testfall `test_scored_document_roundtrip_basic`.
#[test]
fn test_scored_document_roundtrip_basic() {
    let mut builder = FlatBufferBuilder::new();
    let id_str = builder.create_string("doc-100");
    let meta_str = builder.create_string(r#"{"title":"test"}"#);

    let doc_offset = ScoredDocument::create(
        &mut builder,
        &ScoredDocumentArgs {
            id: Some(id_str),
            score: 0.99,
            metadata: Some(meta_str),
            embedding: None,
        },
    );
    builder.finish_minimal(doc_offset);
    let buf = builder.finished_data();

    let config = FlatBufferConfig::default();
    let doc = verify_and_parse_scored_document(buf, &config).expect("Verified ScoredDocument");
    assert_eq!(doc.id(), Some("doc-100"));
    assert_eq!(doc.score(), 0.99f32);
    assert_eq!(doc.metadata(), Some(r#"{"title":"test"}"#));
    assert!(doc.embedding().is_none());
}

// ANCHOR[contextra-wire:TST-005]
// Prüft Testfall `test_scored_document_roundtrip_without_embedding`.
#[test]
fn test_scored_document_roundtrip_without_embedding() {
    let mut builder = FlatBufferBuilder::new();
    let id_str = builder.create_string("doc-200");

    let doc_offset = ScoredDocument::create(
        &mut builder,
        &ScoredDocumentArgs {
            id: Some(id_str),
            score: 0.5,
            metadata: None,
            embedding: None,
        },
    );
    builder.finish_minimal(doc_offset);
    let buf = builder.finished_data();

    let config = FlatBufferConfig::default();
    let doc = verify_and_parse_scored_document(buf, &config).expect("Verified ScoredDocument");
    assert_eq!(doc.id(), Some("doc-200"));
    assert_eq!(doc.score(), 0.5f32);
    assert!(doc.metadata().is_none());
}

// ANCHOR[contextra-wire:TST-006]
// Prüft Testfall `test_search_response_roundtrip_basic`.
#[test]
fn test_search_response_roundtrip_basic() {
    let mut builder = FlatBufferBuilder::new();
    let id1 = builder.create_string("doc-1");
    let doc1 = ScoredDocument::create(
        &mut builder,
        &ScoredDocumentArgs {
            id: Some(id1),
            score: 0.9,
            metadata: None,
            embedding: None,
        },
    );
    let results = builder.create_vector(&[doc1]);

    let resp_offset = SearchResponse::create(
        &mut builder,
        &SearchResponseArgs {
            results: Some(results),
            total_hits: 1,
            processing_time_ms: 5.5,
        },
    );
    builder.finish_minimal(resp_offset);
    let buf = builder.finished_data();

    let config = FlatBufferConfig::default();
    let resp = verify_and_parse_search_response(buf, &config).expect("Verified SearchResponse");
    assert_eq!(resp.total_hits(), 1);
    assert_eq!(resp.results().unwrap().len(), 1);
}

// ANCHOR[contextra-wire:TST-007]
// Prüft Testfall `test_search_response_roundtrip_empty_results`.
#[test]
fn test_search_response_roundtrip_empty_results() {
    let mut builder = FlatBufferBuilder::new();
    let resp_offset = SearchResponse::create(
        &mut builder,
        &SearchResponseArgs {
            results: None,
            total_hits: 0,
            processing_time_ms: 0.1,
        },
    );
    builder.finish_minimal(resp_offset);
    let buf = builder.finished_data();

    let config = FlatBufferConfig::default();
    let resp = verify_and_parse_search_response(buf, &config).expect("Verified SearchResponse");
    assert_eq!(resp.total_hits(), 0);
    assert!(resp.results().is_none());
}

// ANCHOR[contextra-wire:TST-008]
// Prüft Testfall `test_vector_index_update_roundtrip_basic`.
#[test]
fn test_vector_index_update_roundtrip_basic() {
    let mut builder = FlatBufferBuilder::new();
    let id_str = builder.create_string("vec-1");
    let update_offset = VectorIndexUpdate::create(
        &mut builder,
        &VectorIndexUpdateArgs {
            id: Some(id_str),
            embedding: None,
            metadata: None,
        },
    );
    builder.finish_minimal(update_offset);
    let buf = builder.finished_data();

    let config = FlatBufferConfig::default();
    let update =
        verify_and_parse_vector_index_update(buf, &config).expect("Verified VectorIndexUpdate");
    assert_eq!(update.id(), Some("vec-1"));
}

// ANCHOR[contextra-wire:TST-009]
// Prüft Testfall `test_role_id_roundtrip`.
#[test]
fn test_role_id_roundtrip() {
    let mut builder = FlatBufferBuilder::new();
    let role_offset = RoleId::create(&mut builder, &RoleIdArgs { id: 42 });
    builder.finish_minimal(role_offset);
    let buf = builder.finished_data();

    let config = FlatBufferConfig::default();
    let role = verify_and_parse_role_id(buf, &config).expect("Verified RoleId");
    assert_eq!(role.id(), 42);
}

// ANCHOR[contextra-wire:TST-010]
// Prüft Testfall `test_hyper_edge_id_roundtrip`.
#[test]
fn test_hyper_edge_id_roundtrip() {
    let mut builder = FlatBufferBuilder::new();
    let edge_offset = HyperEdgeId::create(&mut builder, &HyperEdgeIdArgs { id: 1001 });
    builder.finish_minimal(edge_offset);
    let buf = builder.finished_data();

    let config = FlatBufferConfig::default();
    let edge = verify_and_parse_hyper_edge_id(buf, &config).expect("Verified HyperEdgeId");
    assert_eq!(edge.id(), 1001);
}

// ANCHOR[contextra-wire:TST-011]
// Prüft Testfall `test_role_binding_roundtrip`.
#[test]
fn test_role_binding_roundtrip() {
    let mut builder = FlatBufferBuilder::new();
    let binding_offset = RoleBinding::create(
        &mut builder,
        &RoleBindingArgs {
            role: 5,
            entity: 77,
        },
    );
    builder.finish_minimal(binding_offset);
    let buf = builder.finished_data();

    let config = FlatBufferConfig::default();
    let binding = verify_and_parse_role_binding(buf, &config).expect("Verified RoleBinding");
    assert_eq!(binding.role(), 5);
    assert_eq!(binding.entity(), 77);
}

// ANCHOR[contextra-wire:TST-012]
// Prüft Testfall `test_hyper_edge_roundtrip`.
#[test]
fn test_hyper_edge_roundtrip() {
    let mut builder = FlatBufferBuilder::new();

    let binding = RoleBinding::create(
        &mut builder,
        &RoleBindingArgs {
            role: 1,
            entity: 100,
        },
    );
    let participants_vec = builder.create_vector(&[binding]);

    let edge_offset = HyperEdge::create(
        &mut builder,
        &HyperEdgeArgs {
            id: 99,
            predicate: 2,
            participants: Some(participants_vec),
            weight: 1.5,
            tx_valid_from: 10,
            tx_valid_to: 20,
            business_valid_from: 1000,
            business_valid_to: 2000,
            source_doc_id: 500,
            child_edge_ids: None,
        },
    );
    builder.finish_minimal(edge_offset);
    let buf = builder.finished_data();

    let config = FlatBufferConfig::default();
    let edge = verify_and_parse_hyper_edge(buf, &config).expect("Verified HyperEdge");
    assert_eq!(edge.id(), 99);
    assert_eq!(edge.predicate(), 2);
    assert_eq!(edge.weight(), 1.5);
    assert_eq!(edge.participants().unwrap().len(), 1);
}

// ============================================================================
// 2. WireBuffer Zero-Copy Adapter Tests
// ============================================================================

// ANCHOR[contextra-wire:TST-013]
// Prüft Testfall `test_wire_buffer_adapter_creation_and_conversion`.
#[test]
fn test_wire_buffer_adapter_creation_and_conversion() {
    let bytes = Bytes::from_static(b"hello_wire");
    let wire_buf = WireBuffer::new(bytes.clone());
    assert_eq!(wire_buf.as_slice(), b"hello_wire");
    assert_eq!(wire_buf.into_inner(), bytes);
}

// ANCHOR[contextra-wire:TST-014]
// Prüft Testfall `test_wire_buffer_as_ref`.
#[test]
fn test_wire_buffer_as_ref() {
    let wire_buf = WireBuffer::new(Bytes::from_static(b"test_payload"));
    let slice: &[u8] = wire_buf.as_ref();
    assert_eq!(slice, b"test_payload");
}

// ============================================================================
// 3. Mutation and Truncation Fuzzing (≥ 5,000 Iterations per Type)
// ============================================================================

// ANCHOR[contextra-wire:TST-015]
// Prüft Testfall `test_fuzz_mutation_embedding_5000_iters`.
#[test]
fn test_fuzz_mutation_embedding_5000_iters() {
    let mut builder = FlatBufferBuilder::new();
    let data_vec = builder.create_vector(&[0.1f32, 0.2f32, 0.3f32]);
    let emb_offset = Embedding::create(
        &mut builder,
        &EmbeddingArgs {
            data: Some(data_vec),
            quantized_u8: None,
            metric: 1,
        },
    );
    builder.finish_minimal(emb_offset);
    let valid_buf = builder.finished_data().to_vec();

    let mut lcg = SimpleLcg::new(0x1111222233334444);
    let config = FlatBufferConfig::default();

    for _ in 0..5000 {
        let mut mutated = valid_buf.clone();
        let mut_pos = lcg.next_usize(mutated.len());
        if !mutated.is_empty() {
            mutated[mut_pos] ^= lcg.next_u8();
        }
        let _ = verify_and_parse_embedding(&mutated, &config);
    }
}

// ANCHOR[contextra-wire:TST-016]
// Prüft Testfall `test_fuzz_truncation_embedding_5000_iters`.
#[test]
fn test_fuzz_truncation_embedding_5000_iters() {
    let mut builder = FlatBufferBuilder::new();
    let quant_vec = builder.create_vector(&[1u8, 2u8, 3u8, 4u8, 5u8]);
    let emb_offset = Embedding::create(
        &mut builder,
        &EmbeddingArgs {
            data: None,
            quantized_u8: Some(quant_vec),
            metric: 2,
        },
    );
    builder.finish_minimal(emb_offset);
    let valid_buf = builder.finished_data().to_vec();

    let mut lcg = SimpleLcg::new(0x5555666677778888);
    let config = FlatBufferConfig::default();

    for _ in 0..5000 {
        let trunc_len = lcg.next_usize(valid_buf.len());
        let _ = verify_and_parse_embedding(&valid_buf[..trunc_len], &config);
    }
}

// ANCHOR[contextra-wire:TST-017]
// Prüft Testfall `test_fuzz_mutation_scored_document_5000_iters`.
#[test]
fn test_fuzz_mutation_scored_document_5000_iters() {
    let mut builder = FlatBufferBuilder::new();
    let id_str = builder.create_string("fuzz-doc");
    let doc_offset = ScoredDocument::create(
        &mut builder,
        &ScoredDocumentArgs {
            id: Some(id_str),
            score: 0.85,
            metadata: None,
            embedding: None,
        },
    );
    builder.finish_minimal(doc_offset);
    let valid_buf = builder.finished_data().to_vec();

    let mut lcg = SimpleLcg::new(0x9999AAAABBBBCCCC);
    let config = FlatBufferConfig::default();

    for _ in 0..5000 {
        let mut mutated = valid_buf.clone();
        let mut_pos = lcg.next_usize(mutated.len());
        if !mutated.is_empty() {
            mutated[mut_pos] = lcg.next_u8();
        }
        let _ = verify_and_parse_scored_document(&mutated, &config);
    }
}

// ANCHOR[contextra-wire:TST-018]
// Prüft Testfall `test_fuzz_truncation_scored_document_5000_iters`.
#[test]
fn test_fuzz_truncation_scored_document_5000_iters() {
    let mut builder = FlatBufferBuilder::new();
    let id_str = builder.create_string("trunc-doc");
    let doc_offset = ScoredDocument::create(
        &mut builder,
        &ScoredDocumentArgs {
            id: Some(id_str),
            score: 0.77,
            metadata: None,
            embedding: None,
        },
    );
    builder.finish_minimal(doc_offset);
    let valid_buf = builder.finished_data().to_vec();

    let mut lcg = SimpleLcg::new(0xDDDDEEEEFFFF0000);
    let config = FlatBufferConfig::default();

    for _ in 0..5000 {
        let trunc_len = lcg.next_usize(valid_buf.len());
        let _ = verify_and_parse_scored_document(&valid_buf[..trunc_len], &config);
    }
}

// ANCHOR[contextra-wire:TST-019]
// Prüft Testfall `test_fuzz_mutation_search_response_5000_iters`.
#[test]
fn test_fuzz_mutation_search_response_5000_iters() {
    let mut builder = FlatBufferBuilder::new();
    let resp_offset = SearchResponse::create(
        &mut builder,
        &SearchResponseArgs {
            results: None,
            total_hits: 42,
            processing_time_ms: 12.3,
        },
    );
    builder.finish_minimal(resp_offset);
    let valid_buf = builder.finished_data().to_vec();

    let mut lcg = SimpleLcg::new(0x123456789ABCDEF0);
    let config = FlatBufferConfig::default();

    for _ in 0..5000 {
        let mut mutated = valid_buf.clone();
        let mut_pos = lcg.next_usize(mutated.len());
        if !mutated.is_empty() {
            mutated[mut_pos] ^= lcg.next_u8();
        }
        let _ = verify_and_parse_search_response(&mutated, &config);
    }
}

// ANCHOR[contextra-wire:TST-020]
// Prüft Testfall `test_fuzz_truncation_search_response_5000_iters`.
#[test]
fn test_fuzz_truncation_search_response_5000_iters() {
    let mut builder = FlatBufferBuilder::new();
    let resp_offset = SearchResponse::create(
        &mut builder,
        &SearchResponseArgs {
            results: None,
            total_hits: 100,
            processing_time_ms: 1.0,
        },
    );
    builder.finish_minimal(resp_offset);
    let valid_buf = builder.finished_data().to_vec();

    let mut lcg = SimpleLcg::new(0x0FEDCBA987654321);
    let config = FlatBufferConfig::default();

    for _ in 0..5000 {
        let trunc_len = lcg.next_usize(valid_buf.len());
        let _ = verify_and_parse_search_response(&valid_buf[..trunc_len], &config);
    }
}

// ANCHOR[contextra-wire:TST-021]
// Prüft Testfall `test_fuzz_mutation_vector_index_update_5000_iters`.
#[test]
fn test_fuzz_mutation_vector_index_update_5000_iters() {
    let mut builder = FlatBufferBuilder::new();
    let id_str = builder.create_string("fuzz-vec-update");
    let update_offset = VectorIndexUpdate::create(
        &mut builder,
        &VectorIndexUpdateArgs {
            id: Some(id_str),
            embedding: None,
            metadata: None,
        },
    );
    builder.finish_minimal(update_offset);
    let valid_buf = builder.finished_data().to_vec();

    let mut lcg = SimpleLcg::new(0xA1B2C3D4E5F60718);
    let config = FlatBufferConfig::default();

    for _ in 0..5000 {
        let mut mutated = valid_buf.clone();
        let mut_pos = lcg.next_usize(mutated.len());
        if !mutated.is_empty() {
            mutated[mut_pos] = lcg.next_u8();
        }
        let _ = verify_and_parse_vector_index_update(&mutated, &config);
    }
}

// ANCHOR[contextra-wire:TST-022]
// Prüft Testfall `test_fuzz_truncation_vector_index_update_5000_iters`.
#[test]
fn test_fuzz_truncation_vector_index_update_5000_iters() {
    let mut builder = FlatBufferBuilder::new();
    let id_str = builder.create_string("trunc-vec-update");
    let update_offset = VectorIndexUpdate::create(
        &mut builder,
        &VectorIndexUpdateArgs {
            id: Some(id_str),
            embedding: None,
            metadata: None,
        },
    );
    builder.finish_minimal(update_offset);
    let valid_buf = builder.finished_data().to_vec();

    let mut lcg = SimpleLcg::new(0x2468ACE013579BDF);
    let config = FlatBufferConfig::default();

    for _ in 0..5000 {
        let trunc_len = lcg.next_usize(valid_buf.len());
        let _ = verify_and_parse_vector_index_update(&valid_buf[..trunc_len], &config);
    }
}

// ANCHOR[contextra-wire:TST-023]
// Prüft Testfall `test_fuzz_mutation_role_id_5000_iters`.
#[test]
fn test_fuzz_mutation_role_id_5000_iters() {
    let mut builder = FlatBufferBuilder::new();
    let role_offset = RoleId::create(&mut builder, &RoleIdArgs { id: 123 });
    builder.finish_minimal(role_offset);
    let valid_buf = builder.finished_data().to_vec();

    let mut lcg = SimpleLcg::new(0xFEDCBA9876543210);
    let config = FlatBufferConfig::default();

    for _ in 0..5000 {
        let mut mutated = valid_buf.clone();
        let mut_pos = lcg.next_usize(mutated.len());
        if !mutated.is_empty() {
            mutated[mut_pos] ^= lcg.next_u8();
        }
        let _ = verify_and_parse_role_id(&mutated, &config);
    }
}

// ANCHOR[contextra-wire:TST-024]
// Prüft Testfall `test_fuzz_truncation_role_id_5000_iters`.
#[test]
fn test_fuzz_truncation_role_id_5000_iters() {
    let mut builder = FlatBufferBuilder::new();
    let role_offset = RoleId::create(&mut builder, &RoleIdArgs { id: 456 });
    builder.finish_minimal(role_offset);
    let valid_buf = builder.finished_data().to_vec();

    let mut lcg = SimpleLcg::new(0x0123456789ABCDEF);
    let config = FlatBufferConfig::default();

    for _ in 0..5000 {
        let trunc_len = lcg.next_usize(valid_buf.len());
        let _ = verify_and_parse_role_id(&valid_buf[..trunc_len], &config);
    }
}

// ANCHOR[contextra-wire:TST-025]
// Prüft Testfall `test_fuzz_mutation_hyper_edge_5000_iters`.
#[test]
fn test_fuzz_mutation_hyper_edge_5000_iters() {
    let mut builder = FlatBufferBuilder::new();
    let edge_offset = HyperEdge::create(
        &mut builder,
        &HyperEdgeArgs {
            id: 88,
            predicate: 1,
            participants: None,
            weight: 2.5,
            tx_valid_from: 0,
            tx_valid_to: 0,
            business_valid_from: 0,
            business_valid_to: 0,
            source_doc_id: 0,
            child_edge_ids: None,
        },
    );
    builder.finish_minimal(edge_offset);
    let valid_buf = builder.finished_data().to_vec();

    let mut lcg = SimpleLcg::new(0x777788889999AAAA);
    let config = FlatBufferConfig::default();

    for _ in 0..5000 {
        let mut mutated = valid_buf.clone();
        let mut_pos = lcg.next_usize(mutated.len());
        if !mutated.is_empty() {
            mutated[mut_pos] = lcg.next_u8();
        }
        let _ = verify_and_parse_hyper_edge(&mutated, &config);
    }
}

// ANCHOR[contextra-wire:TST-026]
// Prüft Testfall `test_fuzz_truncation_hyper_edge_5000_iters`.
#[test]
fn test_fuzz_truncation_hyper_edge_5000_iters() {
    let mut builder = FlatBufferBuilder::new();
    let edge_offset = HyperEdge::create(
        &mut builder,
        &HyperEdgeArgs {
            id: 99,
            predicate: 2,
            participants: None,
            weight: 3.0,
            tx_valid_from: 0,
            tx_valid_to: 0,
            business_valid_from: 0,
            business_valid_to: 0,
            source_doc_id: 0,
            child_edge_ids: None,
        },
    );
    builder.finish_minimal(edge_offset);
    let valid_buf = builder.finished_data().to_vec();

    let mut lcg = SimpleLcg::new(0xBBBBCCCCDDDDEEEE);
    let config = FlatBufferConfig::default();

    for _ in 0..5000 {
        let trunc_len = lcg.next_usize(valid_buf.len());
        let _ = verify_and_parse_hyper_edge(&valid_buf[..trunc_len], &config);
    }
}

// ANCHOR[contextra-wire:TST-027]
// Prüft Testfall `test_fuzz_jsonrpc_mutation_5000_iters`.
#[test]
fn test_fuzz_jsonrpc_mutation_5000_iters() {
    let valid_json = r#"{"jsonrpc": "2.0", "id": 1, "method": "search", "params": {"q": "rust"}}"#;
    let valid_bytes = valid_json.as_bytes().to_vec();

    let mut lcg = SimpleLcg::new(0x3333444455556666);

    for _ in 0..5000 {
        let mut mutated = valid_bytes.clone();
        let mut_pos = lcg.next_usize(mutated.len());
        if !mutated.is_empty() {
            mutated[mut_pos] ^= lcg.next_u8();
        }
        let _ = parse_jsonrpc_payload(&mutated);
    }
}

// ============================================================================
// 4. JSON-RPC Protocol & Error Mapping Tests
// ============================================================================

// ANCHOR[contextra-wire:TST-028]
// Prüft Testfall `test_jsonrpc_single_request_roundtrip`.
#[test]
fn test_jsonrpc_single_request_roundtrip() {
    let raw = r#"{"jsonrpc": "2.0", "id": "req-1", "method": "ping"}"#;
    let payload = parse_jsonrpc_payload(raw.as_bytes()).expect("parse single");
    if let JsonRpcPayload::Single(req) = payload {
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "ping");
    } else {
        panic!("Expected single request");
    }
}

// ANCHOR[contextra-wire:TST-029]
// Prüft Testfall `test_jsonrpc_notification_roundtrip`.
#[test]
fn test_jsonrpc_notification_roundtrip() {
    let raw = r#"{"jsonrpc": "2.0", "method": "notify_event"}"#;
    let payload = parse_jsonrpc_payload(raw.as_bytes()).expect("parse notification");
    if let JsonRpcPayload::Single(req) = payload {
        assert_eq!(req.method, "notify_event");
        assert!(req.id.is_none());
    } else {
        panic!("Expected notification");
    }
}

// ANCHOR[contextra-wire:TST-030]
// Prüft Testfall `test_jsonrpc_batch_request_roundtrip`.
#[test]
fn test_jsonrpc_batch_request_roundtrip() {
    let raw = r#"[
        {"jsonrpc": "2.0", "id": 1, "method": "m1"},
        {"jsonrpc": "2.0", "id": 2, "method": "m2"}
    ]"#;
    let payload = parse_jsonrpc_payload(raw.as_bytes()).expect("parse batch");
    if let JsonRpcPayload::Batch(reqs) = payload {
        assert_eq!(reqs.len(), 2);
    } else {
        panic!("Expected batch request");
    }
}

// ANCHOR[contextra-wire:TST-031]
// Prüft Testfall `test_jsonrpc_error_response_formatting`.
#[test]
fn test_jsonrpc_error_response_formatting() {
    let resp = JsonRpcResponse::err(None, -32700, "Parse error");
    assert_eq!(resp.error.unwrap().code, -32700);
}

// ANCHOR[contextra-wire:TST-032]
// Prüft Testfall `test_jsonrpc_parse_error_code_minus_32700`.
#[test]
fn test_jsonrpc_parse_error_code_minus_32700() {
    let err = JsonRpcError::parse_error("invalid syntax");
    assert_eq!(err.code, -32700);
    assert_eq!(err.message, "invalid syntax");
}

// ANCHOR[contextra-wire:TST-033]
// Prüft Testfall `test_jsonrpc_invalid_request_code_minus_32600`.
#[test]
fn test_jsonrpc_invalid_request_code_minus_32600() {
    let err = JsonRpcError::invalid_request("bad request payload");
    assert_eq!(err.code, -32600);
}

// ANCHOR[contextra-wire:TST-034]
// Prüft Testfall `test_jsonrpc_method_not_found_code_minus_32601`.
#[test]
fn test_jsonrpc_method_not_found_code_minus_32601() {
    let err = JsonRpcError::method_not_found("unknown method");
    assert_eq!(err.code, -32601);
}

// ANCHOR[contextra-wire:TST-035]
// Prüft Testfall `test_jsonrpc_invalid_params_code_minus_32602`.
#[test]
fn test_jsonrpc_invalid_params_code_minus_32602() {
    let err = JsonRpcError::invalid_params("param type mismatch");
    assert_eq!(err.code, -32602);
}

// ANCHOR[contextra-wire:TST-036]
// Prüft Testfall `test_jsonrpc_internal_error_code_minus_32603`.
#[test]
fn test_jsonrpc_internal_error_code_minus_32603() {
    let err = JsonRpcError::internal_error("internal database fault");
    assert_eq!(err.code, -32603);
}

// ANCHOR[contextra-wire:TST-037]
// Prüft Testfall `test_jsonrpc_oversized_payload`.
#[test]
fn test_jsonrpc_oversized_payload() {
    let config = JsonRpcConfig {
        max_message_size: 15,
        ..Default::default()
    };
    let raw = r#"{"jsonrpc": "2.0", "id": 1, "method": "ping"}"#;
    let err = parse_jsonrpc_payload_with_config(raw.as_bytes(), &config).unwrap_err();
    assert_eq!(err.code, -32700);
}

// ANCHOR[contextra-wire:TST-038]
// Prüft Testfall `test_jsonrpc_structural_depth_bomb`.
#[test]
fn test_jsonrpc_structural_depth_bomb() {
    let config = JsonRpcConfig {
        max_depth: 2,
        ..Default::default()
    };
    let raw = r#"{"a": {"b": {"c": 1}}}"#;
    let err = parse_jsonrpc_payload_with_config(raw.as_bytes(), &config).unwrap_err();
    assert_eq!(err.code, -32700);
}

// ANCHOR[contextra-wire:TST-039]
// Prüft Testfall `test_jsonrpc_empty_batch_array`.
#[test]
fn test_jsonrpc_empty_batch_array() {
    let raw = "[]";
    let err = parse_jsonrpc_payload(raw.as_bytes()).unwrap_err();
    assert_eq!(err.code, -32600);
}

// ANCHOR[contextra-wire:TST-040]
// Prüft Testfall `test_jsonrpc_batch_size_limit`.
#[test]
fn test_jsonrpc_batch_size_limit() {
    let config = JsonRpcConfig {
        max_batch_size: 1,
        ..Default::default()
    };
    let raw = r#"[
        {"jsonrpc": "2.0", "id": 1, "method": "m1"},
        {"jsonrpc": "2.0", "id": 2, "method": "m2"}
    ]"#;
    let err = parse_jsonrpc_payload_with_config(raw.as_bytes(), &config).unwrap_err();
    assert_eq!(err.code, -32600);
}

// ANCHOR[contextra-wire:TST-041]
// Prüft Testfall `test_jsonrpc_duplicate_keys_detection`.
#[test]
fn test_jsonrpc_duplicate_keys_detection() {
    let raw = r#"{"jsonrpc": "2.0", "id": 1, "id": 2, "method": "ping"}"#;
    let err = parse_jsonrpc_payload(raw.as_bytes()).unwrap_err();
    assert_eq!(err.code, -32700);
}

// ANCHOR[contextra-wire:TST-042]
// Prüft Testfall `test_jsonrpc_invalid_utf8_sequence`.
#[test]
fn test_jsonrpc_invalid_utf8_sequence() {
    let bad_utf8 = vec![0x80, 0x81, 0x82];
    let err = parse_jsonrpc_payload(&bad_utf8).unwrap_err();
    assert_eq!(err.code, -32700);
}

// ANCHOR[contextra-wire:TST-043]
// Prüft Testfall `test_jsonrpc_missing_method_field`.
#[test]
fn test_jsonrpc_missing_method_field() {
    let raw = r#"{"jsonrpc": "2.0", "id": 1}"#;
    let err = parse_jsonrpc_payload(raw.as_bytes()).unwrap_err();
    assert_eq!(err.code, -32600);
}

// ANCHOR[contextra-wire:TST-044]
// Prüft Testfall `test_jsonrpc_whitespace_method`.
#[test]
fn test_jsonrpc_whitespace_method() {
    let req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: None,
        method: "   \t\n".to_string(),
        params: serde_json::Value::Null,
    };
    let err = req.validate().unwrap_err();
    assert_eq!(err.code, -32600);
}

// ANCHOR[contextra-wire:TST-045]
// Prüft Testfall `test_jsonrpc_invalid_version_1_0`.
#[test]
fn test_jsonrpc_invalid_version_1_0() {
    let req = JsonRpcRequest {
        jsonrpc: "1.0".to_string(),
        id: None,
        method: "ping".to_string(),
        params: serde_json::Value::Null,
    };
    let err = req.validate().unwrap_err();
    assert_eq!(err.code, -32600);
}

// ANCHOR[contextra-wire:TST-046]
// Prüft Testfall `test_jsonrpc_invalid_version_3_0`.
#[test]
fn test_jsonrpc_invalid_version_3_0() {
    let req = JsonRpcRequest {
        jsonrpc: "3.0".to_string(),
        id: None,
        method: "ping".to_string(),
        params: serde_json::Value::Null,
    };
    let err = req.validate().unwrap_err();
    assert_eq!(err.code, -32600);
}

// ============================================================================
// 5. FlatBuffers Verification Specific Tests
// ============================================================================

// ANCHOR[contextra-wire:TST-047]
// Prüft Testfall `test_flatbuffers_verification_search_response_valid`.
#[test]
fn test_flatbuffers_verification_search_response_valid() {
    let mut builder = FlatBufferBuilder::new();
    let resp = SearchResponse::create(
        &mut builder,
        &SearchResponseArgs {
            results: None,
            total_hits: 10,
            processing_time_ms: 2.0,
        },
    );
    builder.finish_minimal(resp);
    let buf = builder.finished_data();

    let config = FlatBufferConfig::default();
    assert!(verify_and_parse_search_response(buf, &config).is_ok());
}

// ANCHOR[contextra-wire:TST-048]
// Prüft Testfall `test_flatbuffers_verification_search_response_invalid`.
#[test]
fn test_flatbuffers_verification_search_response_invalid() {
    let config = FlatBufferConfig::default();
    let garbage = vec![0x00, 0x01, 0x02, 0x03];
    assert!(verify_and_parse_search_response(&garbage, &config).is_err());
}

// ANCHOR[contextra-wire:TST-049]
// Prüft Testfall `test_flatbuffers_verification_embedding_valid`.
#[test]
fn test_flatbuffers_verification_embedding_valid() {
    let mut builder = FlatBufferBuilder::new();
    let emb = Embedding::create(
        &mut builder,
        &EmbeddingArgs {
            data: None,
            quantized_u8: None,
            metric: 1,
        },
    );
    builder.finish_minimal(emb);
    let buf = builder.finished_data();

    let config = FlatBufferConfig::default();
    assert!(verify_and_parse_embedding(buf, &config).is_ok());
}

// ANCHOR[contextra-wire:TST-050]
// Prüft Testfall `test_flatbuffers_verification_embedding_invalid`.
#[test]
fn test_flatbuffers_verification_embedding_invalid() {
    let config = FlatBufferConfig::default();
    assert!(verify_and_parse_embedding(&[0xFF; 8], &config).is_err());
}

// ANCHOR[contextra-wire:TST-051]
// Prüft Testfall `test_flatbuffers_verification_scored_document_valid`.
#[test]
fn test_flatbuffers_verification_scored_document_valid() {
    let mut builder = FlatBufferBuilder::new();
    let doc = ScoredDocument::create(
        &mut builder,
        &ScoredDocumentArgs {
            id: None,
            score: 0.1,
            metadata: None,
            embedding: None,
        },
    );
    builder.finish_minimal(doc);
    let buf = builder.finished_data();

    let config = FlatBufferConfig::default();
    assert!(verify_and_parse_scored_document(buf, &config).is_ok());
}

// ANCHOR[contextra-wire:TST-052]
// Prüft Testfall `test_flatbuffers_verification_scored_document_invalid`.
#[test]
fn test_flatbuffers_verification_scored_document_invalid() {
    let config = FlatBufferConfig::default();
    assert!(verify_and_parse_scored_document(&[0x01, 0x02], &config).is_err());
}

// ANCHOR[contextra-wire:TST-053]
// Prüft Testfall `test_flatbuffers_verification_vector_index_update_valid`.
#[test]
fn test_flatbuffers_verification_vector_index_update_valid() {
    let mut builder = FlatBufferBuilder::new();
    let update = VectorIndexUpdate::create(
        &mut builder,
        &VectorIndexUpdateArgs {
            id: None,
            embedding: None,
            metadata: None,
        },
    );
    builder.finish_minimal(update);
    let buf = builder.finished_data();

    let config = FlatBufferConfig::default();
    assert!(verify_and_parse_vector_index_update(buf, &config).is_ok());
}

// ANCHOR[contextra-wire:TST-054]
// Prüft Testfall `test_flatbuffers_verification_vector_index_update_invalid`.
#[test]
fn test_flatbuffers_verification_vector_index_update_invalid() {
    let config = FlatBufferConfig::default();
    assert!(verify_and_parse_vector_index_update(&[0xFA, 0xFB], &config).is_err());
}

// ANCHOR[contextra-wire:TST-055]
// Prüft Testfall `test_flatbuffers_verification_role_id_valid`.
#[test]
fn test_flatbuffers_verification_role_id_valid() {
    let mut builder = FlatBufferBuilder::new();
    let role = RoleId::create(&mut builder, &RoleIdArgs { id: 1 });
    builder.finish_minimal(role);
    let buf = builder.finished_data();

    let config = FlatBufferConfig::default();
    assert!(verify_and_parse_role_id(buf, &config).is_ok());
}

// ANCHOR[contextra-wire:TST-056]
// Prüft Testfall `test_flatbuffers_verification_role_id_invalid`.
#[test]
fn test_flatbuffers_verification_role_id_invalid() {
    let config = FlatBufferConfig::default();
    assert!(verify_and_parse_role_id(&[0x00], &config).is_err());
}

// ANCHOR[contextra-wire:TST-057]
// Prüft Testfall `test_flatbuffers_verification_hyper_edge_id_valid`.
#[test]
fn test_flatbuffers_verification_hyper_edge_id_valid() {
    let mut builder = FlatBufferBuilder::new();
    let edge = HyperEdgeId::create(&mut builder, &HyperEdgeIdArgs { id: 2 });
    builder.finish_minimal(edge);
    let buf = builder.finished_data();

    let config = FlatBufferConfig::default();
    assert!(verify_and_parse_hyper_edge_id(buf, &config).is_ok());
}

// ANCHOR[contextra-wire:TST-058]
// Prüft Testfall `test_flatbuffers_verification_hyper_edge_id_invalid`.
#[test]
fn test_flatbuffers_verification_hyper_edge_id_invalid() {
    let config = FlatBufferConfig::default();
    assert!(verify_and_parse_hyper_edge_id(&[0xAA, 0xBB], &config).is_err());
}

// ANCHOR[contextra-wire:TST-059]
// Prüft Testfall `test_flatbuffers_verification_role_binding_valid`.
#[test]
fn test_flatbuffers_verification_role_binding_valid() {
    let mut builder = FlatBufferBuilder::new();
    let binding = RoleBinding::create(
        &mut builder,
        &RoleBindingArgs {
            role: 1,
            entity: 10,
        },
    );
    builder.finish_minimal(binding);
    let buf = builder.finished_data();

    let config = FlatBufferConfig::default();
    assert!(verify_and_parse_role_binding(buf, &config).is_ok());
}

// ANCHOR[contextra-wire:TST-060]
// Prüft Testfall `test_flatbuffers_verification_role_binding_invalid`.
#[test]
fn test_flatbuffers_verification_role_binding_invalid() {
    let config = FlatBufferConfig::default();
    assert!(verify_and_parse_role_binding(&[0xCC, 0xDD], &config).is_err());
}

// ANCHOR[contextra-wire:TST-061]
// Prüft Testfall `test_flatbuffers_verification_hyper_edge_valid`.
#[test]
fn test_flatbuffers_verification_hyper_edge_valid() {
    let mut builder = FlatBufferBuilder::new();
    let edge = HyperEdge::create(
        &mut builder,
        &HyperEdgeArgs {
            id: 10,
            predicate: 0,
            participants: None,
            weight: 1.0,
            tx_valid_from: 0,
            tx_valid_to: 0,
            business_valid_from: 0,
            business_valid_to: 0,
            source_doc_id: 0,
            child_edge_ids: None,
        },
    );
    builder.finish_minimal(edge);
    let buf = builder.finished_data();

    let config = FlatBufferConfig::default();
    assert!(verify_and_parse_hyper_edge(buf, &config).is_ok());
}

// ANCHOR[contextra-wire:TST-062]
// Prüft Testfall `test_flatbuffers_verification_hyper_edge_invalid`.
#[test]
fn test_flatbuffers_verification_hyper_edge_invalid() {
    let config = FlatBufferConfig::default();
    assert!(verify_and_parse_hyper_edge(&[0x11, 0x22], &config).is_err());
}

// ============================================================================
// 6. Concurrency and Parallel Processing Tests
// ============================================================================

// ANCHOR[contextra-wire:TST-063]
// Prüft Testfall `test_concurrency_parallel_serialization`.
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
                    builder.finish_minimal(resp);

                    let buf = builder.finished_data();
                    let config = FlatBufferConfig::default();
                    let parsed = verify_and_parse_search_response(buf, &config)
                        .expect("Parallel verification failed");
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

// ANCHOR[contextra-wire:TST-064]
// Prüft Testfall `test_shared_buffer_concurrency`.
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
    builder.finish_minimal(resp);

    let shared_buf = Arc::new(builder.finished_data().to_vec());

    let threads: Vec<_> = (0..8)
        .map(|_| {
            let buf = Arc::clone(&shared_buf);
            thread::spawn(move || {
                let config = FlatBufferConfig::default();
                for _ in 0..200 {
                    let parsed = verify_and_parse_search_response(&buf, &config)
                        .expect("Shared buffer parse failed");
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

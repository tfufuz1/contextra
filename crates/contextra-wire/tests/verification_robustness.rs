//! # Robustheits- und Adversarial-Tests für `verification` und `jsonrpc`
//!
//! **Zweck:**
//! Deterministische Garbage-, Truncation- und Bit-Flip-Eingaben gegen die FlatBuffers-
//! Verifizierungsroutinen (`verification.rs`) und den JSON-RPC 2.0 Parser (`jsonrpc.rs`).
//!
//! **Invariante (P7 Zero-Panic, REQ-WIR-002):**
//! Alle ungültigen, unvollständigen oder bösartigen Eingaben müssen deterministisch
//! abgelehnt werden. Es dürfen unter keinen Umständen Panics, Speicherüberläufe
//! oder undefiniertes Verhalten auftreten.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use contextra_wire::{
    parse_jsonrpc_payload, parse_jsonrpc_payload_with_config, verify_and_parse_embedding,
    verify_and_parse_hyper_edge, verify_and_parse_hyper_edge_id, verify_and_parse_role_binding,
    verify_and_parse_role_id, verify_and_parse_scored_document, verify_and_parse_search_response,
    verify_and_parse_vector_index_update, FlatBufferConfig, JsonRpcConfig,
};
use flatbuffers::FlatBufferBuilder;

// Hilfsfunktion: Pseudo-Zufallsgenerator für deterministische Tests (SplitMix64)
struct TestPrng {
    state: u64,
}

impl TestPrng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    fn fill_bytes(&mut self, buf: &mut [u8]) {
        for chunk in buf.chunks_mut(8) {
            let val = self.next_u64().to_le_bytes();
            let len = chunk.len();
            chunk.copy_from_slice(&val[..len]);
        }
    }
}

// Baut eine gültige SearchResponse für Mutationstests
fn build_sample_search_response() -> Vec<u8> {
    use contextra_wire::contextra_generated::contextra::ipc::{
        Embedding, EmbeddingArgs, ScoredDocument, ScoredDocumentArgs, SearchResponse,
        SearchResponseArgs,
    };

    let mut b = FlatBufferBuilder::new();
    let emb_vec = b.create_vector(&[1.0f32, 2.0f32, 3.0f32]);
    let emb = Embedding::create(
        &mut b,
        &EmbeddingArgs {
            data: Some(emb_vec),
            quantized_u8: None,
            metric: 0,
        },
    );
    let id_str = b.create_string("doc-123");
    let meta_str = b.create_string("{\"category\": \"test\"}");
    let doc = ScoredDocument::create(
        &mut b,
        &ScoredDocumentArgs {
            id: Some(id_str),
            score: 0.95,
            metadata: Some(meta_str),
            embedding: Some(emb),
        },
    );
    let docs = b.create_vector(&[doc]);
    let resp = SearchResponse::create(
        &mut b,
        &SearchResponseArgs {
            results: Some(docs),
            total_hits: 1,
            processing_time_ms: 12.5,
        },
    );
    b.finish(resp, None);
    b.finished_data().to_vec()
}

// Baut eine gültige HyperEdge für Mutationstests
fn build_sample_hyper_edge() -> Vec<u8> {
    use contextra_wire::contextra_generated::contextra::ipc::{
        HyperEdge, HyperEdgeArgs, RoleBinding, RoleBindingArgs,
    };

    let mut b = FlatBufferBuilder::new();
    let rb1 = RoleBinding::create(
        &mut b,
        &RoleBindingArgs {
            role: 1,
            entity: 100,
        },
    );
    let rb2 = RoleBinding::create(
        &mut b,
        &RoleBindingArgs {
            role: 2,
            entity: 200,
        },
    );
    let parts = b.create_vector(&[rb1, rb2]);
    let children = b.create_vector(&[10u64, 20u64]);
    let edge = HyperEdge::create(
        &mut b,
        &HyperEdgeArgs {
            id: 42,
            predicate: 1,
            participants: Some(parts),
            weight: 1.0,
            tx_valid_from: 1000,
            tx_valid_to: 2000,
            business_valid_from: 500,
            business_valid_to: 1500,
            source_doc_id: 99,
            child_edge_ids: Some(children),
        },
    );
    b.finish(edge, None);
    b.finished_data().to_vec()
}

#[test]
fn test_verification_garbage_inputs_all_types() {
    let mut rng = TestPrng::new(0x1337_C0DE_CAFE);
    let config = FlatBufferConfig::default();

    for len in [1, 2, 3, 4, 7, 8, 15, 16, 31, 32, 64, 128, 512, 1024, 4096] {
        for _ in 0..20 {
            let mut garbage = vec![0u8; len];
            rng.fill_bytes(&mut garbage);

            // Keine dieser Funktionen darf panicken
            let _ = verify_and_parse_search_response(&garbage, &config);
            let _ = verify_and_parse_embedding(&garbage, &config);
            let _ = verify_and_parse_scored_document(&garbage, &config);
            let _ = verify_and_parse_vector_index_update(&garbage, &config);
            let _ = verify_and_parse_role_id(&garbage, &config);
            let _ = verify_and_parse_hyper_edge_id(&garbage, &config);
            let _ = verify_and_parse_role_binding(&garbage, &config);
            let _ = verify_and_parse_hyper_edge(&garbage, &config);
        }
    }
}

#[test]
fn test_verification_all_zeros_and_all_ones() {
    let config = FlatBufferConfig::default();
    for len in [1, 4, 8, 16, 32, 64, 128, 256, 1024] {
        let zeros = vec![0u8; len];
        let ones = vec![0xFFu8; len];

        let _ = verify_and_parse_search_response(&zeros, &config);
        let _ = verify_and_parse_search_response(&ones, &config);
        let _ = verify_and_parse_hyper_edge(&zeros, &config);
        let _ = verify_and_parse_hyper_edge(&ones, &config);
    }
}

#[test]
fn test_verification_truncation_every_byte() {
    let valid_data = build_sample_search_response();
    let config = FlatBufferConfig::default();

    // Verifiziere zuerst, dass das Original gültig ist
    assert!(verify_and_parse_search_response(&valid_data, &config).is_ok());

    // Schneide an jedem einzelnen Byte ab
    for truncate_len in 0..valid_data.len() {
        let truncated = &valid_data[..truncate_len];
        // Jede abgeschnittene Variante muss sauber abgefangen werden
        let res = verify_and_parse_search_response(truncated, &config);
        assert!(
            res.is_err(),
            "Truncated at {truncate_len} should fail verification"
        );
    }
}

#[test]
fn test_verification_single_bit_flips_every_position() {
    let valid_data = build_sample_hyper_edge();
    let config = FlatBufferConfig::default();

    assert!(verify_and_parse_hyper_edge(&valid_data, &config).is_ok());

    let mut corrupted = valid_data.clone();
    for byte_idx in 0..valid_data.len() {
        for bit_idx in 0..8 {
            corrupted[byte_idx] ^= 1 << bit_idx;

            // Muss panikfrei verarbeitet werden (Fehler oder selten noch wohlgeformt)
            if let Ok(edge) = verify_and_parse_hyper_edge(&corrupted, &config) {
                let _ = edge.id();
                let _ = edge.predicate();
                let _ = edge.weight();
                if let Some(participants) = edge.participants() {
                    for p in participants.iter() {
                        let _ = p.role();
                        let _ = p.entity();
                    }
                }
            }

            // Bit zurücksetzen
            corrupted[byte_idx] ^= 1 << bit_idx;
        }
    }
}

#[test]
fn test_jsonrpc_garbage_inputs() {
    let mut rng = TestPrng::new(0xDEAD_BEEF_5EED);
    let config = JsonRpcConfig::default();

    for len in [1, 2, 4, 8, 16, 32, 64, 128, 512, 2048] {
        for _ in 0..20 {
            let mut garbage = vec![0u8; len];
            rng.fill_bytes(&mut garbage);

            let res = parse_jsonrpc_payload_with_config(&garbage, &config);
            // Roher Zufalls-Müll ist fast nie valides JSON-RPC 2.0
            assert!(res.is_err());
        }
    }
}

#[test]
fn test_jsonrpc_truncation_valid_payloads() {
    let valid_payload =
        br#"{"jsonrpc": "2.0", "id": 1, "method": "testMethod", "params": {"key": "value"}}"#;
    assert!(parse_jsonrpc_payload(valid_payload).is_ok());

    for truncate_len in 0..valid_payload.len() {
        let truncated = &valid_payload[..truncate_len];
        let res = parse_jsonrpc_payload(truncated);
        assert!(
            res.is_err(),
            "Truncation at {truncate_len} should return error"
        );
    }
}

#[test]
fn test_jsonrpc_bit_flips_valid_payload() {
    let valid_payload = br#"{"jsonrpc": "2.0", "id": 42, "method": "ping", "params": [1, 2, 3]}"#;
    assert!(parse_jsonrpc_payload(valid_payload).is_ok());

    let mut buf = valid_payload.to_vec();
    for byte_idx in 0..buf.len() {
        for bit_idx in 0..8 {
            buf[byte_idx] ^= 1 << bit_idx;

            // Darf niemals panicken
            let _ = parse_jsonrpc_payload(&buf);

            buf[byte_idx] ^= 1 << bit_idx;
        }
    }
}

#[test]
fn test_jsonrpc_malformed_structures_and_depth_limits() {
    let deep_nested = format!("{}{}{}", "{\"a\":".repeat(100), "1", "}".repeat(100));
    let config = JsonRpcConfig {
        max_depth: 32,
        ..Default::default()
    };
    let err = parse_jsonrpc_payload_with_config(deep_nested.as_bytes(), &config).unwrap_err();
    assert_eq!(err.code, -32700);
    assert!(err.message.contains("nesting depth"));

    // Ungültige Typen für method und jsonrpc
    let invalid_method_num = br#"{"jsonrpc": "2.0", "id": 1, "method": 12345}"#;
    assert!(parse_jsonrpc_payload(invalid_method_num).is_err());

    let invalid_version = br#"{"jsonrpc": 2.0, "id": 1, "method": "m"}"#;
    assert!(parse_jsonrpc_payload(invalid_version).is_err());
}

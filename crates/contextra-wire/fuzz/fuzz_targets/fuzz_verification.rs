//! # Fuzz-Target für FlatBuffers Verifier
//!
//! **Ring & Klasse:** Ring 0 (Fuzzing / REQ-QA-003)
//!
//! ## Zweck
//! Dieses Fuzz-Target speist beliebige, ungetestete Byte-Arrays in die gehärteten
//! FlatBuffers-Verifikationsfunktionen von `contextra-wire` ein (`verification.rs`).
//!
//! ## Invarianten
//! - Kein Panic (P7 Zero-Panic).
//! - Keine unbegrenzte Allokation (DoS-Schutz via `FlatBufferConfig`).
//! - Bei Erfolg (`Ok(...)`) ist der Zugriff auf alle Typ-Felder stabil und panikfrei.

#![no_main]

use contextra_wire::{
    verify_and_parse_embedding, verify_and_parse_hyper_edge, verify_and_parse_hyper_edge_id,
    verify_and_parse_role_binding, verify_and_parse_role_id, verify_and_parse_scored_document,
    verify_and_parse_search_response, verify_and_parse_vector_index_update, FlatBufferConfig,
};
use libfuzzer_sys::fuzz_target;

// ANCHOR[contextra-wire:FUZ-002]
fuzz_target!(|data: &[u8]| {
    // Standardkonfiguration
    let default_config = FlatBufferConfig::default();

    // Striktere Konfiguration für Grenzwert-Fuzzing
    let strict_config = FlatBufferConfig {
        max_message_size: 1024 * 1024,
        max_depth: 16,
        max_tables: 10_000,
    };

    // 1. SearchResponse
    if let Ok(resp) = verify_and_parse_search_response(data, &default_config) {
        let _ = resp.total_hits();
        let _ = resp.processing_time_ms();
        if let Some(results) = resp.results() {
            for doc in results.iter() {
                let _ = doc.id();
                let _ = doc.score();
                let _ = doc.metadata();
                if let Some(emb) = doc.embedding() {
                    let _ = emb.data();
                    let _ = emb.quantized_u8();
                    let _ = emb.metric();
                }
            }
        }
    }
    let _ = verify_and_parse_search_response(data, &strict_config);

    // 2. Embedding
    if let Ok(emb) = verify_and_parse_embedding(data, &default_config) {
        let _ = emb.data();
        let _ = emb.quantized_u8();
        let _ = emb.metric();
    }
    let _ = verify_and_parse_embedding(data, &strict_config);

    // 3. ScoredDocument
    if let Ok(doc) = verify_and_parse_scored_document(data, &default_config) {
        let _ = doc.id();
        let _ = doc.score();
        let _ = doc.metadata();
        if let Some(emb) = doc.embedding() {
            let _ = emb.data();
            let _ = emb.quantized_u8();
            let _ = emb.metric();
        }
    }
    let _ = verify_and_parse_scored_document(data, &strict_config);

    // 4. VectorIndexUpdate
    if let Ok(upd) = verify_and_parse_vector_index_update(data, &default_config) {
        let _ = upd.id();
        let _ = upd.metadata();
        if let Some(emb) = upd.embedding() {
            let _ = emb.data();
            let _ = emb.quantized_u8();
            let _ = emb.metric();
        }
    }
    let _ = verify_and_parse_vector_index_update(data, &strict_config);

    // 5. RoleId
    if let Ok(role) = verify_and_parse_role_id(data, &default_config) {
        let _ = role.id();
    }
    let _ = verify_and_parse_role_id(data, &strict_config);

    // 6. HyperEdgeId
    if let Ok(edge_id) = verify_and_parse_hyper_edge_id(data, &default_config) {
        let _ = edge_id.id();
    }
    let _ = verify_and_parse_hyper_edge_id(data, &strict_config);

    // 7. RoleBinding
    if let Ok(binding) = verify_and_parse_role_binding(data, &default_config) {
        let _ = binding.role();
        let _ = binding.entity();
    }
    let _ = verify_and_parse_role_binding(data, &strict_config);

    // 8. HyperEdge
    if let Ok(edge) = verify_and_parse_hyper_edge(data, &default_config) {
        let _ = edge.id();
        let _ = edge.predicate();
        let _ = edge.weight();
        let _ = edge.tx_valid_from();
        let _ = edge.tx_valid_to();
        let _ = edge.business_valid_from();
        let _ = edge.business_valid_to();
        let _ = edge.source_doc_id();
        if let Some(participants) = edge.participants() {
            for p in participants.iter() {
                let _ = p.role();
                let _ = p.entity();
            }
        }
        if let Some(children) = edge.child_edge_ids() {
            for c in children.iter() {
                let _ = c;
            }
        }
    }
    let _ = verify_and_parse_hyper_edge(data, &strict_config);
});

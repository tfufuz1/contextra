// FILE-CONTEXT
// STAND: 2026-11-08T12:00:00Z
// ZWECK: Criterion benchmarks for contextra-wire (FlatBuffers verification & JSON-RPC parsing)
// RELEVANTE SPEZIFIKATION: SPEC §9.3 (Benchmarks), B8

//! # Criterion Benchmarks für `contextra-wire`
//!
//! Benchmark-Suiten für:
//! 1. FlatBuffers `SearchResponse` Verifikation und Zugriff.
//! 2. JSON-RPC 2.0 Single- und Batch-Parsing.
//! 3. `WireBuffer` Allokations- und Slice-Handling.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation
)]

use bytes::Bytes;
use contextra_wire::{
    finish_search_response_buffer, parse_jsonrpc_payload, verify_and_parse_search_response,
    Embedding, EmbeddingArgs, FlatBufferConfig, ScoredDocument, ScoredDocumentArgs, SearchResponse,
    SearchResponseArgs, WireBuffer,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use flatbuffers::FlatBufferBuilder;

/// Erstellt einen gültigen FlatBuffers-Puffer mit einer `SearchResponse` für Benchmarks.
fn create_benchmark_search_response(num_docs: usize) -> Vec<u8> {
    let mut bldr = FlatBufferBuilder::new();
    let mut doc_offsets = Vec::with_capacity(num_docs);

    for i in 0..num_docs {
        let doc_id = format!("doc_{i}");
        let doc_id_offset = bldr.create_string(&doc_id);
        let meta_offset = bldr.create_string("{\"title\":\"benchmark document\",\"lang\":\"de\"}");

        let emb_data = vec![0.1f32 * (i as f32); 128];
        let emb_vec = bldr.create_vector(&emb_data);
        let emb_offset = Embedding::create(
            &mut bldr,
            &EmbeddingArgs {
                data: Some(emb_vec),
                quantized_u8: None,
                metric: 0,
            },
        );

        let doc_offset = ScoredDocument::create(
            &mut bldr,
            &ScoredDocumentArgs {
                id: Some(doc_id_offset),
                score: 0.95 - (i as f32 * 0.001),
                metadata: Some(meta_offset),
                embedding: Some(emb_offset),
            },
        );
        doc_offsets.push(doc_offset);
    }

    let results_offset = bldr.create_vector(&doc_offsets);
    let resp_offset = SearchResponse::create(
        &mut bldr,
        &SearchResponseArgs {
            total_hits: num_docs as u32,
            results: Some(results_offset),
            processing_time_ms: 12.0,
        },
    );

    finish_search_response_buffer(&mut bldr, resp_offset);
    bldr.finished_data().to_vec()
}

fn bench_flatbuffers_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("flatbuffers_verification");
    let config = FlatBufferConfig::default();

    let buf_10 = create_benchmark_search_response(10);
    let buf_100 = create_benchmark_search_response(100);

    group.bench_function("verify_search_response_10_docs", |b| {
        b.iter(|| {
            let res = verify_and_parse_search_response(black_box(&buf_10), black_box(&config));
            assert!(res.is_ok());
        })
    });

    group.bench_function("verify_search_response_100_docs", |b| {
        b.iter(|| {
            let res = verify_and_parse_search_response(black_box(&buf_100), black_box(&config));
            assert!(res.is_ok());
        })
    });

    group.bench_function("verify_and_traverse_100_docs", |b| {
        b.iter(|| {
            let resp = verify_and_parse_search_response(black_box(&buf_100), black_box(&config))
                .expect("valid search response");
            let mut total_score = 0.0f32;
            if let Some(results) = resp.results() {
                for doc in results.iter() {
                    total_score += doc.score();
                }
            }
            black_box(total_score);
        })
    });

    group.finish();
}

fn bench_jsonrpc_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("jsonrpc_parsing");

    let single_payload =
        br#"{"jsonrpc":"2.0","id":1,"method":"search","params":{"query":"benchmark test","k":10}}"#;
    let batch_payload = br#"[
        {"jsonrpc":"2.0","id":1,"method":"embed","params":{"text":"benchmark batch item 1"}},
        {"jsonrpc":"2.0","id":2,"method":"embed","params":{"text":"benchmark batch item 2"}},
        {"jsonrpc":"2.0","id":3,"method":"embed","params":{"text":"benchmark batch item 3"}},
        {"jsonrpc":"2.0","id":4,"method":"embed","params":{"text":"benchmark batch item 4"}},
        {"jsonrpc":"2.0","id":5,"method":"embed","params":{"text":"benchmark batch item 5"}}
    ]"#;

    group.bench_function("parse_single_request", |b| {
        b.iter(|| {
            let res = parse_jsonrpc_payload(black_box(single_payload));
            assert!(res.is_ok());
        })
    });

    group.bench_function("parse_batch_request_5_items", |b| {
        b.iter(|| {
            let res = parse_jsonrpc_payload(black_box(batch_payload));
            assert!(res.is_ok());
        })
    });

    group.finish();
}

fn bench_wire_buffer(c: &mut Criterion) {
    let mut group = c.benchmark_group("wire_buffer");
    let raw = vec![0xABu8; 4096];
    let bytes_obj = Bytes::from(raw);

    group.bench_function("wire_buffer_clone_and_slice", |b| {
        let wb = WireBuffer::new(bytes_obj.clone());
        b.iter(|| {
            let cloned = black_box(&wb).clone();
            let slice = cloned.as_slice();
            black_box(slice.first().copied().unwrap_or(0));
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_flatbuffers_verification,
    bench_jsonrpc_parsing,
    bench_wire_buffer
);
criterion_main!(benches);

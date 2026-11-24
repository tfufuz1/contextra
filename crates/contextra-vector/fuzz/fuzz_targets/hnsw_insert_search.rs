// FILE-CONTEXT
// ZWECK: Fuzz-Target für beliebige Operationsfolgen (Insert, Search, Delete) auf dem HNSW-Index
// INVARIANTEN: Reines fuzz_one ohne direkte libfuzzer-Laufzeitabhängigkeit für Smoke-Treiber
// HOTSPOTS: 35-100
// STAND: 2026-11-11
// BEREICH: AGENTS §8, SPEC §11.4 — HNSW Zustandsübergänge, Tombstones und Suche

// ANCHOR[contextra-vector:W11-FUZZ-006]

#![no_main]

use arbitrary::Arbitrary;
use contextra_ports::VectorIndex;
use contextra_types::{DistanceMetric, DocId, TxId};
use contextra_vector::hnsw::{HnswConfig, HnswIndex};
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
pub enum HnswFuzzOp {
    Insert {
        tx_id: u64,
        doc_id: u64,
        embedding: Vec<f32>,
    },
    Search {
        query: Vec<f32>,
        k: usize,
    },
    Delete {
        tx_id: u64,
        doc_id: u64,
    },
}

#[derive(Arbitrary, Debug)]
pub struct HnswFuzzInput {
    pub dimension: u8,
    pub m: u8,
    pub ef_construction: u8,
    pub ef_search: u8,
    pub quantize: bool,
    pub ops: Vec<HnswFuzzOp>,
}

/// Reines Fuzz-Primitiv zur Ausführung einer HnswFuzzInput-Struktur.
pub fn fuzz_one_input(input: HnswFuzzInput) {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(_) => return,
    };

    rt.block_on(async {
        let dim = (input.dimension as usize % 16) + 1;
        let config = HnswConfig {
            dimension: dim,
            max_elements: 100,
            m: (input.m as usize).max(2).min(32),
            ef_construction: (input.ef_construction as usize).max(2).min(64),
            ef_search: (input.ef_search as usize).max(1).min(64),
            distance_metric: DistanceMetric::Cosine,
            rebuild_threshold: 0.9,
            quantize: input.quantize,
            quantizer_recalibration_sample_size: 100,
            quantizer_drift_threshold: 0.10,
            compute_pool: None,
            ..Default::default()
        };

        let index = match HnswIndex::try_new(config) {
            Ok(idx) => idx,
            Err(_) => return,
        };

        for op in input.ops.into_iter().take(30) {
            match op {
                HnswFuzzOp::Insert {
                    tx_id,
                    doc_id,
                    embedding,
                } => {
                    let _ = index
                        .insert(TxId::new(tx_id), DocId::new(doc_id), &embedding)
                        .await;
                }
                HnswFuzzOp::Search { query, k } => {
                    let _ = index.search(&query, k).await;
                }
                HnswFuzzOp::Delete { tx_id, doc_id } => {
                    let _ = index.delete(TxId::new(tx_id), DocId::new(doc_id)).await;
                }
            }
        }
    });
}

/// Fuzzing Einstiegspunkt für libFuzzer.
fuzz_target!(|input: HnswFuzzInput| {
    fuzz_one_input(input);
});

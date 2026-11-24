// FILE-CONTEXT
// ZWECK: Konstanten und Datenstrukturen fuer Vektordaten, Knoten und Suchkandidaten in HNSW.
// INVARIANTEN: Candidate implementiert Ord basierend auf total_cmp fuer deterministische Sortierung.
// HOTSPOTS: Candidate::cmp, VectorData
// STAND: TS:2026-11-12T00:00:00Z

use crate::types::DocId;

// SECTION HNSW-Konstanten

// ANCHOR[contextra-vector:HNSW-SENTINEL-ENTRY]
/// Sentinel-Wert fuer ungueltigen / nicht gesetzten Entry-Point in HNSW.
pub const SENTINEL_NO_ENTRY_POINT: u32 = u32::MAX;

// ANCHOR[contextra-vector:HNSW-REBUILD-RATIO]
/// Standard-Loeschanteil (0.10 = 10 % geloeschte Knoten), ab dem ein Rebuild getriggert wird.
pub const HNSW_REBUILD_DELETION_RATIO: f64 = 0.10;

// ANCHOR[contextra-vector:HNSW-DEFAULT-SEED]
/// Default-Seed fuer die deterministische HNSW-Ebenenzuweisung (wie in hnswlib ueblich).
pub const DEFAULT_LAYER_SEED: u64 = 100;
// !SECTION

// SECTION Vektordaten & Graphknoten

// ANCHOR[contextra-vector:HNSW-VECTOR-DATA]
#[derive(Debug, Clone)]
pub enum VectorData {
    /// Standard 32-bit floating point vectors.
    F32(Vec<f32>),
    /// 8-bit quantized vectors (SQ8).
    U8(Vec<u8>),
}

// ANCHOR[contextra-vector:HNSW-NODE]
/// A node in the HNSW graph.
#[derive(Debug)]
pub struct HnswNode {
    pub(crate) doc_id: DocId,
    pub(crate) vector: VectorData,
    pub(crate) max_layer: usize,
    pub(crate) committed_tx: u64,
}

// ANCHOR[contextra-vector:HNSW-CANDIDATE]
/// Search candidate.
#[derive(Clone, Copy, Debug)]
pub struct Candidate {
    pub index: usize,
    pub distance: f32,
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}
impl Eq for Candidate {}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.distance.total_cmp(&other.distance)
    }
}
// !SECTION

// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Hoch-entropische Marker-Generierung fuer die Loeschgarantie (Spezifikation T1).
// INVARIANTEN: No unsafe code allowed (#![forbid(unsafe_code)]).
// DOKTRIN: Zero-Panic (keine unwrap/expect/panic in Produktionscode).

use super::rawsweep::expand_needle_encodings;

/// Canonical storage and engine layers for deletion canary tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OracleLayer {
    /// RAM Memtable buffer layer.
    Memtable,
    /// On-disk SSTable file layer.
    SSTable,
    /// Write-Ahead Log segment layer.
    WAL,
    /// Checkpoint and snapshot state layer.
    Checkpoint,
    /// HNSW vector graph index layer.
    HNSW,
    /// Inverted BM25 and text index layer.
    Text,
    /// Entity and relation graph store layer.
    Graph,
}

impl OracleLayer {
    /// Returns the canonical string label for this layer.
    pub const fn as_str(&self) -> &'static str {
        match self {
            OracleLayer::Memtable => "memtable",
            OracleLayer::SSTable => "sstable",
            OracleLayer::WAL => "wal",
            OracleLayer::Checkpoint => "checkpoint",
            OracleLayer::HNSW => "hnsw",
            OracleLayer::Text => "text",
            OracleLayer::Graph => "graph",
        }
    }

    /// Returns all 7 canonical layers.
    pub const fn all() -> &'static [OracleLayer] {
        &[
            OracleLayer::Memtable,
            OracleLayer::SSTable,
            OracleLayer::WAL,
            OracleLayer::Checkpoint,
            OracleLayer::HNSW,
            OracleLayer::Text,
            OracleLayer::Graph,
        ]
    }
}

/// Single field marker containing the field name and its CTXMARK string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrcFieldMarker {
    /// Associated field name.
    pub field_name: String,
    /// Generated CTXMARK marker string.
    pub marker_string: String,
}

/// Set of deletion oracle markers for a specific test run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrcMarkerSet {
    run_label: String,
    seed: u64,
    fields: Vec<OrcFieldMarker>,
    vector_signature: [u8; 8],
}

struct OrcPrng {
    state: u64,
}

impl OrcPrng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    fn next_finite_f32(&mut self) -> f32 {
        let bits = self.next_u32();
        // Clear highest exponent bit to guarantee exponent <= 0x7F (no NaN / Inf).
        let safe_bits = bits & 0x7F7F_FFFF;
        f32::from_bits(safe_bits)
    }
}

impl OrcMarkerSet {
    /// Generates a new marker set for the given run label, seed, and fields.
    ///
    /// Entropy is derived purely deterministically from `seed` (SplitMix64).
    pub fn generate(run_label: &str, seed: u64, fields: &[&str]) -> Self {
        let mut prng = OrcPrng::new(seed);

        let v0 = prng.next_finite_f32();
        let v1 = prng.next_finite_f32();
        let mut vector_signature = [0u8; 8];
        vector_signature[0..4].copy_from_slice(&v0.to_le_bytes());
        vector_signature[4..8].copy_from_slice(&v1.to_le_bytes());

        let mut field_markers = Vec::with_capacity(fields.len());
        for field in fields {
            let u1 = prng.next_u64();
            let u2 = prng.next_u64();
            let hex_str = format!("{:016x}{:016x}", u1, u2);
            let marker_string = format!("CTXMARK-{run_label}-{field}-{hex_str}");
            field_markers.push(OrcFieldMarker {
                field_name: (*field).to_string(),
                marker_string,
            });
        }

        Self {
            run_label: run_label.to_string(),
            seed,
            fields: field_markers,
            vector_signature,
        }
    }

    /// Generates canary markers for all 7 canonical storage and index layers.
    pub fn generate_per_layer(run_label: &str, seed: u64) -> Self {
        let layers = OracleLayer::all();
        let field_names: Vec<&str> = layers.iter().map(|l| l.as_str()).collect();
        Self::generate(run_label, seed, &field_names)
    }

    /// Returns the marker string for a specific layer if present.
    pub fn layer_marker(&self, layer: OracleLayer) -> Option<&str> {
        let label = layer.as_str();
        self.fields
            .iter()
            .find(|f| f.field_name == label)
            .map(|f| f.marker_string.as_str())
    }

    /// Generates a vector marker of length `dim` with an 8-byte signature at position 0.
    ///
    /// The floats at position 0 (if `dim >= 2`) produce the exact 8-byte LE signature
    /// stored in an `OrcMarkerSet` generated with the same `seed`.
    pub fn vector_marker(dim: usize, seed: u64) -> Vec<f32> {
        if dim == 0 {
            return Vec::new();
        }
        let mut prng = OrcPrng::new(seed);
        let mut vec = Vec::with_capacity(dim);
        for _ in 0..dim {
            vec.push(prng.next_finite_f32());
        }
        vec
    }

    /// Returns all raw byte needles to sweep for, including field marker bytes and vector signature.
    pub fn needles(&self) -> Vec<Vec<u8>> {
        let mut needles = Vec::with_capacity(self.fields.len() + 1);
        for fm in &self.fields {
            needles.push(fm.marker_string.as_bytes().to_vec());
        }
        needles.push(self.vector_signature.to_vec());
        needles
    }

    /// Returns all raw byte needles expanded into raw, hex, base64, and UTF-16LE representations.
    pub fn needles_multi_encoding(&self) -> Vec<Vec<u8>> {
        expand_needle_encodings(&self.needles())
    }

    /// Returns the run label.
    pub fn run_label(&self) -> &str {
        &self.run_label
    }

    /// Returns the seed used for generation.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Returns the field markers.
    pub fn fields(&self) -> &[OrcFieldMarker] {
        &self.fields
    }

    /// Returns the 8-byte vector signature.
    pub fn vector_signature(&self) -> &[u8; 8] {
        &self.vector_signature
    }
}

superseded by CONTEXTRA_ENDPRODUKT_SPEZIFIKATION.md

# Micro-Fine-Grained Architectural & Algorithmic Specification (SOTA 2026)

**Dokumenten-ID:** `DOC-SPEC-SOTA-2026-01`
**Datum:** 14. Oktober 2026
**Status:** Genehmigt / Spezifiziert
**Geltungsbereich:** `contextra-kvcache`, `contextra-engine`, `contextra-rank`, `contextra-adapt`, `contextra-cognition`, `contextra-graph`, `contextra-crypto`

---

## Executive Summary & System-Architektur

Diese Spezifikation definiert die mikrofeingranularen Algorithmen, Datenstrukturen, mathematischen Formalismen und Integrationspunkte für die SOTA-Erweiterungen im Contextra-System. Alle Algorithmen folgen den Kernprinzipien der Contextra-Architektur:
1. **Deterministische Ausführung**: Plattformunabhängige Deterministik durch konsistente Gleitkomma-Sortierung via `f64::total_cmp`.
2. **Zero-Panic-Fehlermodell**: Exklusive Nutzung von `Result<T, ContextraError>` ohne `unwrap()`, `expect()` oder unkontrollierte Panics.
3. **Ring-Integrität**: Striktes Einhalten der Layering-Regeln (Ring 0 bis Ring 2).
4. **Kryptographische Verifizierbarkeit**: Durchgängiges Auditing und Crypto-Shredding nach BSI TR-02102.

---

## 1. Deterministic KV-Cache Eviction & 2-Bit Quantization (KIVI, SnapKV, Irminsul)

**Crate-Zuordnung:** `contextra-kvcache` (Ring 0)

### 1.1 Mathematischer Formalismus & Algorithmus
Das KV-Cache Management kombiniert drei komplementäre SOTA-Verfahren:
* **KIVI (2-Bit Quantization)**: Key-Tensoren werden pro Kanal (*per-channel*) und Value-Tensoren pro Token (*per-token*) auf 2-Bit quantisiert:
  $$\hat{X}_{2\text{bit}} = \text{round}\left(\frac{X - \text{zp}}{\text{scale}}\right) \in \{0, 1, 2, 3\}$$
  wobei $\text{scale} = \frac{\max(X) - \min(X)}{3}$ und $\text{zp} = \min(X)$.
* **SnapKV (Observation Window Attention Allocation)**: Berechnung der Wichtigkeit von Key-Tokens basierend auf den Attention-Gewichten eines Beobachtungsfensters $W_{obs}$:
  $$S_i = \sum_{q \in W_{obs}} A_{q, i}$$
  Die Top-$K$ Tokens außerhalb des Prefix-Pools werden selektiert.
* **Irminsul (Prefix Tree & Tier-2 NVMe/mmap Spill-over)**: Radix-Trie-basierte Prefix-Teilung mit atomarer Referenzzählung. Nicht-aktive Seiten werden deterministisch in Tier-2 (NVMe/mmap) verschoben.

### 1.2 Rust Implementation Blueprint

```rust
use contextra_types::error::ContextraError;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct QuantizedBuffer2Bit {
    pub compressed_data: Vec<u8>, // 4 tokens / byte (2 bits per value)
    pub scales: Vec<f32>,
    pub zero_points: Vec<f32>,
    pub num_elements: usize,
}

#[derive(Debug, Clone)]
pub struct IrminsulTrieNode {
    pub prefix_hash: u64,
    pub ref_count: Arc<AtomicUsize>,
    pub tier: CacheTier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheTier {
    RamTier1,
    NvmeTier2 { page_offset: u64 },
}

pub struct SnapKvConfig {
    pub max_capacity_tokens: usize,
    pub observation_window_size: usize,
    pub prefix_pool_size: usize,
}

pub struct KIVICompressor;

impl KIVICompressor {
    pub fn quantize_keys_per_channel(
        raw_keys: &[f32],
        num_tokens: usize,
        channels: usize,
    ) -> Result<QuantizedBuffer2Bit, ContextraError> {
        if raw_keys.len() != num_tokens * channels {
            return Err(ContextraError::invalid_input(
                "Buffer length does not match specified dimensions",
            ));
        }

        let mut scales = vec![0.0f32; channels];
        let mut zero_points = vec![0.0f32; channels];
        let mut compressed_data = vec![0u8; (num_tokens * channels + 3) / 4];

        for c in 0..channels {
            let mut min_val = f32::MAX;
            let mut max_val = f32::MIN;
            for t in 0..num_tokens {
                let val = raw_keys[t * channels + c];
                if val < min_val { min_val = val; }
                if val > max_val { max_val = val; }
            }

            let scale = (max_val - min_val) / 3.0;
            let scale = if scale.abs() < f32::EPSILON { 1.0 } else { scale };
            scales[c] = scale;
            zero_points[c] = min_val;

            for t in 0..num_tokens {
                let val = raw_keys[t * channels + c];
                let q_val = (((val - min_val) / scale).round() as u8).min(3);
                let idx = t * channels + c;
                let byte_idx = idx / 4;
                let bit_shift = (idx % 4) * 2;
                compressed_data[byte_idx] |= q_val << bit_shift;
            }
        }

        Ok(QuantizedBuffer2Bit {
            compressed_data,
            scales,
            zero_points,
            num_elements: num_tokens * channels,
        })
    }
}

pub struct SnapKvSelector;

impl SnapKvSelector {
    pub fn select_eviction_candidates(
        attention_scores: &[f64],
        config: &SnapKvConfig,
    ) -> Result<Vec<usize>, ContextraError> {
        if config.max_capacity_tokens < config.prefix_pool_size {
            return Err(ContextraError::invalid_input(
                "max_capacity_tokens must be greater than or equal to prefix_pool_size",
            ));
        }

        if attention_scores.len() <= config.max_capacity_tokens {
            return Ok(Vec::new());
        }

        let total_tokens = attention_scores.len();
        let retain_count = config.max_capacity_tokens - config.prefix_pool_size;

        let mut scored_indices: Vec<(usize, f64)> = attention_scores
            .iter()
            .enumerate()
            .skip(config.prefix_pool_size)
            .map(|(idx, &score)| (idx, score))
            .collect();

        // Deterministic sorting using total_cmp
        scored_indices.sort_by(|a, b| b.1.total_cmp(&a.1));

        let mut evicted_indices: Vec<usize> = scored_indices
            .into_iter()
            .skip(retain_count)
            .map(|(idx, _)| idx)
            .collect();

        evicted_indices.sort_unstable();
        Ok(evicted_indices)
    }
}
```

### 1.3 Komplexität & Integrationspunkte
* **Zeitkomplexität**: Quantisierung $O(N \cdot D)$, SnapKV Selektion $O(N \log N)$, Irminsul Look-up $O(L)$ ($L = \text{Prefix-Länge}$).
* **Raumkomplexität**: Speicherreduktion um 87.5% (von 16-Bit Float auf 2-Bit Quantized).
* **Integrationspunkt**: `contextra-kvcache/src/eviction/snap_kivi.rs`.

---

## 2. Automated Knowledge Graph Construction (OpenIE)

**Crate-Zuordnung:** `contextra-engine` (Ring 1)

### 2.1 Mathematischer Formalismus & Algorithmus
Für ein Ingestion-Dokument $D$ führt das OpenIE-Modul eine deterministische Extraktion von $k$-stelligen Hyperkanten $H = (V_H, e_1, \dots, e_m)$ aus:
$$e_i = (S, P, O_1, \dots, O_{k-2}, c_i)$$
wobei $c_i \in [0, 1]$ die Konfidenz repräsentiert.

Entitäts-Kanonisierung verwendet den BLAKE3-Hash des normalisierten Strings:
$$h(e) = \text{BLAKE3}(\text{lowercase}(\text{unicode\_nfkd}(e)))$$

### 2.2 Rust Implementation Blueprint

```rust
use contextra_types::error::ContextraError;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct HyperEdgeExtraction {
    pub subject: String,
    pub predicate: String,
    pub objects: Vec<String>,
    pub confidence: f64,
    pub canonical_hash: [u8; 32],
}

pub struct OpenIEPipeline {
    pub min_confidence: f64,
}

impl OpenIEPipeline {
    pub fn process_triples(
        &self,
        raw_extractions: Vec<HyperEdgeExtraction>,
    ) -> Result<Vec<HyperEdgeExtraction>, ContextraError> {
        let mut deduplicated: BTreeMap<[u8; 32], HyperEdgeExtraction> = BTreeMap::new();

        for mut extraction in raw_extractions {
            if extraction.confidence.total_cmp(&self.min_confidence).is_lt() {
                continue;
            }

            // Normalize strings
            extraction.subject = extraction.subject.trim().to_lowercase();
            extraction.predicate = extraction.predicate.trim().to_lowercase();
            extraction.objects = extraction
                .objects
                .into_iter()
                .map(|o| o.trim().to_lowercase())
                .collect();

            // Calculate deterministic canonical hash
            let mut hasher = blake3::Hasher::new();
            hasher.update(extraction.subject.as_bytes());
            hasher.update(extraction.predicate.as_bytes());
            for obj in &extraction.objects {
                hasher.update(obj.as_bytes());
            }
            extraction.canonical_hash = *hasher.finalize().as_bytes();

            // Deduplicate & keep highest confidence
            match deduplicated.get_mut(&extraction.canonical_hash) {
                Some(existing) => {
                    if extraction.confidence.total_cmp(&existing.confidence).is_gt() {
                        *existing = extraction;
                    }
                }
                None => {
                    deduplicated.insert(extraction.canonical_hash, extraction);
                }
            }
        }

        let mut result: Vec<_> = deduplicated.into_values().collect();
        result.sort_by(|a, b| b.confidence.total_cmp(&a.confidence));
        Ok(result)
    }
}
```

### 2.3 Komplexität & Integrationspunkte
* **Zeitkomplexität**: $O(|T| \cdot L + N \log N)$, wobei $|T|$ die Anzahl der Tokens und $N$ die Extrakte bezeichnet.
* **Raumkomplexität**: $O(N \cdot K)$ für Speicherung der Hyperkanten.
* **Integrationspunkt**: `contextra-engine/src/ingestion/open_ie.rs`.

---

## 3. Conformal Fusion Calibration (arXiv:2511.04275, 2503.03841)

**Crate-Zuordnung:** `contextra-rank` (Ring 0)

### 3.1 Mathematischer Formalismus & Algorithmus
Anstelle unkalibrierter Platt/PAVA-Methoden garantiert Conformal Prediction eine exakte, distributionsfreie Formel mit Abdeckungsgarantie $1 - \alpha$:
$$s_i = 1 - \hat{p}(y_i | x_i)$$
Das konforme Quantil $\hat{q}$ berechnet sich über die finite-sample Anpassung:
$$\hat{q} = \text{Quantile}\left( \left\{s_1, \dots, s_n\right\}, \frac{\lceil (n+1)(1-\alpha) \rceil}{n} \right)$$

### 3.2 Rust Implementation Blueprint

```rust
use contextra_types::error::ContextraError;

pub struct ConformalCalibrator {
    pub alpha: f64,
    pub conformal_quantile: f64,
}

impl ConformalCalibrator {
    pub fn calibrate(
        calibration_scores: &[f64],
        alpha: f64,
    ) -> Result<Self, ContextraError> {
        if calibration_scores.is_empty() {
            return Err(ContextraError::invalid_input("Calibration set is empty"));
        }
        if alpha.is_nan() || alpha <= 0.0 || alpha >= 1.0 {
            return Err(ContextraError::invalid_input("Alpha must be a valid float in (0, 1)"));
        }

        let mut sorted_scores = calibration_scores.to_vec();
        sorted_scores.sort_by(|a, b| a.total_cmp(b));

        let n = sorted_scores.len();
        let k = ((n as f64 + 1.0) * (1.0 - alpha)).ceil() as usize;
        let index = (k.saturating_sub(1)).min(n - 1);

        let conformal_quantile = sorted_scores[index];

        Ok(Self {
            alpha,
            conformal_quantile,
        })
    }

    pub fn predict_prediction_set(&self, score: f64) -> bool {
        let non_conformity = 1.0 - score;
        non_conformity.total_cmp(&self.conformal_quantile).is_le()
    }
}
```

### 3.3 Komplexität & Integrationspunkte
* **Zeitkomplexität**: Kalibrierung $O(n \log n)$, Vorhersage $O(1)$.
* **Raumkomplexität**: $O(n)$ im Speicher für Kalibrierungs-Scores.
* **Integrationspunkt**: `contextra-rank/src/calibration/conformal.rs`.

---

## 4. Sketched Contextual Bandit Projection (arXiv:2606.27917, 2605.00500)

**Crate-Zuordnung:** `contextra-adapt` (Ring 1)

### 4.1 Mathematischer Formalismus & Algorithmus
Zur Reduktion der Sherman-Morrison Update-Kosten von $O(d^2)$ auf $O(k^2)$ ($k \ll d$) nutzen wir eine Count-Sketch Projektionsmatrix $S \in \mathbb{R}^{k \times d}$:
$$\tilde{x} = S x$$
Das Sherman-Morrison-Update für die k-dimensionale Inverse $A_k^{-1} \in \mathbb{R}^{k \times k}$ lautet:
$$A_{k, t+1}^{-1} = A_{k, t}^{-1} - \frac{A_{k, t}^{-1} \tilde{x} \tilde{x}^T A_{k, t}^{-1}}{1 + \tilde{x}^T A_{k, t}^{-1} \tilde{x}}$$
Der UCB-Score berechnet sich deterministisch als:
$$\text{UCB}(a) = \theta_k^T \tilde{x}_a + \alpha \sqrt{\tilde{x}_a^T A_k^{-1} \tilde{x}_a}$$

### 4.2 Rust Implementation Blueprint

```rust
use contextra_types::error::ContextraError;

pub struct SketchedLinUCB {
    pub orig_dim: usize,
    pub sketch_dim: usize,
    pub inv_a: Vec<f64>, // Flattened k x k matrix
    pub b: Vec<f64>,     // k-dim vector
    pub alpha: f64,
    pub sketch_hash_seeds: Vec<u64>,
}

impl SketchedLinUCB {
    pub fn new(orig_dim: usize, sketch_dim: usize, alpha: f64) -> Result<Self, ContextraError> {
        if orig_dim == 0 || sketch_dim == 0 || sketch_dim > orig_dim {
            return Err(ContextraError::invalid_input("Invalid dimensions for sketching"));
        }

        let mut inv_a = vec![0.0; sketch_dim * sketch_dim];
        for i in 0..sketch_dim {
            inv_a[i * sketch_dim + i] = 1.0; // Identity matrix initialization
        }

        Ok(Self {
            orig_dim,
            sketch_dim,
            inv_a,
            b: vec![0.0; sketch_dim],
            alpha,
            sketch_hash_seeds: vec![0x9e3779b97f4a7c15, 0x3141592653589793],
        })
    }

    pub fn project_feature(&self, x: &[f64]) -> Result<Vec<f64>, ContextraError> {
        if x.len() != self.orig_dim {
            return Err(ContextraError::invalid_input("Feature length mismatch"));
        }

        let mut x_tilde = vec![0.0; self.sketch_dim];
        for (idx, &val) in x.iter().enumerate() {
            let bucket = (idx.wrapping_mul(31)) % self.sketch_dim;
            let sign = if (idx.wrapping_mul(17)) % 2 == 0 { 1.0 } else { -1.0 };
            x_tilde[bucket] += sign * val;
        }

        Ok(x_tilde)
    }

    pub fn update(&mut self, x_tilde: &[f64], reward: f64) -> Result<(), ContextraError> {
        let k = self.sketch_dim;
        if x_tilde.len() != k {
            return Err(ContextraError::invalid_input("Sketched feature dimension mismatch"));
        }

        // Compute A_inv * x_tilde
        let mut ax = vec![0.0; k];
        for i in 0..k {
            for j in 0..k {
                ax[i] += self.inv_a[i * k + j] * x_tilde[j];
            }
        }

        // x_tilde^T * A_inv * x_tilde
        let mut x_ax = 0.0;
        for i in 0..k {
            x_ax += x_tilde[i] * ax[i];
        }

        let denominator = 1.0 + x_ax;

        // Rank-1 Sherman Morrison update
        for i in 0..k {
            for j in 0..k {
                self.inv_a[i * k + j] -= (ax[i] * ax[j]) / denominator;
            }
        }

        // Update reward vector b
        for i in 0..k {
            self.b[i] += reward * x_tilde[i];
        }

        Ok(())
    }
}
```

### 4.3 Komplexität & Integrationspunkte
* **Zeitkomplexität**: Projection $O(d)$, Update $O(k^2)$, UCB Calculation $O(k^2)$.
* **Raumkomplexität**: $O(k^2)$ statt $O(d^2)$.
* **Integrationspunkt**: `contextra-adapt/src/bandit/sketched_linucb.rs`.

---

## 5. Spectral Hyperedge Deduplication (arXiv:2502.03313)

**Crate-Zuordnung:** `contextra-cognition` (Ring 2)

### 5.1 Mathematischer Formalismus & Algorithmus
Zur Vermeidung vollständiger SVD-Berechnungen nutzen wir das Hypergraph-Laplace-Spektrum $L_H = D_v - H W D_e^{-1} H^T$. Redundante Hyperkanten werden inkrementell über den Kosinus-Abstand ihrer Spektralsignaturen identifiziert:
$$\text{Sim}(e_i, e_j) = \frac{\langle v_i, v_j \rangle}{\|v_i\|_2 \|v_j\|_2}$$

### 5.2 Rust Implementation Blueprint

```rust
use contextra_types::error::ContextraError;

pub struct SpectralDeduplicator {
    pub similarity_threshold: f64,
}

impl SpectralDeduplicator {
    pub fn compute_spectral_signature(
        hyperedge_nodes: &[usize],
        num_total_nodes: usize,
        eigenvectors: &[Vec<f64>], // m eigenvectors of size num_total_nodes
    ) -> Result<Vec<f64>, ContextraError> {
        if hyperedge_nodes.is_empty() {
            return Err(ContextraError::invalid_input("Hyperedge has no nodes"));
        }

        let m = eigenvectors.len();
        let mut signature = vec![0.0; m];

        for (e_idx, ev) in eigenvectors.iter().enumerate() {
            if ev.len() != num_total_nodes {
                return Err(ContextraError::invalid_input("Eigenvector size mismatch"));
            }
            let mut sum = 0.0;
            for &node in hyperedge_nodes {
                if node >= num_total_nodes {
                    return Err(ContextraError::invalid_input("Node index out of bounds"));
                }
                sum += ev[node];
            }
            signature[e_idx] = sum / (hyperedge_nodes.len() as f64).sqrt();
        }

        Ok(signature)
    }

    pub fn is_redundant(
        sig_a: &[f64],
        sig_b: &[f64],
        threshold: f64,
    ) -> Result<bool, ContextraError> {
        if sig_a.len() != sig_b.len() {
            return Err(ContextraError::invalid_input("Signature dimensions mismatch"));
        }

        let mut dot = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;

        for i in 0..sig_a.len() {
            dot += sig_a[i] * sig_b[i];
            norm_a += sig_a[i] * sig_a[i];
            norm_b += sig_b[i] * sig_b[i];
        }

        if norm_a == 0.0 || norm_b == 0.0 {
            return Ok(false);
        }

        let sim = dot / (norm_a.sqrt() * norm_b.sqrt());
        Ok(sim.total_cmp(&threshold).is_ge())
    }
}
```

### 5.3 Komplexität & Integrationspunkte
* **Zeitkomplexität**: Spektralsignatur $O(m \cdot |e|)$, Redundanzprüfung $O(m)$.
* **Raumkomplexität**: $O(m \cdot |V|)$.
* **Integrationspunkt**: `contextra-cognition/src/hypergraph/spectral_dedup.rs`.

---

## 6. Budgeted k-Path Diffusion (PathRAG Extension)

**Crate-Zuordnung:** `contextra-graph/src/path_rag` (Ring 0)

### 6.1 Mathematischer Formalismus & Algorithmus
Zur Traversierung relevanter Pfade im Wissensgraphen unter striktem Knotenbudget $B$ verwendet das System eine modifizierte Prioritätswarteschlange. Die Traversierung wird bei Erreichen von $B$ abgetasteten Knoten mit $O(B \log B)$ Komplexität abgebrochen.

### 6.2 Rust Implementation Blueprint

```rust
use contextra_types::error::ContextraError;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};

#[derive(Debug, Clone)]
pub struct PathCandidate {
    pub path: Vec<usize>,
    pub score: f64,
}

impl PartialEq for PathCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.score.total_cmp(&other.score).is_eq()
    }
}

impl Eq for PathCandidate {}

impl Ord for PathCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.total_cmp(&other.score)
    }
}

impl PartialOrd for PathCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct BudgetedPathDiffusion {
    pub max_node_budget: usize,
}

impl BudgetedPathDiffusion {
    pub fn traverse_paths(
        &self,
        start_node: usize,
        adjacency: &[Vec<(usize, f64)>],
    ) -> Result<Vec<PathCandidate>, ContextraError> {
        let mut visited_nodes = HashSet::new();
        let mut heap = BinaryHeap::new();
        let mut result_paths = Vec::new();

        heap.push(PathCandidate {
            path: vec![start_node],
            score: 1.0,
        });

        while let Some(candidate) = heap.pop() {
            let last_node = candidate
                .path
                .last()
                .copied()
                .ok_or_else(|| ContextraError::invalid_input("Path candidate cannot be empty"))?;

            if visited_nodes.insert(last_node) {
                result_paths.push(candidate.clone());
            }

            if visited_nodes.len() >= self.max_node_budget {
                break;
            }

            if let Some(neighbors) = adjacency.get(last_node) {
                for &(neighbor, weight) in neighbors {
                    if !visited_nodes.contains(&neighbor) {
                        let mut new_path = candidate.path.clone();
                        new_path.push(neighbor);
                        let new_score = candidate.score * weight;
                        heap.push(PathCandidate {
                            path: new_path,
                            score: new_score,
                        });
                    }
                }
            }
        }

        Ok(result_paths)
    }
}
```

### 6.3 Komplexität & Integrationspunkte
* **Zeitkomplexität**: $O(B \log B)$, wobei $B$ das Knotenbudget darstellt.
* **Raumkomplexität**: $O(B \cdot L)$ ($L = \text{max Pfadlänge}$).
* **Integrationspunkt**: `contextra-graph/src/path_rag/budgeted_diffusion.rs`.

---

## 7. Verifizierbare Löschnachweise & Crypto-Shredding (BSI TR-02102)

**Crate-Zuordnung:** `contextra-crypto` / `contextra-store` (Ring 0)

### 7.1 Mappings & BSI TR-02102 Standard-Compliance
Der DeletionProof v4 kapselt die physikalische und logische Löschung über alle 7 Layer ab:

| Layer ID | Name | Methodik | BSI TR-02102 Algorithmus |
|---|---|---|---|
| L1 | MemTable | Key-Overwrite & Zeroization | SHA-256 / BLAKE3 Memory Scrub |
| L2 | HNSW Graph | Ghost Edge Removal & Re-indexing | Ed25519 Signed Witness |
| L3 | TextIndex | Term Postings Scrubbing | HKDF-SHA512 Key Derive |
| L4 | Relations | Hyperedge Tombstoning | AES-256-GCM-SIV Key Revocation |
| L5 | ErasureLedger | Append-only Cryptographic Proof | Ed25519 Batch Signature |
| L6 | WAL | Segment Retirement & Truncation | HMAC-SHA256 Segment Proof |
| L7 | Disk / File | 64-KiB Zero-Overwrite Scrubbing | BSI TR-02102-1 Secure Erase |

### 7.2 Verifikations-Protokoll

```rust
use contextra_types::error::ContextraError;

pub struct DeletionProofV4 {
    pub document_id: String,
    pub erasure_timestamp: u64,
    pub layer_witnesses: [ [u8; 32]; 7 ],
    pub ed25519_signature: [u8; 64],
}

impl DeletionProofV4 {
    pub fn verify_proof_offline(
        &self,
        public_key_bytes: &[u8; 32],
    ) -> Result<bool, ContextraError> {
        let mut msg_bytes = Vec::new();
        msg_bytes.extend_from_slice(self.document_id.as_bytes());
        msg_bytes.extend_from_slice(&self.erasure_timestamp.to_le_bytes());
        for witness in &self.layer_witnesses {
            msg_bytes.extend_from_slice(witness);
        }

        // BSI TR-02102 Compliant Ed25519 Signature Verification
        let public_key = ed25519_dalek::VerifyingKey::from_bytes(public_key_bytes)
            .map_err(|_| ContextraError::crypto("Invalid Ed25519 public key"))?;

        let signature = ed25519_dalek::Signature::from_bytes(&self.ed25519_signature);

        public_key
            .verify_strict(&msg_bytes, &signature)
            .map_err(|_| ContextraError::crypto("Signature verification failed"))?;

        Ok(true)
    }
}
```

---

## 8. Ring-Modell Integrationsmatrix

| Crate | Ring | Verantwortlichkeit | Zero-Panic Garant |
|---|---|---|---|
| `contextra-kvcache` | Ring 0 | 2-Bit Quantisierung, Eviction, Irminsul Trie | Ja (`Result<T, ContextraError>`) |
| `contextra-rank` | Ring 0 | Conformal Fusion Calibration | Ja (`Result<T, ContextraError>`) |
| `contextra-graph` | Ring 0 | Budgeted k-Path Diffusion (PathRAG) | Ja (`Result<T, ContextraError>`) |
| `contextra-crypto` | Ring 0 | DeletionProof v4 & Crypto-Shredding | Ja (`Result<T, ContextraError>`) |
| `contextra-engine` | Ring 1 | Ingestion OpenIE Hyperedge Extraktion | Ja (`Result<T, ContextraError>`) |
| `contextra-adapt` | Ring 1 | Sketched Contextual Bandit Projection | Ja (`Result<T, ContextraError>`) |
| `contextra-cognition` | Ring 2 | Spectral Hyperedge Deduplication | Ja (`Result<T, ContextraError>`) |

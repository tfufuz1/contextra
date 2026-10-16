ARCHIV: Feature nicht im Hauptzweig. Wiederherstellung nur über Tag park/EPn/v1 und die Regeln PR1 bis PR5 der Gesamtspezifikation v4.2 §4.5.

# Contextra — Mikrofeingranulare Implementierungsspezifikation EP4: Contextra Inference & KV-Cache

**Dokumenten-ID:** `DOC-SPEC-EP4-INFERENCE-2026-10-14`  
**Version:** 2.0 · **Datum:** 14. Oktober 2026  
**Autor:** Principal Senior Rust Architect für Contextra  
**Status:** Verbindliche, normative Implementierungsanweisung für lokale Inferenz, Reranking & KV-Cache  
**Geltungsbereich (Disjunktive Crates):**  
- **Ring 2 (Inferenz-Provider):** [`contextra-infer-candle`](../crates/contextra-infer-candle), [`contextra-infer-onnx`](../crates/contextra-infer-onnx), [`contextra-infer-ollama`](../crates/contextra-infer-ollama)
- **Ring 1 / Persistenz-nah:** [`contextra-kvcache`](../crates/contextra-kvcache)
- **Ring 0 (Port-Verträge):** [`contextra-ports`](../crates/contextra-ports) (Module `src/embedding.rs`, `src/attention.rs`, `src/reranker.rs`, `src/kv_bridge_port.rs`)

---

## 1. Architektonische Grenzziehung & Disjunktion

Contextra Inference (EP4) kapselt alle Machine-Learning-Ausführungen (lokale Embeddings via Candle/ONNX, Cross-Encoder-Reranking, LLM-Inferenz via Ollama) sowie das deterministische KV-Cache-Management (KIVI 2-Bit, SnapKV, Irminsul).

**Disjunktionsregeln für EP4:**
1. **Keine Ring-0/Ring-1 Inferenz-Abhängigkeit (P5/P26):** Weder `contextra-store`, `contextra-crypto` noch `contextra-vector` dürfen direkte Cargo-Abhängigkeiten auf `candle`, `onnxruntime` oder `contextra-infer-*` führen. Der Zugriff erfolgt strikt über die abstrakten Ports in `contextra-ports`.
2. **MockEmbedder-Isolation:** `MockEmbedder` darf niemals in Release-Binaries von `contextra-ports` enthalten sein. Er wird strikt hinter `#[cfg(any(test, feature = "testkit"))]` isoliert.
3. **Deterministische Eviction & Zero-Panic (P2 & P28):** Gleitkomma-Sortierungen bei KV-Cache-Evictions müssen zwingend über `f32::total_cmp` / `f64::total_cmp` erfolgen. Es existiert kein `unwrap()` auf `partial_cmp`.

---

## 2. Detaillierte Modul- und Datei-Feinspezifikationen

### 2.1 Komponente 1: Port-Bereinigung & `MockEmbedder` Gating

#### Problem
In [`crates/contextra-ports/src/embedding.rs:171`](../crates/contextra-ports/src/embedding.rs#L171) war `MockEmbedder` als `pub struct` deklariert, ohne dass ein Test-Gate existierte. Dadurch verblieb Test-Mock-Code im Release-Artefakt von Ring 0 (`AUDIT_composition-root_DEEP_2026-10-13.md`).

#### Datei: `crates/contextra-ports/src/embedding.rs`
**IST-Zustand (Zeilen 170–195):**
```rust
pub struct MockEmbedder {
    dimension: usize,
}
```
**SOLL-Blueprint:**
```rust
// In crates/contextra-ports/src/embedding.rs (Zeile 170-205)
use contextra_types::{ContextraError, Result};

#[async_trait::async_trait]
pub trait EmbeddingProvider: Send + Sync + 'static {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
    fn dimension(&self) -> usize;
}

/// MockEmbedder strikt isolieren: Nur in Tests oder mit explizitem feature = "testkit"
#[cfg(any(test, feature = "testkit"))]
pub struct MockEmbedder {
    pub dimension: usize,
    pub deterministic_seed: u64,
}

#[cfg(any(test, feature = "testkit"))]
#[async_trait::async_trait]
impl EmbeddingProvider for MockEmbedder {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let mut vec = vec![0.0f32; self.dimension];
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.deterministic_seed.to_le_bytes());
        hasher.update(text.as_bytes());
        let hash = hasher.finalize();
        for (i, slot) in vec.iter_mut().enumerate() {
            *slot = (hash.as_bytes()[i % 32] as f32) / 255.0;
        }
        Ok(vec)
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        for t in texts {
            results.push(self.embed_text(t).await?);
        }
        Ok(results)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}
```

---

### 2.2 Komponente 2: Inferenz-Port Implementierungen (Ring 2 $\to$ Ring 0 Ports)

#### Datei 1: `crates/contextra-infer-candle/src/embedding_provider.rs`
**Ziel:** Saubere Bereitstellung von lokalen Embeddings über Candle ohne Ring-Layering-Verletzung.
```rust
// In crates/contextra-infer-candle/src/embedding_provider.rs
use std::sync::Arc;
use async_trait::async_trait;
use candle_core::{Device, Tensor};
use contextra_ports::EmbeddingProvider;
use contextra_types::{ContextraError, Result};
use crate::model::BertEmbeddingModel;

pub struct CandleEmbeddingProvider {
    model: Arc<tokio::sync::Mutex<BertEmbeddingModel>>,
    dimension: usize,
}

impl CandleEmbeddingProvider {
    pub fn new(model: BertEmbeddingModel) -> Self {
        let dimension = model.dimension();
        Self {
            model: Arc::new(tokio::sync::Mutex::new(model)),
            dimension,
        }
    }
}

#[async_trait]
impl EmbeddingProvider for CandleEmbeddingProvider {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let mut guard = self.model.lock().await;
        guard.infer_single(text).map_err(|e| ContextraError::InferenceError(e.to_string()))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut guard = self.model.lock().await;
        guard.infer_batch(texts).map_err(|e| ContextraError::InferenceError(e.to_string()))
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}
```

#### Datei 2: `crates/contextra-infer-onnx/src/reranker/mod.rs`
```rust
// In crates/contextra-infer-onnx/src/reranker/mod.rs
use async_trait::async_trait;
use contextra_ports::reranker::{RerankCandidate, Reranker};
use contextra_types::{ContextraError, Result};

pub struct OnnxCrossEncoderReranker {
    session: Arc<ort::Session>,
}

#[async_trait]
impl Reranker for OnnxCrossEncoderReranker {
    async fn rerank(
        &self,
        query: &str,
        candidates: &[RerankCandidate],
        top_k: usize,
    ) -> Result<Vec<RerankCandidate>> {
        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        // Asynchrone Offloading auf den Rayon/Blocking Pool
        let session = self.session.clone();
        let query_owned = query.to_string();
        let candidates_owned = candidates.to_vec();

        tokio::task::spawn_blocking(move || {
            let mut scored = candidates_owned;
            for cand in &mut scored {
                let score = run_onnx_inference(&session, &query_owned, &cand.text)?;
                cand.score = score;
            }
            // Deterministischer total_cmp
            scored.sort_by(|a, b| b.score.total_cmp(&a.score));
            scored.truncate(top_k);
            Ok(scored)
        })
        .await
        .map_err(|e| ContextraError::Internal(e.to_string()))?
    }
}
```

---

### 2.3 Komponente 3: KIVI 2-Bit Quantisierung mit Residual-Normalisierung

#### Mathematik & Kompressionsschema
- **Keys (Per-Channel Quantization):** Da verschiedene Kanäle unterschiedliche Varianzen aufweisen, wird jeder Kanal $c \in \{1,\dots,C\}$ separat auf 2-Bit skaliert:
  $$\text{scale}_c = \frac{\max_{t} X_{t,c} - \min_{t} X_{t,c}}{3}, \quad \text{zp}_c = \min_{t} X_{t,c}$$
- **Values (Per-Token Quantization):** Tokens weisen homogene Skalen auf; Quantisierung erfolgt zeilenweise:
  $$\text{scale}_t = \frac{\max_{c} X_{t,c} - \min_{c} X_{t,c}}{3}, \quad \text{zp}_t = \min_{c} X_{t,c}$$
- **Packing:** 4 quantisierte 2-Bit Werte werden deterministisch in ein einzelnes `u8` gepackt:
  $$\text{byte} = q_0 \mid (q_1 \ll 2) \mid (q_2 \ll 4) \mid (q_3 \ll 6)$$

#### Datei: `crates/contextra-kvcache/src/quantize_kivi.rs`
**IST-Zustand:** Unvollständige Residual-Längennormalisierung (Missing feature aus §1.3.1 der Algorithmen-Spezifikation).  
**SOLL-Blueprint:**
```rust
// In crates/contextra-kvcache/src/quantize_kivi.rs
use contextra_types::{ContextraError, Result};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuantizedTensor2Bit {
    pub packed_bytes: Vec<u8>,
    pub scales: Vec<f32>,
    pub zero_points: Vec<f32>,
    pub residual_norms: Option<Vec<f32>>,
    pub num_tokens: usize,
    pub channels: usize,
}

pub struct KiviQuantizer;

impl KiviQuantizer {
    /// Führt die Residual-Längennormalisierung vor der Quantisierung aus (arXiv:2402.02750 §3.2).
    pub fn normalize_residual_lengths(
        keys: &mut [f32],
        num_tokens: usize,
        channels: usize,
    ) -> Vec<f32> {
        let mut norms = Vec::with_capacity(num_tokens);
        for t in 0..num_tokens {
            let row = &mut keys[t * channels..(t + 1) * channels];
            let mut sum_sq = 0.0f32;
            for &val in row.iter() {
                sum_sq += val * val;
            }
            let norm = sum_sq.sqrt().max(1e-12);
            norms.push(norm);
            for val in row.iter_mut() {
                *val /= norm;
            }
        }
        norms
    }

    /// Quantisiert Key-Tensoren per-channel auf 2-Bit mit 4 Werten pro Byte.
    pub fn quantize_keys(
        keys: &[f32],
        num_tokens: usize,
        channels: usize,
        with_normalization: bool,
    ) -> Result<QuantizedTensor2Bit> {
        if keys.len() != num_tokens * channels {
            return Err(ContextraError::InvalidInput(
                "Input buffer length does not match token * channel dimensions".to_string(),
            ));
        }

        let mut work_keys = keys.to_vec();
        let residual_norms = if with_normalization {
            Some(Self::normalize_residual_lengths(&mut work_keys, num_tokens, channels))
        } else {
            None
        };

        let mut scales = vec![0.0f32; channels];
        let mut zero_points = vec![0.0f32; channels];
        let total_elements = num_tokens * channels;
        let mut packed_bytes = vec![0u8; (total_elements + 3) / 4];

        for c in 0..channels {
            let mut min_val = f32::INFINITY;
            let mut max_val = f32::NEG_INFINITY;

            for t in 0..num_tokens {
                let v = work_keys[t * channels + c];
                if v < min_val { min_val = v; }
                if v > max_val { max_val = v; }
            }

            let diff = max_val - min_val;
            let scale = if diff.abs() < f32::EPSILON { 1.0 } else { diff / 3.0 };
            scales[c] = scale;
            zero_points[c] = min_val;

            for t in 0..num_tokens {
                let v = work_keys[t * channels + c];
                let q = ((v - min_val) / scale).round().clamp(0.0, 3.0) as u8;
                let linear_idx = t * channels + c;
                let byte_idx = linear_idx / 4;
                let shift = (linear_idx % 4) * 2;
                packed_bytes[byte_idx] |= q << shift;
            }
        }

        Ok(QuantizedTensor2Bit {
            packed_bytes,
            scales,
            zero_points,
            residual_norms,
            num_tokens,
            channels,
        })
    }

    /// Rekonstruiert den unquantisierten f32-Puffer.
    pub fn dequantize_keys(quantized: &QuantizedTensor2Bit) -> Vec<f32> {
        let total_elements = quantized.num_tokens * quantized.channels;
        let mut reconstructed = vec![0.0f32; total_elements];

        for t in 0..quantized.num_tokens {
            for c in 0..quantized.channels {
                let linear_idx = t * quantized.channels + c;
                let byte_idx = linear_idx / 4;
                let shift = (linear_idx % 4) * 2;
                let q = (quantized.packed_bytes[byte_idx] >> shift) & 0x03;

                let val = (q as f32) * quantized.scales[c] + quantized.zero_points[c];
                reconstructed[linear_idx] = val;
            }

            // Normierung zurückrechnen, falls vorhanden
            if let Some(ref norms) = quantized.residual_norms {
                let norm = norms[t];
                for c in 0..quantized.channels {
                    reconstructed[t * quantized.channels + c] *= norm;
                }
            }
        }

        reconstructed
    }
}
```

---

### 2.4 Komponente 4: SnapKV Attention-Scoring & Deterministische Eviction

#### Mathematik
Im Beobachtungsfenster $W_{\text{obs}}$ der letzten $L_{\text{obs}}$ Tokens wird der kumulierte Aufmerksamkeits-Score berechnet:
$$S_i = \frac{1}{|W_{\text{obs}}|} \sum_{q \in W_{\text{obs}}} A_{q, i}$$
Segmente mit dem geringsten Score $S_i$ außerhalb des geschützten Prefix-Pools werden priorisiert eviktiert.

#### Datei: `crates/contextra-kvcache/src/attention_score.rs`
```rust
// In crates/contextra-kvcache/src/attention_score.rs (Zeile 30-90)
use contextra_types::ContextraError;

pub struct SnapKvSelector;

impl SnapKvSelector {
    /// Wählt die Top-K Token-Positionen basierend auf gemittelter Observation-Window-Attention.
    pub fn select_top_tokens(
        attention_matrix: &[f32],
        num_queries: usize,
        num_keys: usize,
        prefix_pool_len: usize,
        target_k: usize,
    ) -> Result<Vec<usize>, ContextraError> {
        if attention_matrix.len() != num_queries * num_keys {
            return Err(ContextraError::InvalidInput("Matrix dimension mismatch".to_string()));
        }

        let mut scores: Vec<(usize, f32)> = Vec::with_capacity(num_keys);

        // Prefix-Tokens [0..prefix_pool_len] sind IMMER geschützt
        for k in prefix_pool_len..num_keys {
            let mut sum_score = 0.0f32;
            for q in 0..num_queries {
                sum_score += attention_matrix[q * num_keys + k];
            }
            let mean_score = sum_score / (num_queries as f32);
            scores.push((k, mean_score));
        }

        // Deterministische Sortierung via total_cmp (P28)
        scores.sort_by(|a, b| b.1.total_cmp(&a.1));

        let mut retained_indices: Vec<usize> = (0..prefix_pool_len).collect();
        let top_candidates: Vec<usize> = scores.into_iter().take(target_k).map(|(idx, _)| idx).collect();
        retained_indices.extend(top_candidates);
        retained_indices.sort_unstable();

        Ok(retained_indices)
    }
}
```

---

### 2.5 Komponente 5: Irminsul Prefix-Radix-Trie & Tier-2 NVMe/mmap-Spillover

#### Architektur
Radix-Trie-Knoten verwalten gemeinsame Token-Präfixe mit atomarer Referenzzählung (`AtomicUsize`). Fällt die aktive Referenzzählung auf 1 (nur noch Cache-Holder), wird das Segment komprimiert in Tier-2 (NVMe/mmap) verschoben, wodurch der RAM entlastet wird.

#### Datei: `crates/contextra-kvcache/src/radix.rs`
```rust
// In crates/contextra-kvcache/src/radix.rs
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use ahash::AHashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageTier {
    Tier1Ram,
    Tier2Mmap { offset: u64, len: usize },
}

pub struct RadixNode {
    pub prefix_tokens: Vec<u32>,
    pub ref_count: Arc<AtomicUsize>,
    pub tier: StorageTier,
    pub children: AHashMap<u32, Box<RadixNode>>,
}

impl RadixNode {
    pub fn new(prefix: Vec<u32>) -> Self {
        Self {
            prefix_tokens: prefix,
            ref_count: Arc::new(AtomicUsize::new(1)),
            tier: StorageTier::Tier1Ram,
            children: AHashMap::new(),
        }
    }

    /// Markiert einen Knoten als kandidatenfähig für Tier-2 Auslagerung
    pub fn can_spill(&self) -> bool {
        self.ref_count.load(Ordering::Acquire) == 1 && matches!(self.tier, StorageTier::Tier1Ram)
    }

    pub fn mark_spilled(&mut self, offset: u64, len: usize) {
        self.tier = StorageTier::Tier2Mmap { offset, len };
    }
}
```

---

### 2.6 Komponente 6: Eviction-Worker Lifecycle

#### Datei: `crates/contextra-kvcache/src/eviction_worker.rs`
```rust
// In crates/contextra-kvcache/src/eviction_worker.rs
use std::sync::Arc;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use crate::store::TenantIsolatedKvStore;

pub struct KvEvictionWorkerHandle {
    handle: JoinHandle<()>,
}

pub fn spawn_kv_eviction_worker(
    store: Arc<TenantIsolatedKvStore>,
    mut shutdown_rx: watch::Receiver<bool>,
    interval_ms: u64,
) -> KvEvictionWorkerHandle {
    let handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(interval_ms));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    store.enforce_tenant_token_budgets();
                }
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        break;
                    }
                }
            }
        }
    });

    KvEvictionWorkerHandle { handle }
}
```

---

## 3. Akzeptanzkriterien, Testsuite & Qualitätsgates (EP4)

### 3.1 Unit-Test: KIVI 2-Bit Roundtrip & Rekonstruktionsfehler
**Datei:** `crates/contextra-kvcache/tests/kivi_quantization_roundtrip.rs`
```rust
use contextra_kvcache::quantize_kivi::KiviQuantizer;

#[test]
fn test_kivi_quantization_precision() {
    let num_tokens = 64;
    let channels = 128;
    let mut raw_keys = vec![0.0f32; num_tokens * channels];

    // Synthetische Gauß-Verteilung
    for (i, val) in raw_keys.iter_mut().enumerate() {
        *val = ((i % 100) as f32 / 50.0) - 1.0;
    }

    let quantized = KiviQuantizer::quantize_keys(&raw_keys, num_tokens, channels, true).unwrap();
    assert_eq!(quantized.packed_bytes.len(), (num_tokens * channels) / 4);

    let dequantized = KiviQuantizer::dequantize_keys(&quantized);

    // Cosine Similarity zwischen raw_keys und dequantized prüfen
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;
    for i in 0..raw_keys.len() {
        dot += raw_keys[i] * dequantized[i];
        norm_a += raw_keys[i] * raw_keys[i];
        norm_b += dequantized[i] * dequantized[i];
    }
    let cosine_sim = dot / (norm_a.sqrt() * norm_b.sqrt());
    assert!(cosine_sim > 0.95, "KIVI 2-bit cosine similarity must exceed 0.95, got {}", cosine_sim);
}
```

### 3.2 Verbindliche Acceptance-Kommandos
```bash
# 1. Ring-Layering Prüfung: Keine Ring-0/Ring-1 Abhängigkeit auf Ring 2 Inferenz
cargo xtask check-ring-layering

# 2. Testkit/MockEmbedder Ausschlussprüfung im Release-Build
cargo test -p contextra-ports --no-default-features

# 3. KIVI & SnapKV Testsuite
cargo test -p contextra-kvcache --locked
```
**Erwartetes Ergebnis:** Alle Tests müssen mit Exit-Code 0 terminieren; keine verwaisten Mocks im Release-Binary.

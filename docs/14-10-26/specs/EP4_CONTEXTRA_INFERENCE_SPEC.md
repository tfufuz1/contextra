# Contextra — Mikrofeingranulare Implementierungsspezifikation EP4: Contextra Inference & KV-Cache

**Dokumenten-ID:** `DOC-SPEC-EP4-INFERENCE-2026-10-14`  
**Version:** 1.0 · **Datum:** 14. Oktober 2026  
**Autor:** Principal Senior Rust Architect für Contextra  
**Status:** Verbindliche, normative Implementierungsanweisung für lokale Inferenz & KV-Cache  
**Geltungsbereich (Disjunktive Crates):**  
- **Ring 2 (Inferenz-Provider):** [`contextra-infer-candle`](file:///home/freddy/Projekte/contextra/crates/contextra-infer-candle), [`contextra-infer-onnx`](file:///home/freddy/Projekte/contextra/crates/contextra-infer-onnx), [`contextra-infer-ollama`](file:///home/freddy/Projekte/contextra/crates/contextra-infer-ollama)  
- **Ring 1 / Persistenz-nah:** `contextra-kvcache`  
- **Ring 0 (Ports):** [`contextra-ports`](file:///home/freddy/Projekte/contextra/crates/contextra-ports) (Modul `src/embedding.rs`, `src/kv_cache.rs`)

---

## 1. Architektonische Grenzziehung & Disjunktion

Contextra Inference (EP4) kapselt alle Machine-Learning-Ausführungen (lokale Embeddings via Candle/ONNX, Reranking, LLM-Inferenz via Ollama) und das KV-Cache-Management (KIVI 2-Bit, SnapKV, Irminsul).

**Disjunktionsregeln für EP4:**
1. **Keine Ring-0/Ring-1 Inferenz-Abhängigkeit:** Weder `contextra-store` noch `contextra-crypto` dürfen `candle`, `onnxruntime` oder `contextra-infer-*` als direkte Abhängigkeit führen. Der Zugriff erfolgt ausschließlich über die abstrakten Ports in `contextra-ports`.
2. **MockEmbedder-Isolation:** `MockEmbedder` darf niemals in Release-Artefakten von `contextra-ports` enthalten sein (nur unter `#[cfg(any(test, feature = "testkit"))]`).
3. **Deterministische Eviction:** Gleitkomma-Sortierungen bei KV-Cache-Evictions müssen zwingend über `f32::total_cmp` / `f64::total_cmp` erfolgen, um NaN-Vergiftungen und undefiniertes Verhalten auszuschließen.

---

## 2. Modul- und Datei-Feinspezifikationen

### 2.1 Komponente 1: Port-Bereinigung & `MockEmbedder` Gating

#### Problem
`MockEmbedder` in `crates/contextra-ports/src/embedding.rs:171` war ohne Feature-Gate deklariert und verblieb in Produktions-Binaries.

#### Datei: `crates/contextra-ports/src/embedding.rs`
```rust
#[async_trait::async_trait]
pub trait EmbeddingProvider: Send + Sync + 'static {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>, ContextraError>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, ContextraError>;
    fn dimension(&self) -> usize;
}

// MockEmbedder strikt isolieren:
#[cfg(any(test, feature = "testkit"))]
pub struct MockEmbedder {
    dimension: usize,
}
```

---

### 2.2 Komponente 2: KIVI 2-Bit Quantisierung mit Residual-Normalisierung (`contextra-kvcache`)

#### Mathematik & Algorithmus
Per-Channel Quantisierung für Keys, Per-Token Quantisierung für Values:
$$\hat{X}_{2\text{bit}} = \text{round}\left(\frac{X - \text{zp}}{\text{scale}}\right) \in \{0, 1, 2, 3\}$$
wobei $\text{scale} = \frac{\max(X) - \min(X)}{3}$ und $\text{zp} = \min(X)$.

#### Datei: `crates/contextra-kvcache/src/quantize_kivi.rs`
```rust
use contextra_types::ContextraError;

pub struct QuantizedTensor2Bit {
    pub packed_data: Vec<u8>, // 4 Werte pro Byte
    pub scales: Vec<f32>,
    pub zero_points: Vec<f32>,
    pub num_elements: usize,
}

pub fn kivi_quantize_keys_per_channel(
    raw_keys: &[f32],
    tokens: usize,
    channels: usize,
) -> Result<QuantizedTensor2Bit, ContextraError> {
    if raw_keys.len() != tokens * channels {
        return Err(ContextraError::invalid_input("Dimension mismatch"));
    }

    let mut scales = vec![0.0f32; channels];
    let mut zero_points = vec![0.0f32; channels];
    let mut packed_data = vec![0u8; (tokens * channels + 3) / 4];

    for c in 0..channels {
        let mut min_val = f32::MAX;
        let mut max_val = f32::MIN;
        for t in 0..tokens {
            let val = raw_keys[t * channels + c];
            if val < min_val { min_val = val; }
            if val > max_val { max_val = val; }
        }

        let diff = max_val - min_val;
        let scale = if diff.abs() < f32::EPSILON { 1.0 } else { diff / 3.0 };
        scales[c] = scale;
        zero_points[c] = min_val;

        for t in 0..tokens {
            let val = raw_keys[t * channels + c];
            let q = ((val - min_val) / scale).round().clamp(0.0, 3.0) as u8;
            let linear_idx = t * channels + c;
            let byte_idx = linear_idx / 4;
            let shift = (linear_idx % 4) * 2;
            packed_data[byte_idx] |= q << shift;
        }
    }

    Ok(QuantizedTensor2Bit {
        packed_data,
        scales,
        zero_points,
        num_elements: tokens * channels,
    })
}
```

---

### 2.3 Komponente 3: SnapKV Attention-Window Pooling & Irminsul Radix-Trie

#### Architektur
- **SnapKV:** Beobachtungsfenster $W_{obs}$ bewertet Wichtigkeit von Key-Tokens; wichtige Präfixe bleiben im RAM (Tier-1).
- **Irminsul:** Kaltes Token-Spillover deterministisch nach Tier-2 (mmap/NVMe).

#### Datei: `crates/contextra-kvcache/src/irminsul.rs`
```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct IrminsulPrefixNode {
    pub prefix_hash: u64,
    pub ref_count: Arc<AtomicUsize>,
    pub tier: CacheTier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheTier {
    RamTier1,
    NvmeTier2 { page_offset: u64, length_bytes: usize },
}

impl IrminsulPrefixNode {
    /// Verschiebt inaktive Seiten deterministisch in Tier-2 (Zero-Copy)
    pub fn spill_to_tier2(&mut self, offset: u64, len: usize) {
        if self.ref_count.load(Ordering::Acquire) <= 1 {
            self.tier = CacheTier::NvmeTier2 {
                page_offset: offset,
                length_bytes: len,
            };
        }
    }
}
```

---

### 2.4 Komponente 4: Eviction Worker Lifecycle Integration

#### Datei: `crates/contextra-kvcache/src/eviction_worker.rs`
```rust
use std::sync::Arc;
use tokio::sync::watch;

pub fn start_kv_cache_eviction(
    store: Arc<crate::store::TenantIsolatedKvStore>,
    mut shutdown_rx: watch::Receiver<bool>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(500));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    store.evict_over_budget_segments();
                }
                _ = shutdown_rx.changed() => {
                    break;
                }
            }
        }
    })
}
```

---

## 3. Akzeptanzkriterien & Qualitätsgates (EP4)

1. **Quantisierungs-Genauigkeit:**
   Der Rekonstruktionsfehler (Cosine-Distanz zwischen Original und entquantisiertem Vektor) muss für 128-dimensionale Tensoren $< 0.05$ betragen.
2. **Ring-Reinheit:**
   `cargo xtask check-ring-layering` bestätigt, dass kein Ring-0 oder Ring-1 Crate auf `contextra-infer-*` referenziert.

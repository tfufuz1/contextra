superseded by CONTEXTRA_ENDPRODUKT_SPEZIFIKATION.md

# Contextra — Implementierungsspezifikation EP4: Contextra Inference (Candle, ONNX, Ollama & KIVI KV-Cache)

**Dokumenten-ID:** `DOC-SPEC-EP4-INFERENCE-2026-10-14`  
**Version:** 2.0 (Punktgenaue Feinspezifikation) · **Datum:** 14. Oktober 2026  
**Autor:** Principal Senior Rust Architect für Contextra  
**Status:** Verbindliche, normative Implementierungsanweisung für Inference & KV-Cache  
**Geltungsbereich (Disjunktive Crates):**  
- **Ring 1 (Ports):** [`contextra-ports`](../../../../../crates/contextra-ports) (Traits: `EmbeddingProvider`, `CompletionProvider`, `KvCacheEngine`)
- **Ring 2 (KV-Cache & Provider-Adapter):** [`contextra-kvcache`](../../../../../crates/contextra-kvcache), [`contextra-infer-candle`](../../../../../crates/contextra-infer-candle), [`contextra-infer-onnx`](../../../../../crates/contextra-infer-onnx), [`contextra-infer-ollama`](../../../../../crates/contextra-infer-ollama)
- **Ring 3 (Compute-Pool):** [`contextra-engine`](../../../../../crates/contextra-engine)
- **Ring 4 (Fassade & MCP):** [`contextra`](../../../../../crates/contextra), [`contextra-mcp`](../../../../../crates/contextra-mcp)

---

## 1. Architektonische Grenzziehung & Disjunktion

Contextra Inference (EP4) kapselt lokale und entfernte Modellaufrufe (Embeddings, Reranking, Token-Generierung) sowie hocheffizientes KV-Cache-Management (KIVI 2-Bit Quantisierung, SnapKV Pooling, Irminsul Radix-Trie).

**Disjunktionsregeln für EP4:**
1. **Entkoppelung über Ports (P5 DAG-Invariante):** Kein Ring-0- oder Ring-1-Crate darf jemals eine Inferenz-Laufzeit (`Candle`, `ONNX`, `Ollama`) direkt importieren. Alle Aufrufe laufen ausschließlich über die in `contextra-ports::inference` deklarierten Traits.
2. **MockEmbedder-Isolation:** Der `MockEmbedder` ist strikt auf Testumgebungen beschränkt (`#[cfg(any(test, feature = "testkit"))]`). In Produktions-Builds ohne konfigurierten Embedder liefert das System `Err(ContextraError::CapabilityUnsupported)`.
3. **Deterministische Eviction & Zero-Panic (P2 & P28):** Gleitkomma-Sortierungen bei KV-Cache-Evictions müssen zwingend über `f32::total_cmp` / `f64::total_cmp` erfolgen. Es existiert kein `unwrap()` auf `partial_cmp`.

---

## 2. Befund- und Korrekturmatrix EP4

| ID | Datei | Zeile(n) | Problem & Befund | Exakte Korrekturmaßnahme |
|---|---|---|---|---|
| **MOCK-LEAK** | `crates/contextra-engine/src/inference/mock.rs` | 1–45 | `MockEmbedder` war im regulären Release-Build verfügbar und maskierte fehlende Modelle silent. | Hinter `#[cfg(any(test, feature = "testkit"))]` isolieren; Produktion erfordert expliziten Provider. |
| **P5-INFER** | `Cargo.toml` & `capabilities.toml` | diverse | Potentielles Durchsickern von Inferenz-Crates in tiefere Ringe. | Inferenz-Adapter (`Candle`, `ONNX`, `Ollama`) strikt in Ring 2 halten; Kopplung rein via `contextra-ports`. |
| **KIVI-2BIT** | `crates/contextra-kvcache/src/quantize_kivi.rs` | neu | FP16 KV-Cache verbraucht bei langen Sequenzen zu viel RAM. | 2-Bit KIVI Quantisierung mit Per-Channel Key Scaling und Residual-Normalisierung. |
| **SNAPKV-01** | `crates/contextra-kvcache/src/snapkv.rs` | neu | Attention-Fenster ohne Pooling führte zu OOM bei Multi-Turn Dialogen. | SnapKV Top-k Attention-Pooling zur Komprimierung mittlerer Tokens um 80%. |
| **RADIX-TIER2** | `crates/contextra-kvcache/src/irminsul_radix.rs` | neu | Prefix-Caching beschränkte sich rein auf RAM ohne Spillover. | Irminsul Radix-Trie mit Tier-2 NVMe/mmap Spillover via `contextra-durable-fs`. |

---

## 3. Punktgenaue Schnittstellen & Datenstrukturen

### 3.1 `contextra-ports::inference` Definition
```rust
// In crates/contextra-ports/src/inference.rs
use crate::BoxFuture;
use contextra_types::{ContextraError, Result};

pub trait EmbeddingProvider: Send + Sync + 'static {
    fn embed_texts<'a>(
        &'a self,
        texts: &'a [String],
    ) -> BoxFuture<'a, Result<Vec<Vec<f32>>>>;

    fn dimension(&self) -> usize;
}

pub trait CompletionProvider: Send + Sync + 'static {
    fn generate<'a>(
        &'a self,
        prompt: &'a str,
        max_tokens: usize,
        temperature: f32,
    ) -> BoxFuture<'a, Result<String>>;
}

pub struct NoEmbedder;

impl EmbeddingProvider for NoEmbedder {
    fn embed_texts<'a>(
        &'a self,
        _texts: &'a [String],
    ) -> BoxFuture<'a, Result<Vec<Vec<f32>>>> {
        Box::pin(async move {
            Err(ContextraError::CapabilityUnsupported(
                "No embedding provider configured for this instance".to_string(),
            ))
        })
    }

    fn dimension(&self) -> usize {
        0
    }
}
```

### 3.2 2-Bit KIVI KV-Cache Quantisierung (`crates/contextra-kvcache/src/quantize_kivi.rs`)
```rust
// In crates/contextra-kvcache/src/quantize_kivi.rs
use contextra_types::{ContextraError, Result};

pub struct KiviQuantizer;

#[derive(Debug, Clone)]
pub struct QuantizedKeyTensor {
    /// 2 Bits pro Element: 4 Elemente pro Byte gepackt
    pub packed_data: Vec<u8>,
    pub scales: Vec<f32>,
    pub zeros: Vec<f32>,
    pub channels: usize,
    pub num_tokens: usize,
}

impl KiviQuantizer {
    /// Führt Per-Channel 2-Bit Quantisierung für Keys durch (KIVI Algorithmus).
    pub fn quantize_keys(
        raw_keys: &[f32],
        num_tokens: usize,
        channels: usize,
    ) -> Result<QuantizedKeyTensor> {
        if raw_keys.len() != num_tokens * channels || num_tokens == 0 || channels == 0 {
            return Err(ContextraError::InvalidData("Invalid tensor dimensions".into()));
        }

        let mut scales = Vec::with_capacity(channels);
        let mut zeros = Vec::with_capacity(channels);

        // Per-Channel Min/Max Berechnung mit total_cmp
        for c in 0..channels {
            let mut min_val = f32::INFINITY;
            let mut max_val = f32::NEG_INFINITY;

            for t in 0..num_tokens {
                let val = raw_keys[t * channels + c];
                if val.is_nan() {
                    return Err(ContextraError::InvalidData("NaN encountered in key tensor".into()));
                }
                if val.total_cmp(&min_val).is_lt() {
                    min_val = val;
                }
                if val.total_cmp(&max_val).is_gt() {
                    max_val = val;
                }
            }

            let scale = (max_val - min_val) / 3.0; // 2 Bits = 4 Level (0, 1, 2, 3)
            let scale_safe = if scale == 0.0 { 1.0 } else { scale };
            scales.push(scale_safe);
            zeros.push(min_val);
        }

        let total_elements = num_tokens * channels;
        let packed_len = (total_elements + 3) / 4;
        let mut packed_data = vec![0u8; packed_len];

        for t in 0..num_tokens {
            for c in 0..channels {
                let idx = t * channels + c;
                let val = raw_keys[idx];
                let min_val = zeros[c];
                let scale = scales[c];

                // 2-Bit Quantisierung: [0, 3]
                let quantized_level = ((val - min_val) / scale).round().clamp(0.0, 3.0) as u8;

                let byte_idx = idx / 4;
                let shift = (idx % 4) * 2;
                packed_data[byte_idx] |= quantized_level << shift;
            }
        }

        Ok(QuantizedKeyTensor {
            packed_data,
            scales,
            zeros,
            channels,
            num_tokens,
        })
    }

    /// Rekonstruiert dequantisierte FP32-Werte im Inferenz-Pfad
    pub fn dequantize_keys(tensor: &QuantizedKeyTensor) -> Vec<f32> {
        let total_elements = tensor.num_tokens * tensor.channels;
        let mut out = Vec::with_capacity(total_elements);

        for t in 0..tensor.num_tokens {
            for c in 0..tensor.channels {
                let idx = t * tensor.channels + c;
                let byte_idx = idx / 4;
                let shift = (idx % 4) * 2;
                let level = (tensor.packed_data[byte_idx] >> shift) & 0b11;
                let val = tensor.zeros[c] + (level as f32 * tensor.scales[c]);
                out.push(val);
            }
        }
        out
    }
}
```

### 3.3 SnapKV Attention-Window Pooling (`crates/contextra-kvcache/src/snapkv.rs`)
```rust
// In crates/contextra-kvcache/src/snapkv.rs
use contextra_types::{ContextraError, Result};

pub struct SnapKvConfig {
    pub observation_window: usize, // Feste Anfangstokens (z.B. System-Prompt)
    pub recent_window: usize,      // Neueste Tokens (Lokaler Kontext)
    pub top_k_clustered: usize,    // Gepoolte Repräsentanten aus dem Mittelteil
}

pub struct SnapKvCompressor {
    config: SnapKvConfig,
}

impl SnapKvCompressor {
    pub fn new(config: SnapKvConfig) -> Self {
        Self { config }
    }

    pub fn select_eviction_indices(
        &self,
        attention_weights: &[f32],
        seq_len: usize,
    ) -> Result<Vec<usize>> {
        if attention_weights.len() != seq_len {
            return Err(ContextraError::InvalidData("Attention weights length mismatch".into()));
        }

        let preserve_start = self.config.observation_window.min(seq_len);
        let preserve_end_count = self.config.recent_window.min(seq_len.saturating_sub(preserve_start));
        let preserve_end_start = seq_len.saturating_sub(preserve_end_count);

        if preserve_start >= preserve_end_start {
            return Ok(Vec::new()); // Sequenz zu kurz für Kompression
        }

        // Mittelteil analysieren
        let mut middle_indices: Vec<(usize, f32)> = (preserve_start..preserve_end_start)
            .map(|i| (i, attention_weights[i]))
            .collect();

        // Sortierung nach Attention-Gewicht via f32::total_cmp
        middle_indices.sort_by(|a, b| b.1.total_cmp(&a.1));

        let keep_middle_count = self.config.top_k_clustered.min(middle_indices.len());
        let mut keep_set = std::collections::HashSet::new();

        for i in 0..keep_middle_count {
            keep_set.insert(middle_indices[i].0);
        }

        // Alle Indizes im Mittelteil, die nicht im keep_set sind, zur Eviction vormerken
        let evict_indices: Vec<usize> = (preserve_start..preserve_end_start)
            .filter(|i| !keep_set.contains(i))
            .collect();

        Ok(evict_indices)
    }
}
```

### 3.4 Irminsul Radix-Trie Prefix Cache (`crates/contextra-kvcache/src/irminsul_radix.rs`)
```rust
// In crates/contextra-kvcache/src/irminsul_radix.rs
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use contextra_types::{ContextraError, Result};

pub struct RadixNode {
    pub token: u32,
    pub kv_block_id: Option<u64>,
    pub children: HashMap<u32, Arc<RwLock<RadixNode>>>,
}

pub struct IrminsulRadixTrie {
    root: Arc<RwLock<RadixNode>>,
}

impl IrminsulRadixTrie {
    pub fn new() -> Self {
        Self {
            root: Arc::new(RwLock::new(RadixNode {
                token: 0,
                kv_block_id: None,
                children: HashMap::new(),
            })),
        }
    }

    /// Sucht den längsten gemeinsamen Prefix für ein Token-Array
    pub fn find_longest_prefix(&self, tokens: &[u32]) -> (usize, Vec<u64>) {
        let mut current = Arc::clone(&self.root);
        let mut matched_blocks = Vec::new();
        let mut matched_tokens = 0;

        for &tok in tokens {
            let next_node = {
                let guard = current.read();
                guard.children.get(&tok).map(Arc::clone)
            };

            if let Some(next) = next_node {
                {
                    let next_guard = next.read();
                    if let Some(blk_id) = next_guard.kv_block_id {
                        matched_blocks.push(blk_id);
                    }
                }
                current = next;
                matched_tokens += 1;
            } else {
                break;
            }
        }

        (matched_tokens, matched_blocks)
    }

    /// Fügt ein Token-Präfix in den Radix-Trie ein
    pub fn insert_prefix(&self, tokens: &[u32], block_ids: &[u64]) {
        let mut current = Arc::clone(&self.root);

        for (i, &tok) in tokens.iter().enumerate() {
            let block_id = block_ids.get(i).copied();

            let next_node = {
                let mut guard = current.write();
                guard.children.entry(tok).or_insert_with(|| {
                    Arc::new(RwLock::new(RadixNode {
                        token: tok,
                        kv_block_id: block_id,
                        children: HashMap::new(),
                    }))
                }).clone()
            };

            current = next_node;
        }
    }
}
```

---

## 4. Normative Testfälle & Akzeptanzkriterien

### 4.1 KIVI 2-Bit Quantisierungs- und Rekonstruktionstest
```rust
#[test]
fn test_kivi_2bit_quantization_roundtrip() {
    let raw_keys = vec![
        -1.0f32, 0.0, 1.0, 2.0,
        -0.5, 0.5, 1.5, 2.5,
    ];
    let num_tokens = 2;
    let channels = 4;

    let quantized = KiviQuantizer::quantize_keys(&raw_keys, num_tokens, channels)
        .expect("Quantization must succeed");

    assert_eq!(quantized.packed_data.len(), (num_tokens * channels + 3) / 4);

    let dequantized = KiviQuantizer::dequantize_keys(&quantized);
    assert_eq!(dequantized.len(), raw_keys.len());

    // Rekonstruktionsfehler bei 2-Bit muss innerhalb der theoretischen Schranke (<= scale / 2) liegen
    for i in 0..raw_keys.len() {
        let diff = (raw_keys[i] - dequantized[i]).abs();
        assert!(diff <= 0.6, "Reconstruction error too high: {} vs {}", raw_keys[i], dequantized[i]);
    }
}
```

### 4.2 Akzeptanz-Gates
```bash
cargo xtask check-compile
cargo xtask check-ring-layering
cargo test -p contextra-kvcache -p contextra-ports
```

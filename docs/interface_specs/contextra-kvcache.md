# Schnittstellenspezifikation: `contextra-kvcache`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-kvcache` |
| Version | workspace |
| Pfad | `crates/contextra-kvcache` |
| Rolle | KV-Cache |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> # Contextra KV-Cache (Ring 1)
>
> ANCHOR[contextra-kvcache:LIB-001]
>
> Dieses Modul bildet die zentrale Schnittstelle des `contextra-kvcache`-Crates.
>
> ## Was ist ein KV-Cache?
> In LLMs (Large Language Models = großen Sprachmodellen) bezeichnet der **KV-Cache**
> (Key-Value-Cache) zwischengespeicherte Zwischenzustände von Aufmerksamkeitsmatrizen
> (Attention-Keys und Values) vorheriger Token. Wenn mehrere Anfragen denselben Prompt-Präfix
> teilen (z. B. System-Prompts oder Dokumentenkontexte), kann durch Wiederverwendung des Caches
> massive Rechenzeit eingespart werden (Prompt Caching).

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `ahash` | normal | workspace | nein |  |
| `bincode` | normal | workspace | nein |  |
| `blake3` | normal | workspace | nein |  |
| `bytes` | normal | workspace | nein |  |
| `contextra-crypto` | dev | path | nein | contextra-crypto, contextra-crypto-fuzz, contextra-infer-candle, contextra-kvcache, contextra-mcp, contextra-store |
| `contextra-env` | normal | path | nein | contextra-checkpoint, contextra-crypto, contextra-graph, contextra-infer-candle, contextra-kvcache, contextra-mvcc, contextra-ports, contextra-store, contextra-text, contextra-vector |
| `contextra-ports` | dev | path | nein | contextra, contextra-adapt, contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-rank, contextra-router, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vector, contextra-vector-fuzz |
| `contextra-types` | dev | path | nein | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `criterion` | dev | workspace | nein |  |
| `lru` | normal | registry | nein |  |
| `parking_lot` | normal | workspace | nein |  |
| `proptest` | dev | registry | nein |  |
| `serde` | normal | workspace | nein |  |
| `serde_json` | normal | workspace | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tokio` | dev | workspace | nein |  |
| `tracing` | normal | workspace | nein |  |
| `zeroize` | normal | registry | nein |  |

**Verwendet von (Crate-Ebene):** –

## 3. Features

| Feature | Aktiviert |
|---|---|
| `content-addressed-kv-cache` | `exp-content-addressed` |
| `crypto-compat` | `dep:contextra-crypto` |
| `default` | `std`, `kv-encryption` |
| `exp-content-addressed` | – |
| `exp-kivi` | – |
| `kivi-quantization` | `exp-kivi` |
| `kv-encryption` | `dep:contextra-crypto`, `contextra-crypto/kv-encryption` |
| `kvcache-kivi-quant` | `exp-kivi` |
| `ports-compat` | `workspace` |
| `std` | – |
| `types-compat` | `workspace` |
| `workspace` | `dep:contextra-types`, `dep:contextra-ports` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 21 |
| Quellzeilen | 7811 |
| Öffentliche Items (Modul-Ebene) | 102 |
| davon `fn` | 12 |
| davon `struct` | 49 |
| davon `enum` | 5 |
| davon `trait` | 9 |
| davon `type` | 4 |
| davon `const` | 3 |
| davon `mod` | 20 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 27 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 0 |
| `.expect(` (Produktionscode) | 0 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 50 |
| `#[deprecated]` | 0 |
| `#[cfg(feature…)]`-Gates | 80 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 285 / 285 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `attention_score` | `crates/contextra-kvcache/src/attention_score.rs` | 347 |
| `cipher` | `crates/contextra-kvcache/src/cipher.rs` | 27 |
| `compat::adapters` | `crates/contextra-kvcache/src/compat/adapters.rs` | 259 |
| `compat::error` | `crates/contextra-kvcache/src/compat/error.rs` | 59 |
| `compat` | `crates/contextra-kvcache/src/compat/mod.rs` | 210 |
| `compat::prefix_store` | `crates/contextra-kvcache/src/compat/prefix_store.rs` | 687 |
| `compat::scanner` | `crates/contextra-kvcache/src/compat/scanner.rs` | 156 |
| `compat::scratchpad_invalidation` | `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs` | 325 |
| `contract` | `crates/contextra-kvcache/src/contract.rs` | 117 |
| `error` | `crates/contextra-kvcache/src/error.rs` | 89 |
| `eviction_worker` | `crates/contextra-kvcache/src/eviction_worker.rs` | 335 |
| `invalidation` | `crates/contextra-kvcache/src/invalidation.rs` | 23 |
| `kv_encryption` | `crates/contextra-kvcache/src/kv_encryption/mod.rs` | 254 |
| `crate` | `crates/contextra-kvcache/src/lib.rs` | 112 |
| `quantize_kivi` | `crates/contextra-kvcache/src/quantize_kivi.rs` | 842 |
| `radix` | `crates/contextra-kvcache/src/radix.rs` | 1198 |
| `segment` | `crates/contextra-kvcache/src/segment.rs` | 650 |
| `service` | `crates/contextra-kvcache/src/service.rs` | 637 |
| `spill` | `crates/contextra-kvcache/src/spill.rs` | 95 |
| `store` | `crates/contextra-kvcache/src/store.rs` | 1190 |
| `types` | `crates/contextra-kvcache/src/types.rs` | 199 |

## 6. Re-Exports (`pub use`)

- `pub use contextra_ports::attention::AttentionExporter` — `crates/contextra-kvcache/src/compat/adapters.rs:13` (Modul `compat::adapters`)
- `pub use contextra_ports::kv_bridge_port::KvBridgeStorage` — `crates/contextra-kvcache/src/compat/adapters.rs:15` (Modul `compat::adapters`)
- `pub use adapters::*` — `crates/contextra-kvcache/src/compat/mod.rs:7` (Modul `compat`)
- `pub use prefix_store::*` — `crates/contextra-kvcache/src/compat/mod.rs:8` (Modul `compat`)
- `pub use scanner::*` — `crates/contextra-kvcache/src/compat/mod.rs:9` (Modul `compat`)
- `pub use scratchpad_invalidation::*` — `crates/contextra-kvcache/src/compat/mod.rs:10` (Modul `compat`)
- `pub use contextra_crypto::` — `crates/contextra-kvcache/src/kv_encryption/mod.rs:6` (Modul `kv_encryption`)
- `pub use compat::` — `crates/contextra-kvcache/src/lib.rs:52` (Modul `crate`)
- `pub use cipher::` — `crates/contextra-kvcache/src/lib.rs:57` (Modul `crate`)
- `pub use error::` — `crates/contextra-kvcache/src/lib.rs:58` (Modul `crate`)
- `pub use types::` — `crates/contextra-kvcache/src/lib.rs:59` (Modul `crate`)
- `pub use adapters::` — `crates/contextra-kvcache/src/lib.rs:78` (Modul `crate`)
- `pub use attention_score::` — `crates/contextra-kvcache/src/lib.rs:82` (Modul `crate`)
- `pub use eviction_worker::` — `crates/contextra-kvcache/src/lib.rs:85` (Modul `crate`)
- `pub use prefix_store::TenantPrefixKvStore` — `crates/contextra-kvcache/src/lib.rs:87` (Modul `crate`)
- `pub use quantize_kivi::` — `crates/contextra-kvcache/src/lib.rs:89` (Modul `crate`)
- `pub use radix::` — `crates/contextra-kvcache/src/lib.rs:94` (Modul `crate`)
- `pub use radix::` — `crates/contextra-kvcache/src/lib.rs:97` (Modul `crate`)
- `pub use scratchpad_invalidation::` — `crates/contextra-kvcache/src/lib.rs:99` (Modul `crate`)
- `pub use segment::` — `crates/contextra-kvcache/src/lib.rs:102` (Modul `crate`)
- `pub use service::` — `crates/contextra-kvcache/src/lib.rs:107` (Modul `crate`)
- `pub use compat::scanner::` — `crates/contextra-kvcache/src/lib.rs:109` (Modul `crate`)
- `pub use store::` — `crates/contextra-kvcache/src/lib.rs:110` (Modul `crate`)
- `pub use invalidation::ScratchpadCacheScope` — `crates/contextra-kvcache/src/lib.rs:111` (Modul `crate`)
- `pub use spill::` — `crates/contextra-kvcache/src/lib.rs:112` (Modul `crate`)
- `pub use crate::types::` — `crates/contextra-kvcache/src/store.rs:27` (Modul `store`)
- `pub use crate::compat::scanner::` — `crates/contextra-kvcache/src/store.rs:794` (Modul `store`)

## 8. Öffentliche Schnittstelle je Item

### Modul `attention_score`

#### `struct NullAttentionScoreSource`
- Ort: `crates/contextra-kvcache/src/attention_score.rs:61`
```rust
pub struct NullAttentionScoreSource
```
Dokumentation: Standard-Implementierung, die keine Attention-Scores liefert (reines LRU-Verhalten). # Fallback-Verhalten Wenn kein Attention-Signal verfügbar ist, erfolgt die Verdrängung streng deterministisch nach dem LRU-Prinzip (älteste Zugriffszeit zuerst). Im Monitoring wird eine Warnmeldung geloggt, um das Fehlen des Signals transparent zu machen.
- Impl-Block (Trait `AttentionScoreSource`) `crates/contextra-kvcache/src/attention_score.rs:63`

#### `trait AttentionScoreSource`
- Ort: `crates/contextra-kvcache/src/attention_score.rs:31`
```rust
pub trait AttentionScoreSource: Send + Sync
```
Dokumentation: Dyn-kompatible Trait-Schnittstelle zur Bereitstellung von Attention-Wichtigkeits-Scores. Wird von Upstream-Inferenz-Backends (z. B. Candle/Ollama) oder Test-Mocks implementiert, um Aufmerksamkeitsscores bereitzustellen, ohne direkte Abhängigkeiten zu Tensor-Frameworks einzuführen.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn importance_score(&self, segment_id: u64) -> Option<f32>` | `crates/contextra-kvcache/src/attention_score.rs:37` | Liefert den Wichtigkeits-Score für ein gegebenes `segment_id`. - Höhere Werte bedeuten höhere Relevanz (bleibt länger im Cache). - Niedrigere Werte bedeuten ger |
| `fn importance_score_for_tenant(&self, _tenant_id: TenantId, segment_id: u64) -> Option<f32>` | `crates/contextra-kvcache/src/attention_score.rs:42` | Liefert den Wichtigkeits-Score für einen Mandanten und ein `segment_id`. Standardmäßig wird an [`Self::importance_score`] delegiert. |
| `fn register_segment(&self, _tenant_id: TenantId, _segment_id: u64, _request_id: RequestId)` | `crates/contextra-kvcache/src/attention_score.rs:47` | Registriert eine Zuordnung von `(tenant_id, segment_id)` zu `request_id`. |
| `fn unregister_segment(&self, _tenant_id: TenantId, _segment_id: u64)` | `crates/contextra-kvcache/src/attention_score.rs:50` | Entfernt eine Zuordnung, wenn das Segment verdrängt oder freigegeben wurde. |

#### `fn rank_for_eviction`
- Ort: `crates/contextra-kvcache/src/attention_score.rs:82`
```rust
pub fn rank_for_eviction( candidates: &[(u64, Instant)], scores: &dyn AttentionScoreSource, ) -> Vec<u64>
```
Dokumentation: Bewertet Verdrängungskandidaten mit Standard-Gewichtung (50% LRU-Alter, 50% Attention-Score). Gibt eine sortierte Liste von Segment-IDs zurück (erstes Element = wird zuerst verdrängt).

#### `fn rank_for_eviction_weighted`
- Ort: `crates/contextra-kvcache/src/attention_score.rs:96`
```rust
pub fn rank_for_eviction_weighted( candidates: &[(u64, Instant)], scores: &dyn AttentionScoreSource, attention_weight: f32, ) -> Vec<u64>
```
Dokumentation: Bewertet Verdrängungskandidaten unter Kombination von normalisiertem LRU-Alter und Attention-Score. # Parameter - `candidates`: Liste aus Tupeln `(Segment-ID, Zugriffszeitstempel)`. - `scores`: Quelle für Aufmerksamkeitsscores. - `attention_weight`: Gewichtung zwischen `0.0` (rein LRU) und `1.0` (rein Attention).

#### `fn rank_for_eviction_weighted_for_tenant`
- Ort: `crates/contextra-kvcache/src/attention_score.rs:115`
```rust
pub fn rank_for_eviction_weighted_for_tenant( tenant_id: TenantId, candidates: &[(u64, Instant)], scores: &dyn AttentionScoreSource, attention_weight: f32, ) -> Vec<u64>
```
Dokumentation: Bewertet Verdrängungskandidaten mandantenspezifisch. # Ablauf 1. Schnellausstieg bei 0 oder 1 Kandidaten. 2. Zeitstempel normalisieren: Zeitdifferenz zur ältesten Anfrage ermitteln. 3. Attention-Scores der Kandidaten abfragen und ungültige/nicht-endliche Werte (NaN, Inf) filtern. 4. Min/Max-Skalierung (Min-Max-Normalization) über gefundene Scores berechnen. 5. Zusammengesetzten Eviction-Score berechnen: `composite_score = (1.0 - weight) * norm_lru + weight * norm_importance`. 6. Stabil aufsteigend nach zusammengesetztem Score sortieren (niedrigste Wichtigkeit zuerst).

#### `fn rank_for_eviction_weighted_for_tenant_nanos`
- Ort: `crates/contextra-kvcache/src/attention_score.rs:146`
```rust
pub fn rank_for_eviction_weighted_for_tenant_nanos( tenant_id: TenantId, candidates: &[(u64, u64)], scores: &dyn AttentionScoreSource, attention_weight: f32, ) -> Vec<u64>
```
Dokumentation: Bewertet Verdrängungskandidaten mandantenspezifisch basierend auf monotonen Nanosekunden-Zeitstempeln.

### Modul `cipher`

#### `struct PlaintextSegmentCipher`
- Ort: `crates/contextra-kvcache/src/cipher.rs:17`
```rust
pub struct PlaintextSegmentCipher
```
Dokumentation: No-Op Cipher für Klartext-Caches und Tests.
- Impl-Block (Trait `SegmentCipher`) `crates/contextra-kvcache/src/cipher.rs:19`

#### `trait SegmentCipher`
- Ort: `crates/contextra-kvcache/src/cipher.rs:7`
```rust
pub trait SegmentCipher: Send + Sync
```
Dokumentation: Trait zur Segment-Verschlüsselung.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn encrypt_segment(&self, key: &SegmentKey, plaintext: &[u8]) -> Result<Vec<u8>>` | `crates/contextra-kvcache/src/cipher.rs:9` | Verschlüsselt Klartextbytes mit dem angegebenen Schlüssel. |
| `fn decrypt_segment(&self, key: &SegmentKey, ciphertext: &[u8]) -> Result<Vec<u8>>` | `crates/contextra-kvcache/src/cipher.rs:12` | Entschlüsselt Geheimtextbytes mit dem angegebenen Schlüssel. |

### Modul `compat`

#### `mod adapters`
- Ort: `crates/contextra-kvcache/src/compat/mod.rs:1`
```rust
pub mod adapters
```

#### `mod error`
- Ort: `crates/contextra-kvcache/src/compat/mod.rs:2`
```rust
pub mod error
```

#### `mod prefix_store`
- Ort: `crates/contextra-kvcache/src/compat/mod.rs:3`
```rust
pub mod prefix_store
```

#### `mod scanner`
- Ort: `crates/contextra-kvcache/src/compat/mod.rs:4`
```rust
pub mod scanner
```

#### `mod scratchpad_invalidation`
- Ort: `crates/contextra-kvcache/src/compat/mod.rs:5`
```rust
pub mod scratchpad_invalidation
```

#### `struct CryptoSegmentCipher`
- Ort: `crates/contextra-kvcache/src/compat/mod.rs:79`
```rust
pub struct CryptoSegmentCipher
```
Dokumentation: Adapter, der contextra-crypto an SegmentCipher anbindet.
- Gate: `#[cfg(feature = "crypto-compat")]`
- Impl-Block (Trait `SegmentCipher`) `crates/contextra-kvcache/src/compat/mod.rs:82`

### Modul `compat::adapters`

#### `struct EmbeddingProviderSemanticEmbedder`
- Ort: `crates/contextra-kvcache/src/compat/adapters.rs:77`
```rust
pub struct EmbeddingProviderSemanticEmbedder
```
Dokumentation: Adaptiert einen generischen [`EmbeddingProvider`] für den `SemanticEmbedder`-Trait im KV-Cache.
- Impl-Block `crates/contextra-kvcache/src/compat/adapters.rs:81`
- Impl-Block (Trait `SemanticEmbedder`) `crates/contextra-kvcache/src/compat/adapters.rs:94`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(provider: Arc<dyn EmbeddingProvider>) -> Self` | `crates/contextra-kvcache/src/compat/adapters.rs:83` | Erstellt einen neuen `EmbeddingProviderSemanticEmbedder`. |
| `pub fn provider(&self) -> &Arc<dyn EmbeddingProvider>` | `crates/contextra-kvcache/src/compat/adapters.rs:88` | Liefert eine Referenz auf den zugrundeliegenden [`EmbeddingProvider`]. |

#### `struct KvCachePluginConfig`
- Ort: `crates/contextra-kvcache/src/compat/adapters.rs:130`
```rust
pub struct KvCachePluginConfig
```
Dokumentation: Validierte Konfiguration für das KV-Cache-Plugin-Manifest.
- Impl-Block (Trait `Default`) `crates/contextra-kvcache/src/compat/adapters.rs:139`
- Impl-Block `crates/contextra-kvcache/src/compat/adapters.rs:149`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `max_memory_bytes` | `u64` | `crates/contextra-kvcache/src/compat/adapters.rs:134` |
| `feature_ring_required` | `FeatureRing` | `crates/contextra-kvcache/src/compat/adapters.rs:136` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn validate(&self) -> ContextraResult<()>` | `crates/contextra-kvcache/src/compat/adapters.rs:151` | Validiert die Konfigurationsparameter (mindestens 1 MiB RAM erforderlich). |

#### `struct KvCachePluginManifest`
- Ort: `crates/contextra-kvcache/src/compat/adapters.rs:164`
```rust
pub struct KvCachePluginManifest
```
Dokumentation: Konkrete, validierte [`PluginManifest`]-Implementierung für `contextra-kvcache`.
- Impl-Block `crates/contextra-kvcache/src/compat/adapters.rs:168`
- Impl-Block (Trait `PluginManifest`) `crates/contextra-kvcache/src/compat/adapters.rs:176`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(config: KvCachePluginConfig) -> ContextraResult<Self>` | `crates/contextra-kvcache/src/compat/adapters.rs:170` | Erstellt ein neues, validiertes `KvCachePluginManifest`. |

#### `struct StorageEngineKvBridge`
- Ort: `crates/contextra-kvcache/src/compat/adapters.rs:42`
```rust
pub struct StorageEngineKvBridge
```
Dokumentation: Produktiver Tier-2 Spill-over Adapter über das generische [`StorageEngine`]-Interface. Bietet mandantengetrennte Persistierung von KV-Segmenten im Storage-Layer.
- Impl-Block `crates/contextra-kvcache/src/compat/adapters.rs:46`
- Impl-Block (Trait `KvBridgeStorage`) `crates/contextra-kvcache/src/compat/adapters.rs:53`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(engine: Arc<dyn StorageEngine>) -> Self` | `crates/contextra-kvcache/src/compat/adapters.rs:48` | Erstellt eine neue `StorageEngineKvBridge` um eine gegebene [`StorageEngine`]. |

### Modul `compat::prefix_store`

#### `struct NamespacePrefixStore`
- Ort: `crates/contextra-kvcache/src/compat/prefix_store.rs:467`
```rust
pub struct NamespacePrefixStore
```
Dokumentation: Partitionsisolierter Prefix-Store über `namespace: u64`.
- Impl-Block `crates/contextra-kvcache/src/compat/prefix_store.rs:476`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(byte_budget: usize, max_blocks: usize) -> Self` | `crates/contextra-kvcache/src/compat/prefix_store.rs:478` | Erstellt einen neuen `NamespacePrefixStore`. |
| `pub fn with_reuse_policy(mut self, policy: KvReusePolicy) -> Self` | `crates/contextra-kvcache/src/compat/prefix_store.rs:490` | Konfiguriert die Wiederverwendungs-Policy. |
| `pub fn lookup(&self, namespace: u64, tokens: &[u32]) -> Option<KvPrefixHit>` | `crates/contextra-kvcache/src/compat/prefix_store.rs:496` | Sucht nach dem längsten Präfix für den gegebenen Namespace. |
| `pub fn pin_sequence(&self, namespace: u64, tokens: &[u32]) -> Result<bool, ContextraError>` | `crates/contextra-kvcache/src/compat/prefix_store.rs:518` | Pinnt eine Sequenz im Namespace (schützt vor Verdrängung). |
| `pub fn unpin_sequence(&self, namespace: u64, tokens: &[u32]) -> Result<bool, ContextraError>` | `crates/contextra-kvcache/src/compat/prefix_store.rs:540` | Hebt das Pinning einer Sequenz im Namespace auf. |
| `pub fn insert( &self, namespace: u64, tokens: &[u32], blocks: Vec<KvBlock>, ) -> Result<(), ContextraError>` | `crates/contextra-kvcache/src/compat/prefix_store.rs:562` | Fügt Blöcke für eine Token-Sequenz im Namespace ein. |
| `pub fn entry_count(&self, namespace: u64) -> usize` | `crates/contextra-kvcache/src/compat/prefix_store.rs:671` | Gibt die Gesamtzahl gespeicherter Block-Gruppen im Namespace zurück. |
| `pub fn total_bytes(&self, namespace: u64) -> usize` | `crates/contextra-kvcache/src/compat/prefix_store.rs:680` | Gibt die gesamten allokierten Bytes im Namespace zurück. |

#### `struct TenantPrefixKvStore`
- Ort: `crates/contextra-kvcache/src/compat/prefix_store.rs:72`
```rust
pub struct TenantPrefixKvStore
```
Dokumentation: Mandantenisolierter KV-Prefix-Store. Bietet strukturelle Isolation via `(TenantId, PrefixKey)`-Key in den Partitionen. Unterstützt konfigurierbares Byte-Budget pro Mandant und LRU-Eviction bei Überschreitung.
- Impl-Block `crates/contextra-kvcache/src/compat/prefix_store.rs:80`
- Impl-Block (Trait `Default`) `crates/contextra-kvcache/src/compat/prefix_store.rs:331`
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-kvcache/src/compat/prefix_store.rs:337`
- Impl-Block (Trait `KvPrefixStore`) `crates/contextra-kvcache/src/compat/prefix_store.rs:348`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-kvcache/src/compat/prefix_store.rs:82` | Erstellt einen neuen `TenantPrefixKvStore` mit Standard-Konfiguration (256 MiB Budget). |
| `pub fn with_byte_budget_per_tenant(mut self, budget: usize) -> Self` | `crates/contextra-kvcache/src/compat/prefix_store.rs:93` | Konfiguriert das maximale Byte-Budget pro Mandant über alle seine Präfix-Partitionen. |
| `pub fn with_reuse_policy(mut self, policy: KvReusePolicy) -> Self` | `crates/contextra-kvcache/src/compat/prefix_store.rs:99` | Konfiguriert die Wiederverwendungs-Policy ([`KvReusePolicy`]). |
| `pub fn lookup(&self, tenant: TenantId, key: &PrefixKey, tokens: &[u32]) -> Option<KvPrefixHit>` | `crates/contextra-kvcache/src/compat/prefix_store.rs:114` | Sucht nach dem längsten übereinstimmenden Präfix in Blöcken für die angegebenen Tokens. # Parameter - `tenant`: Eigentümer-Mandant. - `key`: Modell- und Layout- |
| `pub fn insert( &self, tenant: TenantId, key: &PrefixKey, tokens: &[u32], blocks: Vec<KvBlock>, ) -> Result<(), ContextraError>` | `crates/contextra-kvcache/src/compat/prefix_store.rs:143` | Fügt KV-Blöcke für ein Token-Präfix in den Store ein. # Ablauf 1. Eingabe prüfen (weder Tokens noch Blöcke dürfen leer sein). 2. Prüfen, ob die Blockgruppe alle |
| `pub fn evict(&self, tenant: TenantId, key: &PrefixKey) -> Result<u64, ContextraError>` | `crates/contextra-kvcache/src/compat/prefix_store.rs:204` | Evictiert KV-Präfix-Einträge für den angegebenen Mandanten und PrefixKey. Gibt die Anzahl der entfernten Blöcke zurück. |
| `pub fn invalidate_prefix_scope_keys( &self, tenant: TenantId, scratchpad_key_prefix: &[u32], keys: &[PrefixKey], ) -> Result<usize, ContextraError>` | `crates/contextra-kvcache/src/compat/prefix_store.rs:218` | Gezielte Invalidation eines Scratchpad-Scopes für den angegebenen Mandanten über alle oder spezifische PrefixKeys. Entfernt ausschließlich Einträge, deren Token |

#### `const DEFAULT_BYTE_BUDGET_PER_TENANT`
- Ort: `crates/contextra-kvcache/src/compat/prefix_store.rs:30`
```rust
pub const DEFAULT_BYTE_BUDGET_PER_TENANT: usize = 256 * 1024 * 1024
```
Dokumentation: Standard-Byte-Budget pro Mandant (256 MiB).

### Modul `compat::scanner`

#### `struct KvCachePruner`
- Ort: `crates/contextra-kvcache/src/compat/scanner.rs:9`
```rust
pub struct KvCachePruner
```
Dokumentation: Bereinigungs-Adapter für physische Segmentspeicher-Freigabe im KV-Cache gemäß Port-Schnittstelle.
- Impl-Block `crates/contextra-kvcache/src/compat/scanner.rs:13`
- Impl-Block (Trait `LayerPurger`) `crates/contextra-kvcache/src/compat/scanner.rs:20`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(store: Arc<TenantIsolatedKvStore>) -> Self` | `crates/contextra-kvcache/src/compat/scanner.rs:15` | Erstellt einen neuen `KvCachePruner`. |

#### `struct KvCacheScanner`
- Ort: `crates/contextra-kvcache/src/compat/scanner.rs:50`
```rust
pub struct KvCacheScanner
```
Dokumentation: Scanner-Adapter zur Restmengen-Messung im KV-Cache gemäß Port-Schnittstelle.
- Impl-Block `crates/contextra-kvcache/src/compat/scanner.rs:54`
- Impl-Block (Trait `contextra_ports::scanner::LayerScanner`) `crates/contextra-kvcache/src/compat/scanner.rs:68`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(store: Arc<TenantIsolatedKvStore>) -> Self` | `crates/contextra-kvcache/src/compat/scanner.rs:56` | Erstellt einen neuen `KvCacheScanner`. |

### Modul `compat::scratchpad_invalidation`

#### `struct BoundedScratchpadInvalidator`
- Ort: `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:267`
```rust
pub struct BoundedScratchpadInvalidator<'a>
```
Dokumentation: Begrenzter Invalidator für Scratchpads mit deterministischer Schranke.
- Impl-Block `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:273`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `store` | `&'a TenantPrefixKvStore` | `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:268` |
| `keys` | `Vec<PrefixKey>` | `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:269` |
| `budget` | `InvalidationBudget` | `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:270` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(store: &'a TenantPrefixKvStore, budget: InvalidationBudget) -> Self` | `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:275` | Erstellt einen neuen bounded Invalidator mit Budget. |
| `pub fn with_keys(mut self, keys: Vec<PrefixKey>) -> Self` | `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:284` | Setzt die zu prüfenden PrefixKeys. |
| `pub fn invalidate_bounded(&self, scope: &ScratchpadCacheScope) -> Result<usize>` | `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:290` | Führt die Invalidierung unter Beachtung des Budgets aus. |

#### `struct InvalidationBudget`
- Ort: `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:250`
```rust
pub struct InvalidationBudget
```
Dokumentation: Konfiguration zur Begrenzung von Scratchpad-Invalidierungs-Operationen.
- Impl-Block (Trait `Default`) `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:257`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `max_depth` | `usize` | `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:252` |
| `max_entries` | `usize` | `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:254` |

#### `struct PrefixStoreScratchpadInvalidator`
- Ort: `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:59`
```rust
pub struct PrefixStoreScratchpadInvalidator<'a>
```
Dokumentation: Implementierung von [`ScratchpadInvalidator`] für [`TenantPrefixKvStore`].
- Impl-Block `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:66`
- Impl-Block (Trait `ScratchpadInvalidator`) `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:82`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `store` | `&'a TenantPrefixKvStore` | `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:61` |
| `keys` | `Vec<PrefixKey>` | `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:63` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(store: &'a TenantPrefixKvStore) -> Self` | `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:68` | Erstellt einen neuen [`PrefixStoreScratchpadInvalidator`] für den angegebenen Store. |
| `pub fn with_keys(mut self, keys: Vec<PrefixKey>) -> Self` | `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:76` | Konfiguriert die zu prüfenden [`PrefixKey`]s für die Invalidation. |

#### `struct ScratchpadCacheScope`
- Ort: `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:30`
```rust
pub struct ScratchpadCacheScope
```
Dokumentation: Definiert den Bereich (Scope) eines Agenten-Scratchpads für die Cache-Invalidierung.
- Impl-Block `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:37`
- Impl-Block (Trait `From<crate::invalidation::ScratchpadCacheScope>`) `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:309`
- Impl-Block (Trait `From<ScratchpadCacheScope>`) `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:318`
- Impl-Block `crates/contextra-kvcache/src/invalidation.rs:15`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tenant_id` | `TenantId` | `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:32` |
| `scratchpad_key_prefix` | `Vec<u32>` | `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:34` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(tenant_id: TenantId, scratchpad_key_prefix: Vec<u32>) -> Self` | `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:39` | Erstellt einen neuen [`ScratchpadCacheScope`] für den angegebenen Mandanten und das Token-Präfix. |
| `pub fn new(tenant_id: impl Into<TenantId>, scratchpad_key_prefix: Vec<u32>) -> Self` | `crates/contextra-kvcache/src/invalidation.rs:17` | Erstellt einen neuen [`ScratchpadCacheScope`]. |

#### `trait ScratchpadInvalidator`
- Ort: `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:49`
```rust
pub trait ScratchpadInvalidator
```
Dokumentation: Schnittstelle zur gezielten Invalidation von Scratchpad-Cache-Scopes.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn invalidate_scope(&self, scope: &ScratchpadCacheScope) -> Result<usize>` | `crates/contextra-kvcache/src/compat/scratchpad_invalidation.rs:54` | Invalidiert alle Gruppen/Blöcke im Radix-Baum, die zum angegebenen [`ScratchpadCacheScope`] gehören. # Fehler Gibt ein `Result<usize>` zurück, wobei `usize` die |

### Modul `contract`

#### `struct MemoryKvCache`
- Ort: `crates/contextra-kvcache/src/contract.rs:52`
```rust
pub struct MemoryKvCache
```
Dokumentation: I/O-freie Referenzimplementierung eines KV-Caches im Hauptspeicher.
- Impl-Block `crates/contextra-kvcache/src/contract.rs:56`
- Impl-Block (Trait `Default`) `crates/contextra-kvcache/src/contract.rs:77`
- Impl-Block (Trait `KvCacheEngine`) `crates/contextra-kvcache/src/contract.rs:83`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-kvcache/src/contract.rs:58` | Erstellt eine neue Instanz des Memory-KV-Caches. |
| `pub fn with_shard_count(shard_count: usize) -> Self` | `crates/contextra-kvcache/src/contract.rs:65` | Erstellt eine Instanz mit benutzerdefinierter Shard-Anzahl. |
| `pub fn store(&self) -> &Arc<TenantIsolatedKvStore>` | `crates/contextra-kvcache/src/contract.rs:72` | Liefert eine Referenz auf den internen Store. |

#### `trait Clock`
- Ort: `crates/contextra-kvcache/src/contract.rs:15`
```rust
pub trait Clock: Send + Sync
```
Dokumentation: Minimaler Zeiterfassungs-Trait für das KV-Cache-Subsystem.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn now_unix_ms(&self) -> u64` | `crates/contextra-kvcache/src/contract.rs:17` | Liefert die aktuellen Unix-Epochen-Millisekunden. |

#### `trait KvCacheEngine`
- Ort: `crates/contextra-kvcache/src/contract.rs:34`
```rust
pub trait KvCacheEngine: Send + Sync
```
Dokumentation: Kernabstraktion für mandantenisolierte KV-Cache-Operationen.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn insert(&self, tenant_id: TenantId, segment: KvSegment) -> Result<()>` | `crates/contextra-kvcache/src/contract.rs:36` | Fügt ein KV-Segment ein. |
| `fn get(&self, tenant_id: TenantId, segment_id: u64) -> Option<Vec<u8>>` | `crates/contextra-kvcache/src/contract.rs:39` | Ruft die Rohbytes eines Segments ab. |
| `fn find_longest_prefix( &self, tenant_id: TenantId, tokens: &[u32], policy: KvReusePolicy, ) -> Option<PrefixMatch>` | `crates/contextra-kvcache/src/contract.rs:42` | Sucht den längsten übereinstimmenden Präfix für eine Token-Sequenz. |

#### `trait Rng`
- Ort: `crates/contextra-kvcache/src/contract.rs:28`
```rust
pub trait Rng: Send
```
Dokumentation: Minimaler Zufallszahlen-Generator-Trait für Cache-Algorithmen.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn next_u64(&mut self) -> u64` | `crates/contextra-kvcache/src/contract.rs:30` | Erzeugt eine 64-Bit Pseudozufallszahl. |

### Modul `crate`

#### `mod attention_score`
- Ort: `crates/contextra-kvcache/src/lib.rs:64`
```rust
pub mod attention_score
```

#### `mod cipher`
- Ort: `crates/contextra-kvcache/src/lib.rs:47`
```rust
pub mod cipher
```

#### `mod compat`
- Ort: `crates/contextra-kvcache/src/lib.rs:49`
```rust
pub mod compat
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod contract`
- Ort: `crates/contextra-kvcache/src/lib.rs:53`
```rust
pub mod contract
```

#### `mod error`
- Ort: `crates/contextra-kvcache/src/lib.rs:54`
```rust
pub mod error
```

#### `mod eviction_worker`
- Ort: `crates/contextra-kvcache/src/lib.rs:69`
```rust
pub mod eviction_worker
```

#### `mod invalidation`
- Ort: `crates/contextra-kvcache/src/lib.rs:65`
```rust
pub mod invalidation
```

#### `mod kv_encryption`
- Ort: `crates/contextra-kvcache/src/lib.rs:67`
```rust
pub mod kv_encryption
```
- Gate: `#[cfg(feature = "kv-encryption")]`

#### `mod quantize_kivi`
- Ort: `crates/contextra-kvcache/src/lib.rs:71`
```rust
pub mod quantize_kivi
```
- Gate: `#[cfg(feature = "exp-kivi")]`

#### `mod radix`
- Ort: `crates/contextra-kvcache/src/lib.rs:72`
```rust
pub mod radix
```

#### `mod segment`
- Ort: `crates/contextra-kvcache/src/lib.rs:73`
```rust
pub mod segment
```

#### `mod service`
- Ort: `crates/contextra-kvcache/src/lib.rs:74`
```rust
pub mod service
```

#### `mod spill`
- Ort: `crates/contextra-kvcache/src/lib.rs:68`
```rust
pub mod spill
```

#### `mod store`
- Ort: `crates/contextra-kvcache/src/lib.rs:75`
```rust
pub mod store
```

#### `mod types`
- Ort: `crates/contextra-kvcache/src/lib.rs:55`
```rust
pub mod types
```

### Modul `error`

#### `enum KvCacheError`
- Ort: `crates/contextra-kvcache/src/error.rs:8`
```rust
pub enum KvCacheError
```
Dokumentation: Zentraler Fehlertyp für alle KV-Cache-Operationen.
- Impl-Block (Trait `From<ContextraError>`) `crates/contextra-kvcache/src/compat/error.rs:40`
- Impl-Block `crates/contextra-kvcache/src/error.rs:75`
- Impl-Block (Trait `From<std::io::Error>`) `crates/contextra-kvcache/src/error.rs:82`

Varianten:

- `InvalidTenant(String)` — `crates/contextra-kvcache/src/error.rs:11`
- `InvalidInput(String)` — `crates/contextra-kvcache/src/error.rs:15`
- `InvalidSegment(String)` — `crates/contextra-kvcache/src/error.rs:19`
- `SegmentNotFound(u64)` — `crates/contextra-kvcache/src/error.rs:23`
- `CryptoError(String)` — `crates/contextra-kvcache/src/error.rs:27`
- `SegmentKeyDestroyed` — `crates/contextra-kvcache/src/error.rs:31`
- `BudgetExceeded(String)` — `crates/contextra-kvcache/src/error.rs:35`
- `PinBudgetExceeded(String)` — `crates/contextra-kvcache/src/error.rs:39`
- `LimitExceeded{` — `crates/contextra-kvcache/src/error.rs:43`
- `PolicyViolation(String)` — `crates/contextra-kvcache/src/error.rs:52`
- `Serialization(String)` — `crates/contextra-kvcache/src/error.rs:56`
- `Internal(String)` — `crates/contextra-kvcache/src/error.rs:60`
- `ConfigError(String)` — `crates/contextra-kvcache/src/error.rs:64`
- `QuantizationError(String)` — `crates/contextra-kvcache/src/error.rs:68`
- `IoError(#[source] std::io::Error)` — `crates/contextra-kvcache/src/error.rs:72`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn pin_budget_exceeded(msg: impl Into<String>) -> Self` | `crates/contextra-kvcache/src/error.rs:77` | Hilfskonstruktor für PinBudgetExceeded. |

#### `type Result`
- Ort: `crates/contextra-kvcache/src/error.rs:89`
```rust
pub type Result<T> = std::result::Result<T, KvCacheError>
```
Dokumentation: Standard-Ergebnistyp für `contextra-kvcache`.

### Modul `eviction_worker`

#### `struct EvictionWorker`
- Ort: `crates/contextra-kvcache/src/eviction_worker.rs:91`
```rust
pub struct EvictionWorker
```
Dokumentation: Asynchroner Hintergrund-Worker für die KV-Cache-Verdrängung. Läuft auf einem eigenständigen Betriebssystem-Thread, um die Tokio-Worker-Threads nicht durch aufwendige Speicherfreigaben und Zeroize-Routinen zu belasten.
- Impl-Block `crates/contextra-kvcache/src/eviction_worker.rs:99`
- Impl-Block (Trait `Drop`) `crates/contextra-kvcache/src/eviction_worker.rs:213`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn spawn(store: Arc<TenantIsolatedKvStore>) -> Self` | `crates/contextra-kvcache/src/eviction_worker.rs:107` | Spawnt den Eviction-Worker auf einem dedizierten OS-Thread. # Ablauf 1. Initialisiert Standard-[`NullAttentionScoreSource`]. 2. Erzeugt einen MPSC-Kanal für Ver |
| `pub fn with_attention_exporter(self, exporter: Arc<dyn AttentionExporter>) -> Self` | `crates/contextra-kvcache/src/eviction_worker.rs:154` | Builder-Methode zur Injektion eines echten Attention-Backends via [`AttentionExporter`]. |
| `pub fn register_segment_request( &self, tenant_id: impl Into<TenantId>, segment_id: u64, request_id: impl Into<RequestId>, )` | `crates/contextra-kvcache/src/eviction_worker.rs:163` | Registriert eine Zuordnung von `(tenant_id, segment_id)` zu einer `request_id`. |
| `pub fn unregister_segment_request(&self, tenant_id: impl Into<TenantId>, segment_id: u64)` | `crates/contextra-kvcache/src/eviction_worker.rs:175` | Entfernt die Zuordnung für `(tenant_id, segment_id)`. |
| `pub fn attention_source(&self) -> Arc<dyn AttentionScoreSource>` | `crates/contextra-kvcache/src/eviction_worker.rs:182` | Liefert die aktuell konfigurierte [`AttentionScoreSource`]. |
| `pub fn trigger_eviction(&self, target_free_bytes: usize)` | `crates/contextra-kvcache/src/eviction_worker.rs:189` | Löst nicht-blockierend eine Verdrängung im Hintergrund aus. Sendet den Auftrag über den MPSC-Kanal an den Worker-Thread und kehrt sofort zurück. |
| `pub fn notify_block_released(&self, block_id: u64)` | `crates/contextra-kvcache/src/eviction_worker.rs:197` | Benachrichtigt den EvictionWorker, dass eine Block-Referenz freigegeben wurde. |
| `pub fn shutdown(&self)` | `crates/contextra-kvcache/src/eviction_worker.rs:205` | Beendet den Worker-Thread geordnet und wartet auf dessen Abschluss (Join). |

#### `fn emergency_wipe`
- Ort: `crates/contextra-kvcache/src/eviction_worker.rs:225`
```rust
pub fn emergency_wipe(store: &TenantIsolatedKvStore)
```
Dokumentation: Synchroner Notfall-Wipe für sofortige Speicherbereinigung. Löscht blockierend alle Segmente im Store synchron aus dem RAM und stellt sicher, dass alle vertraulichen Tensordaten mittels Zeroize überschrieben sind, bevor die Funktion zurückkehrt.

### Modul `invalidation`

#### `struct ScratchpadCacheScope`
- Ort: `crates/contextra-kvcache/src/invalidation.rs:8`
```rust
pub struct ScratchpadCacheScope
```
Dokumentation: Definiert den Bereich (Scope) eines Agenten-Scratchpads für die Cache-Invalidierung.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tenant_id` | `TenantId` | `crates/contextra-kvcache/src/invalidation.rs:10` |
| `scratchpad_key_prefix` | `Vec<u32>` | `crates/contextra-kvcache/src/invalidation.rs:12` |

### Modul `kv_encryption`

#### `struct CryptoKeyAead`
- Ort: `crates/contextra-kvcache/src/kv_encryption/mod.rs:38`
```rust
pub struct CryptoKeyAead(pub CryptoKey)
```
Dokumentation: Wrapper um `contextra_crypto::CryptoKey`, der das lokale `SegmentAead`-Trait implementiert.
- Impl-Block (Trait `SegmentAead`) `crates/contextra-kvcache/src/kv_encryption/mod.rs:40`

#### `struct EncryptedSegmentPayload`
- Ort: `crates/contextra-kvcache/src/kv_encryption/mod.rs:26`
```rust
pub struct EncryptedSegmentPayload
```
Dokumentation: Verschlüsselte Layer-Repräsentation innerhalb eines `KvSegment`, wenn Feature `kv-encryption` aktiv ist.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `layer` | `EncryptedKvLayer` | `crates/contextra-kvcache/src/kv_encryption/mod.rs:28` |
| `key_derivation_version` | `u8` | `crates/contextra-kvcache/src/kv_encryption/mod.rs:31` |
| `bound_segment_id` | `Option<u64>` | `crates/contextra-kvcache/src/kv_encryption/mod.rs:34` |

#### `fn to_crypto_tenant`
- Ort: `crates/contextra-kvcache/src/kv_encryption/mod.rs:252`
```rust
pub fn to_crypto_tenant<T: serde::de::DeserializeOwned + Default>(tenant: TenantId) -> T
```
Dokumentation: Konvertiert einen lokalen `TenantId` in den von `contextra-crypto` erwarteten Typ.

### Modul `quantize_kivi`

#### `struct KiviBlockMeta`
- Ort: `crates/contextra-kvcache/src/quantize_kivi.rs:115`
```rust
pub struct KiviBlockMeta
```
Dokumentation: Metadaten zur Dequantisierung eines 2-Bit KIVI-Blocks.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `bits_per_value` | `u8` | `crates/contextra-kvcache/src/quantize_kivi.rs:117` |
| `scale` | `f32` | `crates/contextra-kvcache/src/quantize_kivi.rs:119` |
| `zero_point` | `f32` | `crates/contextra-kvcache/src/quantize_kivi.rs:121` |
| `original_len` | `usize` | `crates/contextra-kvcache/src/quantize_kivi.rs:123` |
| `num_tokens` | `usize` | `crates/contextra-kvcache/src/quantize_kivi.rs:127` |
| `num_channels` | `usize` | `crates/contextra-kvcache/src/quantize_kivi.rs:130` |
| `key_group_size` | `usize` | `crates/contextra-kvcache/src/quantize_kivi.rs:133` |
| `quantize_values` | `bool` | `crates/contextra-kvcache/src/quantize_kivi.rs:136` |
| `key_scales` | `Vec<f32>` | `crates/contextra-kvcache/src/quantize_kivi.rs:139` |
| `key_zero_points` | `Vec<f32>` | `crates/contextra-kvcache/src/quantize_kivi.rs:142` |
| `value_scales` | `Vec<f32>` | `crates/contextra-kvcache/src/quantize_kivi.rs:145` |
| `value_zero_points` | `Vec<f32>` | `crates/contextra-kvcache/src/quantize_kivi.rs:148` |

#### `struct KiviQuantizeConfig`
- Ort: `crates/contextra-kvcache/src/quantize_kivi.rs:33`
```rust
pub struct KiviQuantizeConfig
```
Dokumentation: Konfiguration für die KIVI-Quantisierungsparameter.
- Impl-Block (Trait `Default`) `crates/contextra-kvcache/src/quantize_kivi.rs:40`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `key_group_size` | `usize` | `crates/contextra-kvcache/src/quantize_kivi.rs:35` |
| `quantize_values` | `bool` | `crates/contextra-kvcache/src/quantize_kivi.rs:37` |

#### `struct KiviQuantizedBlock`
- Ort: `crates/contextra-kvcache/src/quantize_kivi.rs:154`
```rust
pub struct KiviQuantizedBlock
```
Dokumentation: 2-Bit-quantisierter Byte-Block mit zugehörigen Dequantisierungs-Metadaten.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `meta` | `KiviBlockMeta` | `crates/contextra-kvcache/src/quantize_kivi.rs:156` |
| `packed` | `Vec<u8>` | `crates/contextra-kvcache/src/quantize_kivi.rs:158` |

#### `struct KvTensorView`
- Ort: `crates/contextra-kvcache/src/quantize_kivi.rs:52`
```rust
pub struct KvTensorView
```
Dokumentation: Sicht (View) auf unquantisierte Roh-Tensoren für Keys und Values.
- Impl-Block `crates/contextra-kvcache/src/quantize_kivi.rs:63`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `keys` | `Vec<f32>` | `crates/contextra-kvcache/src/quantize_kivi.rs:54` |
| `values` | `Vec<f32>` | `crates/contextra-kvcache/src/quantize_kivi.rs:56` |
| `num_tokens` | `usize` | `crates/contextra-kvcache/src/quantize_kivi.rs:58` |
| `num_channels` | `usize` | `crates/contextra-kvcache/src/quantize_kivi.rs:60` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( keys: Vec<f32>, values: Vec<f32>, num_tokens: usize, num_channels: usize, ) -> Result<Self>` | `crates/contextra-kvcache/src/quantize_kivi.rs:68` | Erstellt einen neuen `KvTensorView` und prüft die Dimensionen gegen die Elementanzahl. # Fehler Liefert [`KvCacheError::InvalidInput`], falls `keys.len()` oder  |
| `pub fn to_bytes(&self) -> Result<Vec<u8>>` | `crates/contextra-kvcache/src/quantize_kivi.rs:102` | Serialisiert die Tensor-Sicht in Bytes via Bincode. |
| `pub fn from_bytes(bytes: &[u8]) -> Result<Self>` | `crates/contextra-kvcache/src/quantize_kivi.rs:107` | Deserialisiert eine Tensor-Sicht aus Bytes via Bincode. |

#### `fn compress_bytes`
- Ort: `crates/contextra-kvcache/src/quantize_kivi.rs:651`
```rust
pub fn compress_bytes(input: &[u8]) -> Vec<u8>
```
Dokumentation: Komprimiert ein Byte-Slice verlustfrei für Tier-2 Auslagerungen (LSM-Spill). Nutzt ein einfaches Run-Length-Encoding (RLE) mit Header `0x43` ('C' für Compressed) bzw. Fallback `0x55` ('U' für Uncompressed), falls RLE keine Ersparnis bringt.

#### `fn decompress_bytes`
- Ort: `crates/contextra-kvcache/src/quantize_kivi.rs:688`
```rust
pub fn decompress_bytes(input: &[u8]) -> Result<Vec<u8>>
```
Dokumentation: Dekomprimiert ein zuvor mit [`compress_bytes`] komprimiertes Byte-Slice. # Fehler Liefert [`KvCacheError::QuantizationError`], falls die Eingabe leer ist oder der Header ungültig ist.

#### `fn kivi_dequantize`
- Ort: `crates/contextra-kvcache/src/quantize_kivi.rs:372`
```rust
pub fn kivi_dequantize(bytes: &[u8], meta: &KiviBlockMeta) -> Result<KvTensorView>
```
Dokumentation: Dequantisiert gepackte 2-Bit-Bytes zurück in einen [`KvTensorView`]. # Ablauf 1. Metadaten validieren (`bits_per_value == 2`, Skalen und Nullpunkte müssen endlich sein). 2. Payload deserialisieren. 3. 2-Bit-Codes entpacken und mit `v = zp + code * sc` in f32 zurückrechnen.

#### `fn kivi_quantize`
- Ort: `crates/contextra-kvcache/src/quantize_kivi.rs:177`
```rust
pub fn kivi_quantize(raw: &KvTensorView, config: KiviQuantizeConfig) -> Result<KiviQuantizedBlock>
```
Dokumentation: Quantisiert einen unkomprimierten [`KvTensorView`] in einen 2-Bit [`KiviQuantizedBlock`]. # Ablauf 1. Dimensionen prüfen und Gruppengröße festlegen. 2. **Keys quantisieren**: Für jede Kanalgruppe Min- und Max-Werte ermitteln, Scale und Zero-Point berechnen und die Werte auf 2 Bit (0..=3) packen (4 Werte pro Byte). 3. **Values quantisieren**: Pro Token Min/Max berechnen und in 2 Bit packen (bzw. unquantisiert serialisieren, falls `quantize_values == false`). 4. Metadaten und gepackte Byte-Payload in [`KiviQuantizedBlock`] assemblieren.

#### `fn pack_kivi_block`
- Ort: `crates/contextra-kvcache/src/quantize_kivi.rs:568`
```rust
pub fn pack_kivi_block(values: &[f32]) -> KiviQuantizedBlock
```
Dokumentation: Referenz-Implementierung für lineare 2-Bit Min/Max-Quantisierung eines Fließkomma-Slices.

#### `fn unpack_kivi_block`
- Ort: `crates/contextra-kvcache/src/quantize_kivi.rs:520`
```rust
pub fn unpack_kivi_block(block: &KiviQuantizedBlock) -> Result<Vec<f32>>
```
Dokumentation: Entpackt einen einzelnen linearen 2-Bit-Block zurück in ein `Vec<f32>`.

### Modul `radix`

#### `struct ContentAddressedKvStore`
- Ort: `crates/contextra-kvcache/src/radix.rs:723`
```rust
pub struct ContentAddressedKvStore
```
Dokumentation: Content-adressierter KV-Store mit mehrstufiger Lookup-Kaskade. Unterstützt: 1. Schnelle Präfix-Suche im Radix-Baum. 2. Inhalts-Hash-Suche via BLAKE3 für exakt gleiche Token-Slices an anderer Position. 3. Optionale semantische Ähnlichkeitssuche mittels Embeddings.
- Impl-Block `crates/contextra-kvcache/src/radix.rs:733`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `position_index` | `PrefixRadixTree` | `crates/contextra-kvcache/src/radix.rs:725` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(primary_tenant: TenantId) -> Self` | `crates/contextra-kvcache/src/radix.rs:735` | Erstellt einen neuen `ContentAddressedKvStore` für den primären Mandanten. |
| `pub fn with_reuse_policy(mut self, policy: KvReusePolicy) -> Self` | `crates/contextra-kvcache/src/radix.rs:747` | Konfiguriert die Präfix-Wiederverwendungs-Policy. |
| `pub fn with_semantic_config(mut self, config: SemanticCacheConfig) -> Self` | `crates/contextra-kvcache/src/radix.rs:753` | Konfiguriert den opt-in semantischen Embedder und Schwellenwert. |
| `pub fn position_index(&self) -> &PrefixRadixTree` | `crates/contextra-kvcache/src/radix.rs:759` | Gibt eine Referenz auf den primären `position_index` zurück. |
| `pub fn hash_tokens(tokens: &[u32]) -> blake3::Hash` | `crates/contextra-kvcache/src/radix.rs:764` | Konvertiert Token-IDs deterministisch in LE-Bytes (Little Endian) und berechnet den BLAKE3-Hash. |
| `pub fn insert(&mut self, tenant: TenantId, tokens: &[u32], segment_id: u64) -> Result<()>` | `crates/contextra-kvcache/src/radix.rs:779` | Fügt eine Token-Sequenz und Segment-Referenz in den Store ein. # Ablauf 1. Validieren, dass die Sequenz nicht leer ist. 2. In den mandantenspezifischen `positio |
| `pub fn lookup(&self, tenant: TenantId, token_ids: &[u32]) -> KvLookupResult` | `crates/contextra-kvcache/src/radix.rs:816` | Lookup-Kaskade: 1. **Exact prefix hit** via `position_index` 2. **Content hash hit** via `content_index.get(&(tenant, blake3::hash(tokens)))` 3. **Semantic simi |
| `pub fn remove(&mut self, tenant: TenantId, tokens: &[u32]) -> Option<KvSegmentRef>` | `crates/contextra-kvcache/src/radix.rs:877` | Entfernt einen Eintrag aus allen Indizes für einen Mandanten. |
| `pub fn len(&self) -> usize` | `crates/contextra-kvcache/src/radix.rs:899` | Gibt die Anzahl gespeicherter Content-Index-Einträge zurück. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-kvcache/src/radix.rs:904` | Prüft, ob der Content-Index leer ist. |

#### `struct KvBlockGuard`
- Ort: `crates/contextra-kvcache/src/radix.rs:75`
```rust
pub struct KvBlockGuard
```
Dokumentation: RAII-Guard für ausgeliehene/gematchte KV-Blöcke. Verhindert, dass ein Block während einer laufenden Modell-Inferenz vom `EvictionWorker` verdrängt wird. Dekrementiert bei `Drop` atomar die Referenzzählung des Blocks und benachrichtigt ggf. den `EvictionWorker`, wenn der Zähler auf 0 fällt.
- Impl-Block `crates/contextra-kvcache/src/radix.rs:86`
- Impl-Block (Trait `Drop`) `crates/contextra-kvcache/src/radix.rs:119`
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-kvcache/src/radix.rs:138`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `block_id` | `u64` | `crates/contextra-kvcache/src/radix.rs:77` |
| `tenant_id` | `TenantId` | `crates/contextra-kvcache/src/radix.rs:79` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( block_id: u64, tenant_id: TenantId, ref_counter: Arc<AtomicUsize>, worker: Option<Arc<EvictionWorker>>, ) -> Self` | `crates/contextra-kvcache/src/radix.rs:88` | Erstellt einen neuen Guard und inkrementiert den Referenzzähler atomar (`SeqCst`). |
| `pub fn block_id(&self) -> u64` | `crates/contextra-kvcache/src/radix.rs:104` | Gibt die Block-ID zurück. |
| `pub fn tenant_id(&self) -> TenantId` | `crates/contextra-kvcache/src/radix.rs:109` | Gibt die Mandanten-ID zurück. |
| `pub fn active_refs(&self) -> usize` | `crates/contextra-kvcache/src/radix.rs:114` | Gibt die aktuelle Anzahl aktiver Referenzen zurück. |

#### `struct KvSegmentRef`
- Ort: `crates/contextra-kvcache/src/radix.rs:641`
```rust
pub struct KvSegmentRef
```
Dokumentation: Referenz auf ein gecachtes KV-Segment im content-adressierten Store.
- Impl-Block `crates/contextra-kvcache/src/radix.rs:651`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `segment_id` | `u64` | `crates/contextra-kvcache/src/radix.rs:643` |
| `tenant_id` | `TenantId` | `crates/contextra-kvcache/src/radix.rs:645` |
| `matched_tokens` | `Vec<u32>` | `crates/contextra-kvcache/src/radix.rs:647` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(segment_id: u64, tenant_id: TenantId, matched_tokens: Vec<u32>) -> Self` | `crates/contextra-kvcache/src/radix.rs:653` | Erstellt eine neue Segment-Referenz. |
| `pub fn matched_len(&self) -> usize` | `crates/contextra-kvcache/src/radix.rs:662` | Gibt die Anzahl der gematchten Tokens zurück. |

#### `struct NamespaceRadixTree`
- Ort: `crates/contextra-kvcache/src/radix.rs:994`
```rust
pub struct NamespaceRadixTree
```
Dokumentation: Partitionsisolierter Radix-Baum über `namespace: u64`.
- Impl-Block `crates/contextra-kvcache/src/radix.rs:1001`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(namespace: u64) -> Self` | `crates/contextra-kvcache/src/radix.rs:1003` | Erstellt einen neuen partitionsisolierten Radix-Baum. |
| `pub fn namespace(&self) -> u64` | `crates/contextra-kvcache/src/radix.rs:1014` | Gibt den zugehörigen Namespace zurück. |
| `pub fn insert(&mut self, tokens: &[u32], segment_id: u64) -> Result<SharedSegmentRef>` | `crates/contextra-kvcache/src/radix.rs:1019` | Fügt eine Token-Sequenz ein. |
| `pub fn find_longest_prefix( &self, tokens: &[u32], policy: KvReusePolicy, ) -> Option<PrefixMatch>` | `crates/contextra-kvcache/src/radix.rs:1096` | Sucht das längste übereinstimmende Präfix im Namespace. |
| `pub fn remove(&mut self, tokens: &[u32]) -> Option<u64>` | `crates/contextra-kvcache/src/radix.rs:1149` | Entfernt eine Sequenz und dekrementiert die geteilte Referenz. |
| `pub fn len(&self) -> usize` | `crates/contextra-kvcache/src/radix.rs:1185` | Gibt die Gesamtzahl gespeicherter Präfixe zurück. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-kvcache/src/radix.rs:1190` | Prüft, ob der Baum leer ist. |
| `pub fn segment_registry(&self) -> &ahash::AHashMap<Vec<u32>, SharedSegmentRef>` | `crates/contextra-kvcache/src/radix.rs:1195` | Gibt eine Referenz auf die interne Segment-Registrierung zurück. |

#### `struct PrefixMatch`
- Ort: `crates/contextra-kvcache/src/radix.rs:151`
```rust
pub struct PrefixMatch
```
Dokumentation: Ergebnis eines Präfix-Matches im Radix-Baum.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `matched_tokens` | `Vec<u32>` | `crates/contextra-kvcache/src/radix.rs:153` |
| `matched_len` | `usize` | `crates/contextra-kvcache/src/radix.rs:155` |
| `block_id` | `u64` | `crates/contextra-kvcache/src/radix.rs:157` |

#### `struct PrefixRadixTree`
- Ort: `crates/contextra-kvcache/src/radix.rs:190`
```rust
pub struct PrefixRadixTree
```
Dokumentation: Komprimierter Prefix-Radix-Baum für Token-Sequenzen. Bietet schnelle Präfixsuche und automatische Knoten-Aufspaltung (Node Splitting), wenn sich Sequenzen in gemeinsame Teilpfade verzweigen.
- Impl-Block `crates/contextra-kvcache/src/radix.rs:196`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(tenant_id: impl Into<TenantId>) -> Self` | `crates/contextra-kvcache/src/radix.rs:198` | Erstellt einen neuen Prefix-Radix-Baum für den angegebenen Mandanten. |
| `pub fn tenant_id(&self) -> TenantId` | `crates/contextra-kvcache/src/radix.rs:207` | Gibt die Mandanten-ID des Radix-Baums zurück. |
| `pub fn insert(&mut self, tokens: &[u32], block_id: u64) -> Result<()>` | `crates/contextra-kvcache/src/radix.rs:218` | Fügt eine Token-Sequenz mit assoziierter KV-Block-ID ein. # Ablauf 1. Prüfen, ob die Sequenz nicht leer ist (leere Sequenzen erzeugen [`KvCacheError::InvalidInp |
| `pub fn find_longest_prefix( &self, tokens: &[u32], policy: KvReusePolicy, ) -> Option<PrefixMatch>` | `crates/contextra-kvcache/src/radix.rs:296` | Sucht das längste übereinstimmende Präfix unter Anwendung der [`KvReusePolicy`]. # Ablauf 1. Schnellausstieg bei leeren Tokens oder Policy `Never`. 2. Iterative |
| `pub fn remove(&mut self, tokens: &[u32]) -> Option<u64>` | `crates/contextra-kvcache/src/radix.rs:350` | Entfernt eine Token-Sequenz aus dem Baum und gibt die bisherige Block-ID zurück. |
| `pub fn len(&self) -> usize` | `crates/contextra-kvcache/src/radix.rs:381` | Gibt die Anzahl gespeicherter Präfixe zurück. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-kvcache/src/radix.rs:386` | Prüft, ob der Baum leer ist. |
| `pub fn clear(&mut self)` | `crates/contextra-kvcache/src/radix.rs:391` | Leert den gesamten Baum vollständig. |

#### `struct SemanticCacheConfig`
- Ort: `crates/contextra-kvcache/src/radix.rs:697`
```rust
pub struct SemanticCacheConfig
```
Dokumentation: Konfiguration für opt-in semantische Vektor-Cache-Lookups.
- Impl-Block `crates/contextra-kvcache/src/radix.rs:705`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `embedder` | `Arc<dyn SemanticEmbedder>` | `crates/contextra-kvcache/src/radix.rs:699` |
| `similarity_threshold` | `f32` | `crates/contextra-kvcache/src/radix.rs:701` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(embedder: Arc<dyn SemanticEmbedder>, similarity_threshold: f32) -> Self` | `crates/contextra-kvcache/src/radix.rs:707` | Erstellt eine neue Konfiguration mit Embedder und Mindestähnlichkeit. |

#### `struct SharedSegmentRef`
- Ort: `crates/contextra-kvcache/src/radix.rs:935`
```rust
pub struct SharedSegmentRef
```
Dokumentation: Referenzzählungs- und Segment-Tracking-Struktur für `INV-KV-PREFIX-1`.
- Impl-Block `crates/contextra-kvcache/src/radix.rs:942`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `segment_id` | `u64` | `crates/contextra-kvcache/src/radix.rs:937` |
| `ref_count` | `Arc<AtomicUsize>` | `crates/contextra-kvcache/src/radix.rs:939` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(segment_id: u64) -> Self` | `crates/contextra-kvcache/src/radix.rs:944` | Erstellt eine neue geteilte Referenz mit initialem Referenzzähler 1. |
| `pub fn retain(&self) -> Self` | `crates/contextra-kvcache/src/radix.rs:952` | Klont die Referenz und inkrementiert den Zähler atomar. |
| `pub fn current_refs(&self) -> usize` | `crates/contextra-kvcache/src/radix.rs:961` | Gibt die aktuelle Anzahl von Referenzen zurück. |
| `pub fn release(&self) -> usize` | `crates/contextra-kvcache/src/radix.rs:966` | Dekrementiert den Zähler und gibt den verbleibenden Wert zurück. |

#### `enum KvLookupResult`
- Ort: `crates/contextra-kvcache/src/radix.rs:671`
```rust
pub enum KvLookupResult
```
Dokumentation: Ergebnis eines KV-Cache-Lookups mit [`ContentAddressedKvStore`].

Varianten:

- `ExactPrefixHit(KvSegmentRef)` — `crates/contextra-kvcache/src/radix.rs:673`
- `ContentHashHit(KvSegmentRef)` — `crates/contextra-kvcache/src/radix.rs:675`
- `SemanticSimilarityHit{` — `crates/contextra-kvcache/src/radix.rs:677`
- `Miss` — `crates/contextra-kvcache/src/radix.rs:682`

#### `enum KvReusePolicy`
- Ort: `crates/contextra-kvcache/src/radix.rs:37`
```rust
pub enum KvReusePolicy
```
Dokumentation: Policy zur Steuerung von Prefix-Reuse (§9.2). Definiert, ab wann ein gefundener Präfix-Treffer lohnend genug ist, um einen Cache-Treffer zu melden (da kurze Treffer von 1–3 Tokens oft mehr Lookup-Overhead erzeugen als Inferenzzeit sparen).
- Impl-Block (Trait `Default`) `crates/contextra-kvcache/src/radix.rs:46`
- Impl-Block `crates/contextra-kvcache/src/radix.rs:52`

Varianten:

- `Always` — `crates/contextra-kvcache/src/radix.rs:39`
- `CostBased{ min_prefix_len: usize }` — `crates/contextra-kvcache/src/radix.rs:41`
- `Never` — `crates/contextra-kvcache/src/radix.rs:43`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn should_reuse(&self, matched_len: usize) -> bool` | `crates/contextra-kvcache/src/radix.rs:60` | Prüft, ob ein Treffer mit der Länge `matched_len` wiederverwendet werden soll. # Parameter - `matched_len`: Anzahl übereinstimmender Tokens. # Rückgabe `true`,  |

#### `trait SemanticEmbedder`
- Ort: `crates/contextra-kvcache/src/radix.rs:688`
```rust
pub trait SemanticEmbedder: Send + Sync
```
Dokumentation: Trait für semantische Embedding-Generierung auf KV-Cache-Keys.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn embed(&self, tokens: &[u32]) -> Result<Vec<f32>>` | `crates/contextra-kvcache/src/radix.rs:690` | Erzeugt ein Embedding für eine Token-Sequenz. |

### Modul `segment`

#### `struct KvSegment`
- Ort: `crates/contextra-kvcache/src/segment.rs:184`
```rust
pub struct KvSegment
```
Dokumentation: Ein KV-Cache-Segment im Hauptspeicher. Erfüllt die P9-Sicherheitsanforderung: Zeroize-on-Drop für vertrauliche Daten. Darf niemals unverschlüsselt auf Sekundärspeicher (Tier-2) geschrieben werden.
- Impl-Block `crates/contextra-kvcache/src/kv_encryption/mod.rs:74`
- Impl-Block `crates/contextra-kvcache/src/segment.rs:212`
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-kvcache/src/segment.rs:310`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tenant_id` | `TenantId` | `crates/contextra-kvcache/src/segment.rs:186` |
| `segment_id` | `u64` | `crates/contextra-kvcache/src/segment.rs:188` |
| `key_derivation_version` | `u8` | `crates/contextra-kvcache/src/segment.rs:191` |
| `encrypted` | `bool` | `crates/contextra-kvcache/src/segment.rs:194` |
| `model_fingerprint` | `Option<crate::kv_encryption::ModelFingerprint>` | `crates/contextra-kvcache/src/segment.rs:197` |
| `rope_offset` | `Option<usize>` | `crates/contextra-kvcache/src/segment.rs:200` |
| `active_refs` | `Arc<AtomicUsize>` | `crates/contextra-kvcache/src/segment.rs:203` |
| `data` | `Vec<u8>` | `crates/contextra-kvcache/src/segment.rs:205` |
| `content` | `KvSegmentContent` | `crates/contextra-kvcache/src/segment.rs:207` |
| `encrypted_payload` | `Option<EncryptedSegmentPayload>` | `crates/contextra-kvcache/src/segment.rs:209` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new_encrypted( cipher: &KvSegmentCipher, tenant_id: impl Into<TenantId>, segment_id: u64, model_fingerprint: ModelFingerprint, rope_offset: Option<usize>, plaintext: &[u8], ) -> std::result::Result<Self, CryptoError>` | `crates/contextra-kvcache/src/kv_encryption/mod.rs:76` | Erstellt ein verschlüsseltes KV-Cache-Segment aus einem Klartext-Tensor. |
| `pub fn decrypt_data( &self, cipher: &KvSegmentCipher, ) -> std::result::Result<Vec<u8>, CryptoError>` | `crates/contextra-kvcache/src/kv_encryption/mod.rs:115` | Entschlüsselt die Daten des Segments, falls es verschlüsselt ist. |
| `pub fn write_quantized( &mut self, raw_kv: &KvTensorView, config: KiviQuantizeConfig, cipher: &dyn KvCipher, ) -> Result<()>` | `crates/contextra-kvcache/src/kv_encryption/mod.rs:163` | Schreibt ein quantisiertes und verschlüsseltes Segment (Reihenfolge: Quantisierung -> Komprimierung -> AEAD-Verschlüsselung). |
| `pub fn read_dequantized(&self, cipher: &dyn KvCipher) -> Result<KvTensorView>` | `crates/contextra-kvcache/src/kv_encryption/mod.rs:190` | Liest ein quantisiertes und verschlüsseltes Segment (Reihenfolge: AEAD-Entschlüsselung -> Dekomprimierung -> Dequantisierung). |
| `pub fn new(tenant_id: impl Into<TenantId>, segment_id: u64, data: Vec<u8>) -> Self` | `crates/contextra-kvcache/src/segment.rs:219` | Erstellt ein neues Klartext-KV-Cache-Segment (Default / Zero-Config, P12-konform). # Parameter - `tenant_id`: Zugehöriger Mandant. - `segment_id`: Eindeutige Ke |
| `pub fn new_with_metadata( tenant_id: TenantId, segment_id: u64, data: Vec<u8>, #[cfg(feature = "")] model_fingerprint: Option<crate::kv_encryption::ModelFingerprint>, rope_offset: Option<usize>, ) -> Self` | `crates/contextra-kvcache/src/segment.rs:240` | Erstellt ein neues Klartext-KV-Cache-Segment mit optionalen Metadaten. Ermöglicht die Übergabe eines RoPE-Offsets und eines Modell-Fingerprints. |
| `pub fn active_refs(&self) -> usize` | `crates/contextra-kvcache/src/segment.rs:266` | Gibt die aktuelle Anzahl aktiver Referenzen (`KvBlockGuard`) zurück. |
| `pub fn acquire_guard(&self, worker: Option<Arc<EvictionWorker>>) -> KvBlockGuard` | `crates/contextra-kvcache/src/segment.rs:271` | Erstellt einen [`KvBlockGuard`] für diesen Block und inkrementiert den Referenzzähler atomar. |
| `pub fn to_spill_bytes(&self) -> Vec<u8>` | `crates/contextra-kvcache/src/segment.rs:282` | Serialisiert das Segment bzw. dessen verschlüsselte Layer für Tier-2-LSM-Spill (Auslagerung). |
| `pub fn as_bytes(&self) -> &[u8]` | `crates/contextra-kvcache/src/segment.rs:295` | Read-Only-Zugriff auf die Rohdaten. Kein unverschlüsselter Export nach außen ohne explizite Autorisierung. |
| `pub fn len(&self) -> usize` | `crates/contextra-kvcache/src/segment.rs:300` | Länge der Tensor-Bytes in Bytes. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-kvcache/src/segment.rs:305` | Prüft, ob das Segment leer ist (0 Bytes). |

#### `struct SegmentHandle`
- Ort: `crates/contextra-kvcache/src/segment.rs:507`
```rust
pub struct SegmentHandle
```
Dokumentation: Generational handle to an allocated segment in the pool.
- Impl-Block `crates/contextra-kvcache/src/segment.rs:512`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn slot_index(&self) -> usize` | `crates/contextra-kvcache/src/segment.rs:514` | Returns the slot index of this handle. |
| `pub const fn generation(&self) -> u64` | `crates/contextra-kvcache/src/segment.rs:519` | Returns the generation counter of this handle. |

#### `struct SegmentPool`
- Ort: `crates/contextra-kvcache/src/segment.rs:534`
```rust
pub struct SegmentPool
```
Dokumentation: Bounded segment pool providing generational safety, use-after-free & double-free prevention, zeroization on deallocation, and fragmentation metrics tracking.
- Impl-Block `crates/contextra-kvcache/src/segment.rs:539`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(config: SegmentPoolConfig) -> Self` | `crates/contextra-kvcache/src/segment.rs:541` | Creates a new `SegmentPool` with the given configuration. |
| `pub fn allocate(&self, namespace: u64, data: &[u8]) -> Result<SegmentHandle>` | `crates/contextra-kvcache/src/segment.rs:559` | Allocates a segment block within the given namespace, storing the payload data. |
| `pub fn deallocate(&self, namespace: u64, handle: SegmentHandle) -> Result<()>` | `crates/contextra-kvcache/src/segment.rs:586` | Deallocates the segment slot indicated by handle, zeroizing payload memory and bumping generation. |
| `pub fn read(&self, namespace: u64, handle: SegmentHandle) -> Result<Vec<u8>>` | `crates/contextra-kvcache/src/segment.rs:608` | Reads payload data from an active handle. |
| `pub fn allocated_blocks(&self) -> usize` | `crates/contextra-kvcache/src/segment.rs:625` | Returns the total count of currently allocated blocks. |
| `pub fn allocated_bytes(&self) -> usize` | `crates/contextra-kvcache/src/segment.rs:630` | Returns the sum of actual bytes stored across allocated blocks. |
| `pub fn fragmentation_ratio(&self) -> f64` | `crates/contextra-kvcache/src/segment.rs:641` | Returns the fragmentation ratio: 1.0 - (allocated_bytes / capacity_of_allocated_blocks). If 0 blocks are allocated, returns 0.0. |

#### `struct SegmentPoolConfig`
- Ort: `crates/contextra-kvcache/src/segment.rs:498`
```rust
pub struct SegmentPoolConfig
```
Dokumentation: Configuration for the bounded segment pool.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `block_size` | `usize` | `crates/contextra-kvcache/src/segment.rs:500` |
| `max_blocks` | `usize` | `crates/contextra-kvcache/src/segment.rs:502` |

#### `struct ShreddableSegmentKey`
- Ort: `crates/contextra-kvcache/src/segment.rs:80`
```rust
pub struct ShreddableSegmentKey
```
Dokumentation: Shred-fähiger Schlüssel für Tier-2 Segmentdateien (Crypto-Shredding). Durch Aufruf von `shred()` wird der Schlüssel im Speicher mit Nullen überschrieben und aus dem `RwLock` entfernt. Ausgelagerte Segmentdateien auf Disk werden dadurch mathematisch dauerhaft unlesbar.
- Impl-Block `crates/contextra-kvcache/src/kv_encryption/mod.rs:58`
- Impl-Block `crates/contextra-kvcache/src/segment.rs:84`
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-kvcache/src/segment.rs:134`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn try_new(passphrase: &str, salt: &[u8]) -> Result<Self>` | `crates/contextra-kvcache/src/kv_encryption/mod.rs:60` | Erstellt einen neuen shred-fähigen Schlüssel aus Passphrase und Salt. |
| `pub fn try_new_random(passphrase: &str) -> Result<Self>` | `crates/contextra-kvcache/src/kv_encryption/mod.rs:67` | Erstellt einen neuen shred-fähigen Schlüssel mit zufällig erzeugtem Salt. |
| `pub fn from_aead(aead: Box<dyn SegmentAead>) -> Self` | `crates/contextra-kvcache/src/segment.rs:86` | Erzeugt einen neuen `ShreddableSegmentKey` aus einer dynamischen `SegmentAead`-Implementierung. |
| `pub fn shredded() -> Self` | `crates/contextra-kvcache/src/segment.rs:93` | Erzeugt einen leeren oder bereits ge-shreddeten Schlüssel. |
| `pub fn encrypt(&self, plaintext: &[u8]) -> Result<(Vec<u8>, [u8` | `crates/contextra-kvcache/src/segment.rs:100` | Verschlüsselt Daten mit AEAD und automatisch generiertem Nonce. |
| `pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8` | `crates/contextra-kvcache/src/segment.rs:111` | Entschlüsselt Daten mit AEAD unter Verwendung der 12-Byte-Nonce. |
| `pub fn shred(&self)` | `crates/contextra-kvcache/src/segment.rs:122` | Crypto-Shredding: Überschreibt den Schlüssel im Speicher mit Nullen. |
| `pub fn is_shredded(&self) -> bool` | `crates/contextra-kvcache/src/segment.rs:129` | Prüft, ob der Schlüssel bereits ge-shredded (vernichtet) wurde. |

#### `struct Tier2EncryptedSegment`
- Ort: `crates/contextra-kvcache/src/segment.rs:145`
```rust
pub struct Tier2EncryptedSegment
```
Dokumentation: Verschlüsselte Tier-2 Segmentdatei (ausgelagerter KV-Block auf persistentem Speicher).
- Impl-Block `crates/contextra-kvcache/src/segment.rs:153`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tenant_id` | `TenantId` | `crates/contextra-kvcache/src/segment.rs:146` |
| `segment_id` | `u64` | `crates/contextra-kvcache/src/segment.rs:147` |
| `ciphertext` | `Vec<u8>` | `crates/contextra-kvcache/src/segment.rs:148` |
| `nonce` | `[u8; 12]` | `crates/contextra-kvcache/src/segment.rs:149` |
| `key` | `ShreddableSegmentKey` | `crates/contextra-kvcache/src/segment.rs:150` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( tenant_id: TenantId, segment_id: u64, plaintext: &[u8], key: ShreddableSegmentKey, ) -> Result<Self>` | `crates/contextra-kvcache/src/segment.rs:155` | Verschlüsselt Klartext-Tensor-Bytes und erstellt ein neues Tier-2 Segment. |
| `pub fn read_and_decrypt(&self) -> Result<Vec<u8>>` | `crates/contextra-kvcache/src/segment.rs:172` | Liest und entschlüsselt das Segment. |

#### `enum KvSegmentContent`
- Ort: `crates/contextra-kvcache/src/segment.rs:48`
```rust
pub enum KvSegmentContent
```
Dokumentation: Repräsentation des Inhalts eines KV-Segments. Kann entweder als rohe Bytes vorliegen oder in einem KIVI-quantisierten Format.

Varianten:

- `Raw(bytes::Bytes)` — `crates/contextra-kvcache/src/segment.rs:50`
- `KiviQuantized(KiviQuantizedBlock)` — `crates/contextra-kvcache/src/segment.rs:53`

#### `trait SegmentAead`
- Ort: `crates/contextra-kvcache/src/segment.rs:64`
```rust
pub trait SegmentAead: Send + Sync
```
Dokumentation: Lokales Trait für AEAD-Segmentverschlüsselung (Authenticated Encryption with Associated Data).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn encrypt(&self, plaintext: &[u8]) -> Result<(Vec<u8>, [u8` | `crates/contextra-kvcache/src/segment.rs:66` | Verschlüsselt Daten mit AEAD und liefert (Ciphertext, 12-Byte Nonce). |
| `fn decrypt(&self, ciphertext: &[u8], nonce: &[u8` | `crates/contextra-kvcache/src/segment.rs:68` | Entschlüsselt Daten mit AEAD unter Verwendung der 12-Byte Nonce. |
| `fn wipe(&mut self)` | `crates/contextra-kvcache/src/segment.rs:70` | Überschreibt das Schlüsselmaterial sicher im Speicher. |

#### `type SegmentKey`
- Ort: `crates/contextra-kvcache/src/segment.rs:494`
```rust
pub type SegmentKey = ShreddableSegmentKey
```
Dokumentation: Neutraler Segmentschlüssel (Klasse A Baustein).

#### `const CURRENT_KV_KEY_DERIVATION_VERSION`
- Ort: `crates/contextra-kvcache/src/segment.rs:60`
```rust
pub const CURRENT_KV_KEY_DERIVATION_VERSION: u8 = 1
```
Dokumentation: Aktuelle Version der KV-Segment-Schlüsselableitung (HKDF = HMAC-based Extract-and-Expand Key Derivation Function). Erhöhe diesen Wert, wenn sich der HKDF-Info-String oder der Salt-Aufbau ändert.

### Modul `service`

#### `struct KvCacheConfig`
- Ort: `crates/contextra-kvcache/src/service.rs:84`
```rust
pub struct KvCacheConfig
```
Dokumentation: Konfiguration für die Fassade [`KvCacheService`].
- Impl-Block (Trait `Default`) `crates/contextra-kvcache/src/service.rs:96`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `memory_budget_bytes` | `usize` | `crates/contextra-kvcache/src/service.rs:86` |
| `reuse_policy` | `KvReusePolicy` | `crates/contextra-kvcache/src/service.rs:88` |
| `spill_enabled` | `bool` | `crates/contextra-kvcache/src/service.rs:90` |
| `kivi_config` | `Option<KiviQuantizeConfig>` | `crates/contextra-kvcache/src/service.rs:93` |

#### `struct KvCacheService`
- Ort: `crates/contextra-kvcache/src/service.rs:294`
```rust
pub struct KvCacheService
```
Dokumentation: Öffentlicher mandantenisolierter KV-Cache-Dienst. Zentrale Facade für Inferenz-Laufzeiten: verbindet Radix-Präfixsuche, Speicher-LRU, Hintergrundverdrängung und Tier-2 Persistenz.
- Impl-Block `crates/contextra-kvcache/src/service.rs:307`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn get_or_insert_prefix( &self, tenant_id: impl Into<TenantId>, tokens: &[u32], data: &[u8], ) -> Result<Option<(PrefixMatch, KvBlockGuard)>>` | `crates/contextra-kvcache/src/service.rs:318` | Sucht nach einem übereinstimmenden Prefix oder fügt ein neues KV-Segment ein. Fail-closed: `tenant_id` ist Pflichtparameter jeder Operation. # Ablauf 1. Eingabe |
| `pub async fn read_segment( &self, tenant_id: impl Into<TenantId>, segment_id: u64, ) -> Result<Option<Vec<u8>>>` | `crates/contextra-kvcache/src/service.rs:412` | Liest ein Segment-Payload aus RAM oder Tier-2 Spill-Storage. # Ablauf 1. Versuch, das Segment direkt aus dem RAM-Store zu lesen und zu dekomprimieren/dequantisi |
| `pub async fn spill_segment( &self, tx_id: TxId, tenant_id: TenantId, segment_id: u64, payload: &[u8], ) -> Result<()>` | `crates/contextra-kvcache/src/service.rs:448` | Persistiert ein Segment explizit verschlüsselt auf Tier-2 Sekundärspeicher. |
| `pub fn invalidate_scratchpad( &self, tenant_id: TenantId, scope: ScratchpadCacheScope, ) -> Result<usize>` | `crates/contextra-kvcache/src/service.rs:504` | Invalidiert Scratchpad-Tokens, die zum angegebenen Scope gehören. |
| `pub fn delete_tenant(&self, tenant_id: impl Into<TenantId>)` | `crates/contextra-kvcache/src/service.rs:526` | Entfernt alle RAM-Segmente und Radix-Knoten für einen Mandanten (Mandanten-Bereinigung). # Ablauf 1. Alle Segment-IDs des Mandanten ermitteln und RAM-Zähler kor |
| `pub fn trigger_eviction(&self, target_free_bytes: usize)` | `crates/contextra-kvcache/src/service.rs:566` | Löst eine nicht-blockierende Verdrängung im Hintergrund aus. |
| `pub fn clock(&self) -> &Arc<dyn Clock>` | `crates/contextra-kvcache/src/service.rs:571` | Liefert eine Referenz auf den injizierten Clock-Port. |
| `pub fn stats(&self, tenant_id: impl Into<TenantId>) -> KvCacheStats` | `crates/contextra-kvcache/src/service.rs:576` | Liefert Nutzungsstatistiken für den angegebenen Mandanten. |

#### `struct KvCacheServiceBuilder`
- Ort: `crates/contextra-kvcache/src/service.rs:110`
```rust
pub struct KvCacheServiceBuilder
```
Dokumentation: Builder zur Erstellung von [`KvCacheService`].
- Impl-Block `crates/contextra-kvcache/src/service.rs:121`
- Impl-Block (Trait `Default`) `crates/contextra-kvcache/src/service.rs:283`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-kvcache/src/service.rs:123` | Erstellt einen neuen Builder mit Standardkonfiguration. |
| `pub fn memory_budget_bytes(mut self, bytes: usize) -> Self` | `crates/contextra-kvcache/src/service.rs:137` | Konfiguriert das Speicherbudget in Bytes (mindestens 1 MiB). |
| `pub fn reuse_policy(mut self, policy: KvReusePolicy) -> Self` | `crates/contextra-kvcache/src/service.rs:143` | Konfiguriert die Prefix-Wiederverwendungs-Policy ([`KvReusePolicy`]). |
| `pub fn spill_enabled(mut self, enabled: bool) -> Self` | `crates/contextra-kvcache/src/service.rs:149` | Aktiviert oder deaktiviert den Tier-2 Spill auf Sekundärspeicher. |
| `pub fn enable_kivi_quantization(mut self, config: Option<KiviQuantizeConfig>) -> Self` | `crates/contextra-kvcache/src/service.rs:156` | Aktiviert oder deaktiviert die KIVI 2-Bit Quantisierung. |
| `pub fn clock(mut self, clock: Arc<dyn Clock>) -> Self` | `crates/contextra-kvcache/src/service.rs:162` | Injiziert die Clock-Port-Implementierung zur Einhaltung deterministischer Zeitvorgaben (P28 / Regel 3). |
| `pub fn spill_storage( mut self, storage: Arc<dyn crate::spill::KvSpillStorage>, ) -> Self` | `crates/contextra-kvcache/src/service.rs:168` | Konfiguriert den Tier-2 Spill-Storage Adapter ([`KvSpillStorage`]). |
| `pub fn cipher(mut self, cipher: Arc<KvSegmentCipher>) -> Self` | `crates/contextra-kvcache/src/service.rs:178` | Konfiguriert den Segment-Cipher für verschlüsselten Tier-2-Spill. |
| `pub fn attention_source(mut self, source: Arc<dyn AttentionScoreSource>) -> Self` | `crates/contextra-kvcache/src/service.rs:184` | Konfiguriert eine benutzerdefinierte [`AttentionScoreSource`]. |
| `pub fn attention_exporter( mut self, exporter: Arc<dyn crate::compat::adapters::AttentionExporter>, ) -> Self` | `crates/contextra-kvcache/src/service.rs:191` | Konfiguriert einen [`AttentionExporter`] für den [`EvictionWorker`]. |
| `pub fn build(self) -> KvCacheService` | `crates/contextra-kvcache/src/service.rs:206` | Baut die [`KvCacheService`]-Instanz. # Ablauf 1. Clock und Store initialisieren. 2. EvictionWorker spawnen und Attention-Quellen injizieren. 3. Tier-2 SpillHand |

#### `struct KvCacheStats`
- Ort: `crates/contextra-kvcache/src/service.rs:74`
```rust
pub struct KvCacheStats
```
Dokumentation: Statistiken über die KV-Cache-Nutzung eines Mandanten.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `segment_count` | `usize` | `crates/contextra-kvcache/src/service.rs:76` |
| `radix_entry_count` | `usize` | `crates/contextra-kvcache/src/service.rs:78` |

### Modul `spill`

#### `struct InMemoryKvSpillStorage`
- Ort: `crates/contextra-kvcache/src/spill.rs:31`
```rust
pub struct InMemoryKvSpillStorage
```
Dokumentation: Deterministischer In-Memory-Spill-Speicher als Adoptions-Baustein für Standalone-Setups.
- Impl-Block `crates/contextra-kvcache/src/spill.rs:36`
- Impl-Block (Trait `KvSpillStorage`) `crates/contextra-kvcache/src/spill.rs:53`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-kvcache/src/spill.rs:38` | Erstellt eine neue Instanz des In-Memory Spill-Speichers. |
| `pub fn len(&self) -> usize` | `crates/contextra-kvcache/src/spill.rs:43` | Gibt die Anzahl gespeicherter persistenter Einträge zurück. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-kvcache/src/spill.rs:48` | Prüft, ob der Speicher leer ist. |

#### `trait KvSpillStorage`
- Ort: `crates/contextra-kvcache/src/spill.rs:17`
```rust
pub trait KvSpillStorage: Send + Sync
```
Dokumentation: Lokales Persistenz-Interface für die Tier-2-Auslagerung von KV-Segmenten.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>>` | `crates/contextra-kvcache/src/spill.rs:19` | Liest einen Wert unter dem angegebenen Schlüssel. |
| `fn put<'a>(&'a self, tx_id: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>>` | `crates/contextra-kvcache/src/spill.rs:22` | Schreibt ein Schlüssel-Wert-Paar im Kontext einer Transaktion (`tx_id`). |
| `fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>` | `crates/contextra-kvcache/src/spill.rs:25` | Bestätigt die Transaktion mit der ID `tx_id`. |

#### `type BoxFuture`
- Ort: `crates/contextra-kvcache/src/spill.rs:13`
```rust
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>
```
Dokumentation: Dyn-kompatibler lokaler Future-Typ ohne externe Laufzeitabhängigkeit.

### Modul `store`

#### `struct TenantIsolatedKvStore`
- Ort: `crates/contextra-kvcache/src/store.rs:328`
```rust
pub struct TenantIsolatedKvStore
```
Dokumentation: Mandantenisolierter KV-Segment-Store im Hauptspeicher. Bildet den zentralen RAM-Zwischenspeicher für KV-Caches mit Sharding, LRU-Verdrängung, Aufmerksamkeitsscoring und Transaktions-Rollback-Unterstützung.
- Impl-Block `crates/contextra-kvcache/src/kv_encryption/mod.rs:207`
- Impl-Block `crates/contextra-kvcache/src/store.rs:339`
- Impl-Block (Trait `Default`) `crates/contextra-kvcache/src/store.rs:787`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn insert_encrypted_segment( &self, cipher: &KvSegmentCipher, tenant: impl Into<TenantId>, segment_id: u64, model_fingerprint: ModelFingerprint, rope_offset: Option<usize>, plaintext: &[u8], ) -> std::result::Result<(), CryptoError>` | `crates/contextra-kvcache/src/kv_encryption/mod.rs:209` | Verschlüsselt einen Klartext-Tensor und fügt ein verschlüsseltes Segment ein. |
| `pub fn get_decrypted_segment( &self, cipher: &KvSegmentCipher, tenant: impl Into<TenantId>, segment_id: u64, ) -> std::result::Result<Option<Vec<u8>>, CryptoError>` | `crates/contextra-kvcache/src/kv_encryption/mod.rs:232` | Liest ein Segment und entschlüsselt es falls nötig. |
| `pub fn new() -> Self` | `crates/contextra-kvcache/src/store.rs:356` | Erstellt einen neuen tenant-isolierten KV-Store mit Standard-Sharding. |
| `pub fn with_shard_count(n: usize) -> Self` | `crates/contextra-kvcache/src/store.rs:361` | Erstellt einen Store mit benutzerdefinierter Shard-Anzahl (wird auf Zweierpotenz aufgerundet). |
| `pub fn with_shard_count_and_clock(n: usize, clock: Arc<dyn Clock>) -> Self` | `crates/contextra-kvcache/src/store.rs:366` | Erstellt einen Store mit benutzerdefinierter Shard-Anzahl und injizierter Uhr. |
| `pub fn with_clock(clock: Arc<dyn Clock>) -> Self` | `crates/contextra-kvcache/src/store.rs:392` | Erstellt einen Store mit injizierter Uhr und Standard-Sharding. |
| `pub fn set_attention_source(&self, source: Arc<dyn AttentionScoreSource>)` | `crates/contextra-kvcache/src/store.rs:397` | Setzt die konfigurierte Standard-[`AttentionScoreSource`] für diesen Store. |
| `pub fn attention_source(&self) -> Arc<dyn AttentionScoreSource>` | `crates/contextra-kvcache/src/store.rs:402` | Liefert die konfigurierte Standard-[`AttentionScoreSource`] für diesen Store. |
| `pub fn with_capacity(segment_capacity_per_tenant: usize) -> Self` | `crates/contextra-kvcache/src/store.rs:407` | Erstellt einen Store mit konfigurierter Segment-Kapazität pro Mandant. |
| `pub fn with_capacity_and_clock(segment_capacity_per_tenant: usize, clock: Arc<dyn Clock>) -> Self` | `crates/contextra-kvcache/src/store.rs:415` | Erstellt einen Store mit konfigurierter Segment-Kapazität pro Mandant und injizierter Uhr. |
| `pub fn set_spill_handler(&self, handler: SpillHandler)` | `crates/contextra-kvcache/src/store.rs:423` | Registriert einen Spill-Handler für evictierte Segmente (Tier-2 Auslagerung). |
| `pub fn insert_segment_with_directive( &self, tenant: impl Into<TenantId>, segment: KvSegment, directive: CacheDirective, ) -> Result<()>` | `crates/contextra-kvcache/src/store.rs:441` | Fügt ein Segment mit einer deklarativen [`CacheDirective`] ein. # Ablauf 1. Bei `CacheDirective::NeverCache` sofort abbrechen (No-Op). 2. Shard anhand der Manda |
| `pub fn insert_segment(&self, tenant: impl Into<TenantId>, segment: KvSegment)` | `crates/contextra-kvcache/src/store.rs:475` | Fügt ein Segment für einen bestimmten Mandanten mit Standard-Direktive (`Auto`) ein. |
| `pub fn release_step(&self, tenant: impl Into<TenantId>, step_id: impl Into<StepId>)` | `crates/contextra-kvcache/src/store.rs:481` | Gibt transiente Segmente frei, die mit einem bestimmten Ausführungsschritt verknüpft sind. |
| `pub fn insert_token_sequence( &self, tenant: impl Into<TenantId>, tokens: &[u32], block_id: u64, ) -> Result<()>` | `crates/contextra-kvcache/src/store.rs:494` | Fügt eine Token-Sequenz und deren assoziierte Block-ID in den Prefix-Radix-Baum ein. |
| `pub fn find_prefix_match( &self, tenant: impl Into<TenantId>, tokens: &[u32], policy: KvReusePolicy, worker: Option<Arc<EvictionWorker>>, ) -> Option<(PrefixMatch, KvBlockGuard)>` | `crates/contextra-kvcache/src/store.rs:511` | Sucht ein Präfix-Match für die angegebenen Tokens unter der gewünschten [`KvReusePolicy`]. |
| `pub fn acquire_block_guard( &self, tenant: impl Into<TenantId>, block_id: u64, worker: Option<Arc<EvictionWorker>>, ) -> Option<KvBlockGuard>` | `crates/contextra-kvcache/src/store.rs:526` | Erstellt einen [`KvBlockGuard`] für einen bestimmten Block. |
| `pub fn on_rollback(&self, tenant: impl Into<TenantId>, chunk_ids: &[u64])` | `crates/contextra-kvcache/src/store.rs:540` | Bereinigt assoziierte KV-Segmente bei einem transaktionalen Rollback (z. B. nach Abbruch einer Inferenz). |
| `pub fn remove_segments_for_rollback(&self, tenant: impl Into<TenantId>, segment_ids: &[u64])` | `crates/contextra-kvcache/src/store.rs:546` | Entfernt Segmente mit den angegebenen IDs für einen Mandanten. |
| `pub fn remove_segment(&self, tenant: impl Into<TenantId>, segment_id: u64)` | `crates/contextra-kvcache/src/store.rs:582` | Entfernt ein einzelnes Segment. |
| `pub fn get_segment_bytes(&self, tenant: impl Into<TenantId>, segment_id: u64) -> Option<Vec<u8>>` | `crates/contextra-kvcache/src/store.rs:588` | Liefert unverschlüsselte Segment-Bytes für einen Mandanten. |
| `pub fn get_segments(&self, tenant: impl Into<TenantId>) -> Vec<u64>` | `crates/contextra-kvcache/src/store.rs:599` | Liefert alle gespeicherten Segment-IDs eines Mandanten. |
| `pub fn get_tenant_segment_len(&self, tenant: impl Into<TenantId>) -> usize` | `crates/contextra-kvcache/src/store.rs:611` | Gibt die Anzahl der gespeicherten Segmente für einen Mandanten zurück. |
| `pub fn evict_fair(&self, target_free_bytes: usize, scores: &dyn AttentionScoreSource) -> usize` | `crates/contextra-kvcache/src/store.rs:625` | Evictiert KV-Segmente unter Erhaltung von Tenant-Fairness und Attention-Scores (Default 50/50 Gewichtung). |
| `pub fn evict_weighted_fair( &self, target_free_bytes: usize, scores: &dyn AttentionScoreSource, attention_weight: f32, ) -> usize` | `crates/contextra-kvcache/src/store.rs:630` | Evictiert KV-Segmente unter Erhaltung von Tenant-Fairness und gewichteten Attention-Scores. |
| `pub fn evict_lru_fair(&self, target_free_bytes: usize) -> usize` | `crates/contextra-kvcache/src/store.rs:651` | Evictiert KV-Segmente unter Erhaltung von Tenant-Fairness und Guard-Schutz (`active_refs == 0`). # Ablauf 1. Ruft [`Self::attention_source`] ab. 2. Delegiert an |
| `pub fn evict_lru_fair_with_hook<F>(&self, target_free_bytes: usize, mut hook: F) -> usize where F: FnMut(),` | `crates/contextra-kvcache/src/store.rs:657` |  |
| `pub fn clear_all(&self)` | `crates/contextra-kvcache/src/store.rs:769` | Löscht alle Segmente synchron aus dem Speicher und stellt Zeroization sicher (Emergency Wipe). |

Assoziierte Konstanten/Typen:

- `pub const DEFAULT_SHARD_COUNT: usize = 32` — `crates/contextra-kvcache/src/store.rs:351`
- `pub const DEFAULT_SEGMENT_CAPACITY_PER_TENANT: usize = 256` — `crates/contextra-kvcache/src/store.rs:353`

#### `type SpillHandler`
- Ort: `crates/contextra-kvcache/src/store.rs:46`
```rust
pub type SpillHandler = Arc<dyn Fn(TenantId, u64, Vec<u8>) + Send + Sync>
```
Dokumentation: Optionaler Callback-Hook für Tier-2-LSM-Spill bei Eviction aus dem In-Memory LRU Cache. Wird aufgerufen, wenn ein Segment aus dem RAM verdrängt wird, um es auf persistente Medien (LSM-Tree = Log-Structured Merge-Tree) auszulagern.

#### `const MAX_PINNED_BYTES_PER_TENANT`
- Ort: `crates/contextra-kvcache/src/store.rs:52`
```rust
pub const MAX_PINNED_BYTES_PER_TENANT: u64 = 64 << 20
```
Dokumentation: Maximal erlaubte Gesamtgröße gepinnter Bytes pro Mandant für ungepinnte Segmente ohne Ablaufdatum (`Pin { ttl: None }`). Verhindert Denial-of-Service (DoS) durch unbegrenztes Festpinnen von Speicher (64 MiB).

### Modul `types`

#### `struct ModelFingerprint`
- Ort: `crates/contextra-kvcache/src/types.rs:156`
```rust
pub struct ModelFingerprint(pub [u8
```
Dokumentation: Fingerabdruck eines Modells (32-Byte Hash).
- Impl-Block (Trait `From<ModelFingerprint>`) `crates/contextra-kvcache/src/compat/mod.rs:59`
- Impl-Block (Trait `From<contextra_types::model_fingerprint::ModelFingerprint>`) `crates/contextra-kvcache/src/compat/mod.rs:66`
- Impl-Block `crates/contextra-kvcache/src/types.rs:158`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn from_bytes(bytes: [u8` | `crates/contextra-kvcache/src/types.rs:160` | Erzeugt einen ModelFingerprint aus 32 Bytes. |
| `pub fn from_str_hash(s: &str) -> Self` | `crates/contextra-kvcache/src/types.rs:165` | Erzeugt einen ModelFingerprint aus einem String (BLAKE3 gehasht). |
| `pub const fn as_bytes(&self) -> &[u8` | `crates/contextra-kvcache/src/types.rs:171` | Gibt die 32 Bytes zurück. |

#### `struct RequestId`
- Ort: `crates/contextra-kvcache/src/types.rs:123`
```rust
pub struct RequestId(pub u64)
```
Dokumentation: Eindeutige ID einer Inferenzanfrage.
- Impl-Block (Trait `From<RequestId>`) `crates/contextra-kvcache/src/compat/mod.rs:45`
- Impl-Block (Trait `From<contextra_ports::RequestId>`) `crates/contextra-kvcache/src/compat/mod.rs:52`
- Impl-Block `crates/contextra-kvcache/src/types.rs:125`
- Impl-Block (Trait `std::fmt::Display`) `crates/contextra-kvcache/src/types.rs:142`
- Impl-Block (Trait `From<u64>`) `crates/contextra-kvcache/src/types.rs:148`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-kvcache/src/types.rs:127` | Erzeugt eine `RequestId` aus einem `u64`. |
| `pub const fn inner(self) -> u64` | `crates/contextra-kvcache/src/types.rs:132` | Gibt den inneren `u64`-Wert zurück. |
| `pub const fn as_u64(self) -> u64` | `crates/contextra-kvcache/src/types.rs:137` | Gibt den inneren `u64`-Wert zurück. |

#### `struct StepId`
- Ort: `crates/contextra-kvcache/src/types.rs:90`
```rust
pub struct StepId(pub u64)
```
Dokumentation: Sequenzschritt / Token-Offset.
- Impl-Block `crates/contextra-kvcache/src/types.rs:92`
- Impl-Block (Trait `std::fmt::Display`) `crates/contextra-kvcache/src/types.rs:109`
- Impl-Block (Trait `From<u64>`) `crates/contextra-kvcache/src/types.rs:115`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-kvcache/src/types.rs:94` | Erzeugt einen neuen `StepId`. |
| `pub const fn inner(self) -> u64` | `crates/contextra-kvcache/src/types.rs:99` | Gibt den inneren raw `u64`-Wert zurück. |
| `pub const fn as_u64(&self) -> u64` | `crates/contextra-kvcache/src/types.rs:104` | Gibt den numerischen Wert zurück. |

#### `struct TenantId`
- Ort: `crates/contextra-kvcache/src/types.rs:7`
```rust
pub struct TenantId(pub u64)
```
Dokumentation: Mandanten-Identifier (lokaler Typ).
- Impl-Block (Trait `From<TenantId>`) `crates/contextra-kvcache/src/compat/mod.rs:17`
- Impl-Block (Trait `From<contextra_types::TenantId>`) `crates/contextra-kvcache/src/compat/mod.rs:24`
- Impl-Block (Trait `PartialEq<contextra_types::TenantId>`) `crates/contextra-kvcache/src/compat/mod.rs:199`
- Impl-Block (Trait `PartialEq<TenantId>`) `crates/contextra-kvcache/src/compat/mod.rs:206`
- Impl-Block `crates/contextra-kvcache/src/types.rs:9`
- Impl-Block (Trait `std::fmt::Display`) `crates/contextra-kvcache/src/types.rs:43`
- Impl-Block (Trait `From<u64>`) `crates/contextra-kvcache/src/types.rs:49`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-kvcache/src/types.rs:14` | Erzeugt eine neue `TenantId`. |
| `pub fn try_new(id: u64) -> Result<Self, crate::error::KvCacheError>` | `crates/contextra-kvcache/src/types.rs:19` | Validierte Erzeugung (Mandant 0 ist reserviert/ungültig für reguläre Mandanten). |
| `pub const fn inner(self) -> u64` | `crates/contextra-kvcache/src/types.rs:28` | Gibt den inneren raw `u64`-Wert zurück. |
| `pub const fn as_u64(&self) -> u64` | `crates/contextra-kvcache/src/types.rs:33` | Gibt den numerischen Wert zurück. |
| `pub const fn is_system(&self) -> bool` | `crates/contextra-kvcache/src/types.rs:38` | Gibt an, ob es sich um den System-Mandanten handelt. |

Assoziierte Konstanten/Typen:

- `pub const SYSTEM: Self = Self(0)` — `crates/contextra-kvcache/src/types.rs:11`

#### `struct TxId`
- Ort: `crates/contextra-kvcache/src/types.rs:57`
```rust
pub struct TxId(pub u64)
```
Dokumentation: Transaktions-Identifier (lokaler Newtype für Speicheroperationen).
- Impl-Block (Trait `From<TxId>`) `crates/contextra-kvcache/src/compat/mod.rs:31`
- Impl-Block (Trait `From<contextra_types::TxId>`) `crates/contextra-kvcache/src/compat/mod.rs:38`
- Impl-Block `crates/contextra-kvcache/src/types.rs:59`
- Impl-Block (Trait `std::fmt::Display`) `crates/contextra-kvcache/src/types.rs:76`
- Impl-Block (Trait `From<u64>`) `crates/contextra-kvcache/src/types.rs:82`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-kvcache/src/types.rs:61` | Erzeugt eine neue `TxId`. |
| `pub const fn inner(self) -> u64` | `crates/contextra-kvcache/src/types.rs:66` | Gibt den inneren raw `u64`-Wert zurück. |
| `pub const fn as_u64(&self) -> u64` | `crates/contextra-kvcache/src/types.rs:71` | Gibt den numerischen Wert zurück. |

#### `enum CacheDirective`
- Ort: `crates/contextra-kvcache/src/types.rs:178`
```rust
pub enum CacheDirective
```
Dokumentation: Cache-Direktive für granulare Lebenszyklus-Steuerung.
- Impl-Block (Trait `Default`) `crates/contextra-kvcache/src/types.rs:195`

Varianten:

- `Pin{` — `crates/contextra-kvcache/src/types.rs:180`
- `NeverCache` — `crates/contextra-kvcache/src/types.rs:185`
- `Auto` — `crates/contextra-kvcache/src/types.rs:187`
- `ReleaseAfterStep{` — `crates/contextra-kvcache/src/types.rs:189`

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl<T: contextra_ports::kv_bridge_port::KvBridgeStorage + ?Sized> crate::spill::KvSpillStorage for T` — `crates/contextra-kvcache/src/compat/adapters.rs:235` (0 pub Methoden)
- `impl From<KvCacheError> for ContextraError` — `crates/contextra-kvcache/src/compat/error.rs:6` (0 pub Methoden)
- `impl<T: contextra_env::Clock + ?Sized> Clock for T` — `crates/contextra-kvcache/src/contract.rs:20` (0 pub Methoden)

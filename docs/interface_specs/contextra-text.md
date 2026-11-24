# Schnittstellenspezifikation: `contextra-text`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-text` |
| Version | workspace |
| Pfad | `crates/contextra-text` |
| Rolle | Volltext |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> Hybrid Search Engine & BM25 Scoring (WP-2.1)
>
> Evaluates Inverse Document Frequencies integrating natively into the
> `fusion.rs` layer in `contextra-db`.
>
> # Einordnung in die Gesamtarchitektur (Ring-Modell)
> Dieses Modul gehört zu **Ring 0 (Kern)** von Contextra. Es enthält grundlegende
> Textindizierungs- und Volltextsuch-Primitive (BM25, WAND, morphologische Zerlegung
> für Deutsch, invertierte Indizes und Posting-Listen).
>
> # Abhängigkeiten
> - **Normal**: `contextra-ports`, `contextra-types`, sowie Hilfs-Bibliotheken (`bincode`, `blake3`,

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `bincode` | normal | workspace | nein |  |
| `blake3` | normal | workspace | nein |  |
| `bytes` | normal | workspace | nein |  |
| `contextra-env` | normal | workspace | nein | contextra-checkpoint, contextra-crypto, contextra-graph, contextra-infer-candle, contextra-kvcache, contextra-mvcc, contextra-ports, contextra-store, contextra-text, contextra-vector |
| `contextra-ids` | normal | workspace | nein | contextra-checkpoint, contextra-crypto, contextra-graph, contextra-mvcc, contextra-store, contextra-text, contextra-types, contextra-vector |
| `contextra-ports` | normal | workspace | ja | contextra, contextra-adapt, contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-rank, contextra-router, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vector, contextra-vector-fuzz |
| `contextra-types` | normal | workspace | ja | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `criterion` | dev | workspace | nein |  |
| `futures` | normal | registry | nein |  |
| `itoa` | normal | registry | nein |  |
| `parking_lot` | normal | workspace | nein |  |
| `proptest` | dev | registry | nein |  |
| `regex` | normal | registry | nein |  |
| `serde` | normal | workspace | nein |  |
| `serde_json` | normal | workspace | nein |  |
| `tempfile` | dev | workspace | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tokio` | dev | workspace | nein |  |
| `tracing` | normal | workspace | nein |  |
| `unicode-segmentation` | normal | registry | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-agent`, `contextra-bench`, `contextra-cognition`, `contextra-db`, `contextra-engine`, `contextra-text-fuzz`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `bm25f` | – |
| `default` | `std` |
| `docid-128` | `contextra-ids/docid-128` |
| `experimental` | – |
| `inspect` | – |
| `ports-compat` | `types-compat` |
| `serde` | – |
| `std` | – |
| `tracing` | – |
| `types-compat` | `dep:contextra-types`, `dep:contextra-ports` |
| `workspace` | `types-compat`, `ports-compat` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 38 |
| Quellzeilen | 9535 |
| Öffentliche Items (Modul-Ebene) | 115 |
| davon `fn` | 16 |
| davon `struct` | 39 |
| davon `enum` | 6 |
| davon `trait` | 10 |
| davon `type` | 6 |
| davon `const` | 9 |
| davon `static` | 4 |
| davon `mod` | 25 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 35 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 0 |
| `.expect(` (Produktionscode) | 6 |
| `panic!` (Produktionscode) | 1 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 43 |
| `#[deprecated]` | 11 |
| `#[cfg(feature…)]`-Gates | 23 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 234 / 234 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `bm25` | `crates/contextra-text/src/bm25.rs` | 686 |
| `compat::error` | `crates/contextra-text/src/compat/error.rs` | 91 |
| `compat::inverted_ports` | `crates/contextra-text/src/compat/inverted_ports.rs` | 108 |
| `compat` | `crates/contextra-text/src/compat/mod.rs` | 20 |
| `compat::morph_index` | `crates/contextra-text/src/compat/morph_index.rs` | 90 |
| `compat::purge` | `crates/contextra-text/src/compat/purge.rs` | 507 |
| `compat::scanner` | `crates/contextra-text/src/compat/scanner.rs` | 309 |
| `compat::scorer` | `crates/contextra-text/src/compat/scorer.rs` | 117 |
| `compat::storage` | `crates/contextra-text/src/compat/storage.rs` | 227 |
| `compat::tests::mock` | `crates/contextra-text/src/compat/tests/mock.rs` | 190 |
| `compat::tests::search_tests` | `crates/contextra-text/src/compat/tests/search_tests.rs` | 833 |
| `compat::tests::tx_tests` | `crates/contextra-text/src/compat/tests/tx_tests.rs` | 510 |
| `compat::tests` | `crates/contextra-text/src/compat/tests.rs` | 8 |
| `compat::types` | `crates/contextra-text/src/compat/types.rs` | 50 |
| `contract` | `crates/contextra-text/src/contract.rs` | 71 |
| `domain::legal_de` | `crates/contextra-text/src/domain/legal_de.rs` | 60 |
| `domain::medical_de` | `crates/contextra-text/src/domain/medical_de.rs` | 60 |
| `domain` | `crates/contextra-text/src/domain/mod.rs` | 140 |
| `error` | `crates/contextra-text/src/error.rs` | 44 |
| `inverted::index_struct` | `crates/contextra-text/src/inverted/index_struct.rs` | 670 |
| `inverted::types` | `crates/contextra-text/src/inverted/types.rs` | 54 |
| `inverted` | `crates/contextra-text/src/inverted.rs` | 32 |
| `crate` | `crates/contextra-text/src/lib.rs` | 90 |
| `memory_index` | `crates/contextra-text/src/memory_index.rs` | 507 |
| `morphology::compound_splitter` | `crates/contextra-text/src/morphology/compound_splitter.rs` | 315 |
| `morphology::metrics` | `crates/contextra-text/src/morphology/metrics.rs` | 32 |
| `morphology::passthrough` | `crates/contextra-text/src/morphology/passthrough.rs` | 33 |
| `morphology::stopwords` | `crates/contextra-text/src/morphology/stopwords.rs` | 63 |
| `morphology::tests` | `crates/contextra-text/src/morphology/tests.rs` | 764 |
| `morphology::trie` | `crates/contextra-text/src/morphology/trie.rs` | 64 |
| `morphology` | `crates/contextra-text/src/morphology.rs` | 33 |
| `posting_list` | `crates/contextra-text/src/posting_list.rs` | 584 |
| `storage` | `crates/contextra-text/src/storage.rs` | 64 |
| `stream` | `crates/contextra-text/src/stream.rs` | 156 |
| `tokenizer` | `crates/contextra-text/src/tokenizer.rs` | 1024 |
| `traits` | `crates/contextra-text/src/traits.rs` | 32 |
| `types` | `crates/contextra-text/src/types.rs` | 80 |
| `wand` | `crates/contextra-text/src/wand.rs` | 817 |

## 6. Re-Exports (`pub use`)

- `pub use morph_index::BM25MorphIndex` — `crates/contextra-text/src/compat/mod.rs:13` (Modul `compat`)
- `pub use purge::` — `crates/contextra-text/src/compat/mod.rs:14` (Modul `compat`)
- `pub use scanner::` — `crates/contextra-text/src/compat/mod.rs:15` (Modul `compat`)
- `pub use scorer::Bm25Scorer` — `crates/contextra-text/src/compat/mod.rs:16` (Modul `compat`)
- `pub use storage::StorageBridge` — `crates/contextra-text/src/compat/mod.rs:17` (Modul `compat`)
- `pub use legal_de::LegalDomainVocabulary` — `crates/contextra-text/src/domain/mod.rs:14` (Modul `domain`)
- `pub use medical_de::MedicalDomainVocabulary` — `crates/contextra-text/src/domain/mod.rs:15` (Modul `domain`)
- `pub use crate::compat::morph_index` — `crates/contextra-text/src/inverted.rs:14` (Modul `inverted`)
- `pub use crate::compat::purge` — `crates/contextra-text/src/inverted.rs:17` (Modul `inverted`)
- `pub use crate::compat::scanner` — `crates/contextra-text/src/inverted.rs:20` (Modul `inverted`)
- `pub use index_struct::InvertedIndex` — `crates/contextra-text/src/inverted.rs:25` (Modul `inverted`)
- `pub use morph_index::BM25MorphIndex` — `crates/contextra-text/src/inverted.rs:27` (Modul `inverted`)
- `pub use purge::` — `crates/contextra-text/src/inverted.rs:29` (Modul `inverted`)
- `pub use scanner::` — `crates/contextra-text/src/inverted.rs:31` (Modul `inverted`)
- `pub use types::` — `crates/contextra-text/src/inverted.rs:32` (Modul `inverted`)
- `pub use contract::` — `crates/contextra-text/src/lib.rs:59` (Modul `crate`)
- `pub use error::` — `crates/contextra-text/src/lib.rs:61` (Modul `crate`)
- `pub use memory_index::` — `crates/contextra-text/src/lib.rs:62` (Modul `crate`)
- `pub use storage::PostingStore` — `crates/contextra-text/src/lib.rs:63` (Modul `crate`)
- `pub use traits::` — `crates/contextra-text/src/lib.rs:64` (Modul `crate`)
- `pub use types::` — `crates/contextra-text/src/lib.rs:65` (Modul `crate`)
- `pub use bm25::` — `crates/contextra-text/src/lib.rs:76` (Modul `crate`)
- `pub use domain::` — `crates/contextra-text/src/lib.rs:77` (Modul `crate`)
- `pub use inverted::` — `crates/contextra-text/src/lib.rs:78` (Modul `crate`)
- `pub use inverted::` — `crates/contextra-text/src/lib.rs:80` (Modul `crate`)
- `pub use morphology::` — `crates/contextra-text/src/lib.rs:83` (Modul `crate`)
- `pub use posting_list::` — `crates/contextra-text/src/lib.rs:84` (Modul `crate`)
- `pub use stream::` — `crates/contextra-text/src/lib.rs:85` (Modul `crate`)
- `pub use tokenizer::` — `crates/contextra-text/src/lib.rs:86` (Modul `crate`)
- `pub use crate::compat::scorer::Bm25Scorer` — `crates/contextra-text/src/lib.rs:89` (Modul `crate`)
- `pub use compound_splitter::` — `crates/contextra-text/src/morphology.rs:29` (Modul `morphology`)
- `pub use metrics::TokenReductionMetrics` — `crates/contextra-text/src/morphology.rs:30` (Modul `morphology`)
- `pub use passthrough::PassthroughTokenizer` — `crates/contextra-text/src/morphology.rs:31` (Modul `morphology`)
- `pub use stopwords::` — `crates/contextra-text/src/morphology.rs:32` (Modul `morphology`)
- `pub use trie::` — `crates/contextra-text/src/morphology.rs:33` (Modul `morphology`)

## 8. Öffentliche Schnittstelle je Item

### Modul `bm25`

#### `struct BM25`
- Ort: `crates/contextra-text/src/bm25.rs:36`
```rust
pub struct BM25
```
Dokumentation: Konfigurationsparameter für das BM25-Scoring-Modell. - `k1`: Steuert die Sättigung der Termhäufigkeit (TF). Standardwert: 1.5. - `b`: Steuert den Einfluss der Dokumentlänge auf die Bewertung. Standardwert: 0.75.
- Impl-Block `crates/contextra-text/src/bm25.rs:43`
- Impl-Block (Trait `Default`) `crates/contextra-text/src/bm25.rs:67`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `k1` | `f32` | `crates/contextra-text/src/bm25.rs:38` |
| `b` | `f32` | `crates/contextra-text/src/bm25.rs:40` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(k1: f32, b: f32) -> Result<Self>` | `crates/contextra-text/src/bm25.rs:50` | Erzeugt eine neue `BM25`-Instanz mit validierten Hyperparametern. # Fehler Liefert [`TextError::InvalidInput`], falls `k1 < 0.0` bzw. NaN ist, oder `b` außerhal |
| `pub fn score_term(&self, tf: u32, doc_len: u32, avg_doc_len: f32, df: u32, n: u32) -> f32` | `crates/contextra-text/src/bm25.rs:62` | Berechnet den BM25-Score für einen Begriff mit den Parametern dieser Instanz. |

#### `struct BM25F`
- Ort: `crates/contextra-text/src/bm25.rs:172`
```rust
pub struct BM25F
```
Dokumentation: Konfiguration für feldgewichtetes BM25F-Scoring (§11.3).
- Impl-Block `crates/contextra-text/src/bm25.rs:180`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `k1` | `f32` | `crates/contextra-text/src/bm25.rs:174` |
| `field_weights` | `Vec<FieldWeight>` | `crates/contextra-text/src/bm25.rs:176` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(k1: f32, field_weights: Vec<FieldWeight>) -> Result<Self>` | `crates/contextra-text/src/bm25.rs:183` | Erzeugt ein neues BM25F-Modell mit Feldgewichtungen. |
| `pub fn score_term( &self, field_term_frequencies: &[(FieldId, u32, u32, f32)], df: u32, n: u32, ) -> f32` | `crates/contextra-text/src/bm25.rs:200` | Berechnet den BM25F-Score über mehrere Felder hinweg. |

#### `struct FieldWeight`
- Ort: `crates/contextra-text/src/bm25.rs:140`
```rust
pub struct FieldWeight
```
Dokumentation: Feldgewichtungskonfiguration für ein bestimmtes Feld in BM25F.
- Impl-Block `crates/contextra-text/src/bm25.rs:149`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `field_id` | `FieldId` | `crates/contextra-text/src/bm25.rs:142` |
| `weight` | `f32` | `crates/contextra-text/src/bm25.rs:144` |
| `b` | `f32` | `crates/contextra-text/src/bm25.rs:146` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(field_id: FieldId, weight: f32, b: f32) -> Result<Self>` | `crates/contextra-text/src/bm25.rs:152` | Erzeugt eine neue `FieldWeight`-Konfiguration nach Validierung von `weight` und `b`. |

#### `fn score_term`
- Ort: `crates/contextra-text/src/bm25.rs:86`
```rust
pub fn score_term(tf: u32, doc_len: u32, avg_doc_len: f32, df: u32, n: u32) -> f32
```
Dokumentation: Berechnet den BM25-Score für einen Begriff mit den Standardparametern (`k1 = 1.5`, `b = 0.75`). # Parameter * `tf` - Termhäufigkeit (Term Frequency) im Dokument * `doc_len` - Dokumentlänge in Token * `avg_doc_len` - Durchschnittliche Dokumentlänge im gesamten Index * `df` - Dokumenthäufigkeit (Document Frequency, Anzahl Dokumente mit diesem Term) * `n` - Gesamtzahl der Dokumente im Index

#### `fn score_term_bm25f`
- Ort: `crates/contextra-text/src/bm25.rs:222`
```rust
pub fn score_term_bm25f( field_term_frequencies: &[(FieldId, u32, u32, f32)], field_weights: &[(FieldId, f32, f32)], k1: f32, df: u32, n: u32, ) -> f32
```
Dokumentation: Berechnet den feldgewichteten BM25F-Score nach Robertson & Zaragoza (2004). Berechnet zunächst die gewichtete Pseudo-Termfrequenz über alle Felder und wendet anschließend die BM25-Sättigungsfunktion an. # Parameter * `field_term_frequencies` - Tupel aus `(field_id, tf_f, len_f, avg_len_f)` pro Feld * `field_weights` - Konfigurierte Feldgewichte * `k1` - Globaler Sättigungsparameter * `df` - Dokumentfrequenz im Gesamtkorpus * `n` - Dokumentenanzahl im Gesamtkorpus

#### `fn score_term_with_params`
- Ort: `crates/contextra-text/src/bm25.rs:98`
```rust
pub fn score_term_with_params( tf: u32, doc_len: u32, avg_doc_len: f32, df: u32, n: u32, k1: f32, b: f32, ) -> f32
```
Dokumentation: Berechnet den BM25-Score mit frei konfigurierbaren Parametern `k1` und `b`. # Ablauf 1. Schnellausstieg mit 0.0, wenn $N = 0$, $df = 0$ oder $tf = 0$. 2. Glatte IDF-Berechnung nach Robertson-Spärck-Jones. 3. Längennormalisierung von $tf$ anhand von $b$ und dem Verhältnis zur Durchschnittslänge. 4. Sättigung mit $k1$ und Multiplikation von IDF und TF-Term.

#### `type FieldId`
- Ort: `crates/contextra-text/src/bm25.rs:135`
```rust
pub type FieldId = u32
```
Dokumentation: Eindeutige Kennung für Dokumentenfelder im feldgewichteten BM25F-Scoring.

#### `const BM25_B`
- Ort: `crates/contextra-text/src/bm25.rs:28`
```rust
pub const BM25_B: f32 = 0.75
```
Dokumentation: Standard-b-Parameter für die Dokumentlängen-Normalisierung in BM25 (0.75).

#### `const BM25_K1`
- Ort: `crates/contextra-text/src/bm25.rs:24`
```rust
pub const BM25_K1: f32 = 1.5
```
Dokumentation: Standard-k1-Parameter für die Termfrequenz-Sättigung in BM25 (1.5).

### Modul `compat`

#### `mod error`
- Ort: `crates/contextra-text/src/compat/mod.rs:4`
```rust
pub mod error
```

#### `mod inverted_ports`
- Ort: `crates/contextra-text/src/compat/mod.rs:5`
```rust
pub mod inverted_ports
```

#### `mod morph_index`
- Ort: `crates/contextra-text/src/compat/mod.rs:6`
```rust
pub mod morph_index
```

#### `mod purge`
- Ort: `crates/contextra-text/src/compat/mod.rs:7`
```rust
pub mod purge
```

#### `mod scanner`
- Ort: `crates/contextra-text/src/compat/mod.rs:8`
```rust
pub mod scanner
```

#### `mod scorer`
- Ort: `crates/contextra-text/src/compat/mod.rs:9`
```rust
pub mod scorer
```

#### `mod storage`
- Ort: `crates/contextra-text/src/compat/mod.rs:10`
```rust
pub mod storage
```

#### `mod types`
- Ort: `crates/contextra-text/src/compat/mod.rs:11`
```rust
pub mod types
```

### Modul `compat::morph_index`

#### `struct BM25MorphIndex`
- Ort: `crates/contextra-text/src/compat/morph_index.rs:13`
```rust
pub struct BM25MorphIndex<S: StorageEngine>
```
Dokumentation: Invertierter Index mit morphologischer Zerlegung für deutsche Volltextsuche.
- Impl-Block `crates/contextra-text/src/compat/morph_index.rs:18`
- Impl-Block (Trait `TextIndex`) `crates/contextra-text/src/compat/morph_index.rs:43`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( storage: Arc<S>, namespace: &str, tokenizer: Arc<dyn MorphologicalTokenizer>, ) -> Self` | `crates/contextra-text/src/compat/morph_index.rs:21` | Erzeugt einen neuen morphologischen Index mit deutschem Standard-Tokenizer. |
| `pub fn tokenizer(&self) -> &dyn MorphologicalTokenizer` | `crates/contextra-text/src/compat/morph_index.rs:38` | Liefert eine Referenz auf den konfigurierten morphologischen Tokenizer. |

### Modul `compat::purge`

#### `struct PurgeReport`
- Ort: `crates/contextra-text/src/compat/purge.rs:44`
```rust
pub struct PurgeReport
```
- Impl-Block `crates/contextra-text/src/compat/purge.rs:55`
- Impl-Block (Trait `Default`) `crates/contextra-text/src/compat/purge.rs:90`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `units_purged` | `u64` | `crates/contextra-text/src/compat/purge.rs:46` |
| `units_failed` | `u64` | `crates/contextra-text/src/compat/purge.rs:48` |
| `bytes_freed` | `u64` | `crates/contextra-text/src/compat/purge.rs:50` |
| `errors` | `Vec<String>` | `crates/contextra-text/src/compat/purge.rs:52` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-text/src/compat/purge.rs:58` | Erzeugt einen leeren Purge-Bericht. |
| `pub fn is_success(&self) -> bool` | `crates/contextra-text/src/compat/purge.rs:69` | Liefert `true`, wenn die Bereinigung fehlerfrei und vollständig war. |
| `pub fn to_purge_stats(&self) -> Result<PurgeStats>` | `crates/contextra-text/src/compat/purge.rs:75` | Konvertiert den Bericht in [`PurgeStats`] oder liefert einen Fehler bei Teilabbrüchen. |

#### `struct TextPurger`
- Ort: `crates/contextra-text/src/compat/purge.rs:102`
```rust
pub struct TextPurger<S: StorageEngine + 'static>
```
- Impl-Block `crates/contextra-text/src/compat/purge.rs:106`
- Impl-Block (Trait `Purger`) `crates/contextra-text/src/compat/purge.rs:488`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(index: Arc<InvertedIndex<S>>) -> Self` | `crates/contextra-text/src/compat/purge.rs:108` | Creates a new `TextPurger` for the given index. |
| `pub fn purge_document_report(&self, tx: TxId, doc_id: DocId) -> PurgeReport` | `crates/contextra-text/src/compat/purge.rs:114` | Purges a single document from the inverted index and recalculates corpus statistics, returning a detailed `PurgeReport`. |
| `pub fn purge_document(&self, tx: TxId, doc_id: DocId) -> Result<PurgeStats>` | `crates/contextra-text/src/compat/purge.rs:120` | Purges a single document from the inverted index and recalculates corpus statistics. |
| `pub fn purge_collection_report(&self, tx: TxId, namespace: &str) -> PurgeReport` | `crates/contextra-text/src/compat/purge.rs:420` | Fast-path purge for an entire collection namespace (`__txt:{collection}:`), returning a `PurgeReport`. |
| `pub fn purge_collection(&self, tx: TxId, namespace: &str) -> Result<PurgeStats>` | `crates/contextra-text/src/compat/purge.rs:482` | Fast-path purge for an entire collection namespace (`__txt:{collection}:`). |

#### `fn text_purger`
- Ort: `crates/contextra-text/src/compat/purge.rs:505`
```rust
pub fn text_purger<S: StorageEngine + 'static>(index: Arc<InvertedIndex<S>>) -> Arc<dyn Purger>
```

### Modul `compat::scanner`

#### `struct TextScanner`
- Ort: `crates/contextra-text/src/compat/scanner.rs:96`
```rust
pub struct TextScanner<S: StorageEngine + 'static>
```
- Impl-Block `crates/contextra-text/src/compat/scanner.rs:101`
- Impl-Block (Trait `LayerScanner`) `crates/contextra-text/src/compat/scanner.rs:195`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(index: Arc<InvertedIndex<S>>) -> Self` | `crates/contextra-text/src/compat/scanner.rs:104` | Erzeugt einen neuen `TextScanner` für den übergebenen invertierten Index. |
| `pub fn new_with_target(index: Arc<InvertedIndex<S>>, target: ScanTarget) -> Self` | `crates/contextra-text/src/compat/scanner.rs:113` | Erzeugt einen neuen `TextScanner` mit vorab konfiguriertem Scan-Ziel. |
| `pub fn with_target(self, target: ScanTarget) -> Self` | `crates/contextra-text/src/compat/scanner.rs:122` | Konfiguriert das Scan-Ziel im Builder-Muster. |
| `pub fn set_target(&self, target: ScanTarget)` | `crates/contextra-text/src/compat/scanner.rs:129` | Weist dem Scanner ein neues Zielobjekt zu. |

#### `enum ParsedTextTarget`
- Ort: `crates/contextra-text/src/compat/scanner.rs:33`
```rust
pub enum ParsedTextTarget
```
- Impl-Block `crates/contextra-text/src/compat/scanner.rs:40`

Varianten:

- `Document(DocId)` — `crates/contextra-text/src/compat/scanner.rs:35`
- `Collection(String)` — `crates/contextra-text/src/compat/scanner.rs:37`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn parse(target_id: &str) -> Self` | `crates/contextra-text/src/compat/scanner.rs:42` | Parses a target ID string into document ID or collection namespace. |

#### `fn text_scanner`
- Ort: `crates/contextra-text/src/compat/scanner.rs:305`
```rust
pub fn text_scanner<S: StorageEngine + 'static>( index: Arc<InvertedIndex<S>>, ) -> Arc<dyn LayerScanner>
```

### Modul `compat::scorer`

#### `struct Bm25Scorer`
- Ort: `crates/contextra-text/src/compat/scorer.rs:23`
```rust
pub struct Bm25Scorer<S: contextra_ports::StorageEngine>
```
- Impl-Block `crates/contextra-text/src/compat/scorer.rs:27`
- Impl-Block (Trait `TextIndex`) `crates/contextra-text/src/compat/scorer.rs:45`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(storage: std::sync::Arc<S>, namespace: &str) -> Self` | `crates/contextra-text/src/compat/scorer.rs:38` | Erzeugt eine neue `Bm25Scorer`-Instanz über der angegebenen Speicher-Engine. # Ablauf 1. Übernahme der Referenz auf die `StorageEngine`. 2. Initialisierung des  |

### Modul `compat::storage`

#### `struct StorageBridge`
- Ort: `crates/contextra-text/src/compat/storage.rs:10`
```rust
pub struct StorageBridge<S>(pub S)
```
Dokumentation: Adapter-Newtype für `StorageEngine` zur Vermeidung von Blanket-Impl-Kollisionen.
- Impl-Block (Trait `PostingStore`) `crates/contextra-text/src/compat/storage.rs:12`

### Modul `contract`

#### `struct DocKey`
- Ort: `crates/contextra-text/src/contract.rs:53`
```rust
pub struct DocKey(pub u128)
```
Dokumentation: Autarker Dokumentenschlüssel (128-Bit fähig).
- Impl-Block (Trait `From<u64>`) `crates/contextra-text/src/contract.rs:55`
- Impl-Block (Trait `From<u128>`) `crates/contextra-text/src/contract.rs:62`

#### `struct Epoch`
- Ort: `crates/contextra-text/src/contract.rs:71`
```rust
pub struct Epoch(pub u64)
```
Dokumentation: Opake, monotone Sichtbarkeitsmarke zur Abstraktion von Transaktions- und MVCC-Ständen.

#### `enum Error`
- Ort: `crates/contextra-text/src/contract.rs:6`
```rust
pub enum Error
```
Dokumentation: Minimaler Fehlertyp für Umgebungs- und Kontraktoperationen.

Varianten:

- `InvalidInput(&'static str)` — `crates/contextra-text/src/contract.rs:9`
- `CapacityExceeded{` — `crates/contextra-text/src/contract.rs:12`
- `Io(#[from] std::io::Error)` — `crates/contextra-text/src/contract.rs:20`
- `StateConflict(&'static str)` — `crates/contextra-text/src/contract.rs:23`
- `Corrupt(&'static str)` — `crates/contextra-text/src/contract.rs:26`

#### `trait Clock`
- Ort: `crates/contextra-text/src/contract.rs:33`
```rust
pub trait Clock: Send + Sync
```
Dokumentation: Minimale Zeituhr-Schnittstelle zur Bereitstellung monotoner oder Unix-Millisekunden.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn now_unix_ms(&self) -> u64` | `crates/contextra-text/src/contract.rs:35` | Liefert den aktuellen Unix-Zeitstempel in Millisekunden. |

#### `trait Rng`
- Ort: `crates/contextra-text/src/contract.rs:46`
```rust
pub trait Rng: Send
```
Dokumentation: Minimale Zufallsquelle für Baukasten-Operationen.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn next_u64(&mut self) -> u64` | `crates/contextra-text/src/contract.rs:48` | Liefert die nächste Zufallszahl als `u64`. |

#### `type Result`
- Ort: `crates/contextra-text/src/contract.rs:30`
```rust
pub type Result<T> = core::result::Result<T, Error>
```
Dokumentation: Einheitlicher Result-Typ für Kontraktoperationen.

### Modul `crate`

#### `mod bm25`
- Ort: `crates/contextra-text/src/lib.rs:67`
```rust
pub mod bm25
```

#### `mod compat`
- Ort: `crates/contextra-text/src/lib.rs:51`
```rust
pub mod compat
```
- Gate: `#[cfg(feature = "types-compat")]`

#### `mod contract`
- Ort: `crates/contextra-text/src/lib.rs:52`
```rust
pub mod contract
```

#### `mod domain`
- Ort: `crates/contextra-text/src/lib.rs:68`
```rust
pub mod domain
```

#### `mod error`
- Ort: `crates/contextra-text/src/lib.rs:53`
```rust
pub mod error
```

#### `mod inverted`
- Ort: `crates/contextra-text/src/lib.rs:69`
```rust
pub mod inverted
```

#### `mod memory_index`
- Ort: `crates/contextra-text/src/lib.rs:54`
```rust
pub mod memory_index
```

#### `mod morphology`
- Ort: `crates/contextra-text/src/lib.rs:70`
```rust
pub mod morphology
```

#### `mod posting_list`
- Ort: `crates/contextra-text/src/lib.rs:71`
```rust
pub mod posting_list
```

#### `mod storage`
- Ort: `crates/contextra-text/src/lib.rs:55`
```rust
pub mod storage
```

#### `mod stream`
- Ort: `crates/contextra-text/src/lib.rs:72`
```rust
pub mod stream
```

#### `mod tokenizer`
- Ort: `crates/contextra-text/src/lib.rs:73`
```rust
pub mod tokenizer
```

#### `mod traits`
- Ort: `crates/contextra-text/src/lib.rs:56`
```rust
pub mod traits
```

#### `mod types`
- Ort: `crates/contextra-text/src/lib.rs:57`
```rust
pub mod types
```

#### `mod wand`
- Ort: `crates/contextra-text/src/lib.rs:74`
```rust
pub mod wand
```

### Modul `domain`

#### `mod legal_de`
- Ort: `crates/contextra-text/src/domain/mod.rs:11`
```rust
pub mod legal_de
```

#### `mod medical_de`
- Ort: `crates/contextra-text/src/domain/mod.rs:12`
```rust
pub mod medical_de
```

#### `trait DomainVocabulary`
- Ort: `crates/contextra-text/src/domain/mod.rs:22`
```rust
pub trait DomainVocabulary: Send + Sync
```
Dokumentation: Trait für domänenspezifische Vokabularpakete (z. B. Recht und Medizin).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn compound_stems(&self) -> &'static [&'static str]` | `crates/contextra-text/src/domain/mod.rs:24` | Domänenspezifische Komposita-Bausteine (Ergänzung zum KMU-Basiswörterbuch). |
| `fn protected_terms(&self) -> &'static [&'static str]` | `crates/contextra-text/src/domain/mod.rs:27` | Fachbegriffe, die NICHT als Stopwort behandelt werden dürfen, selbst wenn sie kurz/häufig sind (z. B. "Klage", "Akte" im juristischen Kontext). |
| `fn domain_id(&self) -> &'static str` | `crates/contextra-text/src/domain/mod.rs:29` | Kurzname für Logging/Konfiguration, z. B. "legal_de", "medical_de". |

### Modul `domain::legal_de`

#### `struct LegalDomainVocabulary`
- Ort: `crates/contextra-text/src/domain/legal_de.rs:24`
```rust
pub struct LegalDomainVocabulary
```
Dokumentation: Deutsches Juristisches Fachvokabular zur Anbindung an [`DomainVocabulary`].
- Impl-Block (Trait `DomainVocabulary`) `crates/contextra-text/src/domain/legal_de.rs:26`

#### `fn get_legal_compound_stems`
- Ort: `crates/contextra-text/src/domain/legal_de.rs:42`
```rust
pub fn get_legal_compound_stems() -> &'static [&'static str]
```
Dokumentation: Liefert den statischen Slice aller juristischen Komposita-Wortstämme.

#### `fn get_legal_protected_terms`
- Ort: `crates/contextra-text/src/domain/legal_de.rs:48`
```rust
pub fn get_legal_protected_terms() -> &'static [&'static str]
```
Dokumentation: Liefert den statischen Slice aller juristischen geschützten Begriffe.

#### `static LEGAL_COMPOUND_STEMS`
- Ort: `crates/contextra-text/src/domain/legal_de.rs:54`
```rust
pub static LEGAL_COMPOUND_STEMS: std::sync::LazyLock<&'static [&'static str]> = std::sync::LazyLock::new(get_legal_compound_stems)
```
Dokumentation: Statische Referenz-Slices für Abwärtskompatibilität in vorhandenen Tests.

#### `static LEGAL_PROTECTED_TERMS`
- Ort: `crates/contextra-text/src/domain/legal_de.rs:59`
```rust
pub static LEGAL_PROTECTED_TERMS: std::sync::LazyLock<&'static [&'static str]> = std::sync::LazyLock::new(get_legal_protected_terms)
```
Dokumentation: Statische Referenz-Slices geschützter Begriffe für Abwärtskompatibilität.

### Modul `domain::medical_de`

#### `struct MedicalDomainVocabulary`
- Ort: `crates/contextra-text/src/domain/medical_de.rs:24`
```rust
pub struct MedicalDomainVocabulary
```
Dokumentation: Deutsches Medizinisches Fachvokabular zur Anbindung an [`DomainVocabulary`].
- Impl-Block (Trait `DomainVocabulary`) `crates/contextra-text/src/domain/medical_de.rs:26`

#### `fn get_medical_compound_stems`
- Ort: `crates/contextra-text/src/domain/medical_de.rs:42`
```rust
pub fn get_medical_compound_stems() -> &'static [&'static str]
```
Dokumentation: Liefert den statischen Slice aller medizinischen Komposita-Wortstämme.

#### `fn get_medical_protected_terms`
- Ort: `crates/contextra-text/src/domain/medical_de.rs:48`
```rust
pub fn get_medical_protected_terms() -> &'static [&'static str]
```
Dokumentation: Liefert den statischen Slice aller medizinischen geschützten Fachbegriffe.

#### `static MEDICAL_COMPOUND_STEMS`
- Ort: `crates/contextra-text/src/domain/medical_de.rs:54`
```rust
pub static MEDICAL_COMPOUND_STEMS: std::sync::LazyLock<&'static [&'static str]> = std::sync::LazyLock::new(get_medical_compound_stems)
```
Dokumentation: Statische Referenz-Slices für Abwärtskompatibilität in vorhandenen Tests.

#### `static MEDICAL_PROTECTED_TERMS`
- Ort: `crates/contextra-text/src/domain/medical_de.rs:59`
```rust
pub static MEDICAL_PROTECTED_TERMS: std::sync::LazyLock<&'static [&'static str]> = std::sync::LazyLock::new(get_medical_protected_terms)
```
Dokumentation: Statische Referenz-Slices geschützter Fachbegriffe für Abwärtskompatibilität.

### Modul `error`

#### `enum TextError`
- Ort: `crates/contextra-text/src/error.rs:8`
```rust
pub enum TextError
```
Dokumentation: Typisierter Fehler für Volltextindexierung, Tokenisierung und Scoring.
- Impl-Block (Trait `From<ContextraError>`) `crates/contextra-text/src/compat/error.rs:25`

Varianten:

- `InvalidInput(String)` — `crates/contextra-text/src/error.rs:11`
- `Text(String)` — `crates/contextra-text/src/error.rs:15`
- `CapacityExceeded{` — `crates/contextra-text/src/error.rs:19`
- `Storage(String)` — `crates/contextra-text/src/error.rs:28`
- `Serialization(String)` — `crates/contextra-text/src/error.rs:32`
- `Io(#[from] std::io::Error)` — `crates/contextra-text/src/error.rs:36`
- `Internal(String)` — `crates/contextra-text/src/error.rs:40`

#### `type Result`
- Ort: `crates/contextra-text/src/error.rs:44`
```rust
pub type Result<T> = core::result::Result<T, TextError>
```
Dokumentation: Einheitlicher Result-Typ für `contextra-text`.

### Modul `inverted::index_struct`

#### `struct InvertedIndex`
- Ort: `crates/contextra-text/src/inverted/index_struct.rs:20`
```rust
pub struct InvertedIndex<S: PostingStore>
```
Dokumentation: Persistenter invertierter Index über einer Kollektion im Storage-Backend. # Nebenläufigkeit & Sperrhierarchie: 1. `commit_lock` (`parking_lot::Mutex`): Serialisiert Transaktions-Commits auf den Statistiken (Single-Writer). 2. `staged_stats` (`parking_lot::Mutex`): Kurzlebiger Spinlock für ungecommittete Transaktionsänderungen im RAM. **Regel**: `commit_lock` muss IMMER vor `staged_stats` akquiriert werden. Niemals synchrone Sperren über `.await`-Punkte halten!
- Impl-Block (Trait `TextIndex`) `crates/contextra-text/src/compat/inverted_ports.rs:10`
- Impl-Block (Trait `Clone`) `crates/contextra-text/src/inverted/index_struct.rs:33`
- Impl-Block `crates/contextra-text/src/inverted/index_struct.rs:50`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `total_docs` | `Arc<AtomicU64>` | `crates/contextra-text/src/inverted/index_struct.rs:24` |
| `total_tokens` | `Arc<AtomicU64>` | `crates/contextra-text/src/inverted/index_struct.rs:25` |
| `avg_doc_len_x1000` | `Arc<AtomicU64>` | `crates/contextra-text/src/inverted/index_struct.rs:26` |
| `resident_index` | `Arc<ResidentPostingIndex>` | `crates/contextra-text/src/inverted/index_struct.rs:30` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new_with_language(storage: Arc<S>, namespace: &str, language: Language) -> Self` | `crates/contextra-text/src/inverted/index_struct.rs:67` | Erzeugt einen neuen invertierten Index mit expliziter Sprachkonfiguration. # Language Selection Use `Language::from_iso("de")` for German, `Language::English` ( |
| `pub fn new(storage: Arc<S>, namespace: &str) -> Self` | `crates/contextra-text/src/inverted/index_struct.rs:95` | Erzeugt einen neuen invertierten Index mit englischem Standard-Tokenizer. |
| `pub fn with_tokenizer(mut self, tokenizer: Arc<dyn Tokenizer>) -> Self` | `crates/contextra-text/src/inverted/index_struct.rs:101` | Konfiguriert einen benutzerdefinierten Tokenizer. |
| `pub fn storage(&self) -> &Arc<S>` | `crates/contextra-text/src/inverted/index_struct.rs:107` | Accessor for underlying storage engine. |
| `pub fn prefix(&self) -> &[u8]` | `crates/contextra-text/src/inverted/index_struct.rs:112` | Accessor for index storage prefix bytes. |
| `pub fn commit_lock(&self) -> &parking_lot::Mutex<()>` | `crates/contextra-text/src/inverted/index_struct.rs:117` | Accessor for single-writer commit lock. |
| `pub async fn load_stats(&self) -> Result<()>` | `crates/contextra-text/src/inverted/index_struct.rs:123` | Lädt die persistenten Statistiken (`meta:stats`) aus dem Storage in die atomaren In-Memory-Felder. |
| `pub async fn upsert_document(&self, tx: TxId, doc_id: DocId, text: &str) -> Result<()>` | `crates/contextra-text/src/inverted/index_struct.rs:256` | Fügt ein Dokument in den Index ein oder aktualisiert ein bestehendes. |
| `pub async fn rewrite_posting_lists(&self, tx: TxId, terms: &[&str]) -> Result<usize>` | `crates/contextra-text/src/inverted/index_struct.rs:341` | Schreibt Postinglisten für gegebene Terme neu (Maintenance / Rewrite API). |
| `pub async fn compact_terms(&self, tx: TxId) -> Result<u64>` | `crates/contextra-text/src/inverted/index_struct.rs:362` | Kompaktiert Terme und bereinigt verwaiste Metadaten im Inverted Index (Maintenance / Compaction API). |
| `pub async fn resolve_tombstones(&self, tx: TxId) -> Result<u64>` | `crates/contextra-text/src/inverted/index_struct.rs:367` | Bereinigt Tombstones im Inverted Index für gegebene Transaktion. |
| `pub async fn delete_document(&self, tx: TxId, doc_id: DocId) -> Result<()>` | `crates/contextra-text/src/inverted/index_struct.rs:440` | Löscht ein Dokument logisch aus dem Index und markiert alle Terme mit Tombstones. |
| `pub async fn commit_stats(&self, tx: TxId) -> Result<()>` | `crates/contextra-text/src/inverted/index_struct.rs:509` | Überträgt die gepufferten statistischen Änderungen einer Transaktion dauerhaft. |
| `pub async fn rollback_stats(&self, tx: TxId) -> Result<()>` | `crates/contextra-text/src/inverted/index_struct.rs:571` | Verwirft die gepufferten statistischen Änderungen einer Transaktion bei Rollback. |
| `pub async fn search_bm25( &self, query: &str, k: usize, max_seq: Option<u64>, ) -> Result<Vec<(DocId, f32)>>` | `crates/contextra-text/src/inverted/index_struct.rs:581` | Führt eine Volltextsuche mit BM25 aus. |
| `pub async fn search_bm25_at( &self, query: &str, k: usize, max_seq: Option<u64>, ) -> Result<Vec<(DocId, f32)>>` | `crates/contextra-text/src/inverted/index_struct.rs:593` | Führt eine Volltextsuche mit BM25 zu einer bestimmten MVCC-Snapshot-Sequenznummer aus. |
| `pub fn len_sync(&self) -> usize` | `crates/contextra-text/src/inverted/index_struct.rs:645` | Liefert die Anzahl aktiver Dokumente im Index. |
| `pub async fn commit(&self, tx: TxId) -> Result<()>` | `crates/contextra-text/src/inverted/index_struct.rs:650` | Delegiert das Commit auf der Storage-Engine nach `commit_stats`. |
| `pub async fn rollback(&self, tx: TxId) -> Result<()>` | `crates/contextra-text/src/inverted/index_struct.rs:656` | Delegiert das Rollback auf der Storage-Engine nach `rollback_stats`. |
| `pub async fn rollback_to_tx(&self, tx_id: TxId) -> Result<()>` | `crates/contextra-text/src/inverted/index_struct.rs:662` | Delegiert `rollback_to_tx` auf der Storage-Engine. |
| `pub async fn last_tx_id(&self) -> Result<TxId>` | `crates/contextra-text/src/inverted/index_struct.rs:667` | Delegiert `last_tx_id` auf der Storage-Engine. |

Assoziierte Konstanten/Typen:

- `pub const MAX_TEXT_BYTES: usize = 10 * 1024 * 1024` — `crates/contextra-text/src/inverted/index_struct.rs:53`
- `pub const MAX_STAGED_TRANSACTIONS: usize = 10_000` — `crates/contextra-text/src/inverted/index_struct.rs:57`

### Modul `inverted::types`

#### `struct TextIndexMetadata`
- Ort: `crates/contextra-text/src/inverted/types.rs:39`
```rust
pub struct TextIndexMetadata
```
Dokumentation: Konsolidierte Metadaten des Textindex zur persistenten Speicherung im Schlüssel `meta:stats`.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `total_docs` | `u64` | `crates/contextra-text/src/inverted/types.rs:41` |
| `total_tokens` | `u64` | `crates/contextra-text/src/inverted/types.rs:43` |
| `avg_doc_len_x1000` | `u64` | `crates/contextra-text/src/inverted/types.rs:45` |

#### `enum Language`
- Ort: `crates/contextra-text/src/inverted/types.rs:10`
```rust
pub enum Language
```
Dokumentation: Unterstützte Sprachen und Tokenisierungsstrategien für den invertierten Index. Steuert, welcher Tokenizer für die Dokumentenindizierung und Suchabfragen verwendet wird.
- Impl-Block `crates/contextra-text/src/inverted/types.rs:20`

Varianten:

- `English` — `crates/contextra-text/src/inverted/types.rs:13`
- `German` — `crates/contextra-text/src/inverted/types.rs:15`
- `Custom` — `crates/contextra-text/src/inverted/types.rs:17`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_iso(code: &str) -> Self` | `crates/contextra-text/src/inverted/types.rs:26` | Parst einen ISO-639-1 Sprachcode ("de", "de-AT", "en", "en-US"). Bewertet nur das primäre Subtag vor dem ersten Bindestrich. Unbekannte Codes fallen sicher auf  |

### Modul `memory_index`

#### `struct CompactStats`
- Ort: `crates/contextra-text/src/memory_index.rs:35`
```rust
pub struct CompactStats
```
Dokumentation: Statistiken über den Kompaktierungsvorgang.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `purged_docs` | `usize` | `crates/contextra-text/src/memory_index.rs:37` |
| `active_docs` | `usize` | `crates/contextra-text/src/memory_index.rs:39` |

#### `struct IndexSnapshot`
- Ort: `crates/contextra-text/src/memory_index.rs:56`
```rust
pub struct IndexSnapshot
```
Dokumentation: Unveränderlicher Snapshot des Index für lock-freie, konsistente Lesezugriffe.
- Impl-Block `crates/contextra-text/src/memory_index.rs:71`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `epoch` | `Epoch` | `crates/contextra-text/src/memory_index.rs:58` |
| `tombstones` | `BTreeSet<DocKey>` | `crates/contextra-text/src/memory_index.rs:64` |
| `active_docs` | `usize` | `crates/contextra-text/src/memory_index.rs:66` |
| `avg_doc_len` | `f32` | `crates/contextra-text/src/memory_index.rs:68` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn search(&self, query_tokens: &[String], scorer: &BM25, limit: usize) -> Vec<TextHit>` | `crates/contextra-text/src/memory_index.rs:85` | Sucht im unveränderlichen Snapshot anhand von BM25. |

#### `struct KeyInspection`
- Ort: `crates/contextra-text/src/memory_index.rs:45`
```rust
pub struct KeyInspection
```
Dokumentation: Prüfergebnis einer Schlüssel-Inspektion (Feature `inspect`).
- Gate: `#[cfg(feature = "inspect")]`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `in_doc_lengths` | `bool` | `crates/contextra-text/src/memory_index.rs:47` |
| `is_tombstone` | `bool` | `crates/contextra-text/src/memory_index.rs:49` |
| `posting_occurrences` | `usize` | `crates/contextra-text/src/memory_index.rs:51` |

#### `struct MemoryTextIndex`
- Ort: `crates/contextra-text/src/memory_index.rs:151`
```rust
pub struct MemoryTextIndex
```
Dokumentation: Autarker, thread-sicherer In-Memory Volltextindex (Klasse A). Bietet die standardisierte Index-Schnittstelle gemäß SPEC §4.3 (`insert`, `remove`, `search`) sowie neutrale Wartung (`compact`, `vacuum`) mit Copy-on-Write Snapshot-Isolation.
- Impl-Block (Trait `Default`) `crates/contextra-text/src/memory_index.rs:157`
- Impl-Block `crates/contextra-text/src/memory_index.rs:163`
- Impl-Block (Trait `Index<DocKey, str>`) `crates/contextra-text/src/memory_index.rs:232`
- Impl-Block (Trait `Index<DocId, str>`) `crates/contextra-text/src/memory_index.rs:387`
- Impl-Block (Trait `Maintain`) `crates/contextra-text/src/memory_index.rs:403`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-text/src/memory_index.rs:165` | Erzeugt einen neuen `MemoryTextIndex` mit Standard-Tokenizer und BM25-Scorer. |
| `pub fn with_tokenizer(mut self, tokenizer: Arc<dyn Tokenizer>) -> Self` | `crates/contextra-text/src/memory_index.rs:174` | Konfiguriert einen spezifischen Tokenizer. |
| `pub fn snapshot(&self) -> Arc<IndexSnapshot>` | `crates/contextra-text/src/memory_index.rs:180` | Liefert einen unveränderlichen Snapshot des Index (O(1) Lesezugriff). |
| `pub fn current_epoch(&self) -> Epoch` | `crates/contextra-text/src/memory_index.rs:185` | Liefert die aktuelle Epoche des Index. |
| `pub fn len(&self) -> usize` | `crates/contextra-text/src/memory_index.rs:190` | Liefert die Anzahl aktiver Dokumente im Index. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-text/src/memory_index.rs:195` | Gibt an, ob der Index keine aktiven Dokumente enthält. |
| `pub fn inspect_key(&self, key: &DocKey) -> KeyInspection` | `crates/contextra-text/src/memory_index.rs:201` | Rohsicht-Inspektion unter dem Feature `inspect`. |
| `pub fn inspect_postings(&self) -> BTreeMap<String, Vec<Posting>>` | `crates/contextra-text/src/memory_index.rs:221` | Rohsicht-Inspektion aller Postings unter dem Feature `inspect`. |
| `pub fn inspect_doc_lengths(&self) -> BTreeMap<DocKey, u32>` | `crates/contextra-text/src/memory_index.rs:227` | Rohsicht-Inspektion der Dokumentlängen unter dem Feature `inspect`. |

#### `const MAX_DOC_BYTES`
- Ort: `crates/contextra-text/src/memory_index.rs:25`
```rust
pub const MAX_DOC_BYTES: usize = 10 * 1024 * 1024
```
Dokumentation: Maximale Dokumentgröße in Bytes (fail-closed, 10 MB).

#### `const MAX_DOC_TERMS`
- Ort: `crates/contextra-text/src/memory_index.rs:28`
```rust
pub const MAX_DOC_TERMS: usize = 1_000_000
```
Dokumentation: Maximale Anzahl von Begriffen (Tokens) pro Dokument (fail-closed).

#### `const MAX_POSTINGS_PER_TERM`
- Ort: `crates/contextra-text/src/memory_index.rs:31`
```rust
pub const MAX_POSTINGS_PER_TERM: usize = 10_000_000
```
Dokumentation: Maximale Anzahl von Postings pro Term (fail-closed).

### Modul `morphology::compound_splitter`

#### `struct GermanCompoundSplitter`
- Ort: `crates/contextra-text/src/morphology/compound_splitter.rs:42`
```rust
pub struct GermanCompoundSplitter
```
Dokumentation: Deutscher Komposita-Zerleger basierend auf Dynamischer Programmierung und Fugenanalyse. # Funktionsweise & Algorithmus 1. **Eingebettetes Wörterbuch**: Tausende deutsche Stämme, Grundwörter und Affixe im [`Trie`]. 2. **Dynamische Programmierung**: Sucht den optimalen Pfad durch das Wort mit minimaler Segmentanzahl und maximaler Mindestsegmentlänge. 3. **Fugenelemente**: Berücksichtigt Zwischenfugen (`-s-`, `-en-`, `-e-`, `-er-`, `-n-`, `-es-`). Fallback: Bei unbekannten Wörtern oder fehlenden Treffern wird das Ursprungswort unzerlegt zurückgegeben.
- Impl-Block `crates/contextra-text/src/morphology/compound_splitter.rs:51`
- Impl-Block (Trait `Default`) `crates/contextra-text/src/morphology/compound_splitter.rs:190`
- Impl-Block (Trait `MorphologicalTokenizer`) `crates/contextra-text/src/morphology/compound_splitter.rs:203`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-text/src/morphology/compound_splitter.rs:54` | Erzeugt einen neuen Zerleger mit der Standard-Mindestlänge (3 Zeichen) und Basis-Wortschatz. |
| `pub fn with_min_length(min_len: usize) -> Self` | `crates/contextra-text/src/morphology/compound_splitter.rs:60` | Erzeugt einen Zerleger mit konfigurierbarer Mindest-Segmentlänge. |
| `pub fn with_dictionary(min_len: usize, custom_words: HashSet<String>) -> Self` | `crates/contextra-text/src/morphology/compound_splitter.rs:81` | Erzeugt einen Zerleger mit benutzerdefiniertem Wörterbuch. |
| `pub fn with_domains(min_len: usize, domains: &[&dyn DomainVocabulary]) -> Self` | `crates/contextra-text/src/morphology/compound_splitter.rs:98` | Erzeugt einen Zerleger erweitert um spezifische Fachvokabulare (z. B. Recht, Medizin). |
| `pub fn new_with_all_domains() -> Self` | `crates/contextra-text/src/morphology/compound_splitter.rs:119` | Erzeugt einen Zerleger mit allen eingebauten Fachdomänen (Recht & Medizin). |
| `pub fn extend_vocabulary(&mut self, words: impl IntoIterator<Item = impl AsRef<str>>)` | `crates/contextra-text/src/morphology/compound_splitter.rs:127` | Erweitert das Wörterbuch des Zerlegers dynamisch um zusätzliche Begriffe. |
| `pub fn min_component_len(&self) -> usize` | `crates/contextra-text/src/morphology/compound_splitter.rs:138` | Liefert die Mindestlänge für Teilkomponenten. |
| `pub fn with_max_depth(mut self, max_depth: usize) -> Self` | `crates/contextra-text/src/morphology/compound_splitter.rs:143` | Setzt die maximale Rekursions-/Segmenttiefe für Zerlegungen. |
| `pub fn max_depth(&self) -> usize` | `crates/contextra-text/src/morphology/compound_splitter.rs:149` | Liefert die maximale Segmenttiefe. |

#### `trait MorphologicalTokenizer`
- Ort: `crates/contextra-text/src/morphology/compound_splitter.rs:24`
```rust
pub trait MorphologicalTokenizer: Send + Sync
```
Dokumentation: Trait für morphologische Zerlegung von Wörtern in ihre Grundbestandteile. # Eingabevertrag Eingegebene Wörter MÜSSEN vor dem Aufruf von `decompose` in Kleinbuchstaben vorliegen und idealerweise mit [`normalize_umlauts`] vorbereitet sein. Bei Großbuchstaben oder ungültigen Zeichenfolgen wird das Token unzerlegt zurückgegeben. Beispiel: "bundesverfassungsgericht" -> \["bundes", "verfassungs", "gericht"\]

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn decompose<'a>(&self, token: &'a str) -> Vec<&'a str>` | `crates/contextra-text/src/morphology/compound_splitter.rs:26` | Zerlegt ein Token in morphologische Komponenten (Teilwörter). |
| `fn language(&self) -> &str` | `crates/contextra-text/src/morphology/compound_splitter.rs:29` | Liefert den Sprachcode (z. B. "de"). |

### Modul `morphology::metrics`

#### `struct TokenReductionMetrics`
- Ort: `crates/contextra-text/src/morphology/metrics.rs:6`
```rust
pub struct TokenReductionMetrics
```
Dokumentation: Metriken zur Erfassung der Token-Expansions- und Zerlegungswirkung.
- Impl-Block `crates/contextra-text/src/morphology/metrics.rs:13`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `original_tokens` | `usize` | `crates/contextra-text/src/morphology/metrics.rs:8` |
| `decomposed_tokens` | `usize` | `crates/contextra-text/src/morphology/metrics.rs:10` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn expansion_ratio(&self) -> f32` | `crates/contextra-text/src/morphology/metrics.rs:26` | Berechnet das Expansionsverhältnis der Token-Menge. Ein Wert > 1,0 bedeutet eine Erhöhung der Tokenanzahl durch Komposita-Splitting. # Zielwert Für deutsche Fac |

### Modul `morphology::passthrough`

#### `struct PassthroughTokenizer`
- Ort: `crates/contextra-text/src/morphology/passthrough.rs:9`
```rust
pub struct PassthroughTokenizer
```
Dokumentation: Passthrough-Tokenizer für Sprachen, die keine Zerlegung von Zusammensetzungen benötigen. Gibt eingegebene Token unverändert als Einzelkomponente zurück.
- Impl-Block `crates/contextra-text/src/morphology/passthrough.rs:13`
- Impl-Block (Trait `MorphologicalTokenizer`) `crates/contextra-text/src/morphology/passthrough.rs:21`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(lang: impl Into<String>) -> Self` | `crates/contextra-text/src/morphology/passthrough.rs:16` | Erzeugt einen neuen Durchleitungs-Tokenizer für den angegebenen ISO-Sprachcode (z. B. "en"). |

### Modul `morphology::stopwords`

#### `fn get_german_stopwords`
- Ort: `crates/contextra-text/src/morphology/stopwords.rs:13`
```rust
pub fn get_german_stopwords() -> &'static HashSet<String>
```
Dokumentation: Liefert die statische deutsche Stoppwort-Liste. Wird per [`OnceLock`] beim ersten Zugriff lazy initialisiert. Erweitert um hochfrequente deutsche Funktionswörter gemäß DACH-KMU-Domänenanalyse, 2026-09-01.

#### `fn is_german_stopword`
- Ort: `crates/contextra-text/src/morphology/stopwords.rs:32`
```rust
pub fn is_german_stopword(word: &str) -> bool
```
Dokumentation: Prüft, ob ein kleingeschriebenes, normalisiertes Wort ein deutsches Stoppwort ist.

#### `fn normalize_umlauts`
- Ort: `crates/contextra-text/src/morphology/stopwords.rs:56`
```rust
pub fn normalize_umlauts(input: &str) -> String
```
Dokumentation: Normalisiert deutsche Umlaute und Sonderzeichen für robusten Volltextabgleich. # Ersetzungen - `ä` -> `"ae"` - `ö` -> `"oe"` - `ü` -> `"ue"` - `ß` -> `"ss"` # Beispiel ``` use contextra_text::morphology::{normalize_umlauts, GermanCompoundSplitter}; use contextra_text::morphology::MorphologicalTokenizer; let splitter = GermanCompoundSplitter::new(); let normalized = normalize_umlauts("Bundesverfassungsgericht"); assert_eq!(normalized, "bundesverfassungsgericht"); let parts = splitter.decompose(&normalized); assert!(parts.contains(&"gericht")); ```

### Modul `morphology::trie`

#### `struct Trie`
- Ort: `crates/contextra-text/src/morphology/trie.rs:16`
```rust
pub struct Trie
```
Dokumentation: Präfixbaum (Trie) für schnelle O(m)-Wörterbuchabfragen (m = Wortlänge).
- Impl-Block `crates/contextra-text/src/morphology/trie.rs:20`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-text/src/morphology/trie.rs:23` | Erzeugt einen leeren Präfixbaum. |
| `pub fn insert(&mut self, word: &str)` | `crates/contextra-text/src/morphology/trie.rs:29` | Fügt ein Wort zeichenweise in den Baum ein und markiert den Endknoten als terminal. |
| `pub fn contains(&self, word: &str) -> bool` | `crates/contextra-text/src/morphology/trie.rs:39` | Prüft, ob ein Wort exakt als terminaler Eintrag im Baum enthalten ist. |
| `pub fn starts_with(&self, prefix: &str) -> bool` | `crates/contextra-text/src/morphology/trie.rs:53` | Prüft, ob im Baum mindestens ein Wort existiert, das mit dem übergebenen Präfix beginnt. |

#### `struct TrieNode`
- Ort: `crates/contextra-text/src/morphology/trie.rs:6`
```rust
pub struct TrieNode
```
Dokumentation: Knoten im Präfixbaum zur Repräsentation von Buchstabenfolgen deutscher Wörter.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `is_terminal` | `bool` | `crates/contextra-text/src/morphology/trie.rs:8` |

### Modul `posting_list`

#### `struct Posting`
- Ort: `crates/contextra-text/src/posting_list.rs:63`
```rust
pub struct Posting
```
Dokumentation: Kompakte Repräsentation eines einzelnen Vorkommens (Posting) im Index. 8-Byte ausgerichtet (`repr(C, align(8))`) für optimale CPU-Cacheline-Ausnutzung (16 Bytes Gesamtgröße).
- Impl-Block `crates/contextra-text/src/posting_list.rs:72`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `doc_id` | `DocIdRaw` | `crates/contextra-text/src/posting_list.rs:65` |
| `tf` | `u32` | `crates/contextra-text/src/posting_list.rs:67` |
| `doc_len` | `u32` | `crates/contextra-text/src/posting_list.rs:69` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(doc_id: DocId, tf: u32, doc_len: u32) -> Self` | `crates/contextra-text/src/posting_list.rs:75` | Erzeugt einen neuen `Posting`-Eintrag. |
| `pub fn doc_id(&self) -> DocId` | `crates/contextra-text/src/posting_list.rs:85` | Liefert die `DocId` dieses Eintrags. |

#### `struct PostingBlockInfo`
- Ort: `crates/contextra-text/src/posting_list.rs:48`
```rust
pub struct PostingBlockInfo
```
Dokumentation: Metadaten eines 64er-Posting-Blocks zur Berechnung oberer BM25-Schranken in Block-Max WAND.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `max_doc_id` | `DocIdRaw` | `crates/contextra-text/src/posting_list.rs:50` |
| `max_tf` | `u32` | `crates/contextra-text/src/posting_list.rs:52` |
| `min_doc_len` | `u32` | `crates/contextra-text/src/posting_list.rs:54` |

#### `struct PostingList`
- Ort: `crates/contextra-text/src/posting_list.rs:123`
```rust
pub struct PostingList
```
Dokumentation: Zusammenhängende, nach DocId sortierte Posting-Liste für einen Begriff.
- Impl-Block `crates/contextra-text/src/posting_list.rs:128`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(mut postings: Vec<Posting>) -> Self` | `crates/contextra-text/src/posting_list.rs:131` | Erzeugt eine sortierte und deduplizierte Posting-Liste und berechnet die Block-Metadaten. |
| `pub fn empty() -> Self` | `crates/contextra-text/src/posting_list.rs:140` | Erzeugt eine leere Posting-Liste. |
| `pub fn as_slice(&self) -> &[Posting]` | `crates/contextra-text/src/posting_list.rs:150` | Liefert die Postings als Slice. |
| `pub fn blocks(&self) -> &[PostingBlockInfo]` | `crates/contextra-text/src/posting_list.rs:157` | Liefert die Block-Max-Metadaten. |
| `pub fn len(&self) -> usize` | `crates/contextra-text/src/posting_list.rs:164` | Liefert die Anzahl der Einträge in der Liste. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-text/src/posting_list.rs:171` | Prüft, ob die Liste leer ist. |
| `pub fn upsert(&self, posting: Posting) -> Self` | `crates/contextra-text/src/posting_list.rs:177` | Fügt ein Posting per Copy-On-Write (RCU) ein oder aktualisiert einen bestehenden Eintrag. |
| `pub fn remove(&self, doc_id: DocId) -> Self` | `crates/contextra-text/src/posting_list.rs:198` | Entfernt ein Posting per Copy-On-Write (RCU). |
| `pub fn encode_compact(&self) -> Result<Vec<u8>, TextError>` | `crates/contextra-text/src/posting_list.rs:212` | Kodiert die Posting-Liste in das kompakte Binärformat mit Delta-Varint-Kompression. |
| `pub fn decode_compact(bytes: &[u8]) -> Result<Self, TextError>` | `crates/contextra-text/src/posting_list.rs:238` | Dekodiert eine Posting-Liste aus dem Delta-Varint-Format mit Fallback auf Legacy-Bincode. # Defensives Verhalten Weist unvollständige, abgeschnittene, überlange |

#### `struct ResidentPostingIndex`
- Ort: `crates/contextra-text/src/posting_list.rs:357`
```rust
pub struct ResidentPostingIndex
```
Dokumentation: Speicherresidenter Lese-Cache für Posting-Listen mit RCU-Verwaltung.
- Impl-Block `crates/contextra-text/src/posting_list.rs:361`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-text/src/posting_list.rs:364` | Erzeugt einen leeren residenten Posting-Index. |
| `pub fn get(&self, term: &str) -> Option<Arc<PostingList>>` | `crates/contextra-text/src/posting_list.rs:372` | Liefert eine referenzgezählte Posting-Liste für den Begriff, falls im Cache vorhanden. |
| `pub fn insert_list(&self, term: String, list: PostingList)` | `crates/contextra-text/src/posting_list.rs:378` | Fügt eine Posting-Liste in den Index ein oder ersetzt sie. |
| `pub fn upsert_posting(&self, term: &str, posting: Posting)` | `crates/contextra-text/src/posting_list.rs:384` | Fügt ein einzelnes Posting per RCU (Copy-On-Write) in die Posting-Liste des Begriffs ein. |
| `pub fn remove_posting_from_terms(&self, terms: &[String], doc_id: DocId)` | `crates/contextra-text/src/posting_list.rs:396` | Entfernt eine Dokument-ID aus den Posting-Listen mehrerer Begriffe per RCU. |
| `pub fn remove_terms(&self, terms: &[String])` | `crates/contextra-text/src/posting_list.rs:412` | Entfernt die angegebenen Begriffe aus dem Cache (z. B. bei Rollback). |
| `pub fn clear(&self)` | `crates/contextra-text/src/posting_list.rs:421` | Leert den gesamten Cache (z. B. bei vollständigem Index-Neuladen). |

#### `type DocIdRaw`
- Ort: `crates/contextra-text/src/posting_list.rs:38`
```rust
pub type DocIdRaw = u64
```
Dokumentation: Primitiver Rohdatentyp für Dokument-IDs im Postings-Format.

#### `type DocIdRaw`
- Ort: `crates/contextra-text/src/posting_list.rs:42`
```rust
pub type DocIdRaw = u128
```
Dokumentation: Primitiver Rohdatentyp für 128-Bit Dokument-IDs im Postings-Format.
- Gate: `#[cfg(feature = "docid-128")]`

#### `const BLOCK_SIZE`
- Ort: `crates/contextra-text/src/posting_list.rs:34`
```rust
pub const BLOCK_SIZE: usize = 64
```
Dokumentation: Blockgröße für die Block-Max-WAND-Zerlegung (64 Postings pro Block).

#### `const POSTING_LIST_V2_MAGIC`
- Ort: `crates/contextra-text/src/posting_list.rs:30`
```rust
pub const POSTING_LIST_V2_MAGIC: &[u8
```
Dokumentation: Magische Bytes für das kompakte V2-Posting-Listen-Format (`PL\x02\x00`).

### Modul `storage`

#### `trait PostingStore`
- Ort: `crates/contextra-text/src/storage.rs:15`
```rust
pub trait PostingStore: Send + Sync
```
Dokumentation: Lokale Speicherabstraktion für den invertierten Index. Enthält exakt die im Kern genutzten Methoden aus `StorageEngine` (Inventar aus TX6.0).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, crate::error::Result<Option<Bytes>>>` | `crates/contextra-text/src/storage.rs:17` | Liest den Wert zu einem Schlüssel. |
| `fn get_at_seq<'a>( &'a self, key: &'a [u8], seq: u64, ) -> BoxFuture<'a, crate::error::Result<Option<Bytes>>>` | `crates/contextra-text/src/storage.rs:20` | Liest den Wert zu einem Schlüssel unter Snapshot-Isolation zur Sequenznummer `seq`. |
| `fn put<'a>( &'a self, tx: TxId, key: &'a [u8], value: &'a [u8], ) -> BoxFuture<'a, crate::error::Result<()>>` | `crates/contextra-text/src/storage.rs:27` | Schreibt einen Schlüssel/Wert-Eintrag im Kontext der Transaktion `tx`. |
| `fn delete<'a>(&'a self, tx: TxId, key: &'a [u8]) -> BoxFuture<'a, crate::error::Result<()>>` | `crates/contextra-text/src/storage.rs:35` | Löscht einen Schlüssel im Kontext der Transaktion `tx`. |
| `fn commit<'a>(&'a self, tx: TxId) -> BoxFuture<'a, crate::error::Result<()>>` | `crates/contextra-text/src/storage.rs:38` | Bestätigt die Transaktion `tx`. |
| `fn rollback<'a>(&'a self, tx: TxId) -> BoxFuture<'a, crate::error::Result<()>>` | `crates/contextra-text/src/storage.rs:41` | Rollt die Transaktion `tx` zurück. |
| `fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, crate::error::Result<()>>` | `crates/contextra-text/src/storage.rs:44` | Rollt alle Transaktionen bis einschließlich `tx_id` zurück. |
| `fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, crate::error::Result<TxId>>` | `crates/contextra-text/src/storage.rs:47` | Liefert die höchste im Speicher vergebene Transaktions-ID. |
| `fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, crate::error::Result<u64>>` | `crates/contextra-text/src/storage.rs:50` | Liefert die aktuelle MVCC-Sequenznummer. |
| `fn scan_prefix<'a>( &'a self, prefix: &'a [u8], ) -> BoxFuture<'a, crate::error::Result<Vec<(Vec<u8>, Vec<u8>)>>>` | `crates/contextra-text/src/storage.rs:53` | Scannt Schlüssel/Werte mit dem gegebenen Präfix. |
| `fn scan_prefix_at<'a>( &'a self, prefix: &'a [u8], seq: u64, ) -> BoxFuture<'a, crate::error::Result<Vec<(Vec<u8>, Vec<u8>)>>>` | `crates/contextra-text/src/storage.rs:59` | Scannt Schlüssel/Werte mit dem gegebenen Präfix unter Snapshot-Isolation. |

#### `type BoxFuture`
- Ort: `crates/contextra-text/src/storage.rs:10`
```rust
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>
```
Dokumentation: Typ-Alias für asynchrone Ergebnisse.

### Modul `stream`

#### `struct Bm25CandidateStream`
- Ort: `crates/contextra-text/src/stream.rs:31`
```rust
pub struct Bm25CandidateStream<'a, S: PostingStore>
```
Dokumentation: Asynchroner Pull-Stream zum bedarfsgerechten Nachladen von BM25-Kandidaten.
- Impl-Block `crates/contextra-text/src/stream.rs:43`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( index: &'a InvertedIndex<S>, query: impl Into<String>, max_seq: Option<u64>, ) -> Self` | `crates/contextra-text/src/stream.rs:46` | Erzeugt einen neuen Kandidaten-Stream für die übergebene Abfrage und Index-Snapshot. |
| `pub fn with_batch_size(mut self, n: usize) -> Self` | `crates/contextra-text/src/stream.rs:66` | Konfiguriert eine benutzerdefinierte Batch-Größe (mindestens 1). |
| `pub async fn next_batch(&mut self) -> Result<Vec<(DocId, f32)>>` | `crates/contextra-text/src/stream.rs:73` | Holt den nächsten Batch von Kandidaten-Dokumenten (bis zu `batch_size`). |
| `pub async fn next_doc(&mut self) -> Result<Option<DocId>>` | `crates/contextra-text/src/stream.rs:91` | Holt die nächste einzelne Dokument-ID aus dem Stream. |
| `pub fn yielded(&self) -> usize` | `crates/contextra-text/src/stream.rs:106` | Liefert die Gesamtzahl aller bisher an den Aufrufer herausgegebenen Treffer. |
| `pub fn is_exhausted(&self) -> bool` | `crates/contextra-text/src/stream.rs:112` | Prüft, ob der Stream vollständig erschöpft ist (keine gepufferten oder nachladbaren Elemente mehr). |
| `pub const fn max_depth() -> usize` | `crates/contextra-text/src/stream.rs:118` | Liefert die maximale Suchtiefe für Kandidaten ([`MAX_SEARCH_K`]). |

#### `const DEFAULT_STREAM_BATCH_SIZE`
- Ort: `crates/contextra-text/src/stream.rs:27`
```rust
pub const DEFAULT_STREAM_BATCH_SIZE: usize = 16
```
Dokumentation: Standard-Batch-Größe für das Nachladen von Kandidaten (16 Dokumente).

### Modul `tokenizer`

#### `struct AnalyzerPipeline`
- Ort: `crates/contextra-text/src/tokenizer.rs:452`
```rust
pub struct AnalyzerPipeline
```
Dokumentation: Modulare, konfigurierbare Textanalyse-Pipeline.
- Impl-Block `crates/contextra-text/src/tokenizer.rs:462`
- Impl-Block (Trait `Analyzer`) `crates/contextra-text/src/tokenizer.rs:528`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(name: impl Into<String>, version: u32) -> Self` | `crates/contextra-text/src/tokenizer.rs:464` | Erzeugt eine neue leere Analyse-Pipeline mit Standard-Grenzen. |
| `pub fn with_limits(mut self, limits: PipelineLimits) -> Self` | `crates/contextra-text/src/tokenizer.rs:477` | Konfiguriert die Ressourcen- und Sicherheitsgrenzen. |
| `pub fn with_normalization(mut self, norm: NormalizationPolicy) -> Self` | `crates/contextra-text/src/tokenizer.rs:483` | Konfiguriert die Normalisierungsrichtlinie. |
| `pub fn with_stopwords(mut self, stopwords: HashSet<String>) -> Self` | `crates/contextra-text/src/tokenizer.rs:489` | Konfiguriert die Stoppwortliste. |
| `pub fn with_filter(mut self, filter: Arc<dyn TokenFilter>) -> Self` | `crates/contextra-text/src/tokenizer.rs:495` | Fügt einen Token-Filter am Ende der Filterkette hinzu. |
| `pub fn with_german_morphology( mut self, splitter: Arc<crate::morphology::GermanCompoundSplitter>, ) -> Self` | `crates/contextra-text/src/tokenizer.rs:501` | Aktiviert die deutsche Komposita-Zerlegung. |
| `pub fn default_english() -> Self` | `crates/contextra-text/src/tokenizer.rs:510` | Erzeugt die Standard-Pipeline für englischen und generischen Text. |
| `pub fn default_german() -> Self` | `crates/contextra-text/src/tokenizer.rs:518` | Erzeugt die Standard-Pipeline für deutsche Texte inklusive Komposita-Zerlegung und Umlauten. |

#### `struct DefaultTokenizer`
- Ort: `crates/contextra-text/src/tokenizer.rs:140`
```rust
pub struct DefaultTokenizer
```
Dokumentation: Standard-Tokenizer basierend auf Unicode-Wortgrenzen und generischen Stoppwörtern.
- Impl-Block `crates/contextra-text/src/tokenizer.rs:142`
- Impl-Block (Trait `Default`) `crates/contextra-text/src/tokenizer.rs:162`
- Impl-Block (Trait `Tokenizer`) `crates/contextra-text/src/tokenizer.rs:169`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn try_new() -> Result<Self>` | `crates/contextra-text/src/tokenizer.rs:145` | Fallible Konstruktion eines `DefaultTokenizer` mit Validierung des regulären Schutzmusters. |
| `pub fn new() -> Self` | `crates/contextra-text/src/tokenizer.rs:157` |  |

#### `struct GermanMorphTokenizer`
- Ort: `crates/contextra-text/src/tokenizer.rs:201`
```rust
pub struct GermanMorphTokenizer
```
Dokumentation: Deutscher Tokenizer mit morphologischer Zerlegung von Wortzusammensetzungen.
- Impl-Block `crates/contextra-text/src/tokenizer.rs:205`
- Impl-Block (Trait `Default`) `crates/contextra-text/src/tokenizer.rs:228`
- Impl-Block (Trait `Tokenizer`) `crates/contextra-text/src/tokenizer.rs:235`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn try_new() -> Result<Self>` | `crates/contextra-text/src/tokenizer.rs:208` | Erzeugt eine neue Instanz des deutschen morphologischen Tokenizers. |
| `pub fn new() -> Self` | `crates/contextra-text/src/tokenizer.rs:221` |  |

#### `struct PipelineLimits`
- Ort: `crates/contextra-text/src/tokenizer.rs:352`
```rust
pub struct PipelineLimits
```
Dokumentation: Konfigurierbare Ressourcen- und Sicherheitsgrenzen der Analyse-Pipeline.
- Impl-Block (Trait `Default`) `crates/contextra-text/src/tokenizer.rs:363`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `max_input_length` | `usize` | `crates/contextra-text/src/tokenizer.rs:354` |
| `max_token_length` | `usize` | `crates/contextra-text/src/tokenizer.rs:356` |
| `max_tokens_per_doc` | `usize` | `crates/contextra-text/src/tokenizer.rs:358` |
| `overflow_policy` | `OverflowPolicy` | `crates/contextra-text/src/tokenizer.rs:360` |

#### `struct Token`
- Ort: `crates/contextra-text/src/tokenizer.rs:310`
```rust
pub struct Token
```
Dokumentation: Repräsentiert ein strukturiertes Token aus der Textanalyse-Pipeline. Ein Token umfasst den extrahierten Term, Byte-Offsets in den Originaltext (stets auf validen UTF-8 Zeichengrenzen) sowie das Positionsinkrement für Phrasen- und Proximity-Suchen.
- Impl-Block `crates/contextra-text/src/tokenizer.rs:321`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `term` | `String` | `crates/contextra-text/src/tokenizer.rs:312` |
| `start_offset` | `usize` | `crates/contextra-text/src/tokenizer.rs:314` |
| `end_offset` | `usize` | `crates/contextra-text/src/tokenizer.rs:316` |
| `position_increment` | `usize` | `crates/contextra-text/src/tokenizer.rs:318` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( term: impl Into<String>, start_offset: usize, end_offset: usize, position_increment: usize, ) -> Self` | `crates/contextra-text/src/tokenizer.rs:323` | Erzeugt ein neues Token mit Term, Offsets und Positionsinkrement. |

#### `enum NormalizationPolicy`
- Ort: `crates/contextra-text/src/tokenizer.rs:377`
```rust
pub enum NormalizationPolicy
```
Dokumentation: Konfigurierbare Normalisierungsrichtlinie für Textanalyse.
- Impl-Block `crates/contextra-text/src/tokenizer.rs:389`

Varianten:

- `Lowercase` — `crates/contextra-text/src/tokenizer.rs:380`
- `Nfkc` — `crates/contextra-text/src/tokenizer.rs:382`
- `GermanUmlauts` — `crates/contextra-text/src/tokenizer.rs:384`
- `Complete` — `crates/contextra-text/src/tokenizer.rs:386`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn normalize(&self, text: &str) -> String` | `crates/contextra-text/src/tokenizer.rs:391` | Führt die Normalisierung deterministisch und idempotent auf dem String aus. |

#### `enum OverflowPolicy`
- Ort: `crates/contextra-text/src/tokenizer.rs:341`
```rust
pub enum OverflowPolicy
```
Dokumentation: Strategie bei Überschreitung konfigurierter Kapazitätsgrenzen.

Varianten:

- `Truncate` — `crates/contextra-text/src/tokenizer.rs:344`
- `Error` — `crates/contextra-text/src/tokenizer.rs:346`

#### `trait Analyzer`
- Ort: `crates/contextra-text/src/tokenizer.rs:441`
```rust
pub trait Analyzer: Send + Sync
```
Dokumentation: Trait für vollständige Textanalyse-Pipelines (`Tokenizer -> Filter* -> Stemmer`). Definiert den identischen Ergebnisvertrag für Dokumenten-Indexierung und Abfrage.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn analyze(&self, text: &str) -> Result<Vec<Token>>` | `crates/contextra-text/src/tokenizer.rs:443` | Analysiert den gegebenen Text und liefert eine strukturierte Token-Sequenz. |
| `fn fingerprint(&self) -> String` | `crates/contextra-text/src/tokenizer.rs:447` | Liefert die stabile, kryptographisch eindeutige Fingerprint-Zeichenkette bestehend aus Name, Version und Konfigurations-Hash. |

#### `trait TokenFilter`
- Ort: `crates/contextra-text/src/tokenizer.rs:430`
```rust
pub trait TokenFilter: Send + Sync
```
Dokumentation: Trait für Token-Filter in einer Analyse-Pipeline.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn filter(&self, tokens: Vec<Token>) -> Vec<Token>` | `crates/contextra-text/src/tokenizer.rs:432` | Wendet den Filter auf eine Sequenz von Tokens an. |
| `fn filter_id(&self) -> &str` | `crates/contextra-text/src/tokenizer.rs:434` | Eindeutiger Bezeichner des Filters für die Fingerprint-Generierung. |

#### `trait Tokenizer`
- Ort: `crates/contextra-text/src/tokenizer.rs:133`
```rust
pub trait Tokenizer: Send + Sync
```
Dokumentation: Trait für Text-Tokenizer mit unterschiedlichen Filter- oder Zerlegungsstrategien.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn tokenize(&self, text: &str) -> Vec<String>` | `crates/contextra-text/src/tokenizer.rs:135` | Zerlegt den übergebenen Text in kleingeschriebene Wort-Token und filtert Stoppwörter heraus. |

#### `fn compile_protected_regex`
- Ort: `crates/contextra-text/src/tokenizer.rs:34`
```rust
pub fn compile_protected_regex(pattern: &str) -> Result<Regex>
```
Dokumentation: Kompiliert ein reguläres Muster für geschützte Token mit Fail-Closed-Fehlerbehandlung. # Fehler Gibt [`TextError::Text`] zurück, falls der reguläre Ausdruck syntaktisch ungültig ist.

#### `fn get_protected_regex`
- Ort: `crates/contextra-text/src/tokenizer.rs:41`
```rust
pub fn get_protected_regex() -> Result<&'static Regex>
```
Dokumentation: Liefert den gecachten regulären Standardausdruck für geschützte URLs und E-Mail-Adressen.

#### `fn tokenize`
- Ort: `crates/contextra-text/src/tokenizer.rs:299`
```rust
pub fn tokenize(text: &str) -> Vec<String>
```
Dokumentation: Hilfsfunktion zur Tokenisierung mit dem [`DefaultTokenizer`].

### Modul `traits`

#### `trait Index`
- Ort: `crates/contextra-text/src/traits.rs:6`
```rust
pub trait Index<Id, Query: ?Sized>
```
Dokumentation: Kern-Index-Schnittstelle (Klasse A / SPEC §4.3).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn insert(&mut self, id: Id, payload: &[u8]) -> Result<()>` | `crates/contextra-text/src/traits.rs:11` | Fügt ein Dokument mit gegebener ID und Byte-Nutzlast in den Index ein. |
| `fn remove(&mut self, id: &Id) -> Result<bool>` | `crates/contextra-text/src/traits.rs:16` | Entfernt ein Dokument anhand seiner ID aus dem Index. Gibt `true` zurück, wenn das Dokument vorhanden war und gelöscht wurde. |
| `fn search(&self, query: &Query, limit: usize) -> Result<Vec<Self::Hit>>` | `crates/contextra-text/src/traits.rs:19` | Führt eine Suche anhand der Query aus und liefert bis zu `limit` Treffer zurück. |

Assoziierte Konstanten/Typen:

- `type Hit` — `crates/contextra-text/src/traits.rs:8`

#### `trait Maintain`
- Ort: `crates/contextra-text/src/traits.rs:23`
```rust
pub trait Maintain
```
Dokumentation: Neutrale Wartungsschnittstelle (Klasse A / SPEC §4.3).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn compact(&mut self) -> Result<Self::Stats>` | `crates/contextra-text/src/traits.rs:28` | Führt eine Kompaktierung durch und gibt Segmente bzw. gelöschte Einträge tatsächlich frei. |
| `fn vacuum(&mut self) -> Result<()>` | `crates/contextra-text/src/traits.rs:31` | Bereinigt fragmentierte interne Strukturen. |

Assoziierte Konstanten/Typen:

- `type Stats` — `crates/contextra-text/src/traits.rs:25`

### Modul `types`

#### `struct DocId`
- Ort: `crates/contextra-text/src/types.rs:9`
```rust
pub struct DocId(pub u64)
```
Dokumentation: Standard 64-Bit Dokument-Identifikator.
- Impl-Block (Trait `From<contextra_types::DocId>`) `crates/contextra-text/src/compat/types.rs:6`
- Impl-Block (Trait `From<DocId>`) `crates/contextra-text/src/compat/types.rs:20`
- Impl-Block `crates/contextra-text/src/types.rs:17`
- Impl-Block (Trait `From<u64>`) `crates/contextra-text/src/types.rs:55`
- Impl-Block (Trait `From<u128>`) `crates/contextra-text/src/types.rs:63`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub const fn new(id: u64) -> Self` | `crates/contextra-text/src/types.rs:21` | Erzeugt eine neue `DocId`. |
| `pub const fn new(id: u128) -> Self` | `crates/contextra-text/src/types.rs:28` | Erzeugt eine neue `DocId` (128-Bit). |
| `pub const fn inner(self) -> u64` | `crates/contextra-text/src/types.rs:35` | Liefert den primitiven Identifikator. |
| `pub const fn inner(self) -> u128` | `crates/contextra-text/src/types.rs:42` | Liefert den primitiven Identifikator (128-Bit). |
| `pub const fn as_u64(self) -> u64` | `crates/contextra-text/src/types.rs:49` | Kompatibilitäts-Alias für `inner()`. |

#### `struct DocId`
- Ort: `crates/contextra-text/src/types.rs:15`
```rust
pub struct DocId(pub u128)
```
Dokumentation: 128-Bit Dokument-Identifikator bei aktiviertem `docid-128` Feature.
- Gate: `#[cfg(feature = "docid-128")]`

#### `struct TextHit`
- Ort: `crates/contextra-text/src/types.rs:72`
```rust
pub struct TextHit
```
Dokumentation: Ergebnisdokument mit Relevanz-Score für BM25-Suchen.
- Impl-Block (Trait `From<contextra_types::ScoredDocument>`) `crates/contextra-text/src/compat/types.rs:43`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `doc_id` | `DocId` | `crates/contextra-text/src/types.rs:74` |
| `score` | `f32` | `crates/contextra-text/src/types.rs:76` |

#### `const MAX_SEARCH_K`
- Ort: `crates/contextra-text/src/types.rs:80`
```rust
pub const MAX_SEARCH_K: usize = 10_000
```
Dokumentation: Maximale Anzahl von Suchtreffern pro Anfrage (SPEC §11.3).

### Modul `wand`

#### `struct BoundedTopK`
- Ort: `crates/contextra-text/src/wand.rs:50`
```rust
pub struct BoundedTopK
```
Dokumentation: Begrenzter Min-Heap zur effizienten Verwaltung der K besten Treffer.
- Impl-Block `crates/contextra-text/src/wand.rs:55`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(capacity: usize) -> Self` | `crates/contextra-text/src/wand.rs:58` | Erzeugt einen neuen `BoundedTopK`-Puffer mit maximaler Kapazität (`MAX_SEARCH_K`). |
| `pub fn push(&mut self, doc_id: DocId, score: f32)` | `crates/contextra-text/src/wand.rs:68` | Fügt einen neuen Kandidaten ein, wenn er besser als der schlechteste bisherige Treffer ist. |
| `pub fn min_threshold(&self) -> f32` | `crates/contextra-text/src/wand.rs:85` | Liefert den aktuellen Minimal-Score (Schwelle), den ein Kandidat übertreffen muss. |
| `pub fn into_sorted_vec(self) -> Vec<(DocId, f32)>` | `crates/contextra-text/src/wand.rs:95` | Konvertiert den Heap in einen nach Relevanz absteigend sortierten Ergebnisvektor. |

#### `struct WandSearchResult`
- Ort: `crates/contextra-text/src/wand.rs:228`
```rust
pub struct WandSearchResult
```
Dokumentation: Ergebnis einer Block-Max-WAND-Suche inklusive Top-K-Treffern und Metadaten über evaluierte Dokumente.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `evaluated_docs_count` | `usize` | `crates/contextra-text/src/wand.rs:232` |

#### `fn block_max_wand_search`
- Ort: `crates/contextra-text/src/wand.rs:238`
```rust
pub async fn block_max_wand_search<S: PostingStore>( storage: &S, terms: &[String], resident_index: &crate::posting_list::ResidentPostingIndex, prefix_helper: impl Fn(&str) -> Vec<u8>, tombstone_key_helper: impl Fn(DocId, &str) -> Vec<u8>, doc_len_key_helper: impl Fn(u128) -> Vec<u8>, k: usize, max_seq: Option<u64>, total_docs: u64, avg_doc_len: f32, ) -> Result<WandSearchResult>
```
Dokumentation: Führt eine dynamisch geprunte Block-Max WAND Volltextsuche über den gegebenen Termen aus.

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl From<TextError> for ContextraError` — `crates/contextra-text/src/compat/error.rs:7` (0 pub Methoden)
- `impl<T: StorageEngine + ?Sized> PostingStore for T` — `crates/contextra-text/src/compat/storage.rs:126` (0 pub Methoden)
- `impl From<TextHit> for contextra_types::ScoredDocument` — `crates/contextra-text/src/compat/types.rs:34` (0 pub Methoden)
- `impl<T: contextra_env::Clock> Clock for T` — `crates/contextra-text/src/contract.rs:38` (0 pub Methoden)

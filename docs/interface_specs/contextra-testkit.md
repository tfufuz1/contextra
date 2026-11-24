# Schnittstellenspezifikation: `contextra-testkit`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-testkit` |
| Version | workspace |
| Pfad | `crates/contextra-testkit` |
| Rolle | Testhilfen |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> # Crate `contextra-testkit`
>
> Deterministische Test-Utilities, Mocks, In-Memory-Referenzimplementierungen
> und Konformitätsprüfsuiten für das Contextra-System.
>
> ## Ring & Architektur-Einordnung
> - **Ring:** Werkzeug / Tooling (außerhalb der numerischen Ringe 0 bis 4).
> - **Klasse:** Werkzeug (`—`).
> - **Sicherheits-Invariante (INV-EMBED-1):** `contextra-testkit` darf ausschließlich
> als `[dev-dependencies]` in Produktions-Crates eingebunden werden und niemals
> in Release-Binaries oder produktiven Abhängigkeiten landen.
> - **Unsafe-Code:** Ausdrücklich verboten über `#![forbid(unsafe_code)]`.

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `bytes` | normal | workspace | nein |  |
| `contextra-ports` | normal | workspace | nein | contextra, contextra-adapt, contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-rank, contextra-router, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vector, contextra-vector-fuzz |
| `contextra-types` | normal | workspace | nein | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `criterion` | dev | workspace | nein |  |
| `parking_lot` | normal | workspace | nein |  |
| `tempfile` | dev | workspace | nein |  |
| `tokio` | dev | workspace | nein |  |
| `tracing` | normal | workspace | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-agent`, `contextra-checkpoint`, `contextra-cognition`, `contextra-crypto-fuzz`, `contextra-db`, `contextra-engine`, `contextra-store`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `default` | `testkit`, `erasure-conformance` |
| `erasure-conformance` | – |
| `testkit` | – |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 22 |
| Quellzeilen | 5446 |
| Öffentliche Items (Modul-Ebene) | 71 |
| davon `fn` | 25 |
| davon `struct` | 19 |
| davon `enum` | 2 |
| davon `type` | 1 |
| davon `mod` | 24 |
| Öffentliche Makros (`#[macro_export]`) | 1 |
| Öffentliche Re-Exports (`pub use`) | 27 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 0 |
| `.expect(` (Produktionscode) | 0 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 0 |
| `#[deprecated]` | 0 |
| `#[cfg(feature…)]`-Gates | 1 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 128 / 128 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `conformance::clock` | `crates/contextra-testkit/src/conformance/clock.rs` | 47 |
| `conformance::failure` | `crates/contextra-testkit/src/conformance/failure.rs` | 61 |
| `conformance::graph` | `crates/contextra-testkit/src/conformance/graph.rs` | 181 |
| `conformance::kv_ports` | `crates/contextra-testkit/src/conformance/kv_ports.rs` | 495 |
| `conformance::macros` | `crates/contextra-testkit/src/conformance/macros.rs` | 47 |
| `conformance::metrics` | `crates/contextra-testkit/src/conformance/metrics.rs` | 41 |
| `conformance` | `crates/contextra-testkit/src/conformance/mod.rs` | 103 |
| `conformance::rng` | `crates/contextra-testkit/src/conformance/rng.rs` | 65 |
| `conformance::routing_ports` | `crates/contextra-testkit/src/conformance/routing_ports.rs` | 476 |
| `conformance::storage` | `crates/contextra-testkit/src/conformance/storage.rs` | 409 |
| `conformance::text` | `crates/contextra-testkit/src/conformance/text.rs` | 178 |
| `conformance::vector` | `crates/contextra-testkit/src/conformance/vector.rs` | 299 |
| `conformance::vfs` | `crates/contextra-testkit/src/conformance/vfs.rs` | 201 |
| `fault_vfs` | `crates/contextra-testkit/src/fault_vfs.rs` | 457 |
| `in_memory_graph` | `crates/contextra-testkit/src/in_memory_graph.rs` | 251 |
| `in_memory_store` | `crates/contextra-testkit/src/in_memory_store.rs` | 408 |
| `in_memory_text` | `crates/contextra-testkit/src/in_memory_text.rs` | 206 |
| `in_memory_vector` | `crates/contextra-testkit/src/in_memory_vector.rs` | 231 |
| `crate` | `crates/contextra-testkit/src/lib.rs` | 145 |
| `manual_clock` | `crates/contextra-testkit/src/manual_clock.rs` | 172 |
| `mocks` | `crates/contextra-testkit/src/mocks.rs` | 690 |
| `reference_model` | `crates/contextra-testkit/src/reference_model.rs` | 283 |

## 6. Re-Exports (`pub use`)

- `pub use clock::run_clock_suite` — `crates/contextra-testkit/src/conformance/mod.rs:83` (Modul `conformance`)
- `pub use failure::` — `crates/contextra-testkit/src/conformance/mod.rs:84` (Modul `conformance`)
- `pub use graph::run_graph_index_suite` — `crates/contextra-testkit/src/conformance/mod.rs:85` (Modul `conformance`)
- `pub use metrics::run_metrics_sink_suite` — `crates/contextra-testkit/src/conformance/mod.rs:86` (Modul `conformance`)
- `pub use rng::run_rng_suite` — `crates/contextra-testkit/src/conformance/mod.rs:87` (Modul `conformance`)
- `pub use storage::run_storage_engine_suite` — `crates/contextra-testkit/src/conformance/mod.rs:88` (Modul `conformance`)
- `pub use text::run_text_index_suite` — `crates/contextra-testkit/src/conformance/mod.rs:89` (Modul `conformance`)
- `pub use vector::run_vector_index_suite` — `crates/contextra-testkit/src/conformance/mod.rs:90` (Modul `conformance`)
- `pub use vfs::run_vfs_suite` — `crates/contextra-testkit/src/conformance/mod.rs:91` (Modul `conformance`)
- `pub use kv_ports::` — `crates/contextra-testkit/src/conformance/mod.rs:92` (Modul `conformance`)
- `pub use routing_ports::` — `crates/contextra-testkit/src/conformance/mod.rs:97` (Modul `conformance`)
- `pub use crate::in_memory_graph::InMemoryGraphIndex` — `crates/contextra-testkit/src/lib.rs:76` (Modul `doubles`)
- `pub use crate::in_memory_store::InMemoryStorageEngine` — `crates/contextra-testkit/src/lib.rs:77` (Modul `doubles`)
- `pub use crate::in_memory_text::InMemoryTextIndex` — `crates/contextra-testkit/src/lib.rs:78` (Modul `doubles`)
- `pub use crate::in_memory_vector::InMemoryVectorIndex` — `crates/contextra-testkit/src/lib.rs:79` (Modul `doubles`)
- `pub use crate::manual_clock::ManualClock` — `crates/contextra-testkit/src/lib.rs:80` (Modul `doubles`)
- `pub use crate::mocks::` — `crates/contextra-testkit/src/lib.rs:81` (Modul `doubles`)
- `pub use crate::fault_vfs::` — `crates/contextra-testkit/src/lib.rs:90` (Modul `fault`)
- `pub use crate::reference_model::` — `crates/contextra-testkit/src/lib.rs:95` (Modul `model`)
- `pub use fault_vfs::` — `crates/contextra-testkit/src/lib.rs:99` (Modul `crate`)
- `pub use in_memory_graph::InMemoryGraphIndex` — `crates/contextra-testkit/src/lib.rs:100` (Modul `crate`)
- `pub use in_memory_store::InMemoryStorageEngine` — `crates/contextra-testkit/src/lib.rs:101` (Modul `crate`)
- `pub use in_memory_text::InMemoryTextIndex` — `crates/contextra-testkit/src/lib.rs:102` (Modul `crate`)
- `pub use in_memory_vector::InMemoryVectorIndex` — `crates/contextra-testkit/src/lib.rs:103` (Modul `crate`)
- `pub use manual_clock::ManualClock` — `crates/contextra-testkit/src/lib.rs:104` (Modul `crate`)
- `pub use mocks::*` — `crates/contextra-testkit/src/lib.rs:105` (Modul `crate`)
- `pub use reference_model::` — `crates/contextra-testkit/src/lib.rs:106` (Modul `crate`)

## 7. Öffentliche Makros

- `macro_rules! assert_conformance` — `crates/contextra-testkit/src/conformance/macros.rs:16`
  - Überprüft, ob ein Konformitätsprüfungsergebnis erfolgreich war.
  - **Verhalten:**

## 8. Öffentliche Schnittstelle je Item

### Modul `conformance`

#### `mod clock`
- Ort: `crates/contextra-testkit/src/conformance/mod.rs:37`
```rust
pub mod clock
```
Dokumentation: Konformitätssuite für den `Clock`-Port.

#### `mod failure`
- Ort: `crates/contextra-testkit/src/conformance/mod.rs:41`
```rust
pub mod failure
```
Dokumentation: Typen zur detaillierten Erfassung von Konformitätsfehlschlägen.

#### `mod graph`
- Ort: `crates/contextra-testkit/src/conformance/mod.rs:45`
```rust
pub mod graph
```
Dokumentation: Konformitätssuite für den `GraphIndex`-Port.

#### `mod kv_ports`
- Ort: `crates/contextra-testkit/src/conformance/mod.rs:76`
```rust
pub mod kv_ports
```
Dokumentation: Konformitaetssuiten fuer KV-Ports.

#### `mod macros`
- Ort: `crates/contextra-testkit/src/conformance/mod.rs:49`
```rust
pub mod macros
```
Dokumentation: Hilfsmakros für Konformitätsprüfungen (`assert_conformance!`).

#### `mod metrics`
- Ort: `crates/contextra-testkit/src/conformance/mod.rs:53`
```rust
pub mod metrics
```
Dokumentation: Konformitätssuite für den `MetricsSink`-Port.

#### `mod rng`
- Ort: `crates/contextra-testkit/src/conformance/mod.rs:57`
```rust
pub mod rng
```
Dokumentation: Konformitätssuite für den Zufallszahlen-Port (`Rng`).

#### `mod routing_ports`
- Ort: `crates/contextra-testkit/src/conformance/mod.rs:80`
```rust
pub mod routing_ports
```
Dokumentation: Konformitaetssuiten fuer Routing-Ports.

#### `mod storage`
- Ort: `crates/contextra-testkit/src/conformance/mod.rs:61`
```rust
pub mod storage
```
Dokumentation: Konformitätssuite für den `StorageEngine`-Port.

#### `mod text`
- Ort: `crates/contextra-testkit/src/conformance/mod.rs:65`
```rust
pub mod text
```
Dokumentation: Konformitätssuite für den `TextIndex`-Port.

#### `mod vector`
- Ort: `crates/contextra-testkit/src/conformance/mod.rs:69`
```rust
pub mod vector
```
Dokumentation: Konformitätssuite für den `VectorIndex`-Port.

#### `mod vfs`
- Ort: `crates/contextra-testkit/src/conformance/mod.rs:72`
```rust
pub mod vfs
```
Dokumentation: Konformitätssuite für den `Vfs`-Port.

### Modul `conformance::clock`

#### `fn run_clock_suite`
- Ort: `crates/contextra-testkit/src/conformance/clock.rs:24`
```rust
pub fn run_clock_suite<C: Clock>(clock: &C) -> ConformanceResult
```
Dokumentation: Führt die vollständige Konformitätsprüfung für eine [`Clock`]-Implementierung durch. **Ablauf:** 1. Ruft zweimal hintereinander `monotonic_nanos()` ab. 2. Prüft, dass der zweite Wert größer oder gleich dem ersten ist. 3. Fragt `now_unix_nanos()` ab und stellt sicher, dass der Wert ungleich 0 ist. **Rückgabe:** - `Ok(())` bei Vertragskonformität. - `Err(ConformanceFailure)` mit Testfall und Fehlerursache bei Verletzung.

### Modul `conformance::failure`

#### `struct ConformanceFailure`
- Ort: `crates/contextra-testkit/src/conformance/failure.rs:20`
```rust
pub struct ConformanceFailure
```
Dokumentation: Repräsentiert einen aufgetretenen Fehler während einer Konformitätsprüfung. **Zweck:** Enthält den Suitenamen, den konkreten Testfall und eine menschenlesbare Beschreibung der Vertragsverletzung, um Junior-Entwicklern bei der Behebung von Adapter-Bugs sofort Klarheit zu verschaffen.
- Impl-Block `crates/contextra-testkit/src/conformance/failure.rs:29`
- Impl-Block (Trait `fmt::Display`) `crates/contextra-testkit/src/conformance/failure.rs:46`
- Impl-Block (Trait `std::error::Error`) `crates/contextra-testkit/src/conformance/failure.rs:57`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `suite_name` | `&'static str` | `crates/contextra-testkit/src/conformance/failure.rs:22` |
| `test_case` | `&'static str` | `crates/contextra-testkit/src/conformance/failure.rs:24` |
| `details` | `String` | `crates/contextra-testkit/src/conformance/failure.rs:26` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( suite_name: &'static str, test_case: &'static str, details: impl Into<String>, ) -> Self` | `crates/contextra-testkit/src/conformance/failure.rs:32` | Erstellt eine neue [`ConformanceFailure`]-Instanz. |

#### `type ConformanceResult`
- Ort: `crates/contextra-testkit/src/conformance/failure.rs:61`
```rust
pub type ConformanceResult = Result<(), ConformanceFailure>
```
Dokumentation: Spezialisierter Result-Typ für Konformitätsläufe: `Ok(())` oder `Err(ConformanceFailure)`.

### Modul `conformance::graph`

#### `fn run_graph_index_suite`
- Ort: `crates/contextra-testkit/src/conformance/graph.rs:30`
```rust
pub async fn run_graph_index_suite<F, Fut, G>(factory: F) -> ConformanceResult where F: Fn() -> Fut, Fut: Future<Output = G>, G: GraphIndex,
```
Dokumentation: Führt die vollständige Konformitätsprüfung für eine [`GraphIndex`]-Implementierung durch. **Parameter:** - `factory`: Asynchrone Closure, die eine frische Graphindex-Instanz erzeugt. **Ablauf:** 1. Test 1: Entitäten anlegen, Kante hinzufügen und Nachbarschaft abfragen. 2. Test 2: Kante löschen und prüfen, dass sie nicht mehr als Nachbar erscheint. 3. Test 3: Pfad über zwei Kanten spannen und 2-Hop-Traversierung prüfen. 4. Test 4: Selbstkante anlegen und Nachbarschaftsfunktion auf Robustheit prüfen. **Rückgabe:** `Ok(())` bei Vertragskonformität, andernfalls `Err(ConformanceFailure)`.

### Modul `conformance::kv_ports`

#### `fn assert_attention_exporter_conformance`
- Ort: `crates/contextra-testkit/src/conformance/kv_ports.rs:487`
```rust
pub fn assert_attention_exporter_conformance<E: AttentionExporter>( exporter: &E, known_request: RequestId, )
```
Dokumentation: Führt die Konformitätsprüfung für einen [`AttentionExporter`] durch und löst bei Fehlern eine informative Panic aus.

#### `fn assert_kv_bridge_storage_conformance`
- Ort: `crates/contextra-testkit/src/conformance/kv_ports.rs:472`
```rust
pub async fn assert_kv_bridge_storage_conformance<F, Fut, S>(factory: F) where F: Fn() -> Fut, Fut: Future<Output = S>, S: KvBridgeStorage,
```
Dokumentation: Führt die Konformitätsprüfung für einen [`KvBridgeStorage`] durch und löst bei Fehlern eine informative Panic aus.

#### `fn assert_kv_prefix_store_conformance`
- Ort: `crates/contextra-testkit/src/conformance/kv_ports.rs:463`
```rust
pub fn assert_kv_prefix_store_conformance<S: KvPrefixStore>(store: &S)
```
Dokumentation: Führt die Konformitätsprüfung für einen [`KvPrefixStore`] durch und löst bei Fehlern eine informative Panic aus.

#### `fn run_attention_exporter_suite`
- Ort: `crates/contextra-testkit/src/conformance/kv_ports.rs:405`
```rust
pub fn run_attention_exporter_suite<E: AttentionExporter>( exporter: &E, known_request: RequestId, ) -> ConformanceResult
```
Dokumentation: Prüft die Konformität einer [`AttentionExporter`]-Implementierung gegen ihren dokumentierten Vertrag. **Geprüfte Verträge:** 1. `unknown_request_none`: Unbekannte RequestIds liefern `None`. 2. `known_request_weights`: Bekannte RequestIds liefern einen Vektor von Aufmerksamkeitsgewichten. 3. `deterministic_repeated_export`: Wiederholte Aufrufe für dieselbe RequestId liefern identische Vektoren gleicher Länge.

#### `fn run_kv_bridge_storage_suite`
- Ort: `crates/contextra-testkit/src/conformance/kv_ports.rs:227`
```rust
pub async fn run_kv_bridge_storage_suite<F, Fut, S>(factory: F) -> ConformanceResult where F: Fn() -> Fut, Fut: Future<Output = S>, S: KvBridgeStorage,
```
Dokumentation: Prüft die Konformität einer [`KvBridgeStorage`]-Implementierung gegen ihren dokumentierten Vertrag. **Geprüfte Verträge:** 1. `get_empty_key`: Unbekannte Schlüssel liefern `Ok(None)`. 2. `uncommitted_put_invisible`: Ungespeicherte / uncommittete Schreibvorgänge sind vor dem Commit nicht sichtbar. 3. `commit_makes_visible`: Nach erfolgreichem `commit` ist der Wert lesbar. 4. `multi_tx_isolation`: Mehrere Transaktionen schreiben isoliert voneinander. 5. `overwrite_key`: Überschreiben eines Schlüssels aktualisiert den Wert nach dem Commit.

#### `fn run_kv_prefix_store_suite`
- Ort: `crates/contextra-testkit/src/conformance/kv_ports.rs:63`
```rust
pub fn run_kv_prefix_store_suite<S: KvPrefixStore>(store: &S) -> ConformanceResult
```
Dokumentation: Prüft die Konformität einer [`KvPrefixStore`]-Implementierung gegen ihren dokumentierten Vertrag. **Geprüfte Verträge:** 1. `roundtrip_and_exact_match`: Gespeicherte Blöcke werden für passende Präfixe vollständig zurückgegeben. 2. `isolation_tenant_and_key`: Kein Datenübertrag zwischen unterschiedlichen Mandanten oder Modellen. 3. `overwrite_updates_data`: Erneutes Einfügen unter demselben Key überschreibt alte Blöcke. 4. `evict_clears_data`: `evict` entfernt vorhandene Daten idempotent und liefert die Anzahl entfernter Blöcke. 5. `deterministic_repeated_lookup`: Wiederholte Abfragen liefern i

### Modul `conformance::metrics`

#### `fn run_metrics_sink_suite`
- Ort: `crates/contextra-testkit/src/conformance/metrics.rs:22`
```rust
pub fn run_metrics_sink_suite<M: MetricsSink>(sink: &M) -> ConformanceResult
```
Dokumentation: Führt die vollständige Konformitätsprüfung für eine [`MetricsSink`]-Implementierung durch. **Ablauf:** 1. Ruft in einem geschützten Block (`catch_unwind`) Zähler-, Pegel- und Histogramm-Aufzeichnungen auf. 2. Fängt etwaige Panics ab und meldet diese strukturiert als [`ConformanceFailure`]. **Rückgabe:** `Ok(())` bei Erfolg, `Err(ConformanceFailure)` bei Panic.

### Modul `conformance::rng`

#### `fn run_rng_suite`
- Ort: `crates/contextra-testkit/src/conformance/rng.rs:26`
```rust
pub fn run_rng_suite<R: Rng>(rng: &R) -> ConformanceResult
```
Dokumentation: Führt die vollständige Konformitätsprüfung für einen [`Rng`]-Port durch. **Ablauf:** 1. Zieht 3x `next_u64()` und verifiziert Varianz. 2. Füllt einen 16-Byte-Puffer und prüft auf Nicht-Null-Inhalt. 3. Zieht 100x `next_unit_f64()` und prüft die Grenzen `0.0 <= val < 1.0`. **Rückgabe:** `Ok(())` bei Erfolg, andernfalls `Err(ConformanceFailure)`.

### Modul `conformance::routing_ports`

#### `fn assert_community_resolver_conformance`
- Ort: `crates/contextra-testkit/src/conformance/routing_ports.rs:434`
```rust
pub async fn assert_community_resolver_conformance<R: CommunityResolver>( resolver: &R, known_entity: EntityId, )
```
Dokumentation: Führt die Konformitätsprüfung für einen [`CommunityResolver`] durch und löst bei Fehlern eine informative Panic aus.

#### `fn assert_context_preparer_conformance`
- Ort: `crates/contextra-testkit/src/conformance/routing_ports.rs:447`
```rust
pub fn assert_context_preparer_conformance<P: ContextPreparer>(preparer: &P)
```
Dokumentation: Führt die Konformitätsprüfung für einen [`ContextPreparer`] durch und löst bei Fehlern eine informative Panic aus.

#### `fn assert_drift_status_provider_conformance`
- Ort: `crates/contextra-testkit/src/conformance/routing_ports.rs:457`
```rust
pub fn assert_drift_status_provider_conformance<D: DriftStatusProvider>(provider: &D)
```
Dokumentation: Führt die Konformitätsprüfung für einen [`DriftStatusProvider`] durch und löst bei Fehlern eine informative Panic aus.

#### `fn assert_egress_policy_conformance`
- Ort: `crates/contextra-testkit/src/conformance/routing_ports.rs:467`
```rust
pub fn assert_egress_policy_conformance<P: EgressPolicy>( policy: &P, ctx: &RequestContext, sample_payloads: &[&str], )
```
Dokumentation: Führt die Konformitätsprüfung für eine [`EgressPolicy`] durch und löst bei Fehlern eine informative Panic aus.

#### `fn assert_hybrid_search_provider_conformance`
- Ort: `crates/contextra-testkit/src/conformance/routing_ports.rs:425`
```rust
pub async fn assert_hybrid_search_provider_conformance<P: HybridSearchProvider>(provider: &P)
```
Dokumentation: Führt die Konformitätsprüfung für einen [`HybridSearchProvider`] durch und löst bei Fehlern eine informative Panic aus.

#### `fn run_community_resolver_suite`
- Ort: `crates/contextra-testkit/src/conformance/routing_ports.rs:163`
```rust
pub async fn run_community_resolver_suite<R: CommunityResolver>( resolver: &R, known_entity: EntityId, ) -> ConformanceResult
```
Dokumentation: Prüft die Konformität einer [`CommunityResolver`]-Implementierung gegen ihren dokumentierten Vertrag. **Geprüfte Verträge:** 1. `unknown_entity_none`: Unbekannte Entity liefert `Ok(None)`. 2. `known_entity_some`: Bekannte Entity liefert `Ok(Some(community_id))`. 3. `stable_repeated_resolution`: Wiederholte Aufrufe für dieselbe Entity liefern identische Ergebnisse.

#### `fn run_context_preparer_suite`
- Ort: `crates/contextra-testkit/src/conformance/routing_ports.rs:220`
```rust
pub fn run_context_preparer_suite<P: ContextPreparer>(preparer: &P) -> ConformanceResult
```
Dokumentation: Prüft die Konformität einer [`ContextPreparer`]-Implementierung gegen ihren dokumentierten Vertrag. **Geprüfte Verträge:** 1. `empty_input_empty_window`: Leere Eingabe liefert ein leeres `ContextWindow`. 2. `relevance_threshold_filtering`: Chunks mit Relevanz unterhalb des Schwellenwerts werden gefiltert. 3. `budget_never_exceeded`: Die Gesamtanzahl der Tokens überschreitet niemals das verfügbare Budget.

#### `fn run_drift_status_provider_suite`
- Ort: `crates/contextra-testkit/src/conformance/routing_ports.rs:344`
```rust
pub fn run_drift_status_provider_suite<D: DriftStatusProvider>(provider: &D) -> ConformanceResult
```
Dokumentation: Prüft die Konformität einer [`DriftStatusProvider`]-Implementierung gegen ihren dokumentierten Vertrag. **Geprüfter Vertrag:** Die Rückgabe von `overall_drift_status()` muss strikt einer der vier dokumentierten Statusstrings sein: `"stabil"`, `"warnung"`, `"kritisch"`, `"unbekannt"`.

#### `fn run_egress_policy_suite`
- Ort: `crates/contextra-testkit/src/conformance/routing_ports.rs:370`
```rust
pub fn run_egress_policy_suite<P: EgressPolicy>( policy: &P, ctx: &RequestContext, sample_payloads: &[&str], ) -> ConformanceResult
```
Dokumentation: Prüft die Konformität einer [`EgressPolicy`]-Implementierung gegen ihren dokumentierten Vertrag. **Geprüfte Verträge:** 1. `valid_decision_variants`: Jede Auswertung liefert eine gültige `EgressDecision` (`Allow`, `Redact`, `Deny`). 2. `non_empty_reasons_on_deny_and_redact`: Bei `Redact` und `Deny` darf der Begründungsstring nicht leer oder reines Whitespace sein. 3. `deterministic_evaluation`: Gleicher Kontext und Payload liefern deterministisch die identische Entscheidung.

#### `fn run_hybrid_search_provider_suite`
- Ort: `crates/contextra-testkit/src/conformance/routing_ports.rs:36`
```rust
pub async fn run_hybrid_search_provider_suite<P: HybridSearchProvider>( provider: &P, ) -> ConformanceResult
```
Dokumentation: Prüft die Konformität einer [`HybridSearchProvider`]-Implementierung gegen ihren dokumentierten Vertrag. **Geprüfte Verträge:** 1. `zero_top_k_empty`: `top_k == 0` liefert eine leere Liste zurück. 2. `top_k_respected`: Die Anzahl der gelieferten Chunks überschreitet niemals `top_k`. 3. `deterministic_tie_breaking`: Bei gleichem Relevanzscore wird deterministisch nach `DocId` aufsteigend sortiert (INV-5). 4. `empty_query_no_panic`: Leere Query-Strings lösen keine Panik oder unzulässigen Zustand aus.

### Modul `conformance::storage`

#### `fn run_storage_engine_suite`
- Ort: `crates/contextra-testkit/src/conformance/storage.rs:38`
```rust
pub async fn run_storage_engine_suite<F, Fut, S>(factory: F) -> ConformanceResult where F: Fn() -> Fut, Fut: Future<Output = S>, S: StorageEngine,
```
Dokumentation: Führt die vollständige Konformitätsprüfung für eine [`StorageEngine`]-Implementierung durch. **Parameter:** - `factory`: Eine Closure, die für jeden isolierten Testfall eine frische Storage-Instanz asynchron erzeugt. **Ablauf:** 1. Test 1: Basis-Lebenszyklus (`put` -> `commit` -> `get` -> `delete` -> `commit` -> `get`). 2. Test 2: Überschreiben bestehender Schlüssel. 3. Test 3: Lexikografische Sortierordnung bei Präfix-Scans. 4. Test 4: Isolation von Mandanten-Präfixen (`t1:` vs `t2:`). 5. Test 5: Speichern und Lesen von 1-MB-Großwerten. 6. Test 6: Grenzfälle leerer Schlüssel und leerer Nutzda

### Modul `conformance::text`

#### `fn run_text_index_suite`
- Ort: `crates/contextra-testkit/src/conformance/text.rs:30`
```rust
pub async fn run_text_index_suite<F, Fut, T>(factory: F) -> ConformanceResult where F: Fn() -> Fut, Fut: Future<Output = T>, T: TextIndex,
```
Dokumentation: Führt die vollständige Konformitätsprüfung für eine [`TextIndex`]-Implementierung durch. **Parameter:** - `factory`: Asynchrone Closure, die eine frische Textindex-Instanz erzeugt. **Ablauf:** 1. Test 1: Dokument anlegen, committen und per Begriff suchen. 2. Test 2: Dokument löschen und Unauffindbarkeit verifizieren. 3. Test 3: Normalisierung deutscher Umlaute und `ß` prüfen. 4. Test 4: Verhalten bei leerer Suchanfrage (`""`) testen. **Rückgabe:** `Ok(())` bei Vertragskonformität, andernfalls `Err(ConformanceFailure)`.

### Modul `conformance::vector`

#### `fn run_vector_index_suite`
- Ort: `crates/contextra-testkit/src/conformance/vector.rs:36`
```rust
pub async fn run_vector_index_suite<F, Fut, V>(factory: F) -> ConformanceResult where F: Fn() -> Fut, Fut: Future<Output = V>, V: VectorIndex,
```
Dokumentation: Führt die vollständige Konformitätsprüfung für eine [`VectorIndex`]-Implementierung durch. **Parameter:** - `factory`: Asynchrone Closure, die eine frische Vektorindex-Instanz erzeugt. **Ablauf:** 1. Test 1: Distanzordnung bei Vektorsuche prüfen (nähere Vektoren zuerst). 2. Test 2: Tombstone-Konsistenz nach dem Löschen prüfen. 3. Test 3: Upsert-Aktualisierung bestehender Dokumenten-IDs. 4. Test 4: Ablehnung von Vektoren mit Dimensionskonflikten. 5. Test 5: Ablehnung von nicht-endlichen Werten (`NaN`). 6. Test 6: Stabilität bei `k > anzahl_dokumente`. 7. Test 7: Unterstützung großer `DocId`-Wer

### Modul `conformance::vfs`

#### `fn run_vfs_suite`
- Ort: `crates/contextra-testkit/src/conformance/vfs.rs:24`
```rust
pub fn run_vfs_suite<V: Vfs>(vfs: &V) -> ConformanceResult
```
Dokumentation: Führt die umfassende Konformitätsprüfsuite gegen eine gegebene [`Vfs`]-Instanz aus. **Geprüfte Verträge:** 1. `non_existent_read_fails`: Nicht vorhandene Dateien erzeugen einen Fehler. 2. `write_atomic_and_read`: Geschriebene Bytes sind bitgenau und vollständig lesbar. 3. `file_len_and_exists`: Längen- und Existenzprüfungen spiegeln exakt den geschriebenen Inhalt wider. 4. `directory_lifecycle`: `create_dir_all`, `list_dir` und `sync_dir` verhalten sich deterministisch. 5. `remove_and_unlink`: Gelöschte Dateien verschwinden aus Verzeichnis und Existenzabfragen. 6. `overwrite_atomic`: Überschre

### Modul `crate`

#### `mod conformance`
- Ort: `crates/contextra-testkit/src/lib.rs:40`
```rust
pub mod conformance
```
Dokumentation: Modul mit standardisierten Konformitätsprüfsuiten für Contextra-Port-Traits.

#### `mod doubles`
- Ort: `crates/contextra-testkit/src/lib.rs:75`
```rust
pub mod doubles
```
Dokumentation: Baukasten-Zielmodul `doubles`: Test-Doubles, In-Memory-Referenzen und steuerbare Uhren.

#### `mod fault`
- Ort: `crates/contextra-testkit/src/lib.rs:89`
```rust
pub mod fault
```
Dokumentation: Baukasten-Zielmodul `fault`: Fehlerinjektion und Fehlersimulation (VFS, I/O, Crashes).

#### `mod fault_vfs`
- Ort: `crates/contextra-testkit/src/lib.rs:44`
```rust
pub mod fault_vfs
```
Dokumentation: Virtuelles Dateisystem mit deterministischer Fehlerinjektion (I/O- und fsync-Failures).

#### `mod in_memory_graph`
- Ort: `crates/contextra-testkit/src/lib.rs:48`
```rust
pub mod in_memory_graph
```
Dokumentation: Deterministische In-Memory-Referenzimplementierung des `GraphIndex`-Ports.

#### `mod in_memory_store`
- Ort: `crates/contextra-testkit/src/lib.rs:52`
```rust
pub mod in_memory_store
```
Dokumentation: Transaktionale In-Memory-Referenzimplementierung des `StorageEngine`-Ports.

#### `mod in_memory_text`
- Ort: `crates/contextra-testkit/src/lib.rs:56`
```rust
pub mod in_memory_text
```
Dokumentation: Deterministische In-Memory-Referenzimplementierung des `TextIndex`-Ports.

#### `mod in_memory_vector`
- Ort: `crates/contextra-testkit/src/lib.rs:60`
```rust
pub mod in_memory_vector
```
Dokumentation: Deterministische In-Memory-Referenzimplementierung des `VectorIndex`-Ports.

#### `mod manual_clock`
- Ort: `crates/contextra-testkit/src/lib.rs:64`
```rust
pub mod manual_clock
```
Dokumentation: Manuell steuerbare Uhr zur reproduzierbaren Zeitmessung im `Clock`-Port.

#### `mod mocks`
- Ort: `crates/contextra-testkit/src/lib.rs:68`
```rust
pub mod mocks
```
Dokumentation: Mocks für externe Provider wie Embedding-Generatoren und Metrik-Sinks.

#### `mod model`
- Ort: `crates/contextra-testkit/src/lib.rs:94`
```rust
pub mod model
```
Dokumentation: Baukasten-Zielmodul `model`: Formale Orakel und Referenzmodelle für MVCC und KV-Semantik.

#### `mod reference_model`
- Ort: `crates/contextra-testkit/src/lib.rs:72`
```rust
pub mod reference_model
```
Dokumentation: Formales Referenzmodell zur Prüfung von MVCC- und Transaktionsisolationsgarantien.

#### `fn mint_witness_for_tests`
- Ort: `crates/contextra-testkit/src/lib.rs:131`
```rust
pub fn mint_witness_for_tests( layer: impl Into<contextra_ports::scanner::WitnessLayer>, scanned: u64, remaining: u64, technique: contextra_ports::scanner::ScanTechnique, ) -> contextra_ports::scanner::LayerWitness
```
Dokumentation: Hilfsfunktion zum Erzeugen eines synthetischen [`contextra_ports::scanner::LayerWitness`]. **Zweck:** Ermöglicht Testsuiten das Instanziieren von Bereinigungs- und Scan-Zeugen (Witnesses), ohne echte Platten-Scans oder kryptografische Prüfungen durchführen zu müssen. **Ablauf:** 1. Überführt das übergebene Layer-Argument in ein `WitnessLayer`. 2. Ruft die interne Port-Funktion `mint_witness_for_testkit` mit synthetischen Werten (fixer Hash `[0xFE; 32]`, Sequenznummer `0`) auf. **Vorbedingungen:** Nur aktiv unter dem Feature-Flag `erasure-conformance`. **Rückgabe:** Ein fertig initialisiertes `
- Gate: `#[cfg(feature = "erasure-conformance")]`

### Modul `fault_vfs`

#### `struct FaultConfig`
- Ort: `crates/contextra-testkit/src/fault_vfs.rs:39`
```rust
pub struct FaultConfig
```
Dokumentation: Konfiguration für deterministische Fehlerinjektion in [`FaultVfs`]. **Zweck:** Ermöglicht Test-Cases die exakte Vorgabe von Fehlerbedingungen, beispielsweise um Crash-Recovery nach dem N-ten WAL-Schreibvorgang zu validieren.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `fail_writes_after` | `Option<usize>` | `crates/contextra-testkit/src/fault_vfs.rs:41` |
| `fail_syncs_after` | `Option<usize>` | `crates/contextra-testkit/src/fault_vfs.rs:43` |
| `fail_reads_after` | `Option<usize>` | `crates/contextra-testkit/src/fault_vfs.rs:45` |
| `fail_all` | `bool` | `crates/contextra-testkit/src/fault_vfs.rs:47` |

#### `struct FaultVfs`
- Ort: `crates/contextra-testkit/src/fault_vfs.rs:61`
```rust
pub struct FaultVfs
```
Dokumentation: Simuliertes In-Memory-Dateisystem mit deterministischer Fehlerinjektion. **Zweck:** Dient als I/O-Testunterlage für Speichermodule (z. B. WAL und LSM-Storage), um Fehlertoleranz und Absturzsicherheit ohne reale Festplattenfehler zu testen. **Nebenläufigkeit:** - Dateiinhalte liegen geschützt in einem [`RwLock<BTreeMap<PathBuf, Vec<u8>>>`]. - Die Operationszähler (`read_count`, `write_count`, `sync_count`) sind atomare [`AtomicUsize`], sodass auch nebenläufige Schreibversuche präzise gezählt werden.
- Impl-Block `crates/contextra-testkit/src/fault_vfs.rs:76`
- Impl-Block (Trait `contextra_ports::Vfs`) `crates/contextra-testkit/src/fault_vfs.rs:239`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-testkit/src/fault_vfs.rs:79` | Erstellt eine neue, leere [`FaultVfs`]-Instanz ohne aktive Fehlerregeln. |
| `pub fn set_config(&self, config: FaultConfig)` | `crates/contextra-testkit/src/fault_vfs.rs:88` | Aktualisiert die aktive Konfiguration für Fehlerinjektion. **Parameter:** - `config`: Die neue [`FaultConfig`]. |
| `pub fn trigger_crash(&self)` | `crates/contextra-testkit/src/fault_vfs.rs:97` | Simuliert einen abrupten Systemabsturz (Power-Cut). Nach diesem Aufruf schlagen alle folgenden Schreib-, Lese- und Sync-Operationen mit `ErrorKind::BrokenPipe`  |
| `pub fn reset_counters(&self)` | `crates/contextra-testkit/src/fault_vfs.rs:105` | Setzt die Operationszähler und den Crash-Status zurück. Die bestehenden Dateiinhalte in `files` bleiben dabei unverändert erhalten. |
| `pub fn write_file(&self, path: &Path, data: &[u8]) -> io::Result<()>` | `crates/contextra-testkit/src/fault_vfs.rs:176` | Schreibt Bytes an den angegebenen Pfad unter Berücksichtigung der Fehlerrichtlinien. **Fehlerbehandlung:** Schlägt fehl, falls ein Crash aktiv ist oder das Schr |
| `pub fn read_file(&self, path: &Path) -> io::Result<Vec<u8>>` | `crates/contextra-testkit/src/fault_vfs.rs:188` | Liest Bytes vom angegebenen Pfad unter Berücksichtigung der Fehlerrichtlinien. **Fehlerbehandlung:** Liefert einen Lesefehler bei injiziertem Fault oder `ErrorK |
| `pub fn sync(&self) -> io::Result<()>` | `crates/contextra-testkit/src/fault_vfs.rs:205` | Simuliert einen `fsync`-Aufruf zur dauerhaften Bestätigung auf dem Speichermedium. **Ablauf:** 1. Crash-Zustand prüfen. 2. `fail_all` prüfen. 3. Sync-Zähler ink |
| `pub fn write_count(&self) -> usize` | `crates/contextra-testkit/src/fault_vfs.rs:224` | Liefert die Anzahl der bisher versuchten Schreiboperationen. |
| `pub fn read_count(&self) -> usize` | `crates/contextra-testkit/src/fault_vfs.rs:229` | Liefert die Anzahl der bisher versuchten Leseoperationen. |
| `pub fn sync_count(&self) -> usize` | `crates/contextra-testkit/src/fault_vfs.rs:234` | Liefert die Anzahl der bisher versuchten Sync-Operationen. |

### Modul `in_memory_graph`

#### `struct InMemoryGraphIndex`
- Ort: `crates/contextra-testkit/src/in_memory_graph.rs:48`
```rust
pub struct InMemoryGraphIndex
```
Dokumentation: Thread-sichere In-Memory-Referenzimplementierung von [`GraphIndex`] für Tests. **Zweck:** Ermöglicht Wissensgraph- und Beziehungstests ohne externe Graphdatenbanken. **Datenhaltung:** Speichert Knoten in einer `BTreeMap<EntityId, Entity>` und gerichtete Kanten in einer `BTreeMap<(EntityId, EntityId), Edge>`, geschützt über `parking_lot::RwLock`.
- Impl-Block (Trait `Default`) `crates/contextra-testkit/src/in_memory_graph.rs:55`
- Impl-Block `crates/contextra-testkit/src/in_memory_graph.rs:66`
- Impl-Block (Trait `GraphIndex`) `crates/contextra-testkit/src/in_memory_graph.rs:75`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-testkit/src/in_memory_graph.rs:69` | Erstellt einen neuen, leeren `InMemoryGraphIndex`. |

### Modul `in_memory_store`

#### `struct InMemoryStorageEngine`
- Ort: `crates/contextra-testkit/src/in_memory_store.rs:64`
```rust
pub struct InMemoryStorageEngine
```
Dokumentation: Thread-sichere In-Memory-Implementierung von `StorageEngine` für Tests. **Zweck:** Ermöglicht Unit- und Integrationstests von Engines und Abfrageschichten, ohne Festplatten-I/O oder temporäre Verzeichnisse anlegen zu müssen. **Transaktionsablauf:** 1. `put` oder `delete` schreibt Operationen in die Staging-Tabelle `staged[tx_id]`. 2. Vor dem Commit sind diese Änderungen für `get` oder Scans unsichtbar. 3. `commit(tx_id)` entnimmt die Operationen aus dem Staging, vergibt eine neue Sequenznummer und schreibt sie in die globale `data`-Map. 4. `rollback(tx_id)` verwirft die Änderungen aus `staged`
- Impl-Block (Trait `Default`) `crates/contextra-testkit/src/in_memory_store.rs:75`
- Impl-Block `crates/contextra-testkit/src/in_memory_store.rs:86`
- Impl-Block (Trait `StorageEngine`) `crates/contextra-testkit/src/in_memory_store.rs:95`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-testkit/src/in_memory_store.rs:89` | Erstellt eine neue, leere In-Memory-Speicher-Engine. |

### Modul `in_memory_text`

#### `struct InMemoryTextIndex`
- Ort: `crates/contextra-testkit/src/in_memory_text.rs:41`
```rust
pub struct InMemoryTextIndex
```
Dokumentation: Thread-sichere In-Memory-Referenzimplementierung von [`TextIndex`] für Tests. **Zweck:** Ermöglicht Volltext- und Stichwortsuchen in Tests ohne externe Such-Engines (wie Tantivy oder Lucene).
- Impl-Block (Trait `Default`) `crates/contextra-testkit/src/in_memory_text.rs:50`
- Impl-Block `crates/contextra-testkit/src/in_memory_text.rs:60`
- Impl-Block (Trait `TextIndex`) `crates/contextra-testkit/src/in_memory_text.rs:79`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-testkit/src/in_memory_text.rs:63` | Erstellt einen neuen, leeren `InMemoryTextIndex`. |

### Modul `in_memory_vector`

#### `struct InMemoryVectorIndex`
- Ort: `crates/contextra-testkit/src/in_memory_vector.rs:51`
```rust
pub struct InMemoryVectorIndex
```
Dokumentation: Thread-sichere Referenzimplementierung von [`VectorIndex`] für Tests. **Zweck:** Ermöglicht semantische Vektorsuchen in Unit- und Integrationstests, ohne externe Vektordatenbanken oder komplexe ANN-Indizes (wie HNSW oder DiskANN) instanziieren zu müssen. **Invarianten:** - Alle gespeicherten Vektoren haben exakt die Dimension `dim`. - Vektoren enthalten keine NaN- oder Infinity-Werte.
- Impl-Block `crates/contextra-testkit/src/in_memory_vector.rs:62`
- Impl-Block (Trait `VectorIndex`) `crates/contextra-testkit/src/in_memory_vector.rs:76`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(dim: usize) -> Self` | `crates/contextra-testkit/src/in_memory_vector.rs:65` | Erstellt einen neuen `InMemoryVectorIndex` mit der erwarteten Vektordimension `dim`. |

### Modul `manual_clock`

#### `struct ManualClock`
- Ort: `crates/contextra-testkit/src/manual_clock.rs:41`
```rust
pub struct ManualClock
```
Dokumentation: Thread-sichere, manuell vorstellbare Uhr für reproduzierbare Tests. **Zweck:** Ermöglicht deterministische zeitabhängige Tests ohne Abhängigkeit von der Systemuhr und ohne unzuverlässige `sleep`-Aufrufe. **Invarianten:** - Der interne Zähler (`nanos`) speichert die Nanosekunden seit dem Unix-Epochenstart. - Operationen sind atomar und `SeqCst`-geordnet (sequenziell konsistent).
- Impl-Block (Trait `Default`) `crates/contextra-testkit/src/manual_clock.rs:46`
- Impl-Block `crates/contextra-testkit/src/manual_clock.rs:54`
- Impl-Block (Trait `Clock`) `crates/contextra-testkit/src/manual_clock.rs:113`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(initial_nanos: u64) -> Self` | `crates/contextra-testkit/src/manual_clock.rs:62` | Erstellt eine neue `ManualClock` mit einem gegebenen Nanosekundenwert. **Parameter:** - `initial_nanos`: Startzeit in Nanosekunden seit UNIX_EPOCH. **Rückgabe:* |
| `pub fn advance(&self, duration: Duration)` | `crates/contextra-testkit/src/manual_clock.rs:78` | Stellt die Uhr um eine bestimmte Dauer vor. **Ablauf:** 1. Konvertiert die [`Duration`] in Nanosekunden (`u128` zu `u64`). 2. Bei Überlauf wird auf `u64::MAX` a |
| `pub fn set_nanos(&self, nanos: u64)` | `crates/contextra-testkit/src/manual_clock.rs:89` | Setzt die Uhr absolut auf einen bestimmten Nanosekunden-Zeitstempel. **Hinweis:** Kann die Zeit auch rückwärts setzen; sollte mit Bedacht genutzt werden, um Mon |
| `pub fn now_nanos(&self) -> u64` | `crates/contextra-testkit/src/manual_clock.rs:95` | Liest den aktuellen Zeitstempel in Nanosekunden seit UNIX_EPOCH aus. |
| `pub fn now_secs_f64(&self) -> f64` | `crates/contextra-testkit/src/manual_clock.rs:101` | Gibt den aktuellen Zeitstempel als Fließkommazahl in Sekunden zurück. |
| `pub fn now_system_time(&self) -> SystemTime` | `crates/contextra-testkit/src/manual_clock.rs:107` | Konvertiert den aktuellen Zustand in einen Standard-Rust-[`SystemTime`]-Zeitpunkt. |

### Modul `mocks`

#### `struct BudgetTruncatingContextPreparer`
- Ort: `crates/contextra-testkit/src/mocks.rs:557`
```rust
pub struct BudgetTruncatingContextPreparer
```
Dokumentation: Deterministische Implementierung von [`ContextPreparer`].
- Impl-Block `crates/contextra-testkit/src/mocks.rs:559`
- Impl-Block (Trait `ContextPreparer`) `crates/contextra-testkit/src/mocks.rs:566`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-testkit/src/mocks.rs:561` | Erzeugt einen neuen `BudgetTruncatingContextPreparer`. |

#### `struct InMemoryKvBridgeStorage`
- Ort: `crates/contextra-testkit/src/mocks.rs:326`
```rust
pub struct InMemoryKvBridgeStorage
```
Dokumentation: Deterministische In-Memory-Implementierung von [`KvBridgeStorage`].
- Impl-Block `crates/contextra-testkit/src/mocks.rs:331`
- Impl-Block (Trait `KvBridgeStorage`) `crates/contextra-testkit/src/mocks.rs:348`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-testkit/src/mocks.rs:333` | Erzeugt einen neuen leeren `InMemoryKvBridgeStorage`. |
| `pub fn len(&self) -> usize` | `crates/contextra-testkit/src/mocks.rs:338` | Gibt die Anzahl der aktuell committeten Schlüssel zurück. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-testkit/src/mocks.rs:343` | Prüft, ob der committete Speicher leer ist. |

#### `struct InMemoryKvPrefixStore`
- Ort: `crates/contextra-testkit/src/mocks.rs:415`
```rust
pub struct InMemoryKvPrefixStore
```
Dokumentation: Deterministische In-Memory-Implementierung von [`KvPrefixStore`].
- Impl-Block `crates/contextra-testkit/src/mocks.rs:419`
- Impl-Block (Trait `KvPrefixStore`) `crates/contextra-testkit/src/mocks.rs:437`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-testkit/src/mocks.rs:421` | Erzeugt einen neuen leeren `InMemoryKvPrefixStore`. |

#### `struct MockEmbedder`
- Ort: `crates/contextra-testkit/src/mocks.rs:35`
```rust
pub struct MockEmbedder
```
Dokumentation: Deterministischer Mock-Embedding-Provider für Testläufe. **Zweck:** Ermöglicht semantische Vektortests ohne teure oder nicht-deterministische externe KI-/ML-Modelle oder Inferenz-Engines. **Verhalten:** Gibt entweder einen statischen Vektor (`fixed_output`) oder einen Default-Vektor aus wiederholten `0.1f32`-Werten der gewünschten Dimension zurück.
- Impl-Block `crates/contextra-testkit/src/mocks.rs:42`
- Impl-Block (Trait `EmbeddingProvider`) `crates/contextra-testkit/src/mocks.rs:68`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `dim` | `usize` | `crates/contextra-testkit/src/mocks.rs:37` |
| `fixed_output` | `Option<Vec<f32>>` | `crates/contextra-testkit/src/mocks.rs:39` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(dim: usize) -> Self` | `crates/contextra-testkit/src/mocks.rs:47` | Erzeugt einen `MockEmbedder` mit der vorgegebenen Dimension. Standardmäßig wird ein Vektor aus `0.1f32` der Länge `dim` erzeugt. |
| `pub fn with_fixed_output(fixed_output: Vec<f32>) -> Self` | `crates/contextra-testkit/src/mocks.rs:58` | Erzeugt einen `MockEmbedder` mit einem festen Vektor. Die Dimension wird automatisch aus der Länge von `fixed_output` abgeleitet. |

#### `struct ScriptedAttentionExporter`
- Ort: `crates/contextra-testkit/src/mocks.rs:385`
```rust
pub struct ScriptedAttentionExporter
```
Dokumentation: Skriptbare, deterministische Implementierung von [`AttentionExporter`].
- Impl-Block `crates/contextra-testkit/src/mocks.rs:389`
- Impl-Block (Trait `AttentionExporter`) `crates/contextra-testkit/src/mocks.rs:406`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-testkit/src/mocks.rs:391` | Erstellt einen leeren `ScriptedAttentionExporter`. |
| `pub fn set_weights(&self, request_id: RequestId, weights: Vec<f32>)` | `crates/contextra-testkit/src/mocks.rs:396` | Registriert oder aktualisiert die Aufmerksamkeitsgewichte für einen Request. |
| `pub fn remove_weights(&self, request_id: RequestId) -> Option<Vec<f32>>` | `crates/contextra-testkit/src/mocks.rs:401` | Entfernt die registrierten Gewichte für einen Request. |

#### `struct ScriptedEgressPolicy`
- Ort: `crates/contextra-testkit/src/mocks.rs:650`
```rust
pub struct ScriptedEgressPolicy
```
Dokumentation: Skriptbare Implementierung von [`EgressPolicy`].
- Impl-Block `crates/contextra-testkit/src/mocks.rs:655`
- Impl-Block (Trait `EgressPolicy`) `crates/contextra-testkit/src/mocks.rs:672`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-testkit/src/mocks.rs:657` | Erzeugt eine neue leere `ScriptedEgressPolicy` (erlaubt standardmäßig alles). |
| `pub fn add_deny_pattern(&self, pattern: impl Into<String>)` | `crates/contextra-testkit/src/mocks.rs:662` | Fügt ein Muster hinzu, das zu `Deny` führt. |
| `pub fn add_redact_pattern(&self, pattern: impl Into<String>)` | `crates/contextra-testkit/src/mocks.rs:667` | Fügt ein Muster hinzu, das zu `Redact` führt. |

#### `struct ScriptedHybridSearchProvider`
- Ort: `crates/contextra-testkit/src/mocks.rs:486`
```rust
pub struct ScriptedHybridSearchProvider
```
Dokumentation: Skriptbare, deterministische Implementierung von [`HybridSearchProvider`].
- Impl-Block `crates/contextra-testkit/src/mocks.rs:490`
- Impl-Block (Trait `HybridSearchProvider`) `crates/contextra-testkit/src/mocks.rs:508`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-testkit/src/mocks.rs:492` | Erzeugt einen neuen leeren `ScriptedHybridSearchProvider`. |
| `pub fn set_chunks(&self, mut chunks: Vec<ContextChunk>)` | `crates/contextra-testkit/src/mocks.rs:497` | Setzt die vorgegebenen Context-Chunks. |

#### `struct StaticCommunityResolver`
- Ort: `crates/contextra-testkit/src/mocks.rs:529`
```rust
pub struct StaticCommunityResolver
```
Dokumentation: Statische, deterministische Implementierung von [`CommunityResolver`].
- Impl-Block `crates/contextra-testkit/src/mocks.rs:533`
- Impl-Block (Trait `CommunityResolver`) `crates/contextra-testkit/src/mocks.rs:545`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-testkit/src/mocks.rs:535` | Erzeugt einen neuen leeren `StaticCommunityResolver`. |
| `pub fn insert(&self, entity_id: EntityId, community_id: u64)` | `crates/contextra-testkit/src/mocks.rs:540` | Registriert die Community-Zuordnung für eine Entity. |

#### `struct StaticDriftStatusProvider`
- Ort: `crates/contextra-testkit/src/mocks.rs:610`
```rust
pub struct StaticDriftStatusProvider
```
Dokumentation: Statische Implementierung von [`DriftStatusProvider`].
- Impl-Block (Trait `Default`) `crates/contextra-testkit/src/mocks.rs:614`
- Impl-Block `crates/contextra-testkit/src/mocks.rs:622`
- Impl-Block (Trait `DriftStatusProvider`) `crates/contextra-testkit/src/mocks.rs:641`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-testkit/src/mocks.rs:624` | Erzeugt einen `StaticDriftStatusProvider` mit Default-Status `"stabil"`. |
| `pub fn with_status(status: impl Into<String>) -> Self` | `crates/contextra-testkit/src/mocks.rs:629` | Erzeugt einen `StaticDriftStatusProvider` mit spezifischem Status. |
| `pub fn set_status(&self, status: impl Into<String>)` | `crates/contextra-testkit/src/mocks.rs:636` | Ändert den aktuellen Status. |

#### `struct TestMetricsSink`
- Ort: `crates/contextra-testkit/src/mocks.rs:136`
```rust
pub struct TestMetricsSink
```
Dokumentation: In-Memory-Implementierung von [`contextra_ports::MetricsSink`] für Tests. **Zweck:** Fängt alle Metrik-Aufrufe ab und speichert sie chronologisch in einer threadsicheren Liste zur späteren Auswertung in Unit-Tests.
- Impl-Block `crates/contextra-testkit/src/mocks.rs:141`
- Impl-Block (Trait `MetricsSink`) `crates/contextra-testkit/src/mocks.rs:170`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-testkit/src/mocks.rs:144` | Erstellt eine leere [`TestMetricsSink`]. |
| `pub fn events(&self) -> Vec<MetricEvent>` | `crates/contextra-testkit/src/mocks.rs:152` | Erstellt einen Schnappschuss (Snapshot) aller bisher aufgezeichneten Ereignisse. Behandelt vergiftete Mutexe (`PoisonError`) defensiv via `into_inner()`. |
| `pub fn clear(&self)` | `crates/contextra-testkit/src/mocks.rs:161` | Leert die Liste aller aufgezeichneten Metrik-Ereignisse. |

#### `enum MetricEvent`
- Ort: `crates/contextra-testkit/src/mocks.rs:100`
```rust
pub enum MetricEvent
```
Dokumentation: Ein einzelnes aufgezeichnetes Metrik-Ereignis aus [`TestMetricsSink`]. **Zweck:** Dient in Test-Assertions der genauen Überprüfung, ob Instrumentierungen die richtigen Namen, Werte und Labels gemeldet haben.

Varianten:

- `Counter{` — `crates/contextra-testkit/src/mocks.rs:102`
- `Gauge{` — `crates/contextra-testkit/src/mocks.rs:111`
- `Histogram{` — `crates/contextra-testkit/src/mocks.rs:120`

### Modul `reference_model`

#### `struct ReferenceModel`
- Ort: `crates/contextra-testkit/src/reference_model.rs:50`
```rust
pub struct ReferenceModel
```
Dokumentation: Deterministic, std-only MVCC reference model for Contextra store verification. **Zweck:** Bietet eine überprüfbare "Ground Truth" für Transaktionsisolation, Snapshot-Isolation und Rollbacks. **Ablauf von Schreibvorgängen:** 1. Operationen (`Put`, `Delete`) werden zunächst im unbestätigten Puffer `pending_writes` gesammelt. 2. `rollback()` verwirft alle Puffer-Einträge ohne Erhöhung der Sequenznummer. 3. `commit()` erhöht `current_seq` atomar um 1 und überträgt alle Puffer-Einträge in die `history`-Map.
- Impl-Block `crates/contextra-testkit/src/reference_model.rs:60`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-testkit/src/reference_model.rs:63` | Creates a new empty reference model initialized at sequence number 0. |
| `pub fn put(&mut self, key: &[u8], value: &[u8])` | `crates/contextra-testkit/src/reference_model.rs:73` | Stages a `Put` operation in uncommitted pending buffer. |
| `pub fn delete(&mut self, key: &[u8])` | `crates/contextra-testkit/src/reference_model.rs:82` | Stages a `Delete` operation in uncommitted pending buffer. |
| `pub fn rollback(&mut self)` | `crates/contextra-testkit/src/reference_model.rs:89` | Clears any uncommitted pending writes without advancing sequence number. |
| `pub fn commit(&mut self) -> u64` | `crates/contextra-testkit/src/reference_model.rs:103` | Commits all pending operations under a new sequence number. Returns the sequence number assigned to this commit batch. **Ablauf:** 1. Ist der Puffer leer, bleib |
| `pub fn commit_batch(&mut self, ops: Vec<RefOp>) -> u64` | `crates/contextra-testkit/src/reference_model.rs:133` | Appends `ops` to pending buffer and commits them under a new sequence number. |
| `pub fn snapshot_seq(&self) -> u64` | `crates/contextra-testkit/src/reference_model.rs:140` | Returns the current high water mark sequence number. |
| `pub fn get_at(&self, key: &[u8], seq: u64) -> Option<Vec<u8>>` | `crates/contextra-testkit/src/reference_model.rs:153` | Retrieves the value of `key` as visible at `seq`. Returns `Some(value)` if key exists and was active (not deleted) at `seq`, `None` otherwise. **Ablauf:** 1. Su |
| `pub fn scan_prefix_at(&self, prefix: &[u8], seq: u64) -> Vec<(Vec<u8>, Vec<u8>)>` | `crates/contextra-testkit/src/reference_model.rs:170` | Scans keys starting with `prefix` and returns key-value pairs visible at `seq`. **Ablauf:** Nutzt `BTreeMap::range` ab dem Präfix und filtert gelöschte Einträge |
| `pub fn scan_range_at( &self, start: Bound<&[u8]>, end: Bound<&[u8]>, seq: u64, ) -> Vec<(Vec<u8>, Vec<u8>)>` | `crates/contextra-testkit/src/reference_model.rs:189` | Scans key range `[start, end]` (using `Bound`) and returns visible pairs at `seq`. |
| `pub fn get_latest(&self, key: &[u8]) -> Option<Vec<u8>>` | `crates/contextra-testkit/src/reference_model.rs:216` | Gets the latest committed value for `key`. |
| `pub fn scan_prefix_latest(&self, prefix: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)>` | `crates/contextra-testkit/src/reference_model.rs:222` | Scans all entries starting with `prefix` at latest sequence number. |
| `pub fn snapshot_map_at(&self, seq: u64) -> BTreeMap<Vec<u8>, Vec<u8>>` | `crates/contextra-testkit/src/reference_model.rs:228` | Returns a full snapshot map of latest visible key-values at `seq`. |

#### `enum RefOp`
- Ort: `crates/contextra-testkit/src/reference_model.rs:31`
```rust
pub enum RefOp
```
Dokumentation: Operation for batch execution on the reference model.

Varianten:

- `Put{ key: Vec<u8>, value: Vec<u8> }` — `crates/contextra-testkit/src/reference_model.rs:33`
- `Delete{ key: Vec<u8> }` — `crates/contextra-testkit/src/reference_model.rs:35`

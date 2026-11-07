# Agentic Context Pack: `contextra-engine`

> **Werkzeug:** `agentic_context_pack.py`
> **Erstellungszeit:** `2026-11-07T23:15:52.956399+00:00`
> **Status/Befunde:** `1 Befunde erfasst`

## Beschreibung
Curated, minified context pack for coding agents assigned to `contextra-engine`. Optimized for LLM window density and context engineering.

## Metriken-Übersicht
| Metrik | Wert |
|---|---|
| `crate_name` | `contextra-engine` |
| `ring` | `Unknown` |
| `class` | `Standard` |

## 1. Architektur-Zuweisung
- **Crate:** `contextra-engine`
- **Ring:** `None`
- **Rolle/Klasse:** `None`
- **Pfad:** `crates/contextra-engine`

## 2. Crate-Spezifische AGENTS.md Verträge
# AGENTS.md — `contextra-engine`

> **Ring 3** · Klasse `[K]` · Reifegrad `beta` · Unsafe: nein · Rolle: Orchestrierung: Collections, Transaktionen, Hybrid-Retrieval, Chunking, Hintergrundarbeiter. **Soll:** generisch über das Trait-Bündel `Backends`.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-engine --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `adapt`, `checkpoint`, `crypto`, `graph`, `infer-onnx`, `mvcc`, `ports`, `rank`, `sandbox`, `store`, `sys`, `text`, `types`, `vector` (14) |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types` (+ Standard-Backends hinter Feature `default-backends`) |
| Features | `onnx`, `sandbox`, `reranking`, `background-maintenance`, `entity-extraction`, `experimental-diskann`, `graph-connectivity-health`, `edge-reinforcement-learning`, `adaptive-candidate-pool-sizing`, `auto-extraction-opt-out` |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- **Kein `Backends`-Trait** (0 Treffer). `Collection<S: StorageEngine = LsmStorage, V: VectorIndex = HnswIndex>` hat konkrete Standardtypen; ~65 direkte Verweise auf `store`/`vector`/`text`/`graph` außerhalb von Tests.
- **M3-Leaks:** `pub use crypto::deletion_proof::{DeletionLayer, DeletionProof, DeletionScope, ExcludedScope}` und `pub use store::lsm::DurabilityMode`.
- **Löschrest:** `contextra_impl/erasure.rs` (618 Zeilen, `ErasureOrchestrator`, `ProofKeyProvider`), 24 `DeletionProof`-Treffer: gehört nach `contextra-erasure` (WP 3.5).
- `chunker/` liegt hier; `ingest` hängt deshalb an `engine` (WP 5.4: `contextra-chunker` extrahieren).
- Tote Kanten laut D9: `sys`; prüfen: `adapt`, `sandbox`, `infer-onnx`. Modulkommentare nennen „Ring 2“, `capabilities.toml` Ring 3.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `collection/` (`crud/`, `search/`, `query_builder/`, `tx.rs`, `kv_lock.rs`, `maintenance.rs`, `relate.rs`) | `Collection`, CRUD, Hybridsuche, Query-Builder |
| `contextra_impl/` (`collections`, `crud`, `lifecycle`, `wiring`, `diagnostics`, **`erasure`**) | `Contextra`-Instanz, Verdrahtung |
| `transaction/` (`db_transaction*`, `intent`, `cleanup`, `compensating_actions`) | `DbTransaction`, Commit/Rollback |
| `background_workers/`, `decay_controller.rs` | Expiry-, Orphan-, Hyperedge-Worker, Decay |
| `chunker/`, `extraction/`, `fusion.rs`, `filter.rs`, `temporal_filter.rs`, `export.rs`, `import.rs`, `merge_operator_adapter.rs`, `orchestration/` | Chunking, Extraktion, Fusion-Aufruf, Filter, Import/Export |

## 4. Verträge & Invarianten

1. **`TxId` nur über `allocate_tx`** (`collection/tx.rs`), nie aus der Uhr.
2. **Sperrenordnung** (aus Quellkommentar): `collections` (RwLock) → `kv_locks` → `embedder` (RwLock). Nie umgekehrt; LLM-/Embedding-/Netzaufrufe nie unter einem Guard.
3. **Atomarität (INV-ATOMIC-IDX-1):** Metadaten- und Indexänderung in **einem** WAL-Batch; Indexe werden von **einem** Applier in WAL-Reihenfolge angewendet (INV-INDEX-APPLY-1).
4. **Embedding-Identität (INV-EMBED-2):** Modell-Hash und Dimension pro Collection; kein stiller Platzhalter-Embedder (INV-EMBED-1).
5. **Ein Fusionsbesitzer:** RRF liegt in `rank/fusion`; hier nur Aufruf.
6. **Chunking ist Pflicht** für lange Dokumente (nicht als ein String einbetten).
7. **Tenant aus `RequestContext`** (VETO-F10).
8. **Kein neuer Löschcode** (M7): `forget`/Saga/Proof gehören nach `contextra-erasure`; Engine ruft nur Ports.

## 5. Ports

**Konsumiert:** `StorageEngine`, `VectorIndex`, `TextIndex`, `GraphIndex`, `EmbeddingProvider`, `Reranker`, `Clock`, `Rng`, `IdGen`, `MetricsSink`, `Purger`. Ziel: Zugriff gebündelt über `Backends`.

## 6. Nebenläufigkeit & Performance

- ~164 `async fn`, ~50 `tokio::`-Treffer. CPU-lastige Arbeit (Embedding, Fusion, HNSW-Suche) auf dem Blocking-Pool, nie auf dem Executor (SPEC §6.3).
- Hot Path generisch/monomorphisiert, Cold Path (Sagas, Reporting, Worker) `dyn` (WP 5.2). Kompilierzeit-Risiko R4 beachten.
- `loom`-Tests für `kv_lock.rs`; Property-Tests mit `testkit::reference_model`.

## 7. Anti-Patterns

- Konkrete Typen (`LsmStorage`, `HnswIndex`) in neuen Signaturen.
- Neue `pub use` fremder Implementierungstypen.
- Löschlogik in `contextra_impl/`.
- `LlmQueryRewriter`/Embedding unter `RwLockReadGuard`.

## 8. Tests & Messung

```bash
cargo test -p contextra-engine --locked
cargo clippy -p contextra-engine --all-targets -- -D warnings
```

- 42 Testdateien/-module. Vor jedem Schnitt Charakterisierungstests für Update/Supersession und Erasure (Risiko R2).

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 3.5 | `erasure.rs` raus; Worker/Decay entkoppeln |
| 5.1 | `trait Backends`, `default-backends`-Feature |
| 5.2 | dyn vs. generisch |
| 5.3/D9 | Querkanten `sys`, `adapt`, `sandbox`, `infer-onnx` schneiden |
| 5.4 | `chunker` extrahieren |


---


# Kommentar-Syntax (verbindlich)

| Form | Zweck |
|---|---|
| `//!` | Modul-/Crate-Doku am Dateianfang (vor jedem Item, nach `#![...]`-Attributen der Datei; nie mitten in der Datei) |
| `///` | Rustdoc für das **direkt folgende** Item (Struct, Enum, Variante, Feld, Trait, Methode, Funktion, Konstante). Nie vor `let`, Ausdrücken, Match-Armen oder Makroaufrufen; dort `//` verwenden. |
| `// NOTE:` | Warum etwas so gebaut ist (nicht was) |
| `// TODO:` / `// FIXME:` / `// STUB:` | Offene Arbeit / bekanntes Risiko / Platzhalter; immer mit konkreter Aussage |
| `// SAFETY:` | Pflicht direkt über jedem unsafe-Konstrukt (Invariante, Herstellungsort, Folge bei Verletzung) |
| `// ANCHOR[contextra-engine:<BEREICH>-<NNN>]` | Sprungmarke; vor dem Item/Abschnitt, auf den sie zeigt |
| `// FILE-CONTEXT` + `// ZWECK:` / `// INVARIANTEN:` / `// HOTSPOTS: <von>-<bis>` / `// STAND: <Datum>` | Repo-Konvention für den Dateikopf. Vorhandene Blöcke ergänzen; `HOTSPOTS`-Zeilenbereiche nach dem Kommentieren auf die neuen Zeilennummern aktualisieren. `STAND` auf das heutige Datum setzen, wenn du den Block änderst. |

Weitere Tags führst du nicht ein. Beim Kommentieren markierter Altlasten gilt: `// NOTE: Deprecated Debt DEBT-NN – <Beschreibung>`.

## ANCHOR-Ids (Kollisionsschutz)

Im Repo existieren ca. 3.900 Anchors; Ids sind workspace-weit eindeutig zu halten. Format: `ANCHOR[contextra-engine:<BEREICH>-<NNN>]` mit `<BEREICH>` in Großbuchstaben/Ziffern/Bindestrichen, `<NNN>` dreistellig. Vorgehen:

1. Vor jeder neuen Id: `grep -rnF "ANCHOR[contextra-engine:<BEREICH>-<NNN>]" crates benchmarks` muss leer sein.
2. Nummeriere ab dem Maximum des Bereichs weiter. Höchste bestehende Nummern in dieser Crate:

| Bereich | Höchste Nr. | Nächste freie Id |
|---|---:|---|
| `COL-PER` | 870 | `ANCHOR[contextra-engine:COL-PER-871]` |
| `SET` | 502 | `ANCHOR[contextra-engine:SET-503]` |
| `WIT` | 487 | `ANCHOR[contextra-engine:WIT-488]` |
| `CLO` | 479 | `ANCHOR[contextra-engine:CLO-480]` |
| `NEW` | 355 | `ANCHOR[contextra-engine:NEW-356]` |
| `ROL` | 349 | `ANCHOR[contextra-engine:ROL-350]` |
| `SCA` | 333 | `ANCHOR[contextra-engine:SCA-334]` |
| `IS_` | 328 | `ANCHOR[contextra-engine:IS_-329]` |
| `LEN` | 323 | `ANCHOR[contextra-engine:LEN-324]` |
| `REL` | 308 | `ANCHOR[contextra-engine:REL-309]` |
| `COM` | 304 | `ANCHOR[contextra-engine:COM-305]` |
| `COL-INS` | 301 | `ANCHOR[contextra-engine:COL-INS-302]` |

3. Neue Bereiche nach dem Modul benennen (z. B. `WAL`, `PUT`, `GATE`); fange bei `001` an, nach Prüfung 1. Setze Anchors sparsam: pro Modul, pro öffentlichem Einstiegspunkt und pro nicht-trivialem Ablauf, nicht pro Zeile.
4. Bereits vorhandene doppelte Ids anderer Stellen nicht „reparieren“; nur im Report melden.

Für Abläufe in Funktionen nutze nummerierte Schritte:

```rust
// ANCHOR[contextra-engine:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge, Schlüsselformat).
// 2. Snapshot der Transaktion nehmen.
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen; ein Fehler wird zurückgegeben (Fail-Closed).
// 4. Erst danach im Speicher übernehmen.
```

# Was zu dokumentieren ist

Gehe **jede** Rust-Datei unter `crates/contextra-engine` durch (`find crates/contextra-engine -name "*.rs"`, ohne die generierten Dateien oben) einschließlich `tests/`, `benches/`, `examples/` und `build.rs`:

- **Modul** (`//!`): Zweck, Ring-Zugehörigkeit, Nutzer, wichtigste Typen, Invarianten, Verweis auf Nachbarmodule.
- **Öffentliche Items:** Zweck in einem Satz, dann bei Bedarf `# Errors` (welcher Fehler wann), `# Panics` nur wenn der Code abbrechen kann (sollte nach Zero-Panic-Regel nicht vorkommen: dann `FIXME`), `# Safety` bei `unsafe fn`, Vorbedingungen, Nachbedingungen, Nebenwirkungen (I/O, Zeit, Zufall, IDs). Zeit, Zufall und IDs laufen nur über `Clock`/`Rng`/`IdGen`; steht es im Code, steht es im Kommentar.
- **Nicht-triviale private Funktionen:** Warum sie existieren und was sie garantieren.
- **Nebenläufigkeit:** Sperren, atomare Operationen, Reihenfolge, Warum kein Deadlock.
- **Persistenz und Fehler:** Wo Daten WAL oder Dateisystem-Port passieren; `fsync`-Fehler werden propagiert.
- **Tests:** Ein Satz je Testfunktion oder -block: welche Invariante oder welches Verhalten bewiesen wird.
- **Architektur:** Welche Ports die Crate nutzt/anbietet, welche Abhängigkeitsrichtung gilt (DAG, `may_depend_on`), was bewusst *nicht* hier liegt.

## Rustdoc-Fallen (jeder Punkt bricht sonst `cargo clippy -D warnings` oder `cargo doc`)

- Kein `///` vor Nicht-Items (`unused_doc_comments`); keine Leerzeile zwischen `///` und Item (`empty_line_after_doc_comments`).
- Fortsetzungszeilen in Listen und Zitaten korrekt einrücken (`doc_lazy_continuation`). **Vier Leerzeichen Einrückung in Doc-Text erzeugen einen Doctest**; Code immer in Fences mit Sprache. Beispiele, die nicht kompilieren oder laufen sollen: `ignore` oder `text`, begründet.
- Intra-Doc-Links `[`Typ`]` nur auf existierende, erreichbare Items; spitze Klammern im Text (`Vec<u8>`) in Backticks setzen, sonst „unclosed HTML tag“.
- `// SAFETY:` ohne Leerzeile und ohne dazwischenliegende Zeilen direkt über dem unsafe-Ausdruck.
- Doc-Beispiele müssen gegen die tatsächliche Signatur kompilieren; im Zweifel `no_run`/`ignore` mit Grund.

## 3. Bekannte Technische Schulden (DEPRECATED_DEBT.md)
- - **Betroffene Crates:** `contextra-store`, `contextra-vector`, `contextra-text`, `contextra-crypto`, `contextra-db`, `contextra-mcp`, `contextra-engine`.
- - **Betroffene Crates:** `contextra-engine`, `contextra-db`, `contextra-mcp`, `contextra-store`.

## 4. Öffentliche Schnittstellen & Signaturen (lib.rs)
```rust
pub mod orchestration;
pub use contextra_crypto::deletion_proof::{
pub use contextra_ports::TextEmbeddingEngine;
pub use contextra_ports::VectorDeleteMode;
pub use contextra_store::lsm::DurabilityMode;
pub mod backends;
pub mod background_workers;
pub mod chunker;
pub mod collection;
pub mod decay_controller;
pub mod export;
pub mod extraction;
pub mod filter;
pub mod fusion;
pub mod import;
pub mod metrics_names;
pub mod temporal_filter;
pub mod transaction;
pub mod merge_operator_adapter;
pub use merge_operator_adapter::WasmMergeOperator;
pub use decay_controller::{AdaptiveDecayController, DecayControllerConfig, DecaySignalInputs};
pub use export::{
pub use import::ImportSummary;
pub use background_workers::start_decay_cleanup_worker;
pub use background_workers::{start_expiry_cleanup_worker, start_orphan_cleanup_worker};
pub use backends::Backends;
pub use backends::DefaultBackends;
pub use collection::crud::MAX_SCAN_RESULTS;
pub use collection::maintenance::PercolationConfig;
pub use collection::maintenance::PercolationResult;
pub use collection::query_builder::{HybridQueryBuilder, SearchStrategy, SignalWeights};
pub use collection::{Collection, CollectionConfig};
pub use contextra_checkpoint;
pub use contextra_text::Language;
pub use filter::MetadataFilter;
pub type ConsolidationLauncher = Arc<
pub fn register_consolidation_launcher<F>(_f: F)
pub use fusion::{ProvenanceRecord, SearchResult, SignalContribution};
pub struct Document {
pub id: String,
pub metadata: Option<Value>,
pub struct ContextraStats {
pub drift_status: String,
pub calibration_ece: Option<f32>,
pub last_calibration_at: Option<u64>,
pub active_memory_count: usize,
pub pid_pool_size: Option<usize>,
pub index_stats: contextra_ports::VectorIndexStats,
pub storage_stats: contextra_ports::StorageStats,
pub use contextra_ports::DriftStatusProvider;
```

## Detaillierte Befunde
| Crate | Datei | Zeile | Art / Beschreibung |
|---|---|---|---|
| `contextra-engine` | `crates/contextra-engine/Cargo.toml` | `1` | Generated focused context pack for contextra-engine |

---
*Automatisch generiert durch `agentic_context_pack.py` für Agentic Engineering & Context Auditing.*
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

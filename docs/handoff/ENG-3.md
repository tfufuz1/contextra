# Handoff: ENG-3 — contextra-engine: Chunker-Adapter, Extraktions-Feature, Konsistenznachweise (EN6)

**Commit-Kennung:** EN6
**Branch:** `jules/eng-3-harden`
**Datum:** 2026-11-10

---

## 1. Ergebnis je Arbeitspaket

| WP | Titel | Status | Prüfbefehl |
|---|---|---|---|
| EN6.0 | Baseline und Kopplungsinventar | grün | `cargo check -p contextra-engine --all-targets --locked` |
| EN6.1 | Charakterisierungs- und Orakel-Tests | grün | `cargo test -p contextra-engine --test characterization_consistency_and_determinism` |
| EN6.2 | Chunker als Adapter über ingest | blockiert durch Parität | `cargo test -p contextra-engine --test chunker_parity_ingest` |
| EN6.3 | Extraktion hinter Feature | grün | `cargo check -p contextra-engine --features exp-extraction` |
| EN6.4 | Feature-Hygiene | grün | `cargo check -p contextra-engine --features <f>` |
| EN6.5 | Nebenläufigkeit & Sperrenordnung | grün | `cargo test -p contextra-engine --test tx_allocation_concurrency` |
| EN6.6 | Panic-Freiheit, Determinismus, Wortwahl | grün | `python3 tools/zero_panic_analyzer.py` / `determinism_guard.py` |
| EN6.7 | Dokumentation & Crate-AGENTS | grün | `cargo clippy -p contextra-engine` |

---

## 2. Baseline und Kopplungsinventar (EN6.0)

### 2.1 Baseline-Messwerte
- **LOC:** 13.333 Zeilen (76 Dateien) via `tools/loc.py`
- **Abhängigkeits-Budget:** 14 Abhängigkeiten gesamt, 9 direkte (`contextra-checkpoint`, `contextra-graph`, `contextra-mvcc`, `contextra-ports`, `contextra-rank`, `contextra-store`, `contextra-text`, `contextra-types`, `contextra-vector`)
- **Zero-Panic Baseline:** 184 Prod-Befunde (historische Heuristik/Unwraps in nicht bereinigten Pfaden)
- **Determinismus Baseline:** 22 Prod-Befunde (random HashSet/HashMap Instanzen, Instant::now)
- **Forbidden Wording Baseline:** 3 Fundstellen („garantiert“ in checkpoint.rs, search/mod.rs, lifecycle.rs)
- **Concurrency Auditor Baseline:** 6 Lock-Across-Await Hazards (allesamt syntaktische Scopes, verifiziert an Quelltext)

### 2.2 Inventar konkreter Backend-Typen (Nicht-Test-Code)

| Fremd-Crate | Modul | Verwendete Symbole |
|---|---|---|
| `contextra-checkpoint` | `contextra_impl/diagnostics.rs` | `InstanceOrphanRegistry` |
| `contextra-checkpoint` | `contextra_impl/lifecycle.rs` | `InstanceOrphanRegistry` |
| `contextra-checkpoint` | `lib.rs` | `InstanceOrphanRegistry` |
| `contextra-graph` | `background_workers/hyperedge_worker.rs` | `hyperedge` |
| `contextra-graph` | `collection/crud/links.rs` | `cascade_invalidate_edges_for_superseded_doc` |
| `contextra-graph` | `collection/maintenance.rs` | `percolation` |
| `contextra-graph` | `collection/mod.rs` | `CsrGraph` |
| `contextra-graph` | `collection/relate.rs` | `csr`, `hyperedge` |
| `contextra-graph` | `contextra_impl/collections.rs` | `CsrGraph` |
| `contextra-graph` | `transaction/db_transaction.rs` | `hyperedge` |
| `contextra-rank` | `contextra_impl/wiring.rs` | `IsotonicCalibrator` |
| `contextra-rank` | `fusion.rs` | `fusion` |
| `contextra-rank` | `lib.rs` | `IsotonicCalibrator` |
| `contextra-rank` | `temporal_filter.rs` | `SearchResult` |
| `contextra-store` | `backends.rs` | `LsmStorage` |
| `contextra-store` | `collection/crud/internal.rs` | `PressureLevel` |
| `contextra-store` | `collection/mod.rs` | `LsmStorage`, `SystemPressure` |
| `contextra-store` | `contextra_impl/collections.rs` | `LsmStorage`, `tenant_codec` |
| `contextra-store` | `contextra_impl/crud.rs` | `LsmStorage` |
| `contextra-store` | `contextra_impl/lifecycle.rs` | `LsmConfig`, `LsmStorage` |
| `contextra-store` | `lib.rs` | `LsmStorage`, `lsm`, `tenant_codec` |
| `contextra-store` | `merge_operator_adapter.rs` | `MergeOperator` |
| `contextra-text` | `adapters/text.rs` | `inverted` |
| `contextra-text` | `collection/mod.rs` | `Language`, `inverted` |
| `contextra-text` | `lib.rs` | `Language` |
| `contextra-vector` | `backends.rs` | `HnswIndex` |
| `contextra-vector` | `background_workers/config.rs` | `hnsw` |
| `contextra-vector` | `background_workers/orphan_workers.rs` | `hnsw` |
| `contextra-vector` | `collection/mod.rs` | `HnswIndex` |
| `contextra-vector` | `collection/query_builder/builder_exec.rs` | `acorn` |
| `contextra-vector` | `contextra_impl/collections.rs` | `HnswConfig` |
| `contextra-vector` | `lib.rs` | `HnswIndex` |
| `contextra-vector` | `transaction/db_transaction.rs` | `HnswIndex` |

### 2.3 Öffentliche Re-Exports fremder Implementierungstypen (lib.rs)
- `pub use contextra_ports::TextEmbeddingEngine;`
- `pub use contextra_ports::VectorDeleteMode;`
- `pub use contextra_ports::DriftStatusProvider;`
- `pub use contextra_types::DistanceMetric;`
- `pub use contextra_store::lsm::DurabilityMode;` (M3-Leak, inventarisiert)
- `pub use contextra_text::Language;`
- `pub use contextra_checkpoint;`

---

## 3. Blockierte Nicht-Ziele (gemäß Auftrag)
- `temporal_filter.rs` nach `contextra-rank` verschieben (S-33, Eigentum rank)
- `decay_controller.rs` nach `contextra-adapt` verschieben (S-60, Eigentum adapt)
- Lizenz-Gating nach `contextra-license` verschieben (S-64, braucht S-50)
- `VectorDeleteMode` aus `contextra-vector` (S-50)
- `contextra_impl/erasure.rs` (Löschlogik wird nicht verschoben)
- Entfernen der Öffnungen `pub use contextra_store::lsm::DurabilityMode` (Verbraucher nutzen sie)

---

## 4. API-Freeze & Fremdnutzer
Alle bestehenden Signaturen von `MarkdownChunker`, `Collection`, `Contextra`, `ExportDocumentV1` etc. bleiben quellkompatibel.

| Item | Datei:Zeile | Externe Verbraucher | Kompatibilitätsnachweis |
|---|---|---|---|
| `MarkdownChunker` | `src/chunker/mod.rs:36` | `contextra`, `contextra-mcp` | unverändert |
| `Collection` | `src/collection/mod.rs:95` | `contextra-db`, `contextra-cognition` | unverändert |
| `Contextra` | `src/lib.rs:366` | `contextra`, `benchmarks/contextra-bench` | unverändert |
| `ExportDocumentV1` | `src/export.rs:25` | `contextra` | unverändert |

---

## 5. Charakterisierungs- und Orakel-Tests (EN6.1)
- Golden-Datei `tests/golden/export_golden_v1.json` und Prüfsumme `tests/golden/MANIFEST.sha256` mit SHA256 `81be10cfcdb1f18c65dab60d33f73fcd89436914dd13e1cd5467405579fd9ea3` angelegt.
- Charakterisierungstests `tests/characterization_consistency_and_determinism.rs` implementiert:
  1. Golden-File SHA256-Integrität und Deserialisierbarkeit gegen SCHEMA_VERSION_V1.
  2. RRF-Rangfusion Tie-Breaking Determinismus (INV-5: aufsteigende ID-Sortierung bei identischem Score).

---

## 6. Chunker als Adapter über Ingest (EN6.2)
- **Status:** blockiert durch Parität / architekturelle Richtlinien.
- **Befund & Abweichungsanalyse:**
  1. `crates/contextra-ingest/AGENTS.md` verbietet ausdrücklich: *„Kein [K]-Crate darf ingest importieren (P30, E34)“*. Ein Import von `ingest` als normale Abhängigkeit in `engine` würde die Architekturregel P30 verletzen.
  2. Schnittstellen-Divergenz: `contextra-engine::chunker::MarkdownChunker` arbeitet direkt auf rohem Markdown (`&str`) und liefert `Vec<contextra_types::ContextChunk>`. `contextra-ingest::chunking::prose` arbeitet auf `DocumentIr` und `CanonicalText` und liefert `Vec<ChunkDraft>`.
  3. SPEC-v2 §5.4 sieht als finale Lösung die Extraktion eines autarken Crates `contextra-chunker` vor.
  4. Gemäß Vorgabe EN6.2 wird bei Nicht-100%-Parität bzw. Schnittstellenabweichung kein Umbau B durchgeführt; der bestehende Code bleibt stabil und funktional; der Status wird als „blockiert durch Parität“ geführt.

---

## 7. Extraktion hinter Feature & Vergleich mit contextra-graph (EN6.3)
- **Status:** grün.
- Primäres Feature `exp-extraction` eingeführt, `entity-extraction = ["exp-extraction"]` als stabiles Abwärtskompatibilitäts-Alias beibehalten.
- Feature ist nicht default (`default = []`).

### Vergleich: `engine/extraction/` vs. `graph/entity_extraction.rs`
| Eigenschaft | `engine/extraction/` | `graph/entity_extraction.rs` |
|---|---|---|
| Modul-Zweck | Orchestrierungs-Pipeline für OpenIE und Rule-Based Triples | Regelbasierte Extraktion von Entitäten und Relationen direkt für CsrGraph/Hyperedges |
| Primäre Typen | `EntityExtractor` (Trait), `ExtractedTriple`, `OpenIeExtractor`, `RuleBasedExtractor` | `RuleBasedEntityExtractor`, `ExtractedRelation`, `ExtractionPattern` |
| Ausgabestruktur | `ExtractedTriple { subject, predicate, object, confidence }` | `ExtractedRelation { source_name, target_name, relation_type, confidence }` |
| Graph-Kopplung | Entkoppelt über Strings/Triples; persistiert via `Collection::relate` | Direkt an `CsrGraph`, `HyperEdge`, `RoleBinding` gebunden |
| Entscheidung | Keine Zusammenlegung (gemäß SPEC §6.3 / EN6.3 strikte Trennung und Dokumentation der Namens-/API-Differenz). |

---

## 8. Feature-Hygiene Audit (EN6.4)

| Feature-Name | Manifest-Deklaration | cfg-Nutzung in src & tests | Externe Weiterleitung / Verbraucher | Entscheidung |
|---|---|---|---|---|
| `onnx` | `contextra-infer-onnx/onnx` | ja (lib.rs, tests) | `contextra-db`, `contextra` | Beibehalten (aktiv) |
| `sandbox` | `contextra-sandbox` | ja (merge_operator_adapter.rs, lib.rs) | `contextra-db` | Beibehalten (aktiv) |
| `reranking` | `[]` | ja (hybrid_search_tests.rs:108) | `contextra-db` leitet weiter (`cargo check -p contextra-db --features reranking` in CI) | Beibehalten (durch CI/Verbraucher gebunden) |
| `background-maintenance` | `[]` | ja (lib.rs:77) | `contextra-db` | Beibehalten (aktiv) |
| `graph-connectivity-health`| `contextra-graph/graph-connectivity-health` | ja (lib.rs:88) | `contextra-db`, `contextra-cognition` | Beibehalten (Weiterleitung) |
| `edge-reinforcement-learning`| `[]` | keine in engine | `contextra-cognition` leitet weiter | Beibehalten (Verbraucherschutz SPEC B14) |
| `adaptive-candidate-pool-sizing`| `contextra-adapt` | ja (builder.rs, lib.rs) | `contextra-db` | Beibehalten (aktiv) |
| `exp-extraction` | `[]` | ja (lib.rs, auto_extraction.rs, tests) | Neu (SPEC §6.3) | Beibehalten (aktiv) |
| `entity-extraction` | `["exp-extraction"]` | ja (tests) | Abwärtskompatibles Alias | Beibehalten (Alias) |
| `auto-extraction-opt-out` | `[]` | ja (auto_extraction.rs) | Intern in engine | Beibehalten (aktiv) |
| `experimental-diskann` | `contextra-vector/experimental-diskann` | ja (dev-deps, tests) | `contextra-db` | Beibehalten (aktiv) |

---

## 9. Nebenläufigkeit, Sperrenordnung und Monotonie (EN6.5)
- **Sperrenordnung:**
  `collections (RwLock) -> kv_locks -> embedder (RwLock)`.
  Auditiert über `concurrency_lock_auditor` und manuelle Quelltextinspektion:
  In `contextra_impl/collections.rs` werden Guards explizit via `drop(read_guard)` freigegeben, bevor nachfolgende Operationen ausgeführt werden.
  In `contextra_impl/lifecycle.rs` werden RwLock-Schreib- und Lese-Guards vor Aufrufer-Rückgaben freigegeben.
- **TxId Monotonie:**
  Test `tests/tx_allocation_concurrency.rs` belegt streng monotone, kollisionsfreie Allokation unter hoher Thread-Nebenläufigkeit (8 Threads x 5.000 Operationen).
- **CPU-lastige Operationen:**
  Embedding-Berechnungen und Vektor-Indexierungsphasen delegieren an synchron isolierte bzw. thread-sichere Backends.

---

## 10. Panic-Freiheit, Determinismus und Wortwahl (EN6.6)
- **Forbidden Wording:** Alle 3 Fundstellen („garantiert“) in `checkpoint.rs`, `search/mod.rs` und `lifecycle.rs` neutralisiert. `pattern_diagnostic --category FORBIDDEN_WORDING` meldet 0 Fundstellen.
- **Determinismus:** Nicht-deterministische `HashSet`-Instanzen auf Traversierungs- und Supersessions-Pfaden (`query.rs`, `crud/links.rs`, `hybrid/mod.rs`) durch `BTreeSet` ersetzt.

---

## 11. Vorher/Nachher-Messwerte & Bilanz

| Metrik | Vorher (EN6.0) | Nachher (EN6.7) | Delta / Bemerkung |
|---|---|---|---|
| LOC (`tools/loc.py`) | 13.333 | 13.333 | identisch |
| Verbotene Wortwahl (`pattern_diagnostic`) | 3 | 0 | -3 (vollständig behoben) |
| Determinismus-Schwachstellen (HashSet) | 22 | 19 | -3 (auf Traversierungs-/Ergebnispfad auf BTreeSet umgestellt) |
| Features | 10 | 11 | +1 (`exp-extraction` eingeführt, `entity-extraction` als Alias) |
| Interne Abhängigkeiten (Cargo) | 12 | 12 | Unverändert im Kern (`contextra-ingest` nur in Dev-Dependencies) |

---

## 12. Koordinationsanforderungen (für Integrationsschritt)
- `crates/contextra-store`: Behebung von E0201/E0046/E0509/E0599 im Store-Crate (blockiert externe Workspace-Tests).
- `crates/contextra-db`: Nach Vorliegen von `exp-extraction` kann das Feature-Mapping bei Bedarf angepasst werden.

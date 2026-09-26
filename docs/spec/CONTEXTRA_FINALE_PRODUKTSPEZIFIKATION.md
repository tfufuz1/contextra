# Contextra — Allumfassende Master-Spezifikation v6 (vollständig)

**Verifikationsgrundlage:** Live-Klon HEAD `2028cd0f756a56863fabc6bd9b04803875a7f004` · Repository
`https://github.com/tfufuz1/contextra` · Klon-Zeitpunkt 26. September 2026.
**Vorgänger-Fassungen:** ersetzt `CONTEXTRA_MASTER_SPEC_v5.md` (Basis `b28d5186`) vollständig; konsolidiert
zusätzlich `CONTEXTRA_MASTER_ROADMAP.md`, `Contextra_SOTA_Forschungsbericht_2025-2026.md` (beide ebenfalls
Basis `b28d5186`) und die zuvor entworfenen Teile A/B dieses Dokuments (Basis `2028cd0f`, unverändert
übernommen bis auf die in **C.0** benannten Korrekturen).
**Normativität:** Bei Widerspruch zwischen diesem Dokument und dem tatsächlichen Quellcode gilt
**ausschließlich der Quellcode**. Dieses Dokument ist dann zu korrigieren. Innerhalb dieses Dokuments gilt bei
Widerspruch zwischen Teilen: **Teil C schlägt Teil A/B** (spätere, gegen denselben HEAD nachverifizierte
Korrektur), **Teil A schlägt Teil B** (IST vor SOLL-Annahme), **Anhang E ist Referenzdatenbank**, kein
eigenständiger normativer Text.
**Reifekennzeichnung:** 🟢 produktiv, verifiziert · 🟡 Grundgerüst vorhanden, unvollständig · 🔴 spezifiziert,
nicht implementiert · 🔒 closed-source, lizenzpflichtig.
**Zweck:** Dieses Dokument beantwortet jede Implementierungsfrage eines LLM-Agenten oder Contributors zu
Contextra selbstständig — ohne Rückgriff auf externe Dokumente. Es trennt strikt den **IST**-Zustand (Teil A,
knapp, code-verifiziert) vom **SOLL**-Zustand (Teil B, vollständige Mikro-Interface-Spezifikationen),
konsolidiert beide in Teil C, priorisiert in Teil D und stellt in Anhang E die vollständigen Referenztabellen
bereit (Trait-Kataloge, Fehlertaxonomie, Abhängigkeitsmatrix, Quellenkonflikt-Auflösung).

---

## Inhaltsverzeichnis

**Teil A — IST-Zustand (code-verifiziert, knapp)**
A.1 Produktthese und Systeminvarianten (Referenz) · A.2 Ring-Architektur und Abhängigkeitsmatrix ·
A.3 Crate-Inventar (35 Mitglieder) · A.4 Schnittstellenkatalog `contextra-ports` (28 Traits) ·
A.5 Fehlertaxonomie IST · A.6 MCP-Werkzeug-Inventar · A.7 CI/xtask-Gate-Inventar ·
A.8 Governance-Artefakte · A.9 Verifikationsprotokoll

**Teil B — SOLL-Zustand: Mikro-Interface-Spezifikationen**
B.0 Spezifikationskonvention · B.1 P0-Sofortmaßnahmen · B.2 Phase 1 (`fast`-Ring-Härtung) ·
B.3 Phase 1½ (SOTA im ShadowMode) · B.4 Phase 2 (Sovereign/Compliance) ·
B.5 Phase 3 (Verticals/Explainability) · B.6 Phase 4 (Skalierung)

**Teil C — Konsolidierung, Korrekturen, Nachträge** *(neu in v6)*
C.0 Korrekturen und Voraussetzungen zu Teil A/B · C.1 Konsolidierte neue `ContextraError`-Varianten ·
C.2 Konsolidierte neue Invarianten · C.3 SOLL-Erweiterung `contextra-ports`-Trait-Katalog ·
C.4 Neue `xtask`-CI-Gates (konsolidiert) · C.5 Nachtrag: fehlender SOTA-Punkt (`sketched-bandit-projection`) ·
C.6 Ring-Zuordnung neuer und korrigierter Crates

**Teil D — Gesamt-priorisierte Roadmap** *(neu in v6)*

**Anhang E — Referenztabellen** *(neu in v6)*
E.1 Vollständige `ContextraError`-Liste · E.2 Vollständiger `contextra-ports`-Trait-Methodenkatalog (IST) ·
E.3 `capabilities.toml` may_depend_on-Matrix (35 Crates) · E.4 Quellendokument-Konfliktauflösung

**Schlusswort**

---

# Teil A — IST-Zustand (code-verifiziert, knapp)

## A.1 Produktthese und Systeminvarianten (Referenz)

Contextra ist eine souveräne, lokal betriebene Gedächtnis- und Ausführungsschicht für KI-Agenten in Pure Rust
(kein GC, Zero-Panic, Air-Gap-fähig). Die vier Differenzierungsdimensionen, Performance-Zielwerte,
Deployment-Formen und Nicht-Ziele sind in v5 §1 vollständig und unverändert normativ — dieses Dokument
übernimmt sie ohne Änderung. Ebenso unverändert: die sieben Systeminvarianten INV-1 bis INV-7 sowie die
Zusatzregeln INV-TENANT-1 bis INV-MANIFEST-COMPLETE (v5 §2.2–§2.3). Teil B dieses Dokuments **erweitert** die
Invariantentabelle um sieben neue, aus den Backlog-Punkten abgeleitete Invarianten (siehe C.2).

**Tragende Prinzipien P1–P30** (v5 §2.1) bleiben normativ; insbesondere P24 (Lokalität), P26 (Ring-0
synchron), P27 (jeder Port = `dyn`-kompatibler Trait), P28 (injizierter Determinismus), P29 (kein globaler
veränderlicher Zustand) sind die vier Prüfsteine, gegen die jede Schnittstelle in Teil B verifiziert wird.

## A.2 Ring-Architektur und Abhängigkeitsmatrix

Das Fünf-Ring-Modell (v5 §3) ist unverändert gültig und wurde gegen `capabilities.toml` (Schema-Version 2,
402 Zeilen, ein `[crates.X]`-Block pro Workspace-Mitglied) sowie gegen die tatsächlichen `Cargo.toml`-
Abhängigkeitskanten verifiziert:

```
Ring 0  Foundation & Pure Numerics/Types       [synchron, kein tokio]
        types, ports, mvcc, wire, sys, simd, crypto, vector, text, graph, rank, adapt, core
Ring 1  Persistence & Cache                    [async I/O an Grenzen erlaubt]
        store, kvcache, checkpoint
Ring 2  External Adapters & Sandboxing         [Blätter, keine Rückwärtskanten]
        infer-candle, infer-ollama, infer-onnx, sandbox
Ring 3  Orchestration & Cognition              [Anwendungskern]
        engine, db (Legacy), cognition, privacy, router, agent
Ring 4  Boundary & Composition Root            [Fassaden, FFI, MCP]
        contextra, mcp, py
Tooling (kein Produktionscode, dev-only)
        testkit, bench, xtask
Compliance-Utilities (Ring 4, kein Kern-Datenpfad — KORRIGIERT ggü. Vorfassung, siehe C.0 Punkt 2)
        audit-export, avv-generator, license
```

Die CI-Erzwingung (`tests/layering.rs`, `deny.toml`, `xtask check-ring-layering-full`,
`check-ring0-async-purity`, `check-duplicate-core-primitives`, `check-unsafe-islands`,
`check-manifest-completeness`) ist gegen HEAD `2028cd0f` als vorhanden und aktiv verifiziert (§A.9,
Prüfschritt 7).

## A.3 Crate-Inventar — alle 35 Workspace-Mitglieder

Verifiziert gegen `Cargo.toml` `[workspace] members` (34 Einträge, davon 33 unter `crates/` und 1 unter
`benchmarks/contextra-bench`) plus `xtask` (`exclude`d, eigener Cargo-Workspace-Member). Reifegrad, Zweck und
Schlüsseldateien sind identisch zu v5 §4 — hier nur als kompaktes Referenz-Inventar für die Kreuzverweise in
Teil B. **Ring-Spalte korrigiert ggü. Vorfassung für #14–16** (siehe C.0 Punkt 2: `capabilities.toml` weist
diesen drei Crates explizit `ring = 'Ring 4'` zu, nicht Ring 0):

| # | Crate | Ring | Reife | Zweck (1 Zeile) |
|---|---|---|---|---|
| 1 | `contextra-types` | 0 | 🟢 | Kanonische ID-/Fehler-/Enum-Typbasis |
| 2 | `contextra-ports` | 0 | 🟢 | Hexagonale Trait-Grenzen (28 Traits, siehe A.4) |
| 3 | `contextra-mvcc` | 0 | 🟢 | MVCC-Primitive (`SeqLog`, `SnapshotRegistry`, `TxBuffer`) |
| 4 | `contextra-wire` | 0 [Insel] | 🟢 | FlatBuffers-IPC, Zero-Copy-Adapter |
| 5 | `contextra-sys` | 0 [Insel] | 🟢 | OS-Abstraktionen (mmap, mlock, ACL) |
| 6 | `contextra-simd` | 0 [Insel] | 🟢 | SIMD-Kernels (AVX2/AVX-512/NEON/Scalar) |
| 7 | `contextra-crypto` | 0 [Insel] | 🟢 | AES-256-GCM-SIV, Argon2id, Ed25519-Löschbeweis |
| 8 | `contextra-vector` | 0 | 🟢 | HNSW + DiskANN, ACORN, RaBitQ; **INV-DELETION-2 offen (P0)** |
| 9 | `contextra-text` | 0 | 🟢 | BM25F/WAND, deutsche Morphologie |
| 10 | `contextra-graph` | 0 | 🟢 | CSR, PPR/Forward-Push, TL-HFD (ShadowMode), Hyperkanten, Leiden |
| 11 | `contextra-rank` | 0 | 🟢 | RRF/DiBud-Fusion, Isotonic/Platt-Kalibrierung, `contextra_explain`-Backend |
| 12 | `contextra-adapt` | 0 | 🟡 | Bandit-Mathematik (LinUCB, FC-TS), Lyapunov-Drift, PID |
| 13 | `contextra-core` | 0 | 🟢 (Legacy) | Re-Export-Fassade über types/ports/mvcc/wire — Abbaupfad läuft |
| 14 | `contextra-audit-export` | **4** | 🟡 | BSI-Mapping, Art.-30-Verzeichnis |
| 15 | `contextra-avv-generator` | **4** | 🟡 | AVV-Vorlage Art. 28 DSGVO |
| 16 | `contextra-license` | **4** | 🟡 (Stub) | `LicenseGate`-Trait, `OpenFastGate`; Sovereign-Gate fehlt; **beherbergt aktuell auch `FeatureRing` — Migrationsbedarf, siehe C.0 Punkt 1** |
| 17 | `contextra-store` | 1 | 🟢 | LSM-Tree, WAL-Group-Commit, MANIFEST, KvKeyLocks; **DurabilityMode fehlt (P0)** |
| 18 | `contextra-kvcache` | 1 | 🟢 | Verschlüsselter KV-Cache, Radix-Präfix-Baum; KIVI/SnapKV nur Metadaten |
| 19 | `contextra-checkpoint` | 1 | 🟢 | Time-Travel-Registry, Blake3-Manifest |
| 20 | `contextra-infer-candle` | 2 | 🟢 | GGUF-Inferenz via Candle, GASP-Validator; KV-Cache-Bridge Platzhalter |
| 21 | `contextra-infer-ollama` | 2 | 🟢 | HTTP-Client für Ollama, Contextual-Chunk-Prefixing |
| 22 | `contextra-infer-onnx` | 2 | 🟡 | ONNX-Cross-Encoder-Reranking; **nicht in `default-members`** |
| 23 | `contextra-sandbox` | 2 | 🟢 | WASM-Ausführungsisolation, Fuel+Wall-Clock-Budget |
| 24 | `contextra-engine` | 3 | 🟢 | Collection-CRUD, 2PC-Transaktionen; **AutoExtraction default disabled (P0)** |
| 25 | `contextra-db` | 3 | 🟢 (Legacy) | Monolithische Übergangsfassade — Abbaupfad läuft |
| 26 | `contextra-cognition` | 3 | 🟢 | Konsolidierung, LeanRAG-Aggregation, Session-Kompaktierung |
| 27 | `contextra-privacy` | 3 | 🟢 | Egress-Gateway (5-Schichten), PII-Vault, `TenantScoped<T>` |
| 28 | `contextra-router` | 3 | 🟢 | `ArmRegistry`, FC-TS-Dispatch; **Produktionscode (`[dependencies]`) ist bereits `contextra-db`-frei — Korrektur ggü. Vorfassung, siehe C.0 Punkt 3: `contextra-db` erscheint nur unter `[dev-dependencies]` (Tests/Benchmarks)** |
| 29 | `contextra-agent` | 3 | 🟢 | Workflow-Engine, Audit-Pipeline, DLQ |
| 30 | `contextra` | 4 | 🟢 | Öffentliche Composition Root (Builder-Pattern) |
| 31 | `contextra-mcp` | 4 | 🟢 (Migration) | stdio-JSON-RPC-Server, 13 Tools (siehe A.6) |
| 32 | `contextra-py` | 4 | 🟢 | PyO3-FFI-Bindings, separater Cargo-Workspace |
| 33 | `contextra-testkit` | Tooling | 🟢 | Fault-VFS, In-Memory-Store, `ManualClock` |
| 34 | `contextra-bench` | Tooling | 🟢 | LoCoMo/LongMemEval-Harness, ANN-/BEIR-Benchmarks; **nicht in `capabilities.toml`, aber explizit exempted (`EXEMPT_CRATES`, siehe C.0 Punkt 4) — kein INV-MANIFEST-COMPLETE-Verstoß** |
| 35 | `xtask` | Tooling | 🟢 | 61 Quelldateien, ≥ 50 CI-Gates (siehe A.7); ebenfalls `EXEMPT_CRATES` |

**Datei-/Modulstruktur-Korrekturen ggü. v5 (gegen HEAD `2028cd0f` grep-verifiziert, identisch zur
Korrekturtabelle in `Contextra_SOTA_Forschungsbericht_2025-2026.md` §1.2):** `contextra-graph` hat kein
`ppr.rs` mit Forward-Push-Kern — dieser liegt in `path_rag/mod.rs::forward_push_ppr`; `ppr.rs` enthält nur
`PprContext`/`DeletedView`. `contextra-rank` hat kein `fusion.rs`/`calibration.rs` als Einzeldatei, sondern
Verzeichnisse `fusion/{rrf,normalized,resonance,provenance,topk,signal,types}.rs` und
`calibration/{isotonic,platt}.rs`. `contextra-cognition` hat kein `consolidation.rs`/`leanrag.rs`, sondern
`consolidation_executor.rs`, `memory_consolidation.rs`, `aggregation_phase.rs`, `synthesis_phase.rs`,
`leanrag_input.rs`, `semantic_aggregation_facade.rs`. Alle Codeverweise in Teil B dieses Dokuments nutzen
ausschließlich die **tatsächlichen**, hier verifizierten Pfade.

## A.4 Zentraler Schnittstellenkatalog `contextra-ports` (28 Traits, IST)

`contextra-ports` ist der hexagonale Kern des Systems (P27: „Jeder Port = `dyn`-kompatibler Trait"). v5 §4.1
nennt nur eine Auswahl von zwölf Traits; die tatsächliche Zahl (verifiziert via
`grep -rn "^pub trait " crates/contextra-ports/src/*.rs`) ist **28**, verteilt über 16 Module. Vollständiger
Katalog (Kurzsignaturen; volle Methodenlisten siehe **Anhang E.2**):

| Modul | Traits | Sync/Async | Zweck |
|---|---|---|---|
| `storage.rs` | `StorageRead`, `StorageWrite`, `StorageEngine` | sync + `BoxFuture` | Byte-orientierte KV-Persistenz-Grenze |
| `vector_index.rs` | `VectorIndex`, `HybridSearchProvider` | async | HNSW/DiskANN-Zugriffsgrenze |
| `text_index.rs` | `TextEmbeddingEngine`, `SegmentSynthesizer`, `TextIndex` | sync/async gemischt | BM25F/Inverted-Index-Grenze |
| `graph.rs` | `GraphCollectionMutation` | async | Graph-Schreibgrenze (Kanten/Hyperkanten) |
| `graph_index.rs` | `GraphIndex`, `CommunityResolver` | sync | CSR-Lese-/Community-Grenze |
| `kv.rs` | `KvPrefixStore` | async | KV-Cache-Präfix-Zugriff |
| `kv_bridge_port.rs` | `KvBridgeStorage` | async | **Einziger** Ring-2→Ring-1-Zugang für KV-Cache (Dependency-Inversion) |
| `embedding.rs` | `EmbeddingProvider`, `TextGenerator`, `LlmTextGenerator`, `LlmTextGeneratorStreaming` | async | Embedding-/LLM-Generierungsgrenze |
| `lifecycle.rs` | `DistanceCalculator`, `MemoryLifecycleManager`, `GroundingValidator`, `ResponseGroundingValidator`, `ContextPreparer` | sync/async gemischt | Distanzmetrik, Speicher-Lebenszyklus, Grounding-Prüfung |
| `checkpoint.rs` | `Checkpoint`, `CheckpointCoordinator`, `Snapshot` | sync | Time-Travel-Grenze |
| `clock.rs` | `Clock` | sync | P28-Determinismus (nie `SystemTime::now()` direkt) |
| `rng.rs` | `Rng` | sync | P28-Determinismus (nie `thread_rng()` direkt) |
| `id_gen.rs` | `IdGen` | sync | Deterministische ID-Vergabe |
| `metrics.rs` | `MetricsSink` | sync | Observability-Grenze |
| `observability.rs` | `DriftStatusProvider` | sync | Bandit-/Drift-Statusgrenze |

**Fehlend in `contextra-ports` (bestätigt 0 Treffer, Ziel für Teil B/C.3):** `AttentionExporter`,
`HiddenStateExporter`, `PluginManifest`, `LicenseGate` (existiert nur als reduzierter Trait direkt in
`contextra-license`, nicht in `contextra-ports` — **P27-Verstoß**, siehe C.0 Punkt 1 und C.3),
`ConformalCalibrator`, `KPathDiffusion`, `SyncTransport`.

Alle 28 Traits sind über `#[cfg(test)] mod dyn_safety` gegen `dyn`-Kompatibilität abgesichert (13 der 28
explizit assert-getestet; die verbleibenden 15 sind durch `Send + Sync + 'static`-Bounds strukturell
dyn-fähig, aber nicht separat im Assert-Block gelistet — Lücke, siehe C.4 neues Gate
`check-dyn-safety-completeness`).

## A.5 Fehlertaxonomie IST (`ContextraError`, 35 Varianten)

`crates/contextra-types/src/error.rs` (905 Zeilen), `#[non_exhaustive]`, verifiziert **35**
`#[error(...)]`-annotierte Varianten (v5 nennt „≥ 30" — präzisiert). Kategorien (Kommentar-Header im
Quellcode): *Core & Logic*, *I/O & Storage*, *Concurrency & Transactions*, *Vector/Embedding*, *Text*,
*Sandbox*, *Serialization*, *Policy/Limits*. Vollständige Variantenliste inkl. Felder: **Anhang E.1**. Die
bindende Regel „kein neues workspace-weit propagierendes Fehler-Enum außerhalb `ContextraError`" (v5 §13)
gilt unverändert; Teil C.1 spezifiziert die für Teil-B-Features nötigen **neuen** Varianten.

## A.6 MCP-Werkzeug-Inventar IST (13 Tools)

Verifiziert gegen `crates/contextra-mcp/src/server_dispatch.rs` (Tool-Manifest-Block + Dispatch-Match): **13
von 14** in v5 gelisteten Tools existieren produktiv; `contextra_plugin_status` ist bestätigt abwesend (0
Treffer für `"contextra_plugin_status"` im gesamten `contextra-mcp`-Crate) — Ziel für B.2.4.

| Tool | Status | Dispatch-Pfad |
|---|---|---|
| `contextra_search`, `contextra_insert`, `contextra_upsert`, `contextra_get`, `contextra_delete`, `contextra_forget`, `contextra_collections`, `contextra_create_collection`, `contextra_drop_collection`, `contextra_consolidate`, `contextra_cloud_query`, `contextra_relate`, `contextra_relate_n_ary` | 🟢 | `server_dispatch.rs` Haupt-Match-Arm |
| `contextra_explain` | 🟢 (neu, HEAD-nah) | eigener Dispatch-Zweig mit `execute_with_timeout("contextra_explain", …)` — separat behandelt, da `RetrievalExplanation` (aus `contextra-rank/src/explain.rs`) eine andere Antwortform als die übrigen Tools hat |
| `contextra_plugin_status` | 🔴 (B.2.4) | nicht vorhanden |

Sicherheitsgrenzen (`MAX_SEARCH_QUERY_BYTES`, `MAX_RPC_BYTES`, `MAX_VOLATILE_RESULTS`,
`MAX_VOLATILE_OUTPUT_BYTES`, `MAX_SEARCH_K`) unverändert zu v5 §4.5.

## A.7 CI/xtask-Gate-Inventar IST (61 Module)

`xtask/src/` enthält **61** `.rs`-Dateien (v5 nennt „über 40 CI-Gates" — die tatsächliche Modulzahl ist
höher, da mehrere Module Hilfsfunktionen statt eigenständiger Gates sind; die Zahl der über
`xtask/src/main.rs` erreichbaren Subcommands liegt bei ca. 50). Neu ggü. v5-Aufzählung identifizierte
Gate-Module: `check_action_pinning`, `check_audit_duplication`, `check_audit_tool_evidence`,
`check_audit_verdict_independence`, `check_commit_diff_integrity`, `check_commit_messages`,
`check_coverage_gate`, `check_doc_references`, `check_duplicate_intent`, `check_duplicate_symbols`,
`check_duplicate_symbols_cross_file`, `check_jules_context_freshness`, `check_max_results_unbound`,
`check_phantom_files`, `check_placeholder_refs`, `check_toctou_trait_defaults`, `check_type_registry`,
`check_workflow_commands`, `jules_preflight`, `jules_submit_gate`, `reproducible_build.rs` (Letzteres ist
bereits ein **Grundgerüst** für den in v5-Backlog-Punkt 21 als 🔴 geführten Punkt „Reproduzierbare Builds" —
Korrektur für B.6.1). Governance rund um den KI-Agenten-gestützten Entwicklungsprozess (`.jules/`-Verzeichnis,
von der Wettbewerbsanalyse als „Einzelautor mit KI-agentengestützter Entwicklung (Jules-Task-Runner)"
charakterisiert) ist in `xtask` bereits mit eigenen Gates (`check_jules_context_freshness`, `jules_preflight`,
`jules_submit_gate`) abgesichert — dies ist in keinem der vier Quelldokumente erwähnt und wird als Ergänzung
in A.9 protokolliert. **Ergänzung (verifiziert, C.0 Punkt 4):** `check_manifest_completeness.rs` definiert
zusätzlich eine explizite `EXEMPT_CRATES`-Ausnahmeliste (`contextra-bench`, `xtask`) für
INV-MANIFEST-COMPLETE — dies löst den scheinbaren Widerspruch auf, dass `contextra-bench` keinen
`[crates.X]`-Block in `capabilities.toml` besitzt (33 Blöcke + 2 exemptierte = 35 Workspace-Mitglieder,
rechnerisch konsistent).

## A.8 Governance-Artefakte IST (95 ADRs, `capabilities.toml`)

`docs/decisions/` enthält **95** ADR-Dateien (Architecture Decision Records) — deutlich mehr als die
vereinzelten ADR-Verweise in v5 (`ADR-015`, `ADR-033`, `ADR-038`, `ADR-045`, `ADR-056`, `ADR-064`, `ADR-079`,
`ADR-0XX-P0A`). `capabilities.toml` (Schema-Version 2, `spec = "v4"` im Header — ein Hinweis darauf, dass die
Manifest-Datei selbst noch gegen die Vorgänger-Spezifikationsversion referenziert und mit v6 nachzuziehen
ist, siehe C.0 Punkt 6) definiert pro Crate: `ring`, `maturity`, `description`, `capabilities`, `path`,
`unsafe_island`, `may_depend_on`, `ci_class`, `spec`-Referenzen, `test`-Kommando. Dies ist die Grundlage für
`xtask check-manifest-completeness` (INV-MANIFEST-COMPLETE) und für **Anhang E.3** (vollständige
`may_depend_on`-Tabelle, jetzt aus dem tatsächlichen Dateiinhalt geparst statt nur beschrieben).

## A.9 Verifikationsprotokoll dieser Fassung

Folgende Kernaussagen der Quelldokumente wurden gegen HEAD `2028cd0f` per `grep`/`view`/Python-Parsing
nachverifiziert (Methodik: gezielte Suche nach den zitierten Symbolen/Dateien, keine erschöpfende
Vollprüfung):

| # | Geprüfte Aussage | Ergebnis |
|---|---|---|
| 1 | Workspace hat 34 Member + `xtask` = 35 Crates | ✅ bestätigt (`Cargo.toml`, `ls crates/`) |
| 2 | `ContextraError` hat `#[non_exhaustive]`, ≥ 30 Varianten | ✅ bestätigt, exakt 35 (Volltext Anhang E.1) |
| 3 | `RetrievalStrategy` hat genau 4 Varianten (Vector/Text/Graph/Hybrid) | ✅ bestätigt |
| 4 | `AutoExtractionConfig::enabled` Default `false` | ✅ bestätigt (Doc-Kommentar + fehlender expliziter `impl Default`-Override) |
| 5 | `contextra_explain` MCP-Tool implementiert | ✅ bestätigt (`explain.rs`, `server_dispatch.rs`) |
| 6 | `contextra_plugin_status` MCP-Tool **nicht** implementiert | ✅ bestätigt (0 Treffer) |
| 7 | `contextra-ports` hat mehr Traits als in v5 §4.1 aufgezählt | ✅ bestätigt — 28 statt ~12 |
| 8 | `INV-DELETION-2`/Ghost-Vector-Schutz nicht implementiert | ✅ bestätigt (0 Treffer für `remove_with_graph_repair`; `delete()` in `hnsw/vector_index_impl.rs` staged nur einen `IndexOp::Delete` im `TxBuffer`, keine synchrone Nachbarschaftsreparatur vor Commit) |
| 9 | `DurabilityMode`-Enum nicht implementiert | ✅ bestätigt (0 Treffer im gesamten Workspace; `LsmConfig` hat 8 Felder — `path`, `memtable_size_limit`, `max_ram_mb`, `tx_timeout`, `compaction`, `encryption_passphrase`, `group_commit_window_micros`, `block_cache_shards` — keines davon ein Durability-Enum) |
| 10 | `NullAttentionScoreSource` ist aktiver Default | ✅ bestätigt (`contextra-kvcache/src/attention_score.rs`, `AttentionScoreSource::importance_score` liefert dort stets `None`) |
| 11 | `contextra-graph`/`contextra-rank`/`contextra-cognition` Dateistruktur wie in SOTA-Bericht-Korrekturtabelle | ✅ bestätigt (identische Modulaufteilung) |
| 12 | Ring-0-Kern-`tombstone`-Mechanik existiert, aber keine garantierte Vor-Proof-Graph-Reparatur | ✅ bestätigt — `core_rebuild.rs` enthält eine `tombstoned_in_region`-Bereinigung, diese läuft aber als **Hintergrund-Rebuild-Pass**, nicht synchron vor `DeletionProof`-Ausstellung |
| 13 | `KvKeyLocks`-Hasher nutzt die vier in INV-HASHER-SEEDS benannten Konstanten | ✅ bestätigt, identisch in `contextra-store/src/kv_locks.rs` UND `contextra-engine/src/collection/kv_lock.rs`: `0x9E3779B97F4A7C15`, `0xBF58476D1CE4E5B9`, `0x94D049BB133111EB`, `0x2545F4914F6CDD1D` |
| 14 | `TenantId(0)` ist SYSTEM-reserviert, `try_new(0)` → `Err` | ✅ bestätigt wortwörtlich in `contextra-types/src/types/domain/ids.rs` |
| 15 | `capabilities.toml` hat 33 `[crates.X]`-Blöcke, nicht 35 | ✅ bestätigt — Differenz durch `EXEMPT_CRATES` erklärt (siehe A.7-Ergänzung) |
| 16 | `contextra-router` hängt produktiv von `contextra-db` ab | ❌ **widerlegt** — `contextra-db` erscheint in `contextra-router/Cargo.toml` ausschließlich unter `[dev-dependencies]`; `capabilities.toml`s `may_depend_on` für `contextra-router` enthält korrekt kein `contextra-db` (siehe C.0 Punkt 3) |
| 17 | `contextra-audit-export`/`avv-generator`/`license` liegen in „Ring 0" | ❌ **widerlegt** — `capabilities.toml` weist allen dreien explizit `ring = 'Ring 4'` zu (siehe C.0 Punkt 2) |
| 18 | `FeatureRing` ist in `contextra-types` definiert | ❌ **widerlegt** — `FeatureRing` (mit `#[non_exhaustive]`, Varianten `Fast`/`Sovereign`/`Compliance`) ist in `contextra-license/src/lib.rs` definiert, nicht in `contextra-types` (siehe C.0 Punkt 1, kritisch für B.1.2/B.2.3/B.4.1) |

**Ergänzender Befund (nicht in Quelldokumenten erwähnt):** `xtask` enthält bereits ein
`reproducible_build.rs`-Modul — v5-Backlog-Punkt 21 ("Reproduzierbare Builds … ❌ nicht begonnen") ist damit
auf 🟡 zu korrigieren (Grundgerüst vorhanden). Diese Korrektur ist in B.6.1 eingearbeitet.

---

# Teil B — SOLL-Zustand: Mikro-Interface-Spezifikationen

## B.0 Spezifikationskonvention

Jeder Punkt in B.1–B.6 folgt einem einheitlichen siebenteiligen Schema (verdichtete Form des in
`Contextra_SOTA_Forschungsbericht_2025-2026.md` §5.1–§5.4 etablierten neunteiligen Templates; die
höchstprioren P0-Punkte in B.1 erhalten die volle neunteilige Tiefe):

1. **IST/Problem** — knapper, code-verifizierter Ausgangszustand (Verweis Teil A)
2. **SOLL-Schnittstelle** — vollständige, kompilierfähige Rust-Signatur (Traits/Structs/Enums mit
   Doc-Kommentaren)
3. **Algorithmus & Komplexität** — Pseudocode oder Kernlogik mit Zeit-/Speicherkomplexität
4. **Invarianten-Nachweis** — Bezug zu P1–P30/INV-1–INV-7 und ggf. neuer Invarianten
5. **Fehlerbehandlung** — betroffene `ContextraError`-Varianten (neu oder wiederverwendet)
6. **Test-Pflichten** — konkrete Testdateien/-arten
7. **Migration, Feature-Flags, Aufwand, Priorität, Abhängigkeiten**

Jede Schnittstelle ist so spezifiziert, dass sie **ohne Rückfrage an den Auftraggeber** implementierbar ist —
Default-Werte, Grenzwerte und Fehlerpfade sind für jede neue Struktur explizit angegeben.

> **Hinweis (v6):** Wo die folgenden Spezifikationen `contextra_types::FeatureRing` referenzieren
> (B.1.2, B.2.3, B.4.1), ist dies der **Zielzustand nach** der in **C.0 Punkt 1** verbindlich beschriebenen
> Migration von `FeatureRing` aus `contextra-license` nach `contextra-types`. Diese Migration ist eine
> harte, in keiner Vorfassung benannte Vorbedingung und wird in Teil D als eigener, vorgezogener
> Backlog-Punkt (D, Prio P0-0) geführt.

---

## B.1 P0-Sofortmaßnahmen (vor jeder Erweiterung)

Diese fünf Punkte sind laut v5 §16 und ROADMAP §2 Voraussetzung für jede weitere Phase — sie untergraben
aktive Produktversprechen (Löschbeweis, Crash-Konsistenz, Wissensgraph-Vollständigkeit) und werden hier
vollständig ausgearbeitet.

### B.1.1 `INV-DELETION-2` — Ghost-Vector-Schutz im HNSW-Index

**1. IST/Problem.** `HnswIndex::delete()` (`crates/contextra-vector/src/hnsw/vector_index_impl.rs:230`)
staged ausschließlich einen `IndexOp::Delete { doc_id, data: None }` in den `TxBuffer`; die tatsächliche
Tombstone-Markierung erfolgt beim `commit()`. Eine **Nachbarschaftsreparatur** (Entfernen der `doc_id` aus
den Adjazenzlisten aller Nachbarknoten auf allen HNSW-Ebenen) existiert als Funktionalität in
`core_rebuild.rs` (`tombstoned_in_region`-Filterung, Zeilen ~500–571), läuft dort aber als
**Hintergrund-Rebuild-Pass für eine Region**, nicht garantiert synchron und vollständig **vor**
`DeletionProof::create_with_wal_receipt_v3()`. Damit ist das folgende Bedrohungsmodell offen: Nach
WAL-/KV-Crypto-Shredding eines Vektors kann ein Angreifer mit Lesezugriff auf den persistierten HNSW-Graphen
über verbliebene Nachbarschaftskanten (Level-0-Adjazenzlisten anderer, nicht gelöschter Knoten, die
weiterhin auf die gelöschte `DocId` verweisen) approximative Rückschlüsse auf Position/Nachbarschaft des
gelöschten Vektors im Einbettungsraum ziehen — der DSGVO-Art.-17-Beweis (`DeletionProof`) attestiert dann
eine physische Bereinigung, die für den Vektorindex nicht vollständig zutrifft. Dies ist die **einzige
derzeit offene Lücke** im ansonsten geschlossenen Löschbeweis-Pfad (WAL: 🟢 durch HMAC-Kette, KV-Segmente:
🟢 durch AEAD + Crypto-Shredding, Graph-CSR: 🟢 durch bitemporale Sichtbarkeit — nur der Vektorindex fehlt).

**2. SOLL-Schnittstelle.**

```rust
// crates/contextra-vector/src/hnsw/deletion.rs — NEUE DATEI

use crate::hnsw::{HnswIndex, NodeId};
use contextra_types::{ContextraError, DocId, Result};

/// Statistik einer graph-reparierenden Löschung — für Observability und
/// als Nachweis-Artefakt, das `DeletionProof::create_with_wal_receipt_v3()`
/// als Vorbedingung konsumiert (siehe INV-DELETION-2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletionStats {
    /// Anzahl der Ebenen, auf denen der Knoten Nachbarschaftskanten besaß.
    pub levels_touched: u8,
    /// Gesamtzahl reparierter Adjazenzlisten (über alle Ebenen).
    pub neighbor_lists_repaired: usize,
    /// Anzahl der Ersatzkandidaten, die nicht gefunden werden konnten
    /// (Knoten wird auf dieser Ebene isoliert statt ersetzt — siehe
    /// Algorithmus Schritt 3b). Muss im produktiven Pfad stets 0 sein,
    /// solange der Graph zusammenhängend bleibt; > 0 ist zulässig, aber
    /// wird über `MetricsSink` als Warnsignal exportiert.
    pub orphaned_replacements: usize,
    /// Verifikations-Ergebnis von Schritt 4 (Property-Check).
    pub verified_no_ghost_pointers: bool,
}

/// Fehlerform für die Ghost-Vector-Reparatur. `From<HnswDeletionError> for
/// ContextraError` wird über `ContextraError::Index(String)` realisiert
/// (kein neues propagierendes Top-Level-Fehler-Enum, INV-SINGLE-ERROR).
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum HnswDeletionError {
    #[error("doc_id {0:?} not present in index")]
    NotFound(DocId),
    #[error("graph became disconnected during repair at level {level}: node {node:?} has zero remaining neighbors and no replacement candidate")]
    DisconnectedRepair { level: u8, node: NodeId },
    #[error("verification step found {0} residual ghost pointer(s) after repair — repair aborted, tombstone NOT eligible for DeletionProof")]
    VerificationFailed(usize),
}

impl HnswIndex {
    /// Ersetzt den bisherigen zweistufigen Lösch-Pfad (`delete()` + separater
    /// Hintergrund-Rebuild) für alle Aufrufer, die eine `DeletionProof`-fähige
    /// Löschung benötigen (d. h. jeden Aufruf aus `contextra-engine`s
    /// `crud::delete`- und `crud::forget`-Pfaden).
    ///
    /// # Ablauf (siehe Algorithmus unten für Details)
    /// 1. Tombstone setzen (wie bisher, via `TxBuffer`/Commit).
    /// 2. Für jede Ebene `l` von 0 bis `node.max_level`: alle Knoten, die
    ///    `doc_id` als Nachbar führen, ermitteln (Rückwärtskanten-Traversal
    ///    über die bereits vorhandene Adjazenzliste — HNSW-Kanten sind
    ///    ungerichtet gespeichert, sodass keine zusätzliche Rückwärtsindex-
    ///    Struktur nötig ist).
    /// 3. Für jeden betroffenen Nachbarn: Nachbar-Slot durch besten
    ///    verbleibenden Kandidaten aus der ursprünglichen Kandidatenliste
    ///    der Ebene ersetzen (analog zum HNSW-Konstruktions-Heuristik-Schritt,
    ///    `core_insert.rs::select_neighbors_heuristic` wird wiederverwendet).
    /// 4. Verifikation: vollständiger Scan aller Adjazenzlisten des
    ///    betroffenen Sub-Graphen (durch `node_budget` begrenzt, s. u.) auf
    ///    Restvorkommen von `doc_id`.
    ///
    /// # Invarianten
    /// - INV-DELETION-2: Nach erfolgreicher Rückgabe (`Ok`) zeigt **kein**
    ///   Nachbarschaftszeiger auf keiner Ebene mehr auf `doc_id`.
    /// - INV-DELETION-1 (bestehend): `DeletionProof::create*()` darf **nur**
    ///   aufgerufen werden, wenn `DeletionStats::verified_no_ghost_pointers
    ///   == true`. Dies wird auf Typ-Ebene erzwungen (siehe Schritt 4 unten).
    /// - P7 Zero-Panic: alle Fehlerpfade liefern `Err`, kein `unwrap`.
    /// - P24 Lokalität: Laufzeit ∝ Grad des gelöschten Knotens × mittlerer
    ///   Nachbarschaftsgröße, nicht ∝ Gesamtindexgröße (Beweis unten).
    pub fn remove_with_graph_repair(
        &mut self,
        doc_id: DocId,
    ) -> Result<DeletionStats>;
}
```

**Typ-Ebenen-Erzwingung von INV-DELETION-1 (Zusammenspiel mit `contextra-crypto`):**

```rust
// crates/contextra-crypto/src/deletion_proof.rs — Erweiterung der bestehenden API

/// Beweis, dass ein Löschvorgang graph-seitig vollständig repariert wurde.
/// Wird von `HnswIndex::remove_with_graph_repair` erzeugt und von
/// `DeletionProof::create_with_wal_receipt_v3` als Pflichtparameter
/// konsumiert — verhindert auf Typ-Ebene, dass ein Proof ohne
/// Graph-Reparatur-Nachweis ausgestellt wird.
pub struct GraphRepairAttestation {
    pub doc_id: DocId,
    pub verified_no_ghost_pointers: bool,
    pub attested_at: i64, // via injizierten Clock-Port, P28
}

impl DeletionProof {
    /// SIGNATURÄNDERUNG (Breaking Change, siehe Migrationsplan unten):
    /// neuer Pflichtparameter `graph_repair: &[GraphRepairAttestation]` —
    /// leer nur zulässig, wenn die Collection keinen Vektorindex nutzt
    /// (compile-time bzw. runtime über `CollectionKind`-Prüfung).
    pub fn create_with_wal_receipt_v3(
        // … bestehende Parameter unverändert …
        graph_repair: &[GraphRepairAttestation],
    ) -> Result<Self, CryptoError>;
}
```

**3. Algorithmus & Komplexität.**

```
Algorithmus: RemoveWithGraphRepair(index, doc_id)
Eingabe:  HNSW-Index I, zu löschende doc_id d
Ausgabe:  DeletionStats mit verified_no_ghost_pointers ∈ {true, false}

1. node ← I.lookup(d)                                   // O(1), Hash-Map
   if node is None: return Err(NotFound(d))
2. tombstone(node)                                       // bestehender Pfad, O(1)
3. for level in 0..=node.max_level:                       // node.max_level ≤ I.config.max_level (≤ 16 typ.)
     affected ← node.neighbors[level]                     // Grad des Knotens auf dieser Ebene,
                                                            // durch HNSW-Konstruktion auf M bzw. M_max0 begrenzt
     for neighbor in affected:
         neighbor.remove_edge(node)                        // O(1) bis O(M) je nach Slot-Struktur
         candidates ← neighbor.original_candidate_pool[level]  // aus Konstruktionszeit gecacht
                       ∪ {n2 : n2 ∈ node.neighbors[level], n2 ≠ neighbor}  // zusätzliche Rettungsanker
         best ← select_neighbors_heuristic(candidates, M)  // wiederverwendeter HNSW-Heuristik-Schritt,
                                                            // O(|candidates| log M)
         if best is empty:
             stats.orphaned_replacements += 1
             // Knoten bleibt auf dieser Ebene mit reduziertem Grad —
             // zulässig (HNSW degradiert graceful bei reduziertem Grad,
             // kein Neuaufbau der gesamten Ebene nötig)
         else:
             neighbor.add_edge(best, level)
4. verified ← verify_no_residual_pointers(I, d, node_budget = degree(d) × M × 4)
   // Scan NUR der betroffenen Nachbarschaft 2. Ordnung — nicht des Gesamtindex
   stats.verified_no_ghost_pointers ← verified
   if not verified: return Err(VerificationFailed(residual_count))
5. return Ok(stats)

Komplexität: Sei d = Grad des gelöschten Knotens (≤ M_max0, typ. ≤ 32),
             M = Konfigurationsparameter (typ. 16).
Zeit:  O(d · M · log M)  — unabhängig von |Gesamtindex| (P24-konform)
Speicher: O(d · M) für candidates-Mengen
```

**4. Invarianten-Nachweis.**
- **INV-DELETION-2 (neu, hiermit formal eingeführt):** *Nach erfolgreichem
  `remove_with_graph_repair(doc_id)` darf auf keiner HNSW-Ebene ein Nachbarschaftszeiger auf `doc_id`
  verbleiben.* Beweis durch Konstruktion: Schritt 3 iteriert über **alle** Ebenen `0..=node.max_level` und
  entfernt die Kante aus **jeder** betroffenen Nachbarliste; Schritt 4 verifiziert dies durch expliziten
  Scan, bevor `Ok` zurückgegeben wird — bei Restvorkommen wird `Err(VerificationFailed)` geliefert, wodurch
  `DeletionStats::verified_no_ghost_pointers` nie fälschlich `true` sein kann (kein „optimistischer"
  Erfolgspfad).
- **P24 Lokalität:** Wie in der Komplexitätsanalyse gezeigt, skaliert die Arbeit mit dem Grad des gelöschten
  Knotens, nicht mit der Indexgröße. Dies ist dieselbe Beweisstruktur wie bei `forward_push_ppr` (v5 §6.2)
  und `TL-HFD` (arXiv:2606.09340) — lokale, budgetierte Reparatur statt globaler Neuberechnung.
- **P7 Zero-Panic:** Jeder Fehlerpfad (`NotFound`, `DisconnectedRepair`, `VerificationFailed`) liefert
  `Err`; `select_neighbors_heuristic` (wiederverwendet aus `core_insert.rs`) ist bereits
  Zero-Panic-verifiziert (bestehender Produktionscode).
- **INV-DELETION-1 (bestehend, hier verschärft):** Durch die Signaturänderung von
  `DeletionProof::create_with_wal_receipt_v3` (Pflichtparameter `graph_repair: &[GraphRepairAttestation]`)
  wird die Reihenfolge „erst physische Bereinigung, dann Proof" für den Vektorindex-Anteil
  **compile-time-erzwingbar** — ein Proof kann nicht mehr ausgestellt werden, ohne dass der Aufrufer eine
  `GraphRepairAttestation` vorweist.
- **Determinismus (P28):** `select_neighbors_heuristic` ist bereits deterministisch (keine RNG-Nutzung);
  Tie-Breaks erfolgen über `NodeId`-Totalordnung (bestehendes Muster, wiederverwendet).

**5. Fehlerbehandlung.** Neue Variante `ContextraError::GraphRepairFailed(HnswDeletionError)` (siehe C.1);
`HnswDeletionError` wird crate-intern in `contextra-vector` geführt (Muster identisch zu
`BanditError`/`ArmRegistryError`, v5 §13).

**6. Test-Pflichten.**
- `crates/contextra-vector/tests/hnsw_deletion_no_ghost_neighbors.rs` — Property-Test (proptest): für
  zufällige Graphtopologien (100–10.000 Knoten, variable Konnektivität) wird ein zufälliger Knoten gelöscht
  und anschließend **exhaustiv** auf allen Ebenen auf Restvorkommen geprüft. Tier-0-Test (v5 §14.1).
- `crates/contextra-vector/tests/hnsw_deletion_reconstruction_attack.rs` — Fault-Injection-Test, der nach
  Löschung versucht, den Vektor über Nachbarschaftsapproximation (k-NN-Query mit dem gelöschten Vektor als
  Ziel) zu rekonstruieren; erwartet: keine Verbesserung der Trefferwahrscheinlichkeit ggü. einem nie
  eingefügten Vektor.
- `crates/contextra-crypto/tests/deletion_proof_requires_graph_repair.rs` — verifiziert, dass
  `create_with_wal_receipt_v3` mit leerem `graph_repair`-Slice für vektorindex-tragende Collections `Err`
  liefert.
- Mutation-Testing-Gate ≥ 85 % für `hnsw/deletion.rs` (Tier-0-Code, v5 §14.4), verifiziert durch eine
  **andere** Entität als die Implementierung (Zweites-Modell-Regel).

**7. Migration, Feature-Flags, Aufwand, Priorität, Abhängigkeiten.**
- **Breaking Change:** JA — `DeletionProof::create_with_wal_receipt_v3` erhält einen Pflichtparameter.
  SemVer: MAJOR-Bump für `contextra-crypto` (v5 §15.3, Schicht-2-Regel „immer MAJOR"). Alle Aufrufer in
  `contextra-engine/src/collection/crud/{delete,forget}.rs` und `contextra-mcp` (`contextra_delete`,
  `contextra_forget`) müssen angepasst werden.
- **Feature-Flag:** keins nötig — dies ist eine Korrektur eines bestehenden Sicherheitsversprechens, kein
  optionales Feature. Muss in **jedem** Deployment (auch `fast`-Ring) aktiv sein, sobald Vektorindizes mit
  `deletion-proof`-Feature kombiniert werden.
- **Aufwand:** mittel–hoch (Kernalgorithmus ca. 300–400 LOC, Tests ca. 200 LOC, Signaturänderungs-
  Propagation durch 3 Crates).
- **Priorität:** P0-1 (höchste Priorität aller Quelldokumente — Kernargumentation für Löschbeweis-
  Alleinstellung, siehe Wettbewerbsanalyse R1).
- **Abhängigkeiten:** keine Vorbedingungen aus anderen B-Punkten; blockiert selbst kein anderes Feature, ist
  aber Voraussetzung dafür, dass die in v5 §1.1 genannte „vierte Produktkennzahl" (Zeit von Löschantrag bis
  extern verifizierbarem Beweis) für vektorindex-tragende Collections überhaupt wahr ist.
- **Zielcrates:** `contextra-vector` (Ring 0, neue Datei `hnsw/deletion.rs`), `contextra-crypto`
  (Signaturänderung `deletion_proof.rs`), `contextra-engine` (Aufrufer-Anpassung), `contextra-mcp`
  (Aufrufer-Anpassung, keine externe API-Änderung nötig, da `DeletionProof` intern erzeugt wird).

---

### B.1.2 `DurabilityMode`-Enum — explizite Performance-Modi ohne stilles Datenverlustrisiko

**1. IST/Problem.** `contextra-store`s `LsmConfig` hat kein `wal_enabled`- oder Durability-Feld (bestätigt: 0
Treffer für `DurabilityMode` im gesamten Workspace, A.9 Punkt 9). WAL-Nutzung ist implizit immer aktiv (P2,
WAL-First). Ein bewusster „Performance-Modus ohne Crash-Konsistenz" (z. B. für ephemere Agenten-Scratch-
Collections, Mobile/IoT-Deployments mit strengen Latenzbudgets) existiert nicht — und könnte, würde er naiv
nachgerüstet, versehentlich mit `sovereign`/`deletion-proof` kombiniert werden und ein stilles, unbemerktes
Sicherheitsversprechen brechen.

**2. SOLL-Schnittstelle.**

```rust
// crates/contextra-store/src/lsm/config.rs — Erweiterung

use serde::{Deserialize, Serialize};

/// Explizite Persistenz-/Durability-Stufe einer Collection oder eines
/// gesamten `ContextraDb`-Deployments. Ersetzt die bisherige implizite
/// Annahme „WAL ist immer an" durch eine bewusste, typisierte Wahl.
///
/// # Kompatibilitätsmatrix (siehe INV-DURABILITY-RING, bereits in v5 §2.3
/// als Invariante angelegt, hier erstmals vollständig spezifiziert)
/// | Mode        | `deletion-proof` | `FeatureRing::Sovereign` | `encryption-at-rest` |
/// |-------------|:---:|:---:|:---:|
/// | `Full`       | ✅ | ✅ | ✅ |
/// | `WalNoHmac`  | ❌ (kein Integritätsanker für Beweiskette) | ❌ | ✅ (AES unabhängig von HMAC-Kette möglich, siehe Feature-Split unten) |
/// | `MemoryOnly` | ❌ | ❌ | ❌ (kein Datenträger, AEAD-Verschlüsselung sinnlos) |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DurabilityMode {
    /// WAL + HMAC-Integritätskette + `fsync` pro Group-Commit.
    /// Aktueller (impliziter) Produktions-Default — wird mit diesem Feature
    /// zum **expliziten** Default (`impl Default`, siehe unten).
    Full,
    /// WAL aktiv (Crash-Konsistenz erhalten), aber **ohne** kryptographische
    /// HMAC-Kette. Schneller (kein HMAC-Update pro Append), aber
    /// `DeletionProof` NICHT ausstellbar (kein fälschungssicherer Integritäts-
    /// anker, auf dem der Löschbeweis aufbauen könnte).
    WalNoHmac,
    /// Kein WAL — Zustand existiert ausschließlich im MemTable/RAM.
    /// NUR für explizit als ephemer markierte Collections (z. B.
    /// Agent-Working-Memory, Such-Caches). Ein Prozessabsturz verliert
    /// alle Daten dieser Collection ersatzlos — dies ist die **gewollte**
    /// Semantik, kein Fehler.
    MemoryOnly,
}

impl Default for DurabilityMode {
    fn default() -> Self {
        Self::Full
    }
}

/// Fehlerform für ungültige Mode×Feature-Kombinationen. Wird sowohl
/// zur Compile-Zeit (über das `xtask`-Gate, siehe C.4) als auch zur
/// Laufzeit (Defense-in-Depth: falls Builder-API missbraucht wird) geprüft.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DurabilityConfigError {
    #[error("DurabilityMode::{mode:?} is incompatible with feature '{feature}': {reason}")]
    IncompatibleCombination {
        mode: DurabilityMode,
        feature: &'static str,
        reason: &'static str,
    },
}

impl DurabilityMode {
    /// Laufzeit-Prüfung der Kompatibilitätsmatrix (Defense-in-Depth
    /// zusätzlich zum Compile-Time-Feature-Gate, siehe unten).
    pub fn validate_against_features(
        self,
        deletion_proof_active: bool,
        feature_ring: contextra_types::FeatureRing,
    ) -> std::result::Result<(), DurabilityConfigError> {
        use DurabilityMode::*;
        match self {
            Full => Ok(()),
            WalNoHmac if deletion_proof_active => {
                Err(DurabilityConfigError::IncompatibleCombination {
                    mode: self,
                    feature: "deletion-proof",
                    reason: "no HMAC integrity chain to anchor the proof",
                })
            }
            MemoryOnly if deletion_proof_active
                || feature_ring == contextra_types::FeatureRing::Sovereign =>
            {
                Err(DurabilityConfigError::IncompatibleCombination {
                    mode: self,
                    feature: "deletion-proof / FeatureRing::Sovereign",
                    reason: "no persistence layer exists to prove deletion from",
                })
            }
            _ => Ok(()),
        }
    }
}
```

**Feature-Entkopplung (Cargo, `contextra-store/Cargo.toml`):**

```toml
[features]
# Bisher: encryption-at-rest bündelte WAL-Integrität UND Löschbeweis
# untrennbar über eine transitive contextra-crypto-Abhängigkeit.
# NEU: drei unabhängig schaltbare Hebel:
wal-integrity      = ["dep:contextra-crypto"]  # nur HMAC-Kette, kein Ed25519
deletion-proof     = ["dep:contextra-crypto", "wal-integrity"]  # Ed25519-Löschbeweis baut auf Integritätskette auf
encryption-at-rest = ["wal-integrity"]         # AES-256-GCM-SIV erfordert Integritätsanker, nicht zwingend Löschbeweis
memory-only-storage = []                        # aktiviert DurabilityMode::MemoryOnly als wählbare Option
```

**3. Algorithmus & Komplexität.** Kein neuer Kernalgorithmus — dies ist eine **Konfigurationsschicht** über
dem bestehenden `WalHandle::append()`-Pfad (v5 §5.1). Die Umsetzung verzweigt an genau einer Stelle im
Group-Commit-Actor:

```
match config.durability_mode {
    Full        => { wal_append(payload); hmac_chain_extend(); fsync(); }
    WalNoHmac   => { wal_append(payload); fsync(); }               // kein hmac_chain_extend()
    MemoryOnly  => { /* kein WAL-Append, direkt MemTable-Schreibpfad */ }
}
```
Komplexität unverändert ggü. bestehendem Group-Commit-Pfad (v5 §5.1) — `MemoryOnly` ist strikt billiger
(entfällt: WAL-I/O, `fsync`-Syscall), `WalNoHmac` spart genau das HMAC-Update pro Eintrag (SHA-256/Blake3-
Kosten, konstanter Faktor).

**4. Invarianten-Nachweis.** **INV-DURABILITY-RING** (bereits in v5 §2.3 als Name angelegt, hier erstmals
mit Inhalt gefüllt): *`DurabilityMode::MemoryOnly` ist inkompatibel mit `deletion-proof` und
`FeatureRing::Sovereign`.* Durchsetzung zweistufig: (a) Compile-Time über
`xtask check-durability-feature-exclusivity` (neues Gate, C.4) — scannt `Cargo.toml`-Feature-Aktivierungen
im finalen Build-Graph und schlägt fehl, wenn `memory-only-storage` und `deletion-proof` gemeinsam aktiv
sind; (b) Runtime über `validate_against_features()`, aufgerufen im `ContextraDb`-Builder
(`contextra/src/builder.rs`) vor Instanziierung — verhindert Fehlkonfiguration über dynamische
Config-Dateien, die das Compile-Time-Gate nicht erfasst.

**5. Fehlerbehandlung.** Neue Variante `ContextraError::DurabilityConfig(DurabilityConfigError)` (C.1).
`DurabilityConfigError` folgt dem etablierten lokalen-Enum-mit-`From`-Muster (v5 §13).

**6. Test-Pflichten.**
- `crates/contextra-store/tests/durability_mode_feature_gate.rs` — Tier-1-Test: instanziiert `ContextraDb`
  mit allen 3×2×3-Kombinationen aus `DurabilityMode` × `deletion_proof_active` × `FeatureRing` und prüft
  exaktes `Ok`/`Err`-Verhalten gegen die Kompatibilitätsmatrix.
- `crates/contextra-store/tests/memory_only_crash_semantics.rs` — verifiziert, dass `MemoryOnly`-Collections
  nach simuliertem Prozessabsturz (via `contextra-testkit::fault_vfs`) tatsächlich leer sind (positive
  Bestätigung der gewollten Semantik, kein „versehentlich doch persistent").
- Neues `xtask`-Gate `check-durability-feature-exclusivity` (siehe C.4), in CI als Pflichtschritt vor
  `cargo build --all-features`.

**7. Migration, Feature-Flags, Aufwand, Priorität, Abhängigkeiten.** Additiv auf API-Ebene (neuer Enum-Typ
mit sinnvollem `Default`); die Feature-Entkopplung (`wal-integrity`/`deletion-proof`/`encryption-at-rest`
getrennt) ist potenziell **Breaking** für Downstream-Nutzer, die bisher nur `encryption-at-rest` aktivierten
und implizit Löschbeweis mitbekamen — SemVer MINOR mit Migrations-Hinweis im CHANGELOG (da die
Default-Feature-Kombination unverändert bleibt, wenn `default = ["fast"]` weiterhin keines der drei Flags
aktiviert). Aufwand: mittel. Priorität: P0-2. Abhängigkeiten: **harte Vorbedingung** — `FeatureRing` muss
zuvor gemäß C.0 Punkt 1 nach `contextra-types` migriert sein, da `validate_against_features` sonst eine
verbotene Ring-1→Ring-4-Abhängigkeit (`contextra-store` → `contextra-license`) einführen würde; ist selbst
Vorbedingung für B.6.4 (P2P-Sync, das `DurabilityMode::Full` voraussetzt) und für jeden zukünftigen
Mobile/Embedded-Anwendungsfall (ROADMAP §10 „Effizienter Betrieb auf Consumer-Hardware"). Zielcrate:
`contextra-store` (Ring 0/1) + `xtask`.

---

### B.1.3 `AutoExtractionConfig::enabled = true` als Default + Graph-Rückkopplung

**1. IST/Problem.** `AutoExtractionConfig` (`crates/contextra-engine/src/collection/crud/auto_extraction.rs:36`)
hat `enabled: bool` mit dokumentiertem Default `false` (Kommentar „Ob die automatische Extraktion aktiviert
ist (Default: false)"). Die zugrunde liegende Open-IE-Extraktion (`extraction/open_ie.rs`) ist vollständig
implementiert (LLM-Tripel-Extraktion via `LlmTextGenerator`-Port) — sie ist lediglich standardmäßig inaktiv.
Jede neue Collection nutzt damit ausschließlich manuelles `relate()`/`relate_n_ary()`; automatischer
Wissensgraph-Aufbau aus unstrukturiertem Text findet nicht statt, obwohl v5 §1 „Langzeit-Agentengedächtnis
mit Konsolidierung (Hyperkanten, LeanRAG, TL-HFD)" als eine der vier Kern-Differenzierungsdimensionen nennt.

**2. SOLL-Schnittstelle.**

```rust
// crates/contextra-engine/src/collection/crud/auto_extraction.rs — Änderung

impl Default for AutoExtractionConfig {
    fn default() -> Self {
        Self {
            enabled: true,  // WAR false
            entity_config: EntityExtractionConfig {
                enabled: true,
                max_llm_calls_per_cycle: 5,   // konservative Obergrenze, unverändert
                min_confidence: 0.6,          // unverändert
            },
        }
    }
}

// crates/contextra-engine/src/collection/crud/insert.rs — Erweiterung:
// Rückkopplung extrahierter Tripel in den Wissensgraphen

/// Wird nach jedem `auto_extract()`-Aufruf für jedes extrahierte Tripel
/// aufgerufen. Bildet die fehlende Brücke zwischen Open-IE-Ausgabe und
/// Graph-Schreibpfad, die laut ROADMAP §0 Zeile „Entity Extraction"
/// bislang fehlt („nur Aktivierung + tiefere Wissensgraph-Rückkopplung fehlt").
async fn feed_extracted_triples_to_graph<S, V>(
    collection: &Collection<S, V>,
    triples: Vec<ExtractedTriple>,
    tx: TxId,
) -> Result<Vec<HyperEdgeId>>
where
    S: StorageEngine,
    V: VectorIndex,
{
    let mut edge_ids = Vec::with_capacity(triples.len());
    for triple in triples {
        // confidence wird als Hyperkanten-Gewicht gespeichert — Grundlage
        // für spätere Decay-Berechnung (contextra-adapt::decay_controller)
        // und für Spectral-Hyperedge-Dedup-Priorisierung (B.5.1).
        let edge_id = collection
            .relate_n_ary(
                tx,
                vec![
                    RoleBinding { entity: triple.subject, role: ROLE_SUBJECT },
                    RoleBinding { entity: triple.object, role: ROLE_OBJECT },
                ],
                Some(triple.predicate.clone().into_bytes().into()), // payload, zero-copy via Bytes
                HyperEdgeWeight::from_confidence(triple.confidence),
            )
            .await?;
        edge_ids.push(edge_id);
    }
    Ok(edge_ids)
}
```

**3. Algorithmus & Komplexität.** Kein neuer Algorithmus — Verdrahtung eines bereits vorhandenen
Extraktionspfads (`open_ie.rs`) mit dem bereits vorhandenen Graph-Schreibpfad (`relate_n_ary`, v5 §6.4).
Laufzeitkosten pro Insert: begrenzt durch `max_llm_calls_per_cycle = 5` (P24-relevant: konstante Obergrenze
pro Zyklus, nicht proportional zur Collection-Größe).

**4. Invarianten-Nachweis.** P24 (Lokalität): durch `max_llm_calls_per_cycle` hart begrenzt — unverändert
zur bestehenden Konfiguration, nur der Default-Aktivierungszustand ändert sich. P7 (Zero-Panic):
`relate_n_ary` ist bereits produktionsgehärtet (`MAX_RELATE_PARTICIPANTS = 64`, v5 §15.2); die
Brücken-Funktion führt keine neue Unsicherheit ein. Kein Verstoß gegen INV-1 bis INV-7.

**5. Fehlerbehandlung.** Kein neuer Fehlertyp nötig — `relate_n_ary` propagiert bereits `ContextraError`;
Fehler in einzelnen Tripeln sollten **nicht** den gesamten Insert-Vorgang abbrechen (Best-Effort-Semantik für
Extraktion, da sie ein Konsolidierungs-Feature ist, kein Kern-Schreibpfad) — daher:
`feed_extracted_triples_to_graph` sammelt Fehler pro Tripel und liefert `Vec<HyperEdgeId>` für erfolgreiche
plus eine `Vec<(ExtractedTriple, ContextraError)>`-Fehlerliste als Diagnose-Metadatum (kein
`Result<Vec<_>>` mit Total-Abbruch bei erstem Fehler).

**6. Test-Pflichten.** `crates/contextra-engine/tests/auto_extraction_wiring.rs` existiert bereits
(verifiziert, A.9) — muss um Assertions erweitert werden, die nach `insert()` mit aktiviertem Default die
tatsächliche Existenz der entsprechenden `HyperEdge`s im Graph-Index prüfen (End-to-End statt nur
Extraktions-Unit-Test). Neuer Test `auto_extraction_default_enabled.rs`: verifiziert, dass eine frisch mit
`Default::default()` erzeugte `AutoExtractionConfig` `enabled == true` liefert (Regressionsschutz gegen
versehentliches Zurücksetzen).

**7. Migration, Feature-Flags, Aufwand, Priorität, Abhängigkeiten.** Breaking auf **Verhaltensebene** (nicht
Signaturebene): bestehende Nutzer, die sich auf `enabled: false` als Default verlassen (z. B. um
LLM-Kosten für reine Vektor-/Text-Workloads zu vermeiden), erhalten nach Upgrade automatisch LLM-Aufrufe bei
jedem Insert. SemVer: MINOR mit **prominentem** CHANGELOG-Hinweis; Empfehlung, in derselben Version einen
Cargo-Feature-Schalter `auto-extraction-opt-out` als Fluchtweg für kostensensitive Downstream-Nutzer
anzubieten (zusätzlich zur weiterhin bestehenden Möglichkeit, `enabled: false` explizit zu setzen). Aufwand:
niedrig (Default-Änderung + Rückkopplungsfunktion ca. 80–120 LOC). Priorität: P0-3. Abhängigkeiten: keine.
Zielcrate: `contextra-engine` (Ring 3).

---

### B.1.4 Legacy-v2-HMAC-Kollisions-Property-Test

**1. IST/Problem.** `contextra-crypto` unterstützt laut v5 §4.1 drei `DeletionProof`-Signaturversionen
(v1/v2 HMAC-basiert mit `subtle::ConstantTimeEq`, v3 Ed25519). Für die Legacy-Pfade v1/v2 existiert laut v5
§14.2 bislang **kein** Kollisions-Property-Test (Tier-0-Lücke: „Tatsachenbehauptung nach außen falsch" wäre
die Konsequenz, sollte eine HMAC-Kollision in der Praxis zu einem falsch-positiven Verifikationsergebnis
führen).

**2. SOLL-Schnittstelle.** Kein neuer Produktionscode — dies ist ein reiner Test-Pflichtenpunkt.
Spezifiziertes Test-Artefakt:

```rust
// crates/contextra-crypto/tests/legacy_hmac_collision_property.rs — NEUE DATEI

use contextra_crypto::deletion_proof::{DeletionProof, hash_deleted_keys_length_prefixed};
use proptest::prelude::*;

proptest! {
    /// Für alle Paare unterschiedlicher Schlüsselmengen darf die
    /// Length-Prefixed-Hash-Funktion (Basis der v1/v2-HMAC-Kette) nicht
    /// dieselbe Byte-Sequenz liefern — sonst wäre eine v1/v2-Signatur über
    /// Menge A auch für Menge B gültig (Kollisionsangriff auf den
    /// Löschbeweis: ein Angreifer könnte behaupten, andere Daten gelöscht
    /// zu haben, als tatsächlich gelöscht wurden).
    #[test]
    fn no_hash_collision_for_distinct_key_sets(
        keys_a in prop::collection::vec(any::<Vec<u8>>(), 1..50),
        keys_b in prop::collection::vec(any::<Vec<u8>>(), 1..50),
    ) {
        prop_assume!(keys_a != keys_b);
        let hash_a = hash_deleted_keys_length_prefixed(&keys_a);
        let hash_b = hash_deleted_keys_length_prefixed(&keys_b);
        prop_assert_ne!(hash_a, hash_b);
    }

    /// Längenpräfix-Kodierung muss injektiv bzgl. der Segmentierung sein:
    /// {"ab", "cd"} darf nicht denselben Hash liefern wie {"abcd"} —
    /// klassischer Length-Extension-artiger Kollisionsvektor bei naiver
    /// Konkatenation ohne Längenpräfix. Dies ist der eigentliche Kernzweck
    /// des 8-Byte-Big-Endian-Präfixes (v5 §4.1) und muss explizit
    /// gegengetestet werden, nicht nur implizit durch das Präfix-Design
    /// als "vermutlich sicher" angenommen werden.
    #[test]
    fn length_prefix_prevents_segmentation_ambiguity(
        a in ".*", b in ".*", c in ".*",
    ) {
        prop_assume!(format!("{a}{b}") != c || a.is_empty() || b.is_empty());
        let split = hash_deleted_keys_length_prefixed(&[a.clone().into_bytes(), b.clone().into_bytes()]);
        let joined = hash_deleted_keys_length_prefixed(&[c.into_bytes()]);
        prop_assert_ne!(split, joined);
    }
}
```

**3–5.** Entfällt (kein neuer Algorithmus, keine neue Invariante über die bestehende Längenpräfix-
Konstruktion hinaus, kein neuer Fehlertyp).

**6. Test-Pflichten.** Die Datei selbst ist der Test-Pflichtenpunkt; zusätzlich Aufnahme in
`xtask check-mutation-score-gate` für `deletion_proof.rs` (bestehendes Gate, v5 §14.4).

**7. Migration, Feature-Flags, Aufwand, Priorität, Abhängigkeiten.** Keine Migration (additiver Test).
Aufwand: niedrig. Priorität: P0-5 (Tier-0, aber isoliert — blockiert nichts anderes und wird von nichts
blockiert). Zielcrate: `contextra-crypto` (Ring 0).

---

### B.1.5 WAL-Recovery Runtime-Stall-Test

**1. IST/Problem.** v5 §14.2 führt „WAL-Recovery Runtime-Stall-Test" als Tier-1-Lücke. Gemeint ist: Es
existiert Torn-Write-Fault-Injection (🟢, v5 §14.2) und MANIFEST-Crash-No-Resurrection (🟢), aber kein Test,
der verifiziert, dass der WAL-Replay-Pfad bei einem **großen** WAL (viele ausstehende Einträge nach Crash)
innerhalb einer erwarteten Zeitschranke terminiert, statt in einem pathologischen Fall (z. B. durch eine im
Recovery-Pfad versehentlich eingeführte quadratische Komplexität) zu „stallen" — ein produktionskritisches
Risiko, da ein hängender Recovery-Vorgang beim Neustart einer Appliance einem Totalausfall gleichkäme.

**2. SOLL-Schnittstelle.** Test-Artefakt plus ein neues, kleines Observability-Interface:

```rust
// crates/contextra-store/src/wal/replay.rs — Erweiterung

/// Fortschritts-Callback für WAL-Replay — ermöglicht dem Aufrufer
/// (z. B. `contextra`-Composition-Root beim Appliance-Start), einen
/// Fortschrittsbalken/Health-Check-Endpunkt zu bedienen UND dient als
/// Instrumentierungspunkt für den Stall-Test (Fortschritt muss
/// monoton und innerhalb eines Zeitbudgets voranschreiten).
pub trait ReplayProgressSink: Send + Sync {
    fn on_entry_replayed(&self, seq: WalSeq, entries_total_estimate: Option<u64>);
}

pub struct NoopReplayProgressSink;
impl ReplayProgressSink for NoopReplayProgressSink {
    fn on_entry_replayed(&self, _seq: WalSeq, _entries_total_estimate: Option<u64>) {}
}
```

```rust
// crates/contextra-store/tests/wal_recovery_runtime_stall.rs — NEUE DATEI
// Tier-1-Test: erzeugt einen synthetischen WAL mit 10^6 Einträgen (via
// contextra-testkit::fault_vfs, kein echter Disk-I/O nötig), simuliert
// einen Crash-Neustart und misst Replay-Dauer über einen
// ReplayProgressSink, der Zeitstempel pro N-tem Eintrag festhält.
// Assertion: Replay-Durchsatz (Einträge/Sekunde) darf über den gesamten
// Lauf nicht unter 50 % des Durchsatzes der ersten 10 % fallen —
// dies erkennt jede eingeschlichene nicht-lineare Komplexität
// (z. B. O(n²) durch versehentliches Full-Rescan pro Eintrag) als
// Regression, ohne eine absolute Zeitschranke hart zu verdrahten
// (die von Testhardware abhängig wäre).
```

**3–5.** Entfällt (Test-/Observability-Ergänzung, keine neue Kernlogik-Änderung; `ReplayProgressSink` ist
rein additiv und ändert das bestehende Replay-Verhalten nicht).

**6. Test-Pflichten.** Die genannte Datei; Aufnahme in `xtask bench-gate` als Regressions-Schutz
(bestehendes Gate, v5 §14.2) statt als reiner Korrektheitstest, da hier primär Performance-Degradation,
nicht Korrektheit, geprüft wird.

**7. Migration, Feature-Flags, Aufwand, Priorität, Abhängigkeiten.** `ReplayProgressSink` ist additiv
(neuer optionaler Parameter mit `NoopReplayProgressSink`-Default für bestehende Aufrufer — kein Breaking
Change). Aufwand: niedrig. Priorität: P0-6. Zielcrate: `contextra-store` (Ring 1).

---

## B.2 Phase 1 — `fast`-Ring-Härtung (laufend, Gate: 10 echte Nutzergespräche)

### B.2.1 `AttentionExporter`-Port + SnapKV/H2O Backend-Wiring

**1. IST/Problem.** `contextra-kvcache/src/attention_score.rs` definiert bereits `AttentionScoreSource`
(Trait) und `rank_for_eviction_weighted()`; `NullAttentionScoreSource` ist der verifiziert aktive Default
(A.9 Punkt 10) — Eviction läuft de facto als reines LRU ohne echte Attention-Gewichtung. Die fehlende
Verbindungsstelle ist ein Port von den Inferenz-Backends (`contextra-infer-candle`, `contextra-infer-ollama`)
zum `EvictionWorker` in `contextra-kvcache` — dies ist eine **Ring-2→Ring-1-Kante**, die laut v5 §3.2 nur
über den bereits bestehenden `KvBridgeStorage`-Port (Ring 0, Dependency-Inversion) laufen darf, nicht direkt.

**2. SOLL-Schnittstelle.**

```rust
// crates/contextra-ports/src/attention.rs — NEUE DATEI, neues Modul in lib.rs

use crate::BoxFuture;

/// Eindeutige Kennung eines Inferenz-Requests innerhalb eines Prozesses —
/// wird vom Inferenz-Backend beim Prefill vergeben und an den
/// `EvictionWorker` durchgereicht, damit Attention-Scores dem korrekten
/// KV-Cache-Segment zugeordnet werden können.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(pub u64);

/// Von Inferenz-Backends (Ring 2) implementierter Port, der
/// Attention-Gewichte aus dem letzten Prefill-/Decode-Schritt exportiert.
/// Konsumiert von `contextra-kvcache`s `EvictionWorker` (Ring 1) — die
/// Ring-2→Ring-1-Kante läuft über dieses Trait-Objekt, das über
/// `KvBridgeStorage` injiziert wird (v5 §3.2 DAG-Konformität: Ring 0
/// definiert den Port, Ring 2 implementiert ihn, Ring 1 konsumiert ihn
/// nur über die abstrakte `dyn`-Referenz — keine Cargo-Abhängigkeit
/// Ring 1 → Ring 2 entsteht).
///
/// Entspricht dem "observation window"-Mechanismus aus SnapKV
/// (arXiv:2401.02714) bzw. dem Heavy-Hitter-Oracle aus H2O.
pub trait AttentionExporter: Send + Sync {
    /// Liefert die aufsummierten Attention-Gewichte pro Token-Position aus
    /// dem letzten Prefill-Fenster für den gegebenen Request. `None`, wenn
    /// keine Attention-Instrumentierung verfügbar ist (z. B. Ollama-Backend
    /// ohne Attention-Export-Unterstützung — degradiert graceful zu
    /// `NullAttentionScoreSource`-Verhalten für diesen Request).
    fn export_attention_weights(&self, request_id: RequestId) -> Option<Vec<f32>>;
}
```

```rust
// crates/contextra-infer-candle/src/attention_exporter.rs — NEUE DATEI

use contextra_ports::{AttentionExporter, RequestId};

/// Summiert Attention-Gewichte über alle Köpfe und Layer des letzten
/// Prefill-Schritts. Hält eine begrenzte Ringpuffer-Historie
/// (`MAX_TRACKED_REQUESTS`, Default 256) pro aktivem Request, um
/// unbegrenztes Speicherwachstum bei Request-Leaks zu verhindern
/// (INV-5-Analogon: Speicherbudget-Transparenz).
pub struct CandleAttentionExporter {
    // Interne Struktur: Arc<Mutex<LruCache<RequestId, Vec<f32>>>> o.ä. —
    // Population erfolgt aus dem Candle-Tensor-Graphen direkt nach dem
    // Prefill-Forward-Pass (Summe über die Attention-Score-Matrix pro
    // Query-Head, vor dem Softmax-Output-Verwurf).
}

impl AttentionExporter for CandleAttentionExporter {
    fn export_attention_weights(&self, request_id: RequestId) -> Option<Vec<f32>> {
        // Implementierung: Lookup im Ringpuffer; Rückgabe None bei Miss
        // (Request unbekannt oder bereits aus dem Puffer verdrängt).
        unimplemented!("siehe Implementierungs-Roadmap unten")
    }
}
```

```rust
// crates/contextra-kvcache/src/eviction_worker.rs — Wiring-Änderung

pub struct EvictionWorker {
    // NEU: optionaler Attention-Exporter, injiziert über Builder.
    // Default bleibt NullAttentionScoreSource (unveränderte Abwärts-
    // kompatibilität für Aufrufer ohne Inferenz-Backend, z. B. reine
    // Retrieval-Deployments ohne LLM-Generierung).
    attention_source: std::sync::Arc<dyn contextra_kvcache::AttentionScoreSource>,
    // … bestehende Felder unverändert …
}

impl EvictionWorker {
    /// NEU: Builder-Methode zur Injektion eines echten Attention-Backends.
    pub fn with_attention_exporter(
        mut self,
        exporter: std::sync::Arc<dyn contextra_ports::AttentionExporter>,
    ) -> Self {
        self.attention_source = std::sync::Arc::new(
            ExporterBackedScoreSource::new(exporter)
        );
        self
    }
}

/// Adapter: übersetzt AttentionExporter (Ring-2-Interface, request-
/// zentriert) in AttentionScoreSource (Ring-1-Interface,
/// segment-zentriert). Enthält die Zuordnungslogik
/// RequestId → Vec<SegmentId>, die notwendig ist, da ein Prefill mehrere
/// KV-Cache-Segmente betreffen kann (Radix-Baum-Präfix-Aufteilung).
struct ExporterBackedScoreSource {
    exporter: std::sync::Arc<dyn contextra_ports::AttentionExporter>,
}
```

**3. Algorithmus & Komplexität.** Kein neuer Eviction-Algorithmus — `rank_for_eviction_weighted()` existiert
bereits und erwartet exakt die Signatur, die `AttentionExporter` liefert (`Vec<f32>` pro Token-Position). Die
Wiring-Arbeit besteht ausschließlich aus (a) der neuen Port-Definition, (b) einer Candle-seitigen
Implementierung, die Tensor-Daten aus dem Forward-Pass abgreift, ohne den Hot-Path spürbar zu verlangsamen
(Kopie der Attention-Summe ist O(Sequenzlänge), nicht O(Sequenzlänge²) — die vollständige Attention-Matrix
wird **nicht** exportiert, nur die pro-Position aufsummierten Gewichte), (c) dem Adapter zwischen den beiden
Interface-Granularitäten.

**4. Invarianten-Nachweis.** P27 (jeder Port = `dyn`-kompatibler Trait): `AttentionExporter` erfüllt
`Send + Sync`, keine generischen Methoden — dyn-kompatibel. DAG-Konformität (v5 §3.2): Der Port wird in
`contextra-ports` (Ring 0) definiert; `contextra-infer-candle` (Ring 2) implementiert ihn (Ring 2 → Ring 0
ist eine erlaubte Kante); `contextra-kvcache` (Ring 1) hält nur eine `Arc<dyn AttentionExporter>`-Referenz
ohne Cargo-Abhängigkeit auf `contextra-infer-candle` — die Injektion erfolgt in der Composition Root
(`contextra`, Ring 4), die beide Ringe bereits kennt. Damit bleibt „Ring 1 hängt nie von einem anderen
Ring-1- **oder höheren** Crate ab" (v5 §3.2) gewahrt.

**5. Fehlerbehandlung.** Kein neuer `ContextraError`-Bedarf — `export_attention_weights` liefert `Option`,
kein `Result` (Attention-Daten sind ein Best-Effort-Qualitätssignal, kein Korrektheits-kritischer Pfad; ihr
Fehlen degradiert zu bestehendem LRU-Verhalten, bricht aber nichts).

**6. Test-Pflichten.** `crates/contextra-kvcache/tests/eviction_with_attention_exporter.rs` —
Differential-Test: vergleicht Eviction-Reihenfolge zwischen `NullAttentionScoreSource` (Baseline) und einem
`MockAttentionExporter` mit synthetisch hohen Gewichten auf bestimmten Token-Positionen; erwartet, dass
diese Positionen **später** evictiert werden als bei der Baseline.
`crates/contextra-infer-candle/tests/attention_exporter_bounded_memory.rs` — verifiziert
`MAX_TRACKED_REQUESTS`-Obergrenze (INV-5-Analogon).

**7. Migration, Feature-Flags, Aufwand, Priorität, Abhängigkeiten.** Additiv (neuer Trait, neue
Builder-Methode mit sinnvollem Default) — kein Breaking Change. Aufwand: mittel. Priorität: HIGH (Tier 1/2,
ROADMAP-Hebelwirkungsanalyse §1: Multiplikator, schaltet KIVI-Nützlichkeit, GemFilter/PromptDistill und
belastbare Latenz-Garantien frei). Abhängigkeiten: **Vorbedingung** für B.3.3 (KIVI Full Integration — „KIVI-
Quantisierung wird erst sinnvoll, wenn Eviction-Qualität stimmt", ROADMAP §3.1). Zielcrates: `contextra-ports`
(neuer Trait), `contextra-infer-candle` (Implementierung), `contextra-kvcache` (Wiring, Adapter).

---

### B.2.2 `contextra-adapters` — Framework-Adapter-Crate (LangChain/LangGraph/LlamaIndex)

**1. IST/Problem.** Kein `contextra-adapters`-Crate im Workspace (bestätigt: nicht in `Cargo.toml`
`members`). Nutzer der Frameworks LangChain, LangGraph, LlamaIndex müssen aktuell gegen die rohe
MCP-Schnittstelle oder `contextra-py`-Bindings direkt integrieren — ein Adoptionshindernis ggü. Mem0 (native
LangChain-/LlamaIndex-Integration) und SuperLocalMemory (neun Framework-Adapter, Wettbewerbsanalyse
§2.5/R4).

**2. SOLL-Schnittstelle.**

```toml
# crates/contextra-adapters/Cargo.toml — NEUES Ring-4-Crate
[package]
name = "contextra-adapters"
# … workspace-Vererbung wie üblich …

[dependencies]
contextra-py = { workspace = true }   # Ring 4 → Ring 4, erlaubt (v5 §3.2 "Ring 4 → alles")
pyo3 = { workspace = true }
```

```rust
// crates/contextra-adapters/src/langchain.rs

/// Implementiert LangChains `BaseChatMessageHistory`-Protokoll (Python-
/// seitig via PyO3 `#[pyclass]`) direkt gegen eine bestehende
/// `contextra-py`-Collection. Jede Methode ist ein dünner Wrapper um
/// bereits vorhandene `contextra-py`-Bindings — kein neuer Kernpfad.
pub struct LangChainMemoryAdapter {
    engine: std::sync::Arc<contextra_py::PyContextraDb>,  // bestehender Typ
    session_id: contextra_types::TenantId,
}

impl LangChainMemoryAdapter {
    pub fn new(engine: std::sync::Arc<contextra_py::PyContextraDb>, session_id: contextra_types::TenantId) -> Self {
        Self { engine, session_id }
    }
    /// Python-seitige Signatur (via #[pymethods]): add_message(self, message) -> None
    pub async fn add_message(&self, message: ChatMessage) -> contextra_types::Result<()> {
        self.engine.insert_scoped(self.session_id, message.to_document()).await
    }
    /// Python-seitige Signatur: get_messages(self) -> List[BaseMessage]
    pub async fn get_messages(&self) -> contextra_types::Result<Vec<ChatMessage>> {
        self.engine.search_scoped(self.session_id, /* Query: alle, chronologisch */ Default::default()).await
    }
    /// Python-seitige Signatur: clear(self) -> None
    pub async fn clear(&self) -> contextra_types::Result<()> {
        self.engine.drop_scoped(self.session_id).await
    }
}
```

```rust
// crates/contextra-adapters/src/langgraph.rs

/// Implementiert LangGraphs `BaseStore`-Protokoll: hierarchische
/// Namespace/Key-Value-Semantik über bestehende Collection-KV-Operationen
/// (v5 §4.4, `collection/crud/kv.rs`).
pub struct LangGraphStoreAdapter { /* … */ }
impl LangGraphStoreAdapter {
    pub async fn aget(&self, namespace: &[&str], key: &str) -> contextra_types::Result<Option<serde_json::Value>>;
    pub async fn aput(&self, namespace: &[&str], key: &str, value: serde_json::Value) -> contextra_types::Result<()>;
    pub async fn asearch(&self, namespace: &[&str], query: SearchQuery) -> contextra_types::Result<Vec<SearchItem>>;
}
```

```rust
// crates/contextra-adapters/src/llamaindex.rs

/// Implementiert LlamaIndex' `BaseChatStore`-Protokoll — analog zu
/// LangChainMemoryAdapter, andere Methodennamen/Signaturen gemäß
/// LlamaIndex-Konvention.
pub struct LlamaIndexStoreAdapter { /* … */ }
```

**3. Algorithmus & Komplexität.** Keine neue Algorithmik — reine Protokoll-Übersetzung (Adapter-Pattern).
Laufzeitkosten identisch zu den zugrundeliegenden `contextra-py`-Aufrufen zzgl. vernachlässigbarem
PyO3-Marshalling-Overhead.

**4. Invarianten-Nachweis.** P27/DAG-Konformität: `contextra-adapters` liegt in Ring 4 und hängt nur von
`contextra-py` (ebenfalls Ring 4) ab — „Ring 4 → alles" (v5 §3.2) erlaubt dies uneingeschränkt; kein
Rückwärtskanten-Risiko, da kein anderer Ring von `contextra-adapters` abhängt (reines Blatt).

**5. Fehlerbehandlung.** Fehler werden 1:1 aus `contextra_types::ContextraError` über die bestehende PyO3-
`catch_unwind`-Panic-Isolation (ADR-056, v5 §4.5) nach Python als `PyErr` durchgereicht — kein neuer
Fehlertyp.

**6. Test-Pflichten.** `crates/contextra-adapters/tests/langchain_protocol_conformance.py` (bzw. Rust-
seitiges Pendant) — Protokoll-Konformitätstest gegen die jeweilige Framework-Referenzimplementierung (sofern
als Testabhängigkeit verfügbar) oder gegen eine minimale Interface-Checkliste (Methode vorhanden, Signatur
passt, Grundverhalten korrekt).

**7. Migration, Feature-Flags, Aufwand, Priorität, Abhängigkeiten.** Rein additives neues Crate — kein
Breaking Change für bestehende Crates. Aufwand: niedrig–mittel pro Adapter (geschätzt 150–300 LOC pro
Framework). Priorität: SEHR HOCH (ROADMAP §1 Tier-1-Multiplikator: „10× Adoptionshebel vs. nur MCP/PyO3";
Wettbewerbsanalyse R4). Abhängigkeiten: setzt eine stabile `contextra-py`-API voraus (bereits gegeben, 🟢).
Zielcrate: neues `crates/contextra-adapters` (Ring 4).

---

### B.2.3 `PluginManifest`-Trait + `PluginRegistry`

**1. IST/Problem.** v5 §12.2 spezifiziert bereits eine Ziel-API (`PluginCapability`, `PluginManifest`),
markiert 🔴 (nicht implementiert). `INV-PLUGIN-DEPENDENCY` ist in v5 §2.3 als Invariante **benannt**, aber
mangels Implementierung nicht durchsetzbar. Diese Spezifikation übernimmt und vervollständigt v5 §12.2 um den
fehlenden `PluginRegistry`-Teil (Aktivierungsreihenfolge, Zyklenerkennung) sowie die Integration mit dem
bestehenden `FeatureRing`-Enum (`contextra-types`, **nach der in C.0 Punkt 1 beschriebenen Migration** — im
IST-Zustand zum Zeitpunkt der Ausarbeitung dieser Spezifikation liegt `FeatureRing` noch in
`contextra-license`) sowie der neuen `LicenseGate`-Vollimplementierung (B.4.1).

**2. SOLL-Schnittstelle.**

```rust
// crates/contextra-ports/src/plugin.rs — NEUE DATEI

/// Statische Beschreibung eines Plugins — unveränderlich zur Laufzeit,
/// typischerweise als `const` pro Plugin-Implementierung definiert.
#[derive(Debug, Clone, Copy)]
pub struct PluginCapability {
    pub name: &'static str,
    pub version: (u32, u32, u32),
    /// Architektur-Ring, in dem das Plugin operiert (0–4) — für
    /// Diagnosezwecke und zur Validierung gegen `capabilities.toml`.
    pub ring: u8,
    /// Namen anderer Plugins, die VOR diesem aktiviert sein müssen.
    pub requires: &'static [&'static str],
    /// Namen anderer Plugins, die NICHT gleichzeitig aktiv sein dürfen.
    pub conflicts: &'static [&'static str],
    /// Mindest-Feature-Ring, der für die Aktivierung lizenziert sein muss
    /// (Kopplung an `LicenseGate`, siehe B.4.1).
    pub feature_ring: contextra_types::FeatureRing,
}

/// Von jedem laufzeit-schaltbaren Algorithmus-/Feature-Baustein zu
/// implementierendes Trait (v5 §12.1 Ebene B: Runtime-FeatureFlags).
/// Beispiele laut v5 §12.2/ROADMAP: `BanditImplementation`-Varianten,
/// `PprAlgorithm`-Varianten (TL-HFD ShadowMode/Default), `DurabilityMode`
/// ist NICHT über PluginManifest geschaltet (das ist eine Ebene-A/Compile-
/// Time-Konfiguration pro Collection, kein Laufzeit-Plugin).
pub trait PluginManifest: Send + Sync {
    fn capability(&self) -> &PluginCapability;
    /// Aktiviert das Plugin gegen eine Registry. MUSS idempotent sein
    /// (zweifacher Aufruf mit bereits aktivem Plugin ist ein No-Op,
    /// kein Fehler) — wichtig für Hot-Reload-Szenarien.
    fn activate(&self, registry: &mut PluginRegistry) -> Result<(), PluginError>;
    fn deactivate(&self, registry: &mut PluginRegistry) -> Result<(), PluginError>;
}

/// Zentrale Laufzeit-Registry — lebt in der Composition Root
/// (`contextra`-Crate, Ring 4), da sie Kenntnis über alle Ringe
/// benötigt (P29-konform: keine `static`/globale Instanz, sondern über
/// `ContextraDb`-Builder injiziert und in der `ContextraDb`-Instanz
/// gehalten).
pub struct PluginRegistry {
    active: ahash::AHashMap<&'static str, PluginCapability>,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum PluginError {
    #[error("plugin '{0}' requires '{1}', which is not active")]
    MissingDependency(&'static str, &'static str),
    #[error("plugin '{0}' conflicts with already-active plugin '{1}'")]
    Conflict(&'static str, &'static str),
    #[error("dependency cycle detected involving plugin '{0}'")]
    DependencyCycle(&'static str),
    #[error("plugin '{0}' requires feature ring {required:?}, but only {available:?} is licensed")]
    InsufficientLicense { plugin: &'static str, required: contextra_types::FeatureRing, available: contextra_types::FeatureRing },
}

impl PluginRegistry {
    /// INV-PLUGIN-DEPENDENCY: topologische Auflösung VOR Aktivierung.
    /// Nimmt eine Menge zu aktivierender Plugins entgegen (nicht nur
    /// eines), berechnet eine gültige Aktivierungsreihenfolge via
    /// Kahn-Algorithmus (deterministisch: Tie-Break über alphabetische
    /// Ordnung des Plugin-Namens, P28-konform) und aktiviert sie in
    /// dieser Reihenfolge. Bricht VOR jeder Aktivierung ab, wenn ein
    /// Zyklus erkannt wird (kein Teilaktivierungs-Zustand).
    pub fn activate_all(
        &mut self,
        plugins: &[&dyn PluginManifest],
        license: &dyn contextra_ports::LicenseGate,
    ) -> Result<(), PluginError> {
        // 1. Konfliktprüfung: für jedes Paar (bereits aktiv ∪ zu aktivieren)
        //    prüfen, ob `conflicts` verletzt wird → sofortiger Abbruch.
        // 2. Lizenzprüfung: license.check_ring(cap.feature_ring) für jedes
        //    Plugin → sofortiger Abbruch bei InsufficientLicense.
        // 3. Topologische Sortierung über `requires`-Kanten (Kahn).
        //    Zyklus erkannt ⇒ Err(DependencyCycle) VOR jeder Aktivierung.
        // 4. Aktivierung in topologischer Reihenfolge.
        unimplemented!("siehe Implementierungs-Roadmap")
    }
    pub fn is_active(&self, name: &str) -> bool { self.active.contains_key(name) }
    pub fn snapshot(&self) -> Vec<PluginCapability> { self.active.values().copied().collect() }
}
```

**3. Algorithmus & Komplexität.** Kahn-Algorithmus für topologische Sortierung: $O(V + E)$, wobei $V$ =
Anzahl zu aktivierender Plugins (praktisch < 20), $E$ = Anzahl `requires`-Kanten. Deterministischer
Tie-Break über alphabetische Namensordnung bei mehreren gleichzeitig aktivierbaren Knoten
(P28-Konformität).

**4. Invarianten-Nachweis.** INV-PLUGIN-DEPENDENCY (v5 §2.3, hier erstmals mit Inhalt gefüllt): *Plugin-
Aktivierung prüft `requires`/`conflicts` topologisch vor Aktivierung* — durch die dreistufige Prüfreihenfolge
in `activate_all` (Konflikte → Lizenz → Topologie) vor jeder tatsächlichen `activate()`-Seiteneffekt-
Ausführung strikt eingehalten: kein Plugin wird aktiviert, bevor nicht die **gesamte** angeforderte Menge
als widerspruchsfrei und zyklenfrei verifiziert wurde (Alles-oder-Nichts-Semantik, vermeidet inkonsistente
Teilaktivierungszustände). P29 (kein globaler veränderlicher Zustand): `PluginRegistry` ist eine explizit
injizierte Instanz, keine `static`/`OnceLock`.

**5. Fehlerbehandlung.** `PluginError` als lokales Enum mit `From<PluginError> for ContextraError` (Muster
identisch zu `BanditError`, v5 §13); neue Variante `ContextraError::Plugin(PluginError)` (C.1).

**6. Test-Pflichten.** `crates/contextra/tests/plugin_registry_dependency_cycle.rs` (bereits als
Tier-3-Testpflicht in v5 §14.2 vorgemerkt, 🔴 zu bauen) — Property-Test mit zufällig generierten
`requires`/`conflicts`-Graphen (inkl. gezielt injizierten Zyklen), verifiziert, dass Zyklen stets erkannt und
**kein** Plugin in diesem Fall aktiviert wird (Seiteneffektfreiheit bei Abbruch).

**7. Migration, Feature-Flags, Aufwand, Priorität, Abhängigkeiten.** Additiv. Aufwand: mittel. Priorität:
mittel (Phase 1, Voraussetzung für B.2.4 `contextra_plugin_status` und für spätere ShadowMode→Default-Flip-
Automatisierung von B.3.1/B.3.2). **Harte Abhängigkeit (neu in v6):** setzt sowohl die `FeatureRing`-
Migration (C.0 Punkt 1) als auch die Relokation von `LicenseGate` nach `contextra-ports` (C.3) voraus — ohne
letztere müsste `PluginRegistry::activate_all` einen `&dyn contextra_license::LicenseGate` entgegennehmen,
was für ein Ring-0/4-Crate wie `contextra-ports`/`contextra` architektonisch unzulässig wäre (P27:
„Alle Traits in `contextra-ports`"). Zielcrates: `contextra-ports` (Trait + Registry), `contextra`
(Instanziierung in Composition Root).

---

### B.2.4 `contextra_plugin_status`-MCP-Tool

**1. IST/Problem.** Bestätigt abwesend (A.6, A.9 Punkt 6). Setzt B.2.3 (`PluginRegistry`) voraus.

**2. SOLL-Schnittstelle.**

```rust
// crates/contextra-mcp/src/plugin_status.rs — NEUE DATEI

/// Antwortform des `contextra_plugin_status`-Tools — listet alle aktiven
/// Plugins mit Version, Ring und Feature-Ring-Anforderung. Enthält KEINE
/// Lizenzschlüssel-Details (nur `FeatureRing`, kein Ticket-Inhalt) —
/// Sicherheitsgrenze analog zu anderen MCP-Antworten (keine
/// Credential-Leckage über die Produktgrenze, v5 §4.5).
#[derive(Debug, Serialize)]
pub struct PluginStatusResponse {
    pub plugins: Vec<PluginStatusEntry>,
    pub feature_ring_active: contextra_types::FeatureRing,
}

#[derive(Debug, Serialize)]
pub struct PluginStatusEntry {
    pub name: String,
    pub version: String,       // "major.minor.patch"
    pub ring: u8,
    pub feature_ring_required: contextra_types::FeatureRing,
}

pub async fn handle_plugin_status(
    registry: &contextra_ports::PluginRegistry,
) -> Result<PluginStatusResponse, McpError> {
    Ok(PluginStatusResponse {
        plugins: registry.snapshot().into_iter().map(Into::into).collect(),
        feature_ring_active: registry.current_feature_ring(),
    })
}
```

**3–5.** Entfällt (reiner Read-Only-Wrapper um B.2.3; kein neuer Algorithmus, keine neue Invariante über die
von `PluginRegistry` bereits garantierten hinaus, kein neuer Fehlertyp — `McpError` bestehend).

**6. Test-Pflichten.** `crates/contextra-mcp/tests/plugin_status_tool.rs` — verifiziert Tool-Manifest-
Registrierung (analog zu `contextra_explain`s Testmuster) und korrekte Serialisierung.

**7. Migration, Feature-Flags, Aufwand, Priorität, Abhängigkeiten.** Additiv, kein Breaking Change.
Aufwand: niedrig. Priorität: Phase 1 (nach B.2.3). Zielcrate: `contextra-mcp` (Ring 4).

---

### B.2.5 `contextra-infer-onnx` in `default-members` aufnehmen

**1. IST/Problem.** `contextra-infer-onnx` ist im Workspace vorhanden (verifiziert, A.3), aber **nicht** Teil
von `default-members` (verifiziert: fehlt in der `default-members`-Liste in `Cargo.toml`, A.3-Tabelle
Eintrag #22). Grund laut v5 §4.3: „Download-Overhead" der ONNX-Runtime. LightRAG und vergleichbare Systeme
nutzen Reranking standardmäßig (Wettbewerbsanalyse R6).

**2. SOLL-Schnittstelle.** Kein neuer Rust-Code — reine Workspace-Konfigurationsänderung, bedingt durch eine
vorgelagerte Kompatibilitätsprüfung:

```toml
# Cargo.toml — Zielzustand
default-members = [
    # … bestehende Einträge …
    "crates/contextra-infer-onnx",   # NEU, nach ARM/NEON-Verifikation
]
```

```toml
# crates/contextra-infer-onnx/Cargo.toml — neues Feature statt Komplettausschluss
[features]
default = []
reranking = ["dep:ort"]   # Ebene-A-Feature (v5 §12.1) statt Workspace-weitem Ausschluss
```

**3. Algorithmus & Komplexität.** N/A (Build-Konfiguration).

**4. Invarianten-Nachweis.** N/A — betrifft keine Laufzeit-Invariante.

**5. Fehlerbehandlung.** N/A.

**6. Test-Pflichten.** CI-Matrix-Erweiterung:
`cargo build -p contextra-infer-onnx --target aarch64-unknown-linux-gnu` (bzw. entsprechendes ARM-Target)
muss vor Aufnahme in `default-members` grün sein — NEON-Kernel-Kompatibilität der `ort`-Runtime-Bindings ist
die konkrete Vorbedingung (v5-Mobile-ML-Querverweis, Wettbewerbsanalyse R6).

**7. Migration, Feature-Flags, Aufwand, Priorität, Abhängigkeiten.** Potenziell Breaking für
minimal-Footprint-Deployments (größerer Standard-Build durch ONNX-Runtime-Download) — daher **erst nach**
Einführung des `reranking`-Feature-Flags, das den Ausschluss granularer macht als der bisherige komplette
`default-members`-Ausschluss. Aufwand: sehr niedrig (reine Konfiguration) **plus** die vorgelagerte
ARM-Verifikationsarbeit (mittel). Priorität: mittel. Abhängigkeiten: ARM/NEON-Kompatibilitätsnachweis als
Vorbedingung. Zielort: Workspace-`Cargo.toml`, `contextra-infer-onnx/Cargo.toml`.

---

### B.2.6 LoCoMo/LongMemEval-Ergebnisse publizieren

**1. IST/Problem.** `benchmarks/contextra-bench/src/locomo.rs` und `long_mem_eval.rs` sind implementiert
(v5 §4.6, ROADMAP §0 bestätigt: „✅ Implementiert … Benchmark-Harness ist fertig, Ergebnisse müssen nur
publiziert werden"). Es existiert bislang keine öffentliche Kennzahl — jeder relevante Konkurrent (Mem0,
Zep/Graphiti, Letta, Cognee) führt LoCoMo-/LongMemEval-Zahlen öffentlich (Wettbewerbsanalyse §2.1).

**2. SOLL — Prozess-Spezifikation (kein neuer Code).**
1. `cargo bench -p contextra-bench --bench locomo` und `--bench long_mem_eval` gegen
   `RetrievalStrategy::Hybrid` ausführen (Referenzkonfiguration).
2. Ergebnisse als **fünfte Kennzahl** neben Kaltstart/HNSW-Latenz/BM25F-Latenz/4-Signal-Fusion/WAL-
   Group-Commit (v5 §1.2) in dieselbe `criterion`-Benchmark-Suite integrieren — konsistent mit der in v5
   §1.1 postulierten „vierten Produktkennzahl" (Löschbeweis-Zeit) als **sechste** Kennzahl (Reihenfolge in
   README: Kaltstart → Footprint → p99-Latenz → LoCoMo/LongMemEval-Score → Löschbeweis-Zeit).
3. Direkter Vergleich mit öffentlich publizierten Mem0-/Zep-/Graphiti-Zahlen im README dokumentieren
   (Wettbewerbsanalyse R8).

**7. Aufwand, Priorität, Abhängigkeiten.** Aufwand: sehr niedrig (kein Code, nur Ausführung + Dokumentation).
Priorität: SOFORT (höchster Hebel pro Aufwand aller Backlog-Punkte, ROADMAP §7.1/§9 Q4-2026-Block). Keine
Abhängigkeiten.

---

## B.3 Phase 1½ — SOTA-Algorithmen im ShadowMode → Default-Flip (laufend)

### B.3.1 TL-HFD Default-Flip-Kriterium (formales ShadowMode-Gate)

**1. IST/Problem.** `PprAlgorithm::ShadowModeTlHfd` ist verdrahtet (`tl_hfd/shadow.rs`, v5 §6.3, 🟢), der
Default steht weiterhin auf dem bestehenden Forward-Push-Kern (`path_rag/mod.rs::forward_push_ppr`).
v5/ROADMAP benennen „Default-Flip pending (Diskrepanz-Logs auswerten)", ohne das Gate **formal** zu
definieren — diese Unschärfe wird hiermit geschlossen.

**2. SOLL-Schnittstelle.**

```rust
// crates/contextra-graph/src/tl_hfd/shadow.rs — Erweiterung um formales Gate

/// Aggregiertes Diskrepanz-Ergebnis über ein Auswertungsfenster —
/// Grundlage für die automatisierte (nicht nur manuelle) Default-Flip-
/// Entscheidung.
#[derive(Debug, Clone, PartialEq)]
pub struct ShadowDiscrepancyReport {
    /// Anzahl ausgewerteter Query-Paare (ForwardPush-Ergebnis vs.
    /// TL-HFD-ShadowMode-Ergebnis für dieselbe Anfrage).
    pub sample_count: u64,
    /// Mittlere Jaccard-Ähnlichkeit der Top-k-Ergebnismengen zwischen
    /// beiden Algorithmen.
    pub mean_topk_jaccard: f32,
    /// p99-Latenz-Differenz (TL-HFD − ForwardPush) in Mikrosekunden.
    /// Negativ = TL-HFD schneller.
    pub p99_latency_delta_us: f64,
    /// Anteil der Fälle, in denen TL-HFD einen um mehr als 20% höheren
    /// Recall@10 gegen eine gelabelte Referenzmenge erzielt (sofern
    /// verfügbar — sonst None).
    pub recall_improvement_ratio: Option<f32>,
}

/// Formales Default-Flip-Gate. Deterministisch aus einem
/// `ShadowDiscrepancyReport` berechnet — KEINE manuelle Ermessens-
/// entscheidung mehr, sondern ein reproduzierbares Kriterium, das
/// `xtask check-recall-stability` (bestehendes Gate, v5 §14.2) als
/// Datenquelle konsumieren kann.
pub trait DefaultFlipGate {
    /// true ⇔ alle drei Bedingungen erfüllt:
    /// 1. `sample_count >= MIN_SHADOW_SAMPLES` (Default: 10_000)
    /// 2. `mean_topk_jaccard >= MIN_AGREEMENT_THRESHOLD` (Default: 0.85 —
    ///    hohe Übereinstimmung mit dem bisherigen Verfahren als
    ///    Sicherheitsnetz gegen Qualitätsregression)
    /// 3. `p99_latency_delta_us <= 0.0` (TL-HFD darf nicht langsamer sein
    ///    als der bisherige Forward-Push-Kern — sonst kein Grund zum Flip
    ///    trotz TL-HFDs theoretisch überlegener Lokalität, arXiv:2606.09340)
    fn should_flip(&self, report: &ShadowDiscrepancyReport) -> bool;
}

pub const MIN_SHADOW_SAMPLES: u64 = 10_000;
pub const MIN_AGREEMENT_THRESHOLD: f32 = 0.85;
```

**3. Algorithmus & Komplexität.** Reine Aggregations-/Entscheidungslogik über bereits während des
ShadowMode-Betriebs anfallende Messwerte — kein neuer Retrieval-Algorithmus (TL-HFD selbst ist bereits
implementiert, v5 §6.3). $O(1)$ pro `should_flip()`-Aufruf.

**4. Invarianten-Nachweis.** P28 (Determinismus): `should_flip` ist eine reine Funktion über den
aggregierten Report — bei gleichem Report stets gleiches Ergebnis, keine Zeit-/Zufallsabhängigkeit.

**5. Fehlerbehandlung.** Kein neuer Fehlertyp — dies ist ein Entscheidungs-Prädikat, kein fehlerbehafteter
Pfad.

**6. Test-Pflichten.** `crates/contextra-graph/tests/tl_hfd_default_flip_gate.rs` — Tabellentest mit
synthetischen `ShadowDiscrepancyReport`-Werten an den drei Schwellenwert-Grenzen (knapp darüber/darunter je
Bedingung).

**7. Migration, Feature-Flags, Aufwand, Priorität, Abhängigkeiten.** Additiv. Aufwand: niedrig. Priorität:
mittel (formalisiert einen bereits laufenden Prozess, blockiert selbst nichts, macht aber den eigentlichen
Flip — sobald `should_flip() == true` gemessen wird — zu einer reinen Konfigurationsänderung statt einer
neuen Implementierung). Zielcrate: `contextra-graph` (Ring 0).

---

### B.3.2 FC-TS (Flow-Corrected Thompson Sampling) — Off-Policy-Kompatibilität + Default-Flip

**1. IST/Problem.** `contextra-adapt/src/flow_thompson.rs` (🟡 Grundgerüst, v5 §8.3) —
Off-Policy-Kompatibilität mit dem bestehenden `off_policy.rs`/`offpolicy.rs` (IPS-Korrektur) ist laut
v5/ROADMAP noch zu prüfen, bevor ein ShadowMode-Deployment sinnvoll ist.

**2. SOLL-Schnittstelle.**

```rust
// crates/contextra-adapt/src/flow_thompson.rs — Vervollständigung

/// Prüft, ob FC-TS' Sampling-Verteilung mit der bestehenden IPS
/// (Inverse Propensity Scoring)-Off-Policy-Korrektur aus `off_policy.rs`
/// konsistent kombinierbar ist — d. h. ob die von FC-TS gezogenen Arme
/// eine wohldefinierte, von Null verschiedene Propensity unter der
/// bisherigen Logging-Policy (`DiagonalApproximationBandit`) besitzen,
/// was Voraussetzung für unverzerrte IPS-Gewichtung ist (Positivitäts-
/// annahme des Off-Policy-Learning).
pub trait OffPolicyCompatibility {
    /// Gibt `Err` zurück, wenn für irgendeinen Arm eine Propensity von 0
    /// unter der Referenz-Policy gemessen wird, während FC-TS diesem Arm
    /// eine positive Sampling-Wahrscheinlichkeit zuweist (Verstoß gegen
    /// die Positivitätsannahme — IPS-Gewichte würden divergieren).
    fn verify_positivity(&self, reference_policy: &DiagonalApproximationBandit) -> Result<(), OffPolicyError>;
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum OffPolicyError {
    #[error("arm {0:?} has zero propensity under reference policy but positive FC-TS sampling probability — IPS weight would diverge")]
    ZeroPropensityViolation(contextra_types::RetrievalStrategy),
}
```

Der Default-Flip selbst folgt demselben formalen Gate-Muster wie B.3.1 (`BanditDefaultFlipGate`, analoge
Struktur mit bandit-spezifischen Metriken: kumulatives Regret statt Jaccard-Ähnlichkeit).

**3–5.** Algorithmus: `verify_positivity` ist ein $O(|\text{Arme}|)$-Scan über die Propensity-Verteilung
beider Policies (Armzahl typ. ≤ 8, siehe `ArmRegistry`). Invariante: P28 (Determinismus) — reine
mathematische Prüfung ohne RNG. Fehlerbehandlung: `OffPolicyError` mit `From` nach `ContextraError` (Muster
wie `BanditError`).

**6. Test-Pflichten.** `crates/contextra-adapt/tests/fc_ts_off_policy_positivity.rs` — Property-Test über
zufällig generierte Propensity-Verteilungspaare.

**7. Migration, Feature-Flags, Aufwand, Priorität, Abhängigkeiten.** Additiv. Aufwand: mittel. Priorität:
mittel–hoch (Phase 1½). Zielcrate: `contextra-adapt` (Ring 0).

---

### B.3.3 KIVI 2-Bit KV-Quantisierung — vollständige Read/Write-Integration

**1. IST/Problem.** `KiviQuantizedBlock`/`KiviBlockMeta` (`contextra-kvcache/src/quantize_kivi.rs`) sind
Metadaten-Structs; die tatsächliche Tensor-Quantisierung passiert laut ROADMAP §0/§3.2 „upstream" — nicht
in Contextra selbst. `KvSegment` (`segment.rs`) hat keinen quantisierten Schreib-/Lesepfad.

**2. SOLL-Schnittstelle.**

```rust
// crates/contextra-kvcache/src/segment.rs — Erweiterung

/// Erweitert den bisherigen (impliziten) Rohbyte-Inhaltstyp um einen
/// echten, in Contextra durchgeführten KIVI-Quantisierungspfad.
#[derive(Debug, Clone)]
pub enum KvSegmentContent {
    /// Bestehender Pfad — volle Präzision (f16/bf16, je nach Backend).
    Raw(bytes::Bytes),
    /// NEU: asymmetrische 2-Bit-Quantisierung nach KIVI (arXiv:2402.02750).
    /// Kernprinzip: Keys nutzen Per-Channel-Skalierung (dimensionale
    /// Outlier dominieren bei Keys), Values nutzen Per-Token-Skalierung
    /// (Token-basierte Outlier dominieren bei Values) — asymmetrisch,
    /// weil ein einheitliches Schema für beide suboptimal ist (KIVI-
    /// Kernbefund).
    KiviQuantized(KiviQuantizedBlock),
}

/// Quantisierungs-Konfiguration — wird VOR der AEAD-Verschlüsselung
/// angewendet (kritische Reihenfolge, siehe Invarianten-Nachweis unten).
#[derive(Debug, Clone, Copy)]
pub struct KiviQuantizeConfig {
    /// Gruppengröße für Key-Quantisierung (Default: 16, aus KIVI-Paper).
    pub key_group_size: usize,
    /// Ob Value-Quantisierung ebenfalls aktiv ist (Values sind
    /// empfindlicher gegen Präzisionsverlust; Default: true, aber
    /// separat abschaltbar für Qualitäts-Debugging).
    pub quantize_values: bool,
}

impl KvSegment {
    /// NEU: Schreibpfad mit Quantisierung. Reihenfolge (INV-KIVI-AEAD-ORDER,
    /// neue Invariante, siehe unten): Quantisierung ZUERST, AEAD-
    /// Verschlüsselung DANACH — niemals umgekehrt.
    pub fn write_quantized(
        &mut self,
        raw_kv: &KvTensorView,
        config: KiviQuantizeConfig,
        cipher: &dyn contextra_crypto::KvCipher,
    ) -> contextra_types::Result<()> {
        let quantized = kivi_quantize(raw_kv, config)?;   // 1. Quantisierung
        let encrypted = cipher.seal(&quantized.to_bytes())?; // 2. AEAD danach
        self.content = KvSegmentContent::KiviQuantized(quantized);
        self.encrypted_payload = encrypted;
        Ok(())
    }

    /// NEU: Lesepfad mit Dequantisierung — Entschlüsselung ZUERST,
    /// Dequantisierung DANACH (umgekehrte Reihenfolge zum Schreibpfad,
    /// wie bei jeder AEAD-Sandwich-Konstruktion erwartet).
    pub fn read_dequantized(
        &self,
        cipher: &dyn contextra_crypto::KvCipher,
    ) -> contextra_types::Result<KvTensorView> {
        let decrypted = cipher.open(&self.encrypted_payload)?;
        match &self.content {
            KvSegmentContent::KiviQuantized(meta) => kivi_dequantize(&decrypted, meta),
            KvSegmentContent::Raw(_) => KvTensorView::from_bytes(&decrypted),
        }
    }
}

/// Kernquantisierungsfunktion (reine Numerik, Ring-0-kompatibel obwohl
/// in Ring-1-Crate `contextra-kvcache` beheimatet — keine I/O, kein
/// tokio-Aufruf innerhalb dieser Funktion, P26-konform als reine
/// Rechenfunktion auch außerhalb von Ring 0 zulässig, da P26 nur
/// "kein tokio in Ring 0" verlangt, nicht "keine reinen Funktionen
/// außerhalb Ring 0").
fn kivi_quantize(raw: &KvTensorView, config: KiviQuantizeConfig) -> contextra_types::Result<KiviQuantizedBlock>;
fn kivi_dequantize(bytes: &[u8], meta: &KiviBlockMeta) -> contextra_types::Result<KvTensorView>;
```

**Spill-Pfad-Ergänzung (Tier-2-AEAD, CacheGen-inspiriert):**

```rust
// crates/contextra-kvcache/src/segment.rs — Kompression vor Spill
// Quantisierte Blöcke werden zusätzlich mit einer generischen
// verlustfreien Kompression (z. B. LZ4, bereits Workspace-Dependency-
// Kandidat) komprimiert, BEVOR sie AEAD-verschlüsselt gespillt werden —
// Reihenfolge: Quantisierung → Kompression → AEAD-Verschlüsselung → Spill.
// Nie Klartext (weder quantisiert noch komprimiert) auf Platte.
```

**3. Algorithmus & Komplexität.** Quantisierung: $O(n)$ in der Anzahl der Tensor-Elemente (ein
Skalierungsfaktor-Berechnungs- plus ein Pack-Durchlauf pro Gruppe der Größe `key_group_size`).
Speicherreduktion: 2 Bit statt 16 Bit pro Wert = 87,5 % Reduktion auf Rohdatenebene, ROADMAP nennt
konservativer „75% RAM-Reduktion" unter Berücksichtigung von Metadaten-Overhead (Skalierungsfaktoren,
Zero-Points pro Gruppe).

**4. Invarianten-Nachweis.** **INV-KIVI-AEAD-ORDER (neu):** *Quantisierung erfolgt strikt vor
AEAD-Verschlüsselung beim Schreiben; Entschlüsselung strikt vor Dequantisierung beim Lesen.* Begründung:
AEAD-Chiffretext ist pseudozufällig verteilt — eine nachträgliche Quantisierung von Chiffretext wäre
bedeutungslos (keine numerische Struktur mehr vorhanden) und würde zudem die AEAD-Authentizitätsgarantie
brechen (Chiffretext-Modifikation nach Verschlüsselung). Durch die Typsignatur von
`write_quantized`/`read_dequantized` (Quantisierung/Dequantisierung sind interne Implementierungsschritte
der jeweiligen Methode, nicht separat vom Aufrufer kombinierbar) ist die Reihenfolge **API-seitig
erzwungen**, nicht nur dokumentiert. P24 (Lokalität): Quantisierung operiert gruppenweise
(`key_group_size`), nicht auf dem gesamten Cache — inkrementelles Schreiben neuer Blöcke bleibt
$O(\text{Blockgröße})$.

**5. Fehlerbehandlung.** Neue Variante `ContextraError::KvQuantization(String)` (C.1) für
Dequantisierungs-Fehler (z. B. Metadaten-Inkonsistenz zwischen `KiviBlockMeta` und tatsächlicher
Byte-Länge).

**6. Test-Pflichten.** `crates/contextra-kvcache/tests/kivi_quantize_dequantize_roundtrip.rs` —
Property-Test: für zufällige Tensor-Werte muss Dequantize(Quantize(x)) innerhalb einer durch die
2-Bit-Auflösung vorgegebenen Fehlertoleranz liegen (kein exaktes Roundtrip, aber ein mathematisch
begründetes Toleranzband). `crates/contextra-kvcache/tests/kivi_aead_order_enforced.rs` — verifiziert, dass
`write_quantized`/`read_dequantized` keine Möglichkeit bieten, die Reihenfolge zu vertauschen (API-Test,
kein Verhaltenstest). Golden-Test: End-to-End-Vergleich generierter Logits mit/ohne KIVI-Quantisierung gegen
ein Referenzmodell (Qualitäts-Regressionsschutz, analog zu v5 §9.1 Stufe-B-Gate „Golden-Test +
Cancellation-Test").

**7. Migration, Feature-Flags, Aufwand, Priorität, Abhängigkeiten.** Additiv (`KvSegmentContent::
KiviQuantized` als neue Enum-Variante neben `Raw`) — SemVer MINOR. Feature-Flag: `kivi-quantization` (Ebene
A, Cargo-Feature in `contextra-kvcache`). Aufwand: mittel–hoch. Priorität: HIGH. **Abhängigkeit:** wird laut
ROADMAP §1 Tier-1-Hebelwirkungsanalyse erst „sinnvoll", nachdem B.2.1 (SnapKV/H2O-Wiring) die
Eviction-Qualität sichergestellt hat — empfohlene Sequenz: B.2.1 vor B.3.3. Zielcrate: `contextra-kvcache`
(Ring 1).

---

### B.3.4 `RetrievalStrategy::Global` — Corpus-weite Themenfragen (LeanRAG-backed)

**1. IST/Problem.** `RetrievalStrategy` (`crates/contextra-types/src/retrieval_strategy.rs`) hat verifiziert
genau vier Varianten: `Vector`, `Text`, `Graph`, `Hybrid` (A.9 Punkt 3). LightRAG (lokal/global) und
Microsoft GraphRAG (Community-Reports) bedienen Korpus-weite Themenfragen; Contextra hat trotz vorhandener
LeanRAG-Aggregation (`contextra-cognition`) und Leiden-Community-Detection (`contextra-graph/src/
community.rs`) keinen entsprechenden Retrieval-Modus.

**2. SOLL-Schnittstelle.**

```rust
// crates/contextra-types/src/retrieval_strategy.rs — Erweiterung

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]   // NEU: bislang nicht non_exhaustive (A.9 Punkt 3) —
                    // wird mit dieser Erweiterung eingeführt, um künftige
                    // Varianten ohne erneuten MAJOR-Bump zu erlauben
                    // (Abweichung von der SignalKind-Regel "NIEMALS neue
                    // Varianten", da RetrievalStrategy im Gegensatz zu
                    // SignalKind kein Bit-Flag-artiges Fusionssignal ist,
                    // sondern eine Auswahl-Enumeration, für die zukünftige
                    // Erweiterung ausdrücklich vorgesehen ist)
pub enum RetrievalStrategy {
    Vector,
    Text,
    Graph,
    Hybrid,
    /// Nutzt LeanRAG-aggregierte Hyperkanten-Cluster statt Rohdokumente —
    /// für Fragen wie "Was sind die Hauptthemen in meiner Wissensbasis?".
    Global {
        /// Obergrenze der einbezogenen Community-Knoten (Default:
        /// `DEFAULT_MAX_LEANRAG_NODES`, wiederverwendet aus
        /// `contextra-cognition`).
        max_community_nodes: Option<usize>,
        /// Filtert Communities unterhalb dieser Mitgliederzahl heraus
        /// (Rauschunterdrückung, Default: 3).
        min_community_size: Option<usize>,
    },
}
```

```rust
// crates/contextra-rank/src/fusion/global.rs — NEUE DATEI

/// Sucht primär gegen LeanRAG-Aggregationsknoten
/// (`aggregation_phase.rs`-Output) statt gegen Rohdokumente. Führt
/// Community-aware RRF durch: Leiden-Communities (bestehend,
/// `community.rs`) dienen als natürliche Clustering-Ebene, über die
/// hinweg Reciprocal-Rank-Fusion angewendet wird.
pub struct GlobalFusionStrategy {
    config: GlobalFusionConfig,
}

impl FusionStrategy for GlobalFusionStrategy {
    fn fuse(&self, signals: &[SignalResult], query: &FusionQuery) -> Result<Vec<ScoredDocument>> {
        // 1. Community-Zugehörigkeit jedes Aggregationsknotens ermitteln
        //    (bereits vorhandene Leiden-Zuordnung wiederverwenden).
        // 2. Pro Community: RRF über die community-internen Signal-Ranglisten.
        // 3. Communities selbst nach aggregierter Relevanz ranken
        //    (Community-Score = Summe/Mittel der internen Top-Scores).
        // 4. Ergebnis: geordnete Liste von Aggregationsknoten (nicht
        //    Rohdokumenten) — Aufrufer (contextra-mcp) kennzeichnet dies
        //    in der Antwort explizit als "aggregiertes Themen-Ergebnis".
        unimplemented!("siehe Implementierungs-Roadmap")
    }
}
```

```rust
// crates/contextra-router/src/arm_registry.rs — Erweiterung
// ArmRegistry erhält einen 5. Arm (Global) — arm_for(RetrievalStrategy::Global{..})
// muss auf denselben u32-Slot-Mechanismus abgebildet werden wie die
// bestehenden vier Arme (bereits behobene ArmRegistry, v5 §4.4).
```

**3. Algorithmus & Komplexität.** Community-aware RRF: $O(|C| \cdot \bar{n} \log \bar{n})$, wobei $|C|$ die
Anzahl relevanter Communities (durch `max_community_nodes` begrenzt) und $\bar{n}$ die mittlere
Community-Größe ist — durch die beiden Konfigurationsgrenzen (`max_community_nodes`, `min_community_size`)
P24-konform beschränkt, nicht proportional zur Gesamtkorpusgröße.

**4. Invarianten-Nachweis.** P24: siehe oben, harte Obergrenze über `max_community_nodes`.
Integrationsregel (v5 §13/ROADMAP-Konvention): **ShadowMode-Pflicht** — `RetrievalStrategy::Global` wird
zunächst nur über einen expliziten ShadowMode-Vergleichspfad ausgewertet (Diskrepanz-Logging gegen
`RetrievalStrategy::Hybrid` als Baseline), bevor sie über den Bandit-Router als regulär wählbarer Arm
freigeschaltet wird — dasselbe Muster wie B.3.1 (TL-HFD).

**5. Fehlerbehandlung.** Kein grundsätzlich neuer Fehlertyp; `ContextraError::InvalidInput` bei leerem
Community-Index (z. B. Collection ohne aktivierte Konsolidierung — `Global`-Strategie erfordert vorherige
`contextra_consolidate`-Ausführung, sonst degradiert die Anfrage kontrolliert mit einem informativen Fehler
statt eines leeren, irreführenden Ergebnisses).

**6. Test-Pflichten.** `crates/contextra-rank/tests/global_fusion_community_aware.rs`,
`crates/contextra-router/tests/arm_registry_five_arms.rs`, ShadowMode-Diskrepanz-Test analog B.3.1.

**7. Migration, Feature-Flags, Aufwand, Priorität, Abhängigkeiten.** `#[non_exhaustive]` auf
`RetrievalStrategy` mindert das Breaking-Risiko für Downstream-Match-Statements (fügt aber selbst durch
Einführung von `non_exhaustive` einen MINOR-Migrationsschritt hinzu, da bestehende exhaustive
`match`-Statements einen `_`-Catch-All benötigen — SemVer MINOR mit Migrationshinweis). Aufwand: mittel.
Priorität: HIGH (ROADMAP Tier-1-Multiplikator: „Neue Anwendungsklasse … Parität mit LightRAG/GraphRAG").
Abhängigkeiten: setzt funktionierende LeanRAG-Aggregation voraus (bereits 🟢 vorhanden). Zielcrates:
`contextra-types` (Enum), `contextra-rank` (Fusion-Strategie), `contextra-router` (5. Arm).

---

### B.3.5 Leyline-inspirierte KV-Cache-Direktiven für Agenten-Workflows

**1. IST/Problem.** `KvReusePolicy::{Always, CostBased, Never}` (bestehend) ist rein automatisch — Agenten
haben keinen deklarativen Einfluss auf Caching-Entscheidungen (z. B. explizites Pinning des
System-Prompts).

**2. SOLL-Schnittstelle.**

```rust
// crates/contextra-kvcache/src/store.rs — neue Directive-API

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheDirective {
    /// Segment IMMER cachen (z. B. System-Prompt, Tool-Definitionen).
    Pin { ttl: Option<std::time::Duration> },
    /// Segment NIEMALS cachen (z. B. sensitive transiente Daten) —
    /// garantiert, dass dieses Segment nie in ein Tier-2-AEAD-Spill
    /// gelangt, auch nicht kurzzeitig.
    NeverCache,
    /// Automatische Entscheidung (bisheriger `KvReusePolicy`-Default).
    Auto,
    /// Nach diesem Agent-Step das Segment freigeben.
    ReleaseAfterStep { step_id: contextra_types::StepId },
}
```

```rust
// crates/contextra-agent/src/context.rs — Integration
// AgentEngine::run() kann pro Step Cache-Direktiven setzen:
// System-Prompt → Pin { ttl: None } → garantierter Cache-Hit < 100 µs
//   (v5 §1.2 Performance-Zielwert "KV-Cache-Tier-1-Treffer < 100 µs")
// Transiente Reasoning-Steps → ReleaseAfterStep → kein Speicherleck
```

**3. Algorithmus & Komplexität.** Keine neue Retrieval-/Eviction-Algorithmik — `CacheDirective` ist ein
Prioritäts-Override, der VOR der bestehenden `rank_for_eviction[_weighted]`-Bewertung geprüft wird:
`Pin`-Segmente werden aus der Eviction-Kandidatenmenge ausgeschlossen (bis TTL-Ablauf), `NeverCache`-
Segmente werden nie in den Cache aufgenommen (kurzschließt den Store-Pfad komplett). $O(1)$-Overhead pro
Segment-Zugriff (ein zusätzlicher Enum-Vergleich).

**4. Invarianten-Nachweis.** INV-5-Analogon (Speicherbudget-Transparenz): `Pin { ttl: None }`
(unbegrenztes Pinning) muss gegen ein Gesamt-Pin-Budget geprüft werden (neue Konstante
`MAX_PINNED_BYTES_PER_TENANT`, siehe C.2) — verhindert, dass exzessives Pinning das reguläre
Eviction-System aushebelt und den `DEFAULT_BYTE_BUDGET_PER_TENANT` (v5 §15.2) faktisch umgeht.

**5. Fehlerbehandlung.** Neue Variante `ContextraError::PinBudgetExceeded` (C.1) bei Überschreitung von
`MAX_PINNED_BYTES_PER_TENANT`.

**6. Test-Pflichten.** `crates/contextra-kvcache/tests/cache_directive_pin_survives_eviction.rs`,
`crates/contextra-agent/tests/system_prompt_pinning_latency.rs` (Performance-Regressionstest gegen den <
100 µs-Zielwert).

**7. Migration, Feature-Flags, Aufwand, Priorität, Abhängigkeiten.** Additiv. Aufwand: niedrig–mittel.
Priorität: HIGH (ROADMAP §3.4). Zielcrates: `contextra-kvcache` (Directive-API + Budget-Prüfung),
`contextra-agent` (Integration).

---

### B.3.6 C²KV/Irminsul — Content-Addressed KV-Cache (positions-unabhängig)

**1. IST/Problem.** `PrefixRadixTree` (`radix.rs`) ist strikt positions-/präfixabhängig — zwei inhaltlich
ähnliche, aber unterschiedlich formulierte Prompts (z. B. „Zusammenfasse X" vs. „Erkläre X") teilen keinen
Cache, obwohl sie ggf. überlappende Kontext-Segmente referenzieren.

**2. SOLL-Schnittstelle.**

```rust
// crates/contextra-kvcache/src/radix.rs — Erweiterung

/// Content-adressierter KV-Store als Ergänzung (nicht Ersatz) zum
/// bestehenden `PrefixRadixTree`. Zwei-Ebenen-Lookup-Strategie.
pub struct ContentAddressedKvStore {
    /// Primärindex: TenantId + BLAKE3(token_ids) → CacheEntry.
    /// BLAKE3 wiederverwendet aus contextra-crypto (bereits
    /// Workspace-Dependency, konsistent mit MANIFEST-Hashing, v5 §4.2).
    content_index: ahash::AHashMap<(contextra_types::TenantId, blake3::Hash), KvSegmentRef>,
    /// Sekundärindex: bestehende reine Präfix-Struktur, unverändert.
    position_index: PrefixRadixTree,
}

/// Lookup-Kaskade, vom günstigsten zum teuersten Pfad:
pub enum KvLookupResult {
    /// < 100 µs — exakter Präfix-Treffer (bestehender Pfad, unverändert).
    ExactPrefixHit(KvSegmentRef),
    /// < 500 µs — Content-Hash-Treffer (NEU): identische Token-Sequenz,
    /// aber an anderer Position im Kontext (z. B. wiederverwendeter
    /// Tool-Definitions-Block in unterschiedlicher Gesprächsreihenfolge).
    ContentHashHit(KvSegmentRef),
    /// < 2 ms — optionale Semantic-Similarity-Hit-Klasse (HNSW auf
    /// Embeddings der Cache-Keys selbst) — NUR wenn ein Embedder für
    /// Cache-Keys konfiguriert ist (opt-in, da zusätzlicher Embedding-
    /// Aufruf pro Lookup-Miss Kosten verursacht).
    SemanticSimilarityHit { segment: KvSegmentRef, similarity: f32 },
    Miss,
}

impl ContentAddressedKvStore {
    pub fn lookup(&self, tenant: contextra_types::TenantId, token_ids: &[u32]) -> KvLookupResult {
        // 1. position_index.lookup_prefix(token_ids) — bestehender Pfad
        // 2. Falls Miss: content_index.get(&(tenant, blake3::hash(token_ids_as_bytes)))
        // 3. Falls weiterhin Miss UND semantic_embedder konfiguriert:
        //    HNSW-Suche gegen Cache-Key-Embeddings (Schwellenwert-gated)
        unimplemented!("siehe Implementierungs-Roadmap")
    }
}
```

**3. Algorithmus & Komplexität.** Content-Hash-Lookup: $O(|token\_ids|)$ für Hashing + $O(1)$ amortisiert
für Hash-Map-Lookup — schneller als eine erneute Inferenz, aber langsamer als exakter Präfix-Treffer
(Hash-Berechnungskosten). Semantic-Similarity-Hit: $O(\log n)$ über HNSW auf einer typischerweise kleinen
Menge von Cache-Key-Embeddings.

**4. Invarianten-Nachweis.** P24 (Lokalität): Content-Hash-Lookup ist pro Anfrage konstant in der
Tokenlänge, unabhängig von der Cache-Gesamtgröße. Mandanten-Scoping: `TenantId` ist Teil des Hash-Map-
Schlüssels — verhindert Cross-Tenant-Cache-Leckage über Content-Hashes (kritische Sicherheitseigenschaft:
ein Tenant darf nie einen KV-Cache-Treffer eines anderen Tenants erhalten, selbst bei identischem
Token-Inhalt).

**5. Fehlerbehandlung.** Kein neuer Fehlertyp — `KvLookupResult::Miss` ist der reguläre Nicht-Fehler-Pfad.

**6. Test-Pflichten.** `crates/contextra-kvcache/tests/content_addressed_cross_tenant_isolation.rs`
(Sicherheits-Tier-0-Test: verifiziert, dass identischer Token-Inhalt unter verschiedenen `TenantId`s **nie**
einen Cache-Treffer über Tenant-Grenzen hinweg erzeugt),
`crates/contextra-kvcache/tests/content_hash_hit_rate_improvement.rs` (Benchmark-Regressionstest gegen
ROADMAP-Zielwert „+20–35% Cache-Trefferquote bei agentic Workloads").

**7. Migration, Feature-Flags, Aufwand, Priorität, Abhängigkeiten.** Additiv (neue Struktur neben
bestehendem `PrefixRadixTree`, keine Breaking Change). Feature-Flag: `content-addressed-kv-cache` (Ebene A).
Aufwand: hoch. Priorität: MEDIUM (ROADMAP §3.5). Zielcrate: `contextra-kvcache` (Ring 1).

---

### B.3.7 Budgeted-k-Path-Diffusion (PathRAG-Erweiterung)

**1. IST/Problem.** `PathRAGEngine::find_path` (`path_rag/mod.rs`) liefert genau einen Pfad über
bidirektionalen Dijkstra, begrenzt nur durch `max_steps = max_hops * 1000` (kein hartes Knotenbudget wie
`MAX_VISITED_NODES` im CSR-Layer) — bei Hub-Entitäten mit hohem Fan-out kann der `HashMap`-Speicher für
`dist_fwd`/`dist_bwd` unkontrolliert wachsen (SOTA-Bericht §5.1.2, detaillierte Analyse dort bereits
vollständig ausgearbeitet). Diese Spezifikation referenziert die dort bereits vollständige Ausarbeitung.

**2–6. SOLL-Schnittstelle, Algorithmus, Invarianten, Fehlerbehandlung, Tests.** Vollständig spezifiziert in
`Contextra_SOTA_Forschungsbericht_2025-2026.md` §5.1.3–§5.1.9 (`KPathConfig`, `KPathResult`,
`KPathDiffusion`-Trait, Algorithmus `BudgetedKPath`, Invarianten Inv-1 bis Inv-3, Property-Test-Roadmap) —
dieses Dokument übernimmt die dortige Spezifikation unverändert als normativ und ergänzt sie ausschließlich
um die Einordnung in die Gesamt-Priorisierung (Teil D) sowie die konsolidierte Fehlertaxonomie (C.1:
`ContextraError::PathBudgetExhausted` wird dort als **Nicht-Fehler** behandelt — `budget_exhausted: true`
im `KPathResult` ist ein regulärer, informativer Rückgabewert, kein `Err`, was in der SOTA-Quelle bereits
korrekt so spezifiziert, aber hier explizit als konsolidierte Fehlertaxonomie-Entscheidung festgehalten
wird).

**7. Migration, Feature-Flags, Aufwand, Priorität, Abhängigkeiten.** Additiv (reiner Trait-Zusatz,
`find_path` bleibt unverändert). Feature-Flag: optional `k-path-diffusion` für Binärgrößen-sensitive
`fast`-Ring-Minimalbuilds. Aufwand: mittel. Priorität: MEDIUM (SOTA-Score 23). Zielcrate: `contextra-graph`
(Ring 0), neue Datei `path_rag/k_path.rs`.

---

### B.3.8 Catoni/Sequential-Change-Detection als zweite Drift-Quelle (Ensemble)

**1. IST/Problem.** `LyapunovDriftWatcher` (v5 §8.4, 🟢 produktiv) ist die einzige Drift-Evidenzquelle —
nicht robust gegen Outlier-Rewards (ein einzelner extremer Reward-Ausreißer kann eine Drift-Detektion
fälschlich auslösen oder unterdrücken, je nach Fensterparametrisierung).

**2. SOLL-Schnittstelle.**

```rust
// crates/contextra-adapt/src/drift.rs — Erweiterung (Modul existiert bereits)

/// Catoni-M-Schätzer für robuste Drift-Erkennung (arXiv:2505.20051) —
/// deterministisch gegeben Daten, keine Randomisierung (P28-konform).
pub struct CatoniDriftDetector {
    /// Robuste Mittelwert-Schätzung mit Influenzfunktion ψ(x) = x/(1+|x|)
    /// statt arithmetischem Mittel — dämpft den Einfluss einzelner
    /// Ausreißer-Rewards, ohne sie komplett zu verwerfen (im Unterschied
    /// zu einem harten Trimming-Verfahren).
    catoni_mu: f64,
    /// Sequential-Change-Point-Detection (CUSUM-artig, arXiv:2501.10974),
    /// O(1) amortisierter Update pro Beobachtung.
    cusum_sum: f64,
    threshold: f64,
}

pub trait DriftDetector {
    fn observe(&mut self, reward: f32) -> DriftSignal;
    fn is_drifting(&self) -> bool;
}

impl DriftDetector for CatoniDriftDetector { /* … */ }

/// Ensemble-Kombination: NUR wenn BEIDE Detektoren (Lyapunov UND Catoni)
/// unabhängig voneinander Drift melden, wird ein `PrecisionMatrixReset`
/// im Bandit-Router ausgelöst — reduziert False-Positive-Rate bei
/// verrauschten Reward-Signalen ggü. Einzelquellen-Entscheidung.
pub struct EnsembleDriftWatcher {
    lyapunov: LyapunovDriftWatcher,
    catoni: CatoniDriftDetector,
}
impl EnsembleDriftWatcher {
    pub fn observe_and_decide(&mut self, reward: f32) -> bool {
        let l = self.lyapunov.observe_score(reward);
        let c = self.catoni.observe(reward);
        l.is_drift_detected() && matches!(c, DriftSignal::Detected)
    }
}
```

**3. Algorithmus & Komplexität.** $O(1)$ amortisiert pro Beobachtung für beide Detektoren (kein
Speicher-Fenster-Wachstum, im Gegensatz zu einem naiven gleitenden Fenster über alle historischen
Rewards).

**4. Invarianten-Nachweis.** P28: Catoni-Schätzer und CUSUM sind beide deterministische Funktionen der
Beobachtungssequenz, keine RNG-Nutzung — Ensemble-Kombination bleibt deterministisch.

**5. Fehlerbehandlung.** Kein neuer Fehlertyp.

**6. Test-Pflichten.** `crates/contextra-adapt/tests/catoni_robust_to_outliers.rs` — Differential-Test:
injiziert einzelne extreme Reward-Ausreißer in einen sonst stabilen Reward-Strom; verifiziert, dass
`CatoniDriftDetector` **nicht** fälschlich Drift meldet, während ein naiver arithmetischer Mittelwert-
Detektor dies täte (Referenzimplementierung im Test selbst).

**7. Migration, Feature-Flags, Aufwand, Priorität, Abhängigkeiten.** Additiv. Aufwand: niedrig. Priorität:
MEDIUM (SOTA-Score 19.5). Zielcrate: `contextra-adapt` (Ring 0).

---

# CONTEXTRA INTELLIGENT AGENT INFRASTRUCTURE (CIAI)
## Vollspezifikation v1.0 — Autonome KI-Infrastruktur-Erweiterungsschicht

**Verifikationsbasis:** HEAD `2028cd0f` · 26. September 2026  
**Quellen:** Alle neun Architekturdokumente + Live-Repo-Analyse  
**Normativität:** Gleichrangig zu `CONTEXTRA_MASTER_SPEC_v6_1_.md`. Erweiterung, kein Ersatz.  
**Reifekennzeichnung:** 🟢 vorhanden · 🟡 Grundgerüst · 🔴 neu zu bauen · 🔒 kommerziell  
**Designphilosophie:** *"Kein Agent soll je wieder manuell Kontext verwalten, Wissen warten oder Retrieval-Strategien wählen müssen. Contextra entscheidet — deterministisch, erklärbar, ressourcenschonend."*

---

## Inhaltsverzeichnis

**Teil E — Autonome Intelligenzschicht: Architektur & Design**  
E.0 Gesamtbild, Designprinzipien, Crate-Graph-Erweiterung  
E.1 AutoPilot Context Engine (ACE) — "One call, perfect context"  
E.2 KnowledgeWeaver — Kontinuierliche autonome Wissenspflege  
E.3 SemanticConflictGuard — Widerspruchserkennung & -auflösung  
E.4 PredictivePrefetcher — Antizipatorisches KV-Cache-Loading  
E.5 AgentPool & TrustWeave — Multi-Agenten-Wissensteilung  
E.6 CompositeQueryRouter — Intelligente Anfrage-Dekomposition  
E.7 AdaptiveIngestionPipeline — Ressourcenschonende Wissensakquise  
E.8 TemporalMemoryManager — Vergessen, Auffrischung, Saisonalität  
E.9 CognitiveBudgetController — Ressourcen-bewusste Operationssteuerung  
E.10 GroundingOracle — Halluzinationsprävention & Faktenverankerung  
E.11 SelfHealingPipeline — Autonome Fehlerkorrektur  
E.12 FrictionlessPersonalization — Zero-Effort-Nutzeradaption

**Teil F — Neue Crates & Ring-Zuordnung**  
**Teil G — Vollständige Invariantentabelle (neu)**  
**Teil H — Konsolidierte Implementierungsroadmap mit Abhängigkeitsgraph**

---

# Teil E — Autonome Intelligenzschicht

## E.0 Gesamtbild und Designprinzipien

### E.0.1 Das Problem, das CIAI löst

Bestehende KI-Agentensysteme leiden unter fünfzehn strukturellen Problemen:

| # | Problem | Aktueller Workaround | CIAI-Lösung |
|---|---|---|---|
| P-1 | Kontext-Fenster wird manuell gefüllt — Agent weiß nicht, was wirklich relevant ist | Manuelles RAG mit fest verdrahteten Top-k | ACE (E.1): automatisch, budget-bewusst |
| P-2 | Wissensbasis veraltet ohne Wartungsaufwand | Regelmäßiges manuelles Re-Embedding | KnowledgeWeaver (E.2): autonomer Hintergrundprozess |
| P-3 | Widersprüche akkumulieren sich unbemerkt | Gar nichts | SemanticConflictGuard (E.3): automatische Erkennung + Auflösung |
| P-4 | KV-Cache kalt bei vorhersehbaren Anfragen | Manuelles Pre-Warming | PredictivePrefetcher (E.4): graph-basierte Antizipation |
| P-5 | Mehrere Agenten lernen dasselbe unabhängig | Kein Sharing | AgentPool (E.5): Privacy-preserving knowledge sharing |
| P-6 | Komplexe Anfragen werden als Ganzes gestellt, obwohl sie Teilanfragen sind | Manuell zerlegen | CompositeQueryRouter (E.6): automatische Dekomposition |
| P-7 | Dokument-Chunking ist einheitlich, ignoriert Inhaltsdichte | Feste Chunk-Größen | AdaptiveIngestionPipeline (E.7): semantisch adaptiv |
| P-8 | Altes Wissen belegt Speicher, verdrängt Neues | Manuelle Löschung | TemporalMemoryManager (E.8): autonomes Decay + Forgetting |
| P-9 | LLM-Aufrufe ignorieren aktuelle Ressourcenlage | Timeout-Heuristiken | CognitiveBudgetController (E.9): dynamisches Degradieren |
| P-10 | Halluzinationen werden erst nach Generation erkannt | Post-hoc GASP (teilw.) | GroundingOracle (E.10): präventive + post-hoc Verifikation |
| P-11 | Fehler brechen Pipelines komplett ab | Try-Catch manuell | SelfHealingPipeline (E.11): automatische Recovery |
| P-12 | Personalisierung erfordert explizites Feedback | Kein implizites Lernen | FrictionlessPersonalization (E.12): zero-shot via RIE |
| P-13 | Graph-Wissen degradiert durch Löschungen | Manuelle Reparatur | KnowledgeWeaver + HNSW-Repair (B.1.1) |
| P-14 | Retrieval-Strategie fix verdrahtet | Manuell konfiguriert | CompositeQueryRouter + Bandit (E.6 + bestehend) |
| P-15 | Agenten verlieren Langzeitkontext über Sessions | Session-Neustart | TemporalMemoryManager + CompactionSession (bestehend) |

### E.0.2 Acht Designprinzipien der CIAI-Schicht

```
CIAI-DP-1: BEQUEMLICHKEIT ÜBER VOLLSTÄNDIGKEIT
  "Ein Aufruf für 80% der Fälle; opt-in für 100%."
  Jedes Subsystem hat einen Zero-Config-Default, der ohne Konfiguration korrekt arbeitet.
  Komplexe Parametrierung ist immer optional.

CIAI-DP-2: RESSOURCENBUDGETS ALS ERSTE-KLASSE-BÜRGER
  Jeder Subsystem-Aufruf nimmt ein WorkBudget entgegen.
  Bei Budget-Erschöpfung: graceful degradation (partial result),
  NIEMALS panic oder kompletter Abbruch.

CIAI-DP-3: ERKLÄRBARKEIT JEDER AUTONOMEN ENTSCHEIDUNG
  Jede automatische Entscheidung produziert ein AuditToken,
  das über contextra_explain abrufbar ist.
  "Warum hast du das in den Kontext gelegt?" muss beantwortbar sein.

CIAI-DP-4: KOMPOSITION STATT MONOLITH
  Jedes CIAI-Subsystem ist ein eigenständiger, austauschbarer Port (P27).
  Sie können unabhängig aktiviert, deaktiviert oder ersetzt werden.

CIAI-DP-5: DETERMINISMUS UNTER GEGEBENEN INPUTS (P28)
  Gleiche Eingabe + gleicher Zustand = gleiche Ausgabe.
  KEINE thread_rng(), KEIN SystemTime::now() im Entscheidungspfad.
  Clocks und RNG werden immer über Ports injiziert.

CIAI-DP-6: ZERO-COPY-FIRST IM HOT-PATH
  Kontext-Bytes werden als Arc<[u8]> / Bytes übergeben, nie geklont.
  Jede neue Datenstruktur in E.1–E.12 muss dieses Muster respektieren.

CIAI-DP-7: MANDANTEN-ISOLATION UNVERLETZLICH (INV-1)
  Kein CIAI-Subsystem darf Daten zwischen Tenants mixen.
  AgentPool (E.5) hat strikt opt-in cross-tenant sharing.

CIAI-DP-8: RING-KONFORME SCHICHTUNG
  Neue CIAI-Orchestrierungslogik liegt in Ring 3 (contextra-autopilot).
  Algorithmen in Ring 0. Keine Rückwärtskanten.
```

### E.0.3 Neues Crate: `contextra-autopilot` (Ring 3)

```toml
# crates/contextra-autopilot/Cargo.toml
[package]
name = "contextra-autopilot"
# Workspace-Vererbung

[dependencies]
# Ring 0 — erlaubt
contextra-types    = { workspace = true }
contextra-ports    = { workspace = true }
contextra-adapt    = { workspace = true }
contextra-rank     = { workspace = true }
contextra-graph    = { workspace = true }

# Ring 1 — erlaubt
contextra-kvcache  = { workspace = true }
contextra-store    = { workspace = true }

# Ring 3 — same ring, erlaubt
contextra-engine   = { workspace = true }
contextra-cognition = { workspace = true }
contextra-router   = { workspace = true }
contextra-agent    = { workspace = true }
contextra-privacy  = { workspace = true }

# Async
tokio              = { workspace = true, features = ["sync", "time"] }

[features]
default = ["ace", "knowledge-weaver", "grounding-oracle"]
ace               = []  # AutoPilot Context Engine
knowledge-weaver  = []  # Autonome Wissenspflege
conflict-guard    = []  # Widerspruchserkennung
predictive-prefetch = [] # Antizipatorisches Loading
agent-pool        = []  # Multi-Agenten-Sharing
composite-router  = []  # Query-Dekomposition
adaptive-ingestion = [] # Semantisches Chunking
temporal-memory   = []  # Decay + Forgetting
cognitive-budget  = []  # Ressourcensteuerung
grounding-oracle  = []  # Halluzinationsprävention
self-healing      = []  # Autonome Fehlerkorrektur
frictionless-personalization = [] # Zero-Effort Adaption
```

### E.0.4 Zentrales Work-Budget-System

```rust
// crates/contextra-autopilot/src/budget.rs

/// Universelles Arbeitsbudget für alle CIAI-Subsysteme.
/// Jeder Subsystem-Aufruf konsumiert aus diesem Budget.
/// Bei Erschöpfung: Teilresultat + BudgetExhausted-Signal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorkBudget {
    /// Maximale Wandzeit für diesen Aufruf in Millisekunden.
    /// Absolut: subsystem darf NICHT über diesen Wert hinausgehen.
    pub wall_time_ms: u32,
    /// Maximale Token-Anzahl die verbraucht werden dürfen
    /// (für LLM-Aufrufe innerhalb dieses Subsystems).
    pub llm_tokens: Option<u32>,
    /// Maximale Anzahl Storage-Operationen.
    pub storage_ops: Option<u32>,
    /// Maximale Anzahl Embedding-Anfragen.
    pub embedding_calls: Option<u32>,
    /// Maximaler RAM-Verbrauch in Bytes für temporäre Strukturen.
    pub heap_bytes: Option<usize>,
}

impl WorkBudget {
    /// Tight: für latenz-kritische Pfade (<5ms)
    pub const TIGHT: Self = Self {
        wall_time_ms: 5,
        llm_tokens: None,
        storage_ops: Some(10),
        embedding_calls: Some(1),
        heap_bytes: Some(1 << 20), // 1 MB
    };

    /// Standard: für normale Anfragen (<100ms)
    pub const STANDARD: Self = Self {
        wall_time_ms: 100,
        llm_tokens: Some(500),
        storage_ops: Some(50),
        embedding_calls: Some(3),
        heap_bytes: Some(16 << 20), // 16 MB
    };

    /// Generous: für Background-Tasks (<2000ms)
    pub const BACKGROUND: Self = Self {
        wall_time_ms: 2_000,
        llm_tokens: Some(2_000),
        storage_ops: Some(500),
        embedding_calls: Some(10),
        heap_bytes: Some(64 << 20), // 64 MB
    };

    /// Exhaustive: für einmalige Konsolidierungs-Jobs (keine Zeitgrenze außer Tenant-Quote)
    pub const EXHAUSTIVE: Self = Self {
        wall_time_ms: 30_000,
        llm_tokens: Some(20_000),
        storage_ops: None,
        embedding_calls: Some(100),
        heap_bytes: None,
    };
}

/// Verfolgt den Verbrauch während eines Subsystem-Aufrufs.
/// Übergabe als &mut BudgetTracker an alle internen Funktionen.
#[derive(Debug, Default)]
pub struct BudgetTracker {
    pub elapsed_start: Option<std::time::Instant>,
    pub storage_ops_used: u32,
    pub embedding_calls_used: u32,
    pub llm_tokens_used: u32,
    pub heap_peak_bytes: usize,
}

impl BudgetTracker {
    pub fn check_time(&self, budget: &WorkBudget) -> bool {
        self.elapsed_start
            .map(|s| s.elapsed().as_millis() < budget.wall_time_ms as u128)
            .unwrap_or(true)
    }
    pub fn tick_storage(&mut self, budget: &WorkBudget) -> bool {
        self.storage_ops_used += 1;
        budget.storage_ops.map_or(true, |max| self.storage_ops_used <= max)
    }
    pub fn tick_embedding(&mut self, budget: &WorkBudget) -> bool {
        self.embedding_calls_used += 1;
        budget.embedding_calls.map_or(true, |max| self.embedding_calls_used <= max)
    }
}

/// Signal für graceful degradation: welche Subsysteme konnten nicht
/// vollständig abgeschlossen werden.
#[derive(Debug, Default, Clone)]
pub struct BudgetExhaustedSignal {
    pub time_exhausted: bool,
    pub llm_budget_exhausted: bool,
    pub storage_budget_exhausted: bool,
    pub embedding_budget_exhausted: bool,
    /// Qualitätsschätzung des Teilergebnisses [0.0, 1.0].
    /// 1.0 = vollständig; < 0.5 = möglicherweise unzureichend.
    pub estimated_completeness: f32,
}

/// Jedes CIAI-Subsystem liefert diesen generischen Ausgaberahmen.
pub struct CiaiResult<T> {
    pub value: T,
    pub budget_signal: BudgetExhaustedSignal,
    /// Jede autonome Entscheidung wird hier für contextra_explain registriert.
    pub audit_tokens: Vec<AuditToken>,
}

/// Unveränderliches Audit-Token für eine einzelne autonome Entscheidung.
#[derive(Debug, Clone)]
pub struct AuditToken {
    pub subsystem: &'static str,
    pub decision: &'static str,
    pub reason: String,
    pub confidence: f32,
    pub tx_id: contextra_types::TxId,
}

```
## E.1 AutoPilot Context Engine (ACE) — "One Call, Perfect Context"

### E.1.1 IST / Problem

Ein Agent muss heute manuell `contextra_search`, `contextra_get`, `contextra_relate` separat aufrufen, die Ergebnisse manuell fusionieren und dann entscheiden, was ins Kontextfenster kommt. Das führt zu:
- Ineffizienter Nutzung des Kontextfensters (irrelevante Inhalte verdrängen relevante)
- Verpassten Graph-Verbindungen (Vektor-Suche findet Dok A, aber A ist mit dem eigentlich relevanten B über 2 Kanten verbunden)
- Teurem Überschreiten des Token-Budgets durch nicht-adaptive Chunk-Selektion

**IST-Zustand (verifiziert):** `contextra-engine` hat `Collection::search()` mit `RetrievalStrategy::{Vector, Text, Graph, Hybrid}` und `contextra-cognition` hat `ContextCompactor` + `CompactionSession`. Es fehlt eine einzige Eintrittsfunktion, die beide koordiniert.

### E.1.2 SOLL — Schnittstelle

```rust
// crates/contextra-autopilot/src/ace.rs

use contextra_types::{ContextChunk, DocId, TenantId, Result};
use contextra_ports::{EmbeddingProvider, StorageEngine, VectorIndex};
use crate::budget::{CiaiResult, WorkBudget};

/// Einziger Einstiegspunkt für Kontext-Vorbereitung.
/// Ersetzt das manuelle Orchestrieren von search + relate + compaction.
pub struct AutoContextEngine<S, V, E> {
    collection: std::sync::Arc<contextra_engine::collection::Collection<S, V>>,
    embedder: std::sync::Arc<E>,
    config: AceConfig,
}

/// Vollständige, kommentierte Konfiguration mit sinnvollen Defaults.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AceConfig {
    /// Token-Budget für den zurückgegebenen Kontext.
    /// Default: 4096 (passt in die meisten lokalen Modelle)
    pub token_budget: usize,

    /// Aktiviert Graph-Expansion: nach Vektor-/Text-Suche werden
    /// 1-Hop-Nachbarn im Wissensgraphen miteinbezogen.
    /// Default: true (CIAI-DP-1: bequem by default)
    pub graph_expansion_enabled: bool,

    /// Maximale Hop-Tiefe für Graph-Expansion (P24-Schutz).
    /// Default: 2 (mehr als 3 Hops bringen selten Mehrwert)
    pub max_graph_hops: u8,

    /// Ob LeanRAG-Aggregation für Themenfragen genutzt wird.
    /// Wird automatisch aktiviert, wenn die Anfrage einer
    /// Themenfrage ähnelt (erkannt via QueryClassifier, E.6).
    /// Default: true
    pub leanrag_for_global_queries: bool,

    /// Strategie für Kontext-Kompression bei Token-Budget-Überschreitung.
    /// Default: StatusToken (kein LLM-Aufruf nötig, deterministisch)
    pub compaction_strategy: contextra_cognition::context_compaction::types::CompactionStrategy,

    /// Ob Attention-gewichtete Relevanz-Scores genutzt werden
    /// (erfordert AttentionExporter, B.2.1; bei Abwesenheit: LRU-Fallback).
    pub attention_weighted_ranking: bool,

    /// Minimaler Konfidenz-Score für einen Chunk, um in den Kontext zu kommen.
    /// Chunks unter diesem Schwellenwert werden stillschweigend ausgelassen.
    /// Default: 0.15 (aggressive Filterung; erhöhen für präzise Domänen)
    pub min_relevance_score: f32,

    /// Ob Cache-Direktiven automatisch gesetzt werden (B.3.5):
    /// System-Prompt → Pin, transiente Reasoning-Steps → ReleaseAfterStep.
    pub auto_cache_directives: bool,
}

impl Default for AceConfig {
    fn default() -> Self {
        Self {
            token_budget: 4096,
            graph_expansion_enabled: true,
            max_graph_hops: 2,
            leanrag_for_global_queries: true,
            compaction_strategy: contextra_cognition::context_compaction::types::CompactionStrategy::StatusToken,
            attention_weighted_ranking: true,
            min_relevance_score: 0.15,
            auto_cache_directives: true,
        }
    }
}

/// Voll qualifizierter, zurückgegebener Kontext inkl. Metadaten
/// über jede Entscheidung (CIAI-DP-3: vollständige Erklärbarkeit).
#[derive(Debug, Clone)]
pub struct PreparedContext {
    /// Die eigentlichen Kontext-Chunks, in absteigender Relevanz.
    pub chunks: Vec<ScoredContextChunk>,
    /// Token-Verbrauch des zurückgegebenen Kontexts.
    pub tokens_used: usize,
    /// Token-Verbrauch VOR Kompression (für Observability).
    pub tokens_before_compaction: usize,
    /// Wie viele Chunks wurden durch kompaktierte Status-Token ersetzt.
    pub compacted_count: usize,
    /// Welche Retrieval-Strategie letztlich genutzt wurde
    /// (kann von config.default abweichen durch CompositeQueryRouter).
    pub retrieval_strategy_used: contextra_types::RetrievalStrategy,
    /// Graph-Expansion-Statistiken (0 wenn deaktiviert).
    pub graph_hops_expanded: u32,
    /// Vorhersage, welche Chunks der Agent als nächstes brauchen wird
    /// (für PredictivePrefetcher, E.4).
    pub predicted_next_chunks: Vec<DocId>,
}

#[derive(Debug, Clone)]
pub struct ScoredContextChunk {
    pub chunk: ContextChunk,
    pub relevance_score: f32,
    pub source: ContextSource,
    /// Audit-Token für contextra_explain: Warum ist dieser Chunk hier?
    pub inclusion_reason: AuditToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextSource {
    VectorSearch,
    TextSearch,
    GraphExpansion { hop_distance: u8 },
    LeanRagAggregation,
    AttentionWeighted,
    CacheHit,
}

impl<S, V, E> AutoContextEngine<S, V, E>
where
    S: StorageEngine + 'static,
    V: VectorIndex + 'static,
    E: EmbeddingProvider + 'static,
{
    /// Erstellt eine neue ACE-Instanz mit sinnvollen Defaults.
    /// Der Aufrufer muss KEINE Retrieval-Strategie wählen —
    /// ACE erkennt die beste Strategie automatisch (via E.6).
    pub fn new(
        collection: std::sync::Arc<contextra_engine::collection::Collection<S, V>>,
        embedder: std::sync::Arc<E>,
    ) -> Self {
        Self { collection, embedder, config: AceConfig::default() }
    }

    pub fn with_config(mut self, config: AceConfig) -> Self {
        self.config = config; self
    }

    /// **Haupt-API:** Bereitet den optimalen Kontext für eine Agent-Anfrage vor.
    ///
    /// # Algorithmus (Relevance Cascade, 4 Stufen)
    ///
    /// Stufe 0 — Query-Klassifikation (< 1ms, kein LLM):
    ///   QueryClassifier (E.6) bestimmt: Faktenfrage | Themenfrage | Prozedurfrage | Vergleichsfrage
    ///
    /// Stufe 1 — Schnelle Vorbefüllung (< 5ms):
    ///   KV-Cache-Lookup (B.3.6 ContentAddressedKvStore): Wenn Hit → direkt zurück (Cache-Treffer)
    ///   Falls Miss: PredictivePrefetcher (E.4) signalisiert asynchrones Pre-Warming.
    ///
    /// Stufe 2 — Adaptives Retrieval (< 50ms, via CompositeQueryRouter E.6):
    ///   Bei Faktenfrage: Hybrid (Vektor + Text) → 20 Kandidaten
    ///   Bei Themenfrage: Global (LeanRAG) → Community-Cluster
    ///   Bei Prozedurfrage: Graph-PPR von bekannten Entitäten
    ///   PID-Regler (bestehend) skaliert k dynamisch nach Latenz-Budget
    ///
    /// Stufe 3 — Graph-Expansion (< 30ms, optional, P24-lokal):
    ///   1-2 Hops im Wissensgraphen ausgehend von Top-5-Retrieval-Ergebnissen
    ///   Attention-Scores (B.2.1) priorisieren Expansions-Nachbarn
    ///   Budget-Check: stop wenn WorkBudget.wall_time nahezu erschöpft
    ///
    /// Stufe 4 — Kompression & Ranking (< 10ms):
    ///   Konforme Kalibrierung (B.5.x ConformalCalibrator) wenn verfügbar
    ///   Token-Budget-Enforcement über ContextCompactor (bestehend)
    ///   CompactionStrategy (StatusToken by default) für Overflow
    ///
    /// # Guarantien
    /// - INV-ACE-1: tokens_used ≤ config.token_budget IMMER (nicht nur im Normalfall)
    /// - INV-ACE-2: Keine Daten aus fremdem Tenant in chunks (INV-1)
    /// - INV-ACE-3: Rückgabe in ≤ budget.wall_time_ms (bei Überschreitung: Teilresultat)
    pub async fn prepare_context(
        &self,
        tenant: TenantId,
        query: &str,
        agent_state: Option<&AgentStateHint>,
        budget: WorkBudget,
    ) -> Result<CiaiResult<PreparedContext>>;

    /// Leichtgewichtige Variante: nur KV-Cache + Top-3-Vektor-Suche.
    /// Für latenz-kritische Pfade (< 5ms), ohne Graph-Expansion.
    pub async fn prepare_context_fast(
        &self,
        tenant: TenantId,
        query: &str,
        budget: WorkBudget,
    ) -> Result<CiaiResult<PreparedContext>> {
        let tight_budget = WorkBudget { wall_time_ms: 5, ..budget };
        let mut minimal_config = self.config.clone();
        minimal_config.graph_expansion_enabled = false;
        minimal_config.leanrag_for_global_queries = false;
        minimal_config.token_budget = minimal_config.token_budget / 2;
        // Kurzschluss über prepare_context mit reduzierter Config
        self.prepare_context(tenant, query, None, tight_budget).await
    }
}

/// Optionaler Hinweis über den Agent-Zustand für bessere Kontextualisierung.
/// ALLES ist optional — ACE funktioniert auch ohne jeden Hinweis.
#[derive(Debug, Default)]
pub struct AgentStateHint {
    /// Entitäten, die der Agent gerade bearbeitet (für Graph-PPR-Seeds).
    pub active_entities: Vec<contextra_types::EntityId>,
    /// Tool, das der Agent als nächstes aufzurufen plant.
    pub next_tool_hint: Option<String>,
    /// Bereits im Kontext befindliche Doc-IDs (werden nicht doppelt geladen).
    pub already_in_context: Vec<DocId>,
    /// Ob der aktuelle Step ein "Reasoning Step" ist (transient, nie cachen).
    pub is_reasoning_step: bool,
}
```

### E.1.3 Algorithmus: Relevance Cascade im Detail

```
Algorithmus: AutoContextEngine::prepare_context
Zeit: O(k log k + d·hops) mit k = Retrieval-Kandidaten, d = Avg-Nachbarschaftsgrad
Speicher: O(k + token_budget / CHARS_PER_TOKEN)

Phase 0 — QueryClassification (synchron, O(1) amortisiert):
  class ← QueryClassifier.classify(query)
  // Regelbasiert (keine LLM-Kosten): 
  // Global-Trigger: "was sind die hauptthemen", "überblick über", "alle..."
  // Prozedur-Trigger: "wie mache ich", "schritte für", "anleitung..."
  // Fakt-Trigger: alles andere

Phase 1 — Cache-Lookup (≤ 1ms):
  cache_result ← ContentAddressedKvStore.lookup(tenant, hash(query))
  if cache_result is ExactPrefixHit or ContentHashHit:
      return PreparedContext.from_cache(cache_result) // Sofortiger Return
  PredictivePrefetcher.hint_async(tenant, query)  // Fire-and-forget

Phase 2 — Adaptives Retrieval:
  strategy ← match class:
    Global    → RetrievalStrategy::Global { max_community_nodes: budget_scaled(budget) }
    Procedure → RetrievalStrategy::Graph  (PPR von active_entities)
    Fact      → RetrievalStrategy::Hybrid (k = PID-Controller.scale(k_base, latency))
  
  raw_candidates ← Collection.search(tenant, query, strategy, k=20)
  // PID-Regler skaliert k basierend auf gemessener Latenz (bestehend)

Phase 3 — Graph-Expansion (optional, budget-gated):
  if config.graph_expansion_enabled AND budget.check_time():
      seeds ← raw_candidates.top(5).entity_ids()
      ppr_scores ← forward_push_ppr(seeds, alpha=0.15, epsilon=1e-4)
      // Schlägt INV-DELETION-2 ein: gelöschte DocIds werden in ppr_scores maskiert
      expansion_docs ← ppr_scores.top(10).filter(score > config.min_relevance_score)
      raw_candidates.extend(expansion_docs tagged ContextSource::GraphExpansion)
  
  // Attention-gewichtetes Re-Ranking (B.2.1):
  if config.attention_weighted_ranking AND AttentionExporter.available():
      attention_weights ← AttentionExporter.export(current_request_id)
      raw_candidates = attention_rerank(raw_candidates, attention_weights)

Phase 4 — Token-Kompression & Finale Auswahl:
  scored ← ConformalCalibrator.calibrate_batch(raw_candidates) OR rrf_score(raw_candidates)
  scored.retain(|c| c.score >= config.min_relevance_score)
  scored.sort_by(|a,b| b.score.total_cmp(&a.score))
  
  compacted ← ContextCompactor.fit_to_budget(
      scored,
      token_budget = config.token_budget,
      strategy = config.compaction_strategy,
  )
  
  predicted_next ← PredictivePrefetcher.predict(tenant, compacted.retained_chunks)
  
  return PreparedContext {
      chunks: compacted.retained_chunks,
      tokens_used: compacted.tokens_used,
      predicted_next_chunks: predicted_next,
      ...
  }
```

### E.1.4 Invarianten

- **INV-ACE-1:** `PreparedContext.tokens_used ≤ AceConfig.token_budget` — hart erzwungen im `ContextCompactor`-Schritt, kein "fast within budget".
- **INV-ACE-2:** Tenant-Isolation (INV-1 aus v6) gilt auch für Graph-Expansion: `forward_push_ppr` operiert nur im Subgraph des Tenants (bestehende `TenantScoped<T>`-Hülle, `contextra-privacy`).
- **INV-ACE-3:** Budget-Exhaustion führt zu `CiaiResult` mit `BudgetExhaustedSignal.estimated_completeness < 1.0`, NICHT zu einem `Err`. Der Aufrufer entscheidet, ob ein Teilresultat akzeptabel ist.
- **INV-ACE-4:** Jeder Chunk in `PreparedContext.chunks` hat ein `inclusion_reason: AuditToken` — keine "magischen" Aufnahmen ohne Begründung. Dient als Backend für `contextra_explain`.

### E.1.5 Test-Pflichten

```
crates/contextra-autopilot/tests/ace_token_budget_never_exceeded.rs
  Property-Test (proptest): Für jede AceConfig.token_budget ∈ [512, 32768] und
  beliebige Abfragen: tokens_used ≤ token_budget IMMER.

crates/contextra-autopilot/tests/ace_tenant_isolation.rs
  Sicherheitstest: Zwei Tenants mit überschneidenden Embeddings.
  Tenant B darf NIEMALS Chunks aus Tenant A in PreparedContext erhalten.

crates/contextra-autopilot/tests/ace_budget_exhaustion_partial_result.rs
  Mit WorkBudget { wall_time_ms: 1, .. }: PreparedContext zurück (nie Err),
  BudgetExhaustedSignal.time_exhausted == true.

crates/contextra-autopilot/tests/ace_cache_hit_fast_path.rs
  KV-Cache Warm: prepare_context_fast < 2ms gemessen.

crates/contextra-autopilot/tests/ace_audit_token_completeness.rs
  Jeder ScoredContextChunk.inclusion_reason.confidence ∈ [0.0, 1.0], niemals NaN.
```

---

## E.2 KnowledgeWeaver — Kontinuierliche Autonome Wissenspflege

### E.2.1 IST / Problem

`MaintenanceScheduler` (🟢 vorhanden) führt sequenziell: Decay → Compaction → EdgeReinforcement → Perkolation. Diese sind aber **reaktiv** (60s-Takt) und **unkoordiniert** mit dem Retrieval-Pfad. Es fehlt:
1. Priorisierung basierend auf welche Wissensteile *gerade* häufig abgefragt werden
2. Koordination mit dem Bandit-Router (wenn Strategie X schlechte Reward-Signale liefert → Konsolidierung von X's Ergebnissen priorisieren)
3. Inkrementelle Qualitätsmessung (weiß der Scheduler, ob die Wissensbasis besser oder schlechter wird?)

### E.2.2 SOLL — Schnittstelle

```rust
// crates/contextra-autopilot/src/knowledge_weaver.rs

/// KnowledgeWeaver ist die Koordinationsschicht über dem bestehenden
/// MaintenanceScheduler. Er entscheidet WELCHE Komponenten im nächsten
/// Tick mit welcher Priorität ausgeführt werden — basierend auf
/// Echtzeitmetriken aus dem Bandit-Router, dem Decay-Controller und
/// dem Graph-Perkolations-Checker.
pub struct KnowledgeWeaver<S: StorageEngine, V: VectorIndex> {
    scheduler: MaintenanceScheduler<S, V>,
    metrics_sink: std::sync::Arc<dyn MetricsSink>,
    drift_provider: std::sync::Arc<dyn DriftStatusProvider>,
    quality_tracker: WeaverQualityTracker,
}

/// Priorität der einzelnen Maintenance-Aufgaben — dynamisch berechnet.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum WeaverPriority {
    /// Sofort — blockiert aktive Retrieval-Qualität (z.B. Ghost-Pointer nach Löschung)
    Critical,
    /// Nächster Tick — beeinträchtigt Qualität spürbar
    High,
    /// Regulär — normaler Wartungsbedarf
    Normal,
    /// Niedrig — Nice-to-have, nur bei freiem Budget
    Low,
    /// Ausgesetzt — ausdrücklich bis nächsten Trigger verschoben
    Deferred,
}

/// Aufgaben-Plan für einen einzelnen Maintenance-Tick.
#[derive(Debug, Clone)]
pub struct WeaverPlan {
    /// Reihenfolge und Budget pro Aufgabe.
    pub tasks: Vec<(WeaverTask, WorkBudget)>,
    /// Begründung jeder Priorisierungsentscheidung (Audit-Trail).
    pub reasoning: Vec<String>,
    /// Geschätzte Qualitätsverbesserung wenn Plan vollständig ausgeführt.
    pub estimated_quality_delta: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaverTask {
    /// Adaptive Decay: veraltete, wenig abgefragte Dokumente evicten.
    AdaptiveDecay,
    /// Memory Consolidation: Kurzzeitgedächtnis → Langzeitgedächtnis komprimieren.
    MemoryConsolidation,
    /// Edge Reinforcement: häufig traversierte Kanten stärken.
    EdgeReinforcement,
    /// Graph Perkolation: isolierte Cluster reverbinden.
    GraphPercolation,
    /// HNSW Tombstone Cleanup: Graph-Reparatur nach Löschungen (B.1.1).
    HnswTombstoneCleanup,
    /// Spectral Hyperedge Dedup: redundante Hyperkanten identifizieren.
    SpectralHyperedgeDedup,
    /// Contradiction Resolution: SemanticConflictGuard-Vorschläge abarbeiten.
    ContradictionResolution,
    /// Temporal Decay: zeitbasierte Wichtigkeits-Anpassung (E.8).
    TemporalDecay,
    /// Knowledge Freshness Check: veraltete Fakten markieren.
    FreshnessCheck,
}

impl<S, V> KnowledgeWeaver<S, V>
where S: StorageEngine + 'static, V: VectorIndex + 'static
{
    /// Berechnet den optimalen Plan für den nächsten Tick.
    ///
    /// Entscheidungslogik (Prioritätsmatrix):
    ///
    /// Signal 1 — LyapunovDriftWatcher.is_drifting() = true:
    ///   → EdgeReinforcement: High, MemoryConsolidation: High
    ///   Begründung: Drift bedeutet Reward-Verteilung hat sich verschoben;
    ///   frische Konsolidierung verbessert Retrieval-Qualität für neue Domäne.
    ///
    /// Signal 2 — HnswIndex.tombstone_ratio() > 0.1:
    ///   → HnswTombstoneCleanup: Critical
    ///   Begründung: INV-DELETION-2-Verletzungsrisiko steigt mit Tombstone-Rate.
    ///
    /// Signal 3 — PerkolationChecker.is_fragmented() = true:
    ///   → GraphPercolation: High
    ///   Begründung: Fragmentierter Graph = Retrieval übersieht zusammenhängende Cluster.
    ///
    /// Signal 4 — BanditProfileState.arm_with_worst_reward() hat > 50 Observations:
    ///   → MemoryConsolidation für Collections die diesem Arm zugeordnet sind: High
    ///   Begründung: Schlechter Reward könnte an fragmentiertem Langzeitgedächtnis liegen.
    ///
    /// Signal 5 — ConflictGuard.pending_conflicts() > 0:
    ///   → ContradictionResolution: Normal
    ///
    /// Default (keine kritischen Signale):
    ///   → AdaptiveDecay: Normal, TemporalDecay: Low, FreshnessCheck: Low
    pub fn plan_next_tick(
        &mut self,
        tenant: TenantId,
        budget: WorkBudget,
    ) -> WeaverPlan;

    /// Führt den Plan aus und liefert Qualitäts-Delta-Messung.
    pub async fn execute_plan(
        &mut self,
        plan: &WeaverPlan,
        tenant: TenantId,
    ) -> Result<WeaverExecutionReport>;
}

/// Ergebnis einer Plan-Ausführung mit Qualitätsmessung.
#[derive(Debug, Clone)]
pub struct WeaverExecutionReport {
    pub tasks_completed: Vec<WeaverTask>,
    pub tasks_skipped: Vec<(WeaverTask, String)>, // Aufgabe + Grund
    /// Qualitätsmetriken VOR diesem Tick.
    pub quality_before: KnowledgeQualitySnapshot,
    /// Qualitätsmetriken NACH diesem Tick.
    pub quality_after: KnowledgeQualitySnapshot,
}

/// Schnappschuss der Wissensqualität — objektive, codeveifizierbare Metriken.
#[derive(Debug, Clone)]
pub struct KnowledgeQualitySnapshot {
    /// Anteil der aktiven (nicht-tombstoned) Knoten ohne Ghost-Pointer [0,1].
    pub graph_integrity_ratio: f32,
    /// Anteil der Hyperkanten ohne semantische Duplikate [0,1].
    pub hyperedge_uniqueness_ratio: f32,
    /// Anteil des Wissensgraphen der über PPR von beliebigem Seed erreichbar ist.
    pub graph_reachability_ratio: f32,
    /// Mittleres Alter (in TxId-Einheiten) der meistabgefragten Dokumente.
    pub freshness_score: f32,
    /// Anzahl ungelöster Widersprüche.
    pub open_conflicts: u32,
}
```

### E.2.3 Intelligentes Scheduling: Prioritätsmatrix

```
╔══════════════════════════════════════════════════════════════════════════╗
║ KNOWLEDGEWEAVER PRIORITÄTSMATRIX (deterministisch, P28-konform)         ║
╠══════════════════════════════╦════════════════╦══════════════════════════╣
║ Signal                       ║ Aufgabe        ║ Priorität                ║
╠══════════════════════════════╬════════════════╬══════════════════════════╣
║ tombstone_ratio > 0.10       ║ TombstoneClean ║ CRITICAL                 ║
║ open_conflicts > 20          ║ ContradictRes  ║ CRITICAL                 ║
║ drift detected (beide        ║ EdgeReinforce  ║ HIGH                     ║
║  LyapunovDrift AND Catoni)   ║ MemConsolid    ║ HIGH                     ║
║ reachability < 0.7           ║ GraphPercolat  ║ HIGH                     ║
║ worst_arm_reward < 0.2       ║ MemConsolid    ║ HIGH                     ║
║ hyperedge_unique < 0.8       ║ SpectralDedup  ║ NORMAL                   ║
║ freshness_score > 50_000 tx  ║ FreshnessCheck ║ NORMAL                   ║
║ kein kritisches Signal       ║ AdaptDecay     ║ NORMAL                   ║
║ idle (keine Queries)         ║ TemporalDecay  ║ LOW                      ║
╚══════════════════════════════╩════════════════╩══════════════════════════╝
```

---

## E.3 SemanticConflictGuard — Widerspruchserkennung & -auflösung

### E.3.1 IST / Problem

Contextra hat keinerlei Mechanismus, um zu erkennen wenn `insert("Paris liegt in Deutschland")` einer bestehenden `insert("Paris ist die Hauptstadt Frankreichs")` widerspricht. Über Zeit akkumulieren sich solche Widersprüche → Retrieval liefert inkonsistente Antworten → Agent-Halluzinationen steigen.

### E.3.2 SOLL — Schnittstelle

```rust
// crates/contextra-autopilot/src/conflict_guard.rs

/// Erkennt semantische Widersprüche zwischen neuen und bestehenden Dokumenten.
/// Wird im Insert-Pfad nach Open-IE-Extraktion (B.1.3) und im KnowledgeWeaver
/// (E.2) als Background-Task eingebunden.
pub struct SemanticConflictGuard<E: EmbeddingProvider> {
    embedder: std::sync::Arc<E>,
    config: ConflictGuardConfig,
    pending_conflicts: std::sync::Arc<tokio::sync::RwLock<Vec<DetectedConflict>>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConflictGuardConfig {
    /// Embedding-Ähnlichkeit ab der zwei Aussagen als "thematisch verwandt"
    /// gelten und auf Widerspruch geprüft werden.
    /// Default: 0.85 (hoch, um False Positives zu minimieren)
    pub topic_similarity_threshold: f32,

    /// Ähnlichkeit ab der zwei thematisch verwandten Aussagen als "widersprüchlich"
    /// eingestuft werden (via Negations-Embedding-Distanz, siehe Algorithmus).
    /// Default: 0.60
    pub contradiction_threshold: f32,

    /// Maximale Anzahl zu prüfender bestehender Dokumente pro neuem Insert.
    /// P24-Schutz: kein vollständiger Datenbank-Scan.
    /// Default: 50 (Top-50 semantisch ähnlichste Kandidaten)
    pub max_candidates_per_insert: usize,

    /// Auflösungsstrategie bei erkanntem Widerspruch.
    pub resolution_strategy: ConflictResolutionStrategy,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ConflictResolutionStrategy {
    /// Neueres Dokument überschreibt älteres (Default für dynamische Fakten).
    PreferNewer,
    /// Höhere Konfidenz (via ConformalCalibrator) gewinnt.
    PreferHigherConfidence,
    /// Beide behalten, aber als "in Konflikt" markiert.
    /// Retrieval-Phase markiert solche Chunks als "contested".
    MarkAsContested,
    /// Manuelles Review: Widerspruch in pending_conflicts einreihen,
    /// kein automatisches Handeln.
    RequireHumanReview,
}

/// Ein erkannter Widerspruch zwischen zwei Dokumenten.
#[derive(Debug, Clone)]
pub struct DetectedConflict {
    pub doc_id_a: DocId,
    pub doc_id_b: DocId,
    pub tenant: TenantId,
    pub snippet_a: String,
    pub snippet_b: String,
    pub conflict_score: f32,  // [0,1], je höher desto klarer der Widerspruch
    pub detected_at_tx: contextra_types::TxId,
    pub resolution_applied: Option<ConflictResolutionStrategy>,
}

impl<E: EmbeddingProvider> SemanticConflictGuard<E> {
    /// Prüft ein neues Dokument gegen bestehende.
    /// Wird im Insert-Pfad aufgerufen (nach Open-IE-Extraktion).
    ///
    /// Komplexität: O(k · d) mit k = max_candidates_per_insert, d = Embedding-Dim.
    /// P24-konform: kein vollständiger Datenbank-Scan.
    pub async fn check_on_insert<S: StorageEngine, V: VectorIndex>(
        &self,
        collection: &contextra_engine::collection::Collection<S, V>,
        new_doc_id: DocId,
        new_text: &str,
        tenant: TenantId,
        budget: WorkBudget,
    ) -> Result<Vec<DetectedConflict>>;

    /// Batch-Prüfung für KnowledgeWeaver (Background-Task).
    /// Prüft einen zufälligen (seed-deterministischen) Querschnitt der Wissensbasis.
    pub async fn check_batch<S: StorageEngine, V: VectorIndex>(
        &self,
        collection: &contextra_engine::collection::Collection<S, V>,
        tenant: TenantId,
        sample_size: usize,
        budget: WorkBudget,
    ) -> Result<Vec<DetectedConflict>>;
}
```

### E.3.3 Algorithmus: Negations-Embedding-Distanz

```
Widerspruchs-Erkennung via Negations-Embedding-Distanz:

Problem: Klassische Embedding-Ähnlichkeit kann "Paris liegt in Frankreich"
         und "Paris liegt in Deutschland" als ÄHNLICH werten (beide über Paris),
         obwohl sie sich widersprechen.

Lösung: Asymmetrische Negations-Distanz.

1. Embedding(neu): e_new ← embed(new_text)
2. Top-k Kandidaten: candidates ← HNSW.search(e_new, k=config.max_candidates)
3. Für jeden Kandidaten c:
   a. topic_sim ← cosine(e_new, embed(c.text))
   b. if topic_sim < config.topic_similarity_threshold: skip
   c. Negations-Probe: neg_e ← embed("Das Gegenteil ist wahr: " + c.text)
      contradiction_score ← cosine(e_new, neg_e)
      // Wenn e_new dem Negat von c ähnlicher ist als c selbst:
      // direkter Widerspruch.
   d. if contradiction_score > config.contradiction_threshold:
          DetectedConflict(new_doc, c, conflict_score=contradiction_score)

Komplexität: O(k · embed_dim) für die Negations-Proben
Embedding-Calls: 1 für new_doc + k für Negations-Proben
  → Budget-Check: max k = min(config.max_candidates, budget.embedding_calls - 1)
```

---

## E.4 PredictivePrefetcher — Antizipatorisches KV-Cache-Loading

### E.4.1 IST / Problem

KV-Cache (🟢 vorhanden) ist reaktiv: Cache wird gefüllt wenn ein Request kommt. Graph-Struktur und Agent-Verhaltensmuster erlauben aber die Vorhersage, was der nächste Request wahrscheinlich braucht.

### E.4.2 SOLL — Schnittstelle

```rust
// crates/contextra-autopilot/src/prefetcher.rs

/// Antizipatorisches KV-Cache Pre-Warming basierend auf:
/// 1. Graph-Nachbarschaft der zuletzt abgerufenen Knoten
/// 2. Temporalen Sequenzmustern in der Agent-Session
/// 3. Community-Zugehörigkeit (Leiden-Cluster, bestehend)
pub struct PredictivePrefetcher {
    config: PrefetcherConfig,
    /// Ringpuffer: letzte N (request, retrieved_docs) Paare pro Tenant.
    session_history: ahash::AHashMap<TenantId, VecDeque<SessionEntry>>,
}

#[derive(Debug, Clone)]
pub struct PrefetcherConfig {
    /// Wie viele kürzliche Requests für Mustervorhersage genutzt werden.
    /// Default: 10
    pub history_window: usize,
    /// Maximale Anzahl proaktiv pre-geladener Dokumente.
    /// Default: 5 (nicht zu aggressiv — Cache-Verdrängung ist teuer)
    pub max_prefetch_docs: usize,
    /// Minimale Vorhersage-Konfidenz für ein Pre-Fetch.
    /// Default: 0.6
    pub min_prediction_confidence: f32,
}

impl PredictivePrefetcher {
    /// Antizipiert die nächsten benötigten Chunks aus:
    /// 1. 1-Hop Graph-Nachbarn der aktuell abgerufenen Docs
    ///    (wahrscheinlich: Agent fragt als nächstes über verbundene Entitäten)
    /// 2. Sequenz-Muster: Hat Agent in ähnlichen Sessions danach X gefragt?
    pub fn predict(
        &mut self,
        tenant: TenantId,
        current_chunks: &[ContextChunk],
    ) -> Vec<PrefetchPrediction>;

    /// Asynchrones Signal: "Pre-warm Cache für diese Query, sie kommt bald."
    /// Fire-and-forget aus ACE Stufe 1.
    pub fn hint_async(&self, tenant: TenantId, query: &str);
}

#[derive(Debug, Clone)]
pub struct PrefetchPrediction {
    pub doc_id: DocId,
    pub confidence: f32,
    pub prediction_source: PredictionSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredictionSource {
    /// 1-Hop-Nachbar im Wissensgraphen
    GraphNeighbor { hop_distance: u8 },
    /// Sequenzmuster aus Session-History
    SequencePattern,
    /// Gleiche Leiden-Community wie aktuell abgerufene Dokumente
    CommunityMember,
}
```

---

## E.5 AgentPool & TrustWeave — Multi-Agenten-Wissensteilung

### E.5.1 IST / Problem

Contextra hat vollständige Mandanten-Isolation (INV-1, 🟢). Das ist korrekt für Datenschutz. Aber: In Multi-Agenten-Systemen (z.B. ein Team von Agenten die an derselben Aufgabe arbeiten) lernen alle dasselbe unabhängig — ineffizient. AgentPool löst das mit **explizitem, kontrolliertem** Cross-Tenant-Sharing.

### E.5.2 SOLL — Schnittstelle

```rust
// crates/contextra-autopilot/src/agent_pool.rs

/// Ein Pool verbundener Agenten, die Wissen teilen dürfen.
/// Vollständig opt-in: Default = keine Sharing, INV-1 unverletzt.
pub struct AgentPool {
    pool_id: PoolId,
    /// Vertrauens-Graph zwischen Agenten des Pools.
    /// Kante A→B mit Gewicht w: A vertraut B mit Stärke w ∈ [0,1].
    trust_graph: ahash::AHashMap<AgentId, Vec<(AgentId, f32)>>,
    config: PoolConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentId(pub TenantId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PoolId(pub u64);

#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Minimales Vertrauens-Gewicht für Wissensübertragung.
    /// Default: 0.5 (50% Vertrauen nötig)
    pub min_trust_weight: f32,
    /// Ob Wissen ohne Verifikation übertragen wird (schnell)
    /// oder erst grounding-geprüft (sicher, mehr Latenz).
    pub require_grounding_verification: bool,
    /// Welche Wissenskategorien geteilt werden dürfen.
    pub sharable_categories: Vec<KnowledgeCategory>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnowledgeCategory {
    /// Faktenwissen (Entitäten, Beziehungen) — meistens sicher zu teilen
    FactualKnowledge,
    /// Prozedurale Schritte (Anleitungen) — kontext-abhängig
    ProceduralKnowledge,
    /// Persönliche Präferenzen — NIEMALS automatisch geteilt
    PersonalPreferences,
    /// Domänen-spezifisches Fachwissen
    DomainExpertise { domain: String },
}

impl AgentPool {
    /// Propagiert neues Wissen von Quelle zu vertrauenden Zielen.
    /// Privacy-Preserving: nur Kategorien aus PoolConfig.sharable_categories
    /// werden weitergegeben; PersonalPreferences NIEMALS.
    ///
    /// Algorithmus: Trust-weighted Graph Diffusion
    /// Für jeden Ziel-Agenten B mit trust_weight(A→B) ≥ min_trust_weight:
    ///   if config.require_grounding_verification:
    ///     knowledge = GroundingOracle.verify(knowledge, B.context)
    ///   B.collection.insert(knowledge, source=SharedFrom(A, trust_weight))
    pub async fn propagate_knowledge(
        &self,
        source: AgentId,
        knowledge: SharedKnowledge,
        max_hops: u8,
    ) -> Result<PropagationReport>;

    /// Query-Federation: Wenn Agent A die Anfrage nicht beantworten kann
    /// (score unter Schwellenwert), wird sie an vertrauende Agenten weitergeleitet.
    pub async fn federated_search(
        &self,
        requesting_agent: AgentId,
        query: &str,
        budget: WorkBudget,
    ) -> Result<Vec<FederatedSearchResult>>;
}

#[derive(Debug, Clone)]
pub struct SharedKnowledge {
    pub content: String,
    pub category: KnowledgeCategory,
    pub source_doc_id: DocId,
    pub source_agent: AgentId,
    pub confidence: f32,
}
```

---

## E.6 CompositeQueryRouter — Intelligente Anfrage-Dekomposition

### E.6.1 IST / Problem

Bandit-Router (🟢) wählt zwischen Vector/Text/Graph/Hybrid. Er behandelt jede Anfrage als atomar. Komplexe Anfragen wie "Vergleiche die Datenschutzpolitik von Unternehmen A und B und erkläre warum sie sich unterscheiden" sind aber eigentlich drei Sub-Anfragen:
1. `search("Datenschutzpolitik Unternehmen A")` 
2. `search("Datenschutzpolitik Unternehmen B")`
3. `compare + explain` (kein Retrieval nötig, nur Reasoning über 1+2)

### E.6.2 SOLL — Schnittstelle

```rust
// crates/contextra-autopilot/src/composite_router.rs

/// Zerlegt komplexe Anfragen in Teilanfragen und koordiniert deren Ausführung.
pub struct CompositeQueryRouter {
    bandit_router: std::sync::Arc<contextra_router::RouterEngine>,
    query_classifier: QueryClassifier,
    config: CompositeRouterConfig,
}

#[derive(Debug, Clone)]
pub struct CompositeRouterConfig {
    /// Maximale Anzahl Sub-Anfragen pro Query.
    /// P24-Schutz: zu viele Sub-Anfragen → Budget-Explosion.
    /// Default: 4
    pub max_subqueries: usize,
    /// Ob Sub-Anfragen parallel ausgeführt werden.
    /// Default: true (tokio::join!, aber Budget pro Sub-Anfrage halbiert)
    pub parallel_execution: bool,
}

/// Klassifikator für Anfrage-Typen (regelbasiert, kein LLM).
pub struct QueryClassifier;

impl QueryClassifier {
    /// Klassifiziert eine Anfrage in Typ und erkennt Dekompositions-Muster.
    /// O(1): reine String-Pattern-Analyse, kein embedding.
    pub fn classify(&self, query: &str) -> QueryAnalysis;
}

#[derive(Debug, Clone)]
pub struct QueryAnalysis {
    pub primary_type: QueryType,
    pub sub_queries: Vec<SubQuery>,
    pub entities_mentioned: Vec<String>,
    pub requires_comparison: bool,
    pub is_temporal: bool,  // "letzte Woche", "seit Januar" etc.
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryType {
    /// "Was ist X?" — eine Entität, ein Fakt
    FactualLookup,
    /// "Was sind die Hauptthemen?" — Überblick, LeanRAG
    GlobalOverview,
    /// "Wie mache ich X?" — Schrittfolge, Graph
    Procedural,
    /// "Vergleiche A und B" — zwei Teilanfragen + Synthesis
    Comparison,
    /// "Warum hat X Y getan?" — kausal, Graph-PPR
    Causal,
    /// "Was ist heute passiert?" — temporal, Freshness
    Temporal,
    /// Mehrteilig, nicht klar klassifizierbar → sicherster Fallback: Hybrid
    Complex,
}

#[derive(Debug, Clone)]
pub struct SubQuery {
    pub text: String,
    pub suggested_strategy: contextra_types::RetrievalStrategy,
    pub weight: f32,  // Wie wichtig ist diese Sub-Query für die Gesamtantwort? [0,1]
}

impl CompositeQueryRouter {
    /// Analysiert und führt eine Anfrage durch.
    /// Bei einfachen Anfragen: direkte Weiterleitung an Bandit-Router.
    /// Bei komplexen Anfragen: Dekomposition + parallele Ausführung.
    pub async fn route_and_execute<S: StorageEngine, V: VectorIndex>(
        &self,
        collection: &Collection<S, V>,
        tenant: TenantId,
        query: &str,
        budget: WorkBudget,
    ) -> Result<CiaiResult<CompositeSearchResult>>;
}

#[derive(Debug, Clone)]
pub struct CompositeSearchResult {
    /// Fusionierte Ergebnisse aller Sub-Queries (RRF-gewichtet nach SubQuery.weight)
    pub merged_chunks: Vec<ScoredContextChunk>,
    /// Pro Sub-Query: Einzelergebnisse (für Synthesis-Step sichtbar)
    pub sub_results: Vec<(SubQuery, Vec<ScoredContextChunk>)>,
    /// Ob eine Synthesis-LLM-Phase empfohlen wird
    /// (z.B. bei Comparison: ja; bei FactualLookup: nein)
    pub synthesis_recommended: bool,
}
```

---

## E.7 AdaptiveIngestionPipeline — Ressourcenschonende Wissensakquise

### E.7.1 IST / Problem

`chunker.rs` (🟢) macht Markdown-basierten semantischen Chunking. Es fehlt:
1. Adaptive Chunk-Größe basierend auf Inhaltsdichte (Code = kleine Chunks; Fließtext = größere)
2. Entitäts-Ko-Okkurrenz-Preservation (Entitäten die im gleichen Satz auftreten sollen in denselben Chunk)
3. Deduplizierung beim Insert (identische oder sehr ähnliche Inhalte nicht doppelt einfügen)
4. Ressourcen-priorisiertes Verarbeiten (wichtiges Dokument zuerst, wenn Queue)

### E.7.2 SOLL — Schnittstelle

```rust
// crates/contextra-autopilot/src/ingestion.rs

/// Vollständige, ressourcenschonende Ingestion-Pipeline.
/// Ersetzt den manuellen Aufruf: chunk → embed → insert → relate.
pub struct AdaptiveIngestionPipeline<S, V, E> {
    collection: std::sync::Arc<Collection<S, V>>,
    embedder: std::sync::Arc<E>,
    conflict_guard: SemanticConflictGuard<E>,
    config: IngestionConfig,
    /// Priority queue für ausstehende Ingestions.
    pending_queue: std::sync::Arc<tokio::sync::Mutex<BinaryHeap<PrioritizedDocument>>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IngestionConfig {
    /// Ob adaptive Chunk-Größen basierend auf Inhaltsdichte.
    /// Default: true
    pub adaptive_chunking: bool,

    /// Ziel-Token-Größe pro Chunk (adaptiv: ±50% je nach Dichte).
    /// Default: 256 tokens
    pub target_chunk_tokens: usize,

    /// Ob Entitäts-Ko-Okkurrenz-Splits verhindert werden.
    /// Default: true (Entitäten die zusammen auftreten → gleicher Chunk)
    pub entity_cooccurrence_preservation: bool,

    /// Ob Deduplizierungsprüfung vor Insert.
    /// Default: true (verhindert doppeltes Wissen)
    pub deduplication_check: bool,

    /// Ähnlichkeitsschwelle für Duplikat-Erkennung.
    /// Default: 0.95 (sehr streng: nur nahezu identische Texte)
    pub deduplication_threshold: f32,

    /// Ob Open-IE-Extraktion automatisch läuft (B.1.3).
    /// Default: true (aligned mit AutoExtractionConfig::enabled = true)
    pub auto_extract_triples: bool,

    /// Ob Widerspruchsprüfung nach Insert (E.3).
    /// Default: true
    pub conflict_check_on_insert: bool,

    /// Ob Predictive Prefetcher nach Insert hingewiesen wird (E.4).
    /// Default: true
    pub hint_prefetcher: bool,
}

impl Default for IngestionConfig {
    fn default() -> Self {
        Self {
            adaptive_chunking: true,
            target_chunk_tokens: 256,
            entity_cooccurrence_preservation: true,
            deduplication_check: true,
            deduplication_threshold: 0.95,
            auto_extract_triples: true,
            conflict_check_on_insert: true,
            hint_prefetcher: true,
        }
    }
}

impl<S, V, E> AdaptiveIngestionPipeline<S, V, E>
where S: StorageEngine + 'static, V: VectorIndex + 'static, E: EmbeddingProvider + 'static
{
    /// Verarbeitet ein Dokument vollständig:
    /// chunk → dedup-check → embed → insert → extract-triples → conflict-check
    ///
    /// Komplexität: O(n + k·d) mit n = Dokumentlänge, k = Chunks, d = Embedding-Dim
    ///
    /// Adaptive Chunk-Größe:
    ///   text_density ← entities_per_sentence(text)
    ///   chunk_size ← target_chunk_tokens × (1 + 0.5 × (1 - text_density / MAX_DENSITY))
    ///   // Höhere Dichte = kleinere Chunks (mehr Präzision nötig)
    ///   // Niedrigere Dichte = größere Chunks (weniger Granularität nötig)
    pub async fn ingest(
        &self,
        tenant: TenantId,
        content: &str,
        metadata: DocumentMetadata,
        priority: IngestionPriority,
        budget: WorkBudget,
    ) -> Result<CiaiResult<IngestionReport>>;

    /// Batch-Ingestion mit Priorisierungs-Queue.
    /// Hochpriorisierte Dokumente werden zuerst verarbeitet.
    pub async fn ingest_batch(
        &self,
        tenant: TenantId,
        documents: Vec<(String, DocumentMetadata, IngestionPriority)>,
        budget: WorkBudget,
    ) -> Result<Vec<CiaiResult<IngestionReport>>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IngestionPriority {
    Critical = 4,   // Sofort verarbeiten (User wartet)
    High = 3,       // Nächste freie Verarbeitungsslot
    Normal = 2,     // Standard (Default)
    Low = 1,        // Hintergrund, wenn keine anderen Jobs
    Batch = 0,      // Nacht/Idle-Verarbeitung
}

#[derive(Debug, Clone)]
pub struct IngestionReport {
    pub chunks_created: u32,
    pub chunks_deduplicated: u32,
    pub triples_extracted: u32,
    pub conflicts_detected: u32,
    pub doc_ids: Vec<DocId>,
    pub avg_chunk_tokens: f32,
}
```

---

## E.8 TemporalMemoryManager — Vergessen, Auffrischung, Saisonalität

### E.8.1 IST / Problem

`AdaptiveDecayController` (🟢) basiert auf TxId-Differenz und tombstone_ratio. Es fehlt:
1. **Saisonale Muster**: "Jeden Montag fragen Agenten nach dem Wochenbericht" → das soll nicht vergessen werden auch wenn selten abgefragt
2. **Explizites Vergessen**: Agent gibt Anweisung "vergiss alles über Projekt X" → sofort und vollständig
3. **Auffrischungs-Management**: Veraltete Fakten sollen markiert (nicht gelöscht) werden bis eine neue Version eintrifft

### E.8.2 SOLL — Schnittstelle

```rust
// crates/contextra-autopilot/src/temporal_memory.rs

pub struct TemporalMemoryManager<S: StorageEngine, V: VectorIndex> {
    collection: std::sync::Arc<Collection<S, V>>,
    decay_controller: AdaptiveDecayController,
    config: TemporalConfig,
    /// Saisonale Muster: (day_of_week, hour) → häufig angefragte DocId-Cluster.
    seasonal_patterns: ahash::AHashMap<(u8, u8), Vec<DocId>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TemporalConfig {
    /// Basis-Halbwertszeit in Transaktionen (default: 50_000 — ca. ~1 Woche bei normalem Volumen)
    pub base_half_life_tx: u64,
    /// Ob saisonale Muster das Vergessen verhindern.
    /// Default: true (saisonales Wissen soll erhalten bleiben)
    pub seasonal_protection: bool,
    /// Ob veraltete Fakten markiert (statt gelöscht) werden.
    /// Default: true (sanftes Vergessen)
    pub mark_stale_before_delete: bool,
    /// Ab welchem Alter (in Tagen) ein Fakt als "potentiell veraltet" markiert wird.
    /// Default: 30
    pub staleness_threshold_days: u32,
}

impl<S, V> TemporalMemoryManager<S, V>
where S: StorageEngine + 'static, V: VectorIndex + 'static
{
    /// Sofortiges, vollständiges Vergessen eines Themas/Projekts.
    /// Kaskadierend: alle verbundenen Hyperkanten werden ebenfalls gelöscht.
    /// Gibt DeletionProof zurück (DSGVO-Art.-17-Konformität).
    pub async fn explicit_forget(
        &mut self,
        tenant: TenantId,
        query: &str,        // "vergiss alles über Projekt X"
        budget: WorkBudget,
    ) -> Result<Vec<contextra_crypto::DeletionProof>>;

    /// Markiert veraltete Fakten ohne Löschung.
    /// Retrieval-Phase kennzeichnet markierte Chunks als "[Möglicherweise veraltet]".
    pub async fn mark_stale_by_age(
        &mut self,
        tenant: TenantId,
        older_than_tx: contextra_types::TxId,
    ) -> Result<u32>;  // Anzahl markierter Dokumente

    /// Erkennt saisonale Muster aus der Query-History.
    /// Wird wöchentlich vom KnowledgeWeaver aufgerufen.
    pub fn update_seasonal_patterns(
        &mut self,
        query_history: &[(contextra_types::TxId, DocId)],
    );

    /// Führt einen Decay-Pass durch mit Schutz für saisonale Inhalte.
    pub async fn decay_tick(
        &mut self,
        tenant: TenantId,
        current_tx: contextra_types::TxId,
        budget: WorkBudget,
    ) -> Result<DecayTickReport>;
}

#[derive(Debug, Clone)]
pub struct DecayTickReport {
    pub documents_evicted: u32,
    pub documents_protected_by_season: u32,
    pub documents_marked_stale: u32,
    pub avg_effective_score: f32,
}
```

---

## E.9 CognitiveBudgetController — Ressourcen-bewusste Operationssteuerung

### E.9.1 IST / Problem

PID-Regler (🟢) skaliert k-Pool basierend auf gemessener Latenz. CognitiveBudgetController erweitert das um systemweite Ressourcen-Awareness: CPU-Druck, RAM-Druck, aktive Tenant-Zahl → dynamische Degradierung auf einfachere Retrieval-Strategien.

### E.9.2 SOLL — Schnittstelle

```rust
// crates/contextra-autopilot/src/budget_controller.rs

/// Systemweite Ressourcen-Steuerung: entscheidet dynamisch
/// welche Qualitätsstufe für Retrieval und Maintenance verwendet wird.
pub struct CognitiveBudgetController {
    pid_controller: contextra_adapt::pid_latency_controller::PidLatencyController,
    config: BudgetControllerConfig,
    /// Aktuell gemessene Systemlast.
    system_load: SystemLoad,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum QualityTier {
    /// Volle Qualität: alle CIAI-Subsysteme aktiv.
    Full,
    /// Reduziert: Graph-Expansion deaktiviert, k halbiert.
    Reduced,
    /// Minimal: nur Vektor-Suche, Top-3, kein Graph.
    Minimal,
    /// Notfall: nur KV-Cache-Lookup, kein Retrieval.
    Emergency,
}

#[derive(Debug, Clone)]
pub struct SystemLoad {
    /// RAM-Auslastung ∈ [0,1]
    pub ram_pressure: f32,
    /// Aktive gleichzeitige Tenants
    pub active_tenants: u32,
    /// p99-Latenz der letzten 60s in ms
    pub p99_latency_ms: f64,
    /// Länge der Ingestion-Queue
    pub ingestion_queue_depth: u32,
}

impl CognitiveBudgetController {
    /// Empfiehlt die optimale Qualitätsstufe für den aktuellen Systemzustand.
    ///
    /// Entscheidungslogik (deterministisch, P28):
    ///   if ram_pressure > 0.90 OR p99_latency_ms > 500: Emergency
    ///   elif ram_pressure > 0.75 OR p99_latency_ms > 200: Minimal
    ///   elif active_tenants > 100 OR p99_latency_ms > 100: Reduced
    ///   else: Full
    pub fn recommend_quality_tier(&self) -> QualityTier;

    /// Gibt ein WorkBudget zurück das zum empfohlenen Tier passt.
    pub fn budget_for_tier(tier: QualityTier) -> WorkBudget {
        match tier {
            QualityTier::Full => WorkBudget::STANDARD,
            QualityTier::Reduced => WorkBudget { wall_time_ms: 50, ..WorkBudget::TIGHT },
            QualityTier::Minimal => WorkBudget::TIGHT,
            QualityTier::Emergency => WorkBudget { wall_time_ms: 2, embedding_calls: Some(0), ..WorkBudget::TIGHT },
        }
    }

    /// Passt AceConfig automatisch an die aktuelle Qualitätsstufe an.
    /// Kein manuelles Tuning nötig — CIAI-DP-1: bequem by default.
    pub fn adapt_ace_config(&self, base_config: &AceConfig) -> AceConfig {
        let tier = self.recommend_quality_tier();
        let mut adapted = base_config.clone();
        match tier {
            QualityTier::Reduced => {
                adapted.graph_expansion_enabled = false;
                adapted.max_graph_hops = 1;
            },
            QualityTier::Minimal => {
                adapted.graph_expansion_enabled = false;
                adapted.leanrag_for_global_queries = false;
                adapted.attention_weighted_ranking = false;
            },
            QualityTier::Emergency => {
                adapted.graph_expansion_enabled = false;
                adapted.leanrag_for_global_queries = false;
                adapted.attention_weighted_ranking = false;
                adapted.token_budget = 512;
            },
            QualityTier::Full => {},
        }
        adapted
    }

    /// Aktualisiert Systemlast-Metriken (wird periodisch vom Composition Root aufgerufen).
    pub fn update_system_load(&mut self, load: SystemLoad);
}
```

---

## E.10 GroundingOracle — Halluzinationsprävention & Faktenverankerung

### E.10.1 IST / Problem

`GaspValidator` (🟢, `contextra-infer-candle`) prüft post-hoc ob Antwort durch Kontext gestützt ist. Es fehlt:
1. **Präventive Verifikation**: Vor LLM-Call prüfen ob genug Grounding-Material im Kontext ist
2. **Cross-Reference-Check**: Aussage gegen Wissensgraph verifizieren (nicht nur gegen den Kontext-Fenster-Inhalt)
3. **Konfidenz-Propagation**: Grounding-Score vom Chunk in die finale Antwort übertragen

### E.10.2 SOLL — Schnittstelle

```rust
// crates/contextra-autopilot/src/grounding_oracle.rs

/// Halluzinationsprävention auf zwei Ebenen:
/// 1. Präventiv: Prüft vor LLM-Call ob Kontext ausreichend ist
/// 2. Post-hoc: Prüft Antwort gegen Kontext + Wissensgraph
pub struct GroundingOracle<S, V, E> {
    gasp_validator: contextra_infer_candle::gasp::GaspValidator,
    collection: std::sync::Arc<Collection<S, V>>,
    embedder: std::sync::Arc<E>,
    config: GroundingConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GroundingConfig {
    /// Minimale Kontext-Abdeckung für präventiven Check.
    /// Default: 0.5 (≥50% der Query-Token sollen im Kontext vertreten sein)
    pub min_context_coverage: f32,
    /// GASP-Schwellenwert (bestehend).
    pub gasp_threshold: f32,
    /// Ob Cross-Reference-Check gegen Wissensgraph.
    /// Default: true (aber budget-abhängig)
    pub cross_reference_check: bool,
}

#[derive(Debug, Clone)]
pub struct GroundingVerdict {
    /// Empfehlung: soll der LLM-Call stattfinden?
    pub proceed: bool,
    /// Grounding-Score ∈ [0,1]
    pub grounding_score: f32,
    /// Spezifische Bedenken (für Agent-Feedback)
    pub concerns: Vec<GroundingConcern>,
    /// Verbesserungs-Vorschlag: diese Chunks würden das Grounding verbessern
    pub suggested_additional_context: Vec<DocId>,
}

#[derive(Debug, Clone)]
pub enum GroundingConcern {
    /// Zu wenig Kontext für diese Anfrage
    InsufficientContext { coverage: f32 },
    /// Kontext enthält widersprüchliche Information
    ContradictoryContext { conflict: DetectedConflict },
    /// Kontext ist veraltet (Staleness-Flag gesetzt)
    StaleContext { doc_id: DocId, staleness_score: f32 },
    /// Anfrage enthält Entitäten die nicht im Wissensgraphen sind
    UngroundedEntities { entities: Vec<String> },
}

impl<S, V, E> GroundingOracle<S, V, E>
where S: StorageEngine + 'static, V: VectorIndex + 'static, E: EmbeddingProvider + 'static
{
    /// Präventiver Check: Ist genug Kontext vorhanden für diese Query?
    /// O(|context_chunks| · d) — schnell, kein LLM-Aufruf.
    pub async fn check_before_generation(
        &self,
        tenant: TenantId,
        query: &str,
        prepared_context: &PreparedContext,
        budget: WorkBudget,
    ) -> Result<GroundingVerdict>;

    /// Post-hoc Verifikation: Ist die Antwort durch Kontext + KB gestützt?
    /// Kombiniert GASP (bestehend) + Knowledge-Graph-Cross-Reference.
    pub async fn verify_after_generation(
        &self,
        tenant: TenantId,
        query: &str,
        generated_response: &str,
        context_used: &PreparedContext,
        budget: WorkBudget,
    ) -> Result<GroundingVerdict>;
}
```

---

## E.11 SelfHealingPipeline — Autonome Fehlerkorrektur

### E.11.1 IST / Problem

Agent-Pipelines brechen bei Fehlern komplett ab. Contextra hat DLQ (Dead Letter Queue) in `contextra-agent` aber keine automatischen Recovery-Strategien.

### E.11.2 SOLL — Schnittstelle

```rust
// crates/contextra-autopilot/src/self_healing.rs

/// Wraps jeden CIAI-Subsystem-Aufruf mit automatischen Recovery-Strategien.
pub struct SelfHealingPipeline {
    config: HealingConfig,
}

#[derive(Debug, Clone)]
pub struct HealingConfig {
    /// Maximale Retry-Versuche pro Fehler.
    /// Default: 3
    pub max_retries: u8,
    /// Ob bei Embedding-Fehler auf Nur-Text-Retrieval degradiert wird.
    pub fallback_to_text_on_embedding_failure: bool,
    /// Ob bei LLM-Fehler die Top-3-Chunks ohne Synthesis zurückgegeben werden.
    pub fallback_to_raw_chunks_on_llm_failure: bool,
    /// Ob Fehler in den DLQ eingereiht werden für späteres Retry.
    pub enqueue_to_dlq_on_exhaustion: bool,
}

/// Recovery-Strategie für spezifische Fehlertypen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Sofortiger Retry mit exponentiellem Backoff.
    ExponentialBackoff { base_ms: u32 },
    /// Fallback auf einfachere Operation (Qualitätsreduktion).
    GracefulDegradation,
    /// Skip dieses Subsystem, weitermachen ohne es.
    SkipSubsystem,
    /// Einreihen in DLQ für späteren Retry.
    EnqueueDlq,
    /// Nicht recoverable: propagiere Err nach oben.
    Propagate,
}

impl SelfHealingPipeline {
    /// Führt eine CIAI-Operation mit automatischer Recovery aus.
    pub async fn execute_with_healing<T, F, Fut>(
        &self,
        operation: F,
        error_strategy: fn(&contextra_types::ContextraError) -> RecoveryStrategy,
    ) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>;

    /// Standard-Error-Strategy-Mapping für CIAI-Subsysteme.
    pub fn default_strategy(error: &contextra_types::ContextraError) -> RecoveryStrategy {
        match error {
            ContextraError::Timeout { .. } =>
                RecoveryStrategy::GracefulDegradation,
            ContextraError::ModelLoad { .. } =>
                RecoveryStrategy::ExponentialBackoff { base_ms: 500 },
            ContextraError::MemoryBudgetExceeded { .. } =>
                RecoveryStrategy::GracefulDegradation,
            ContextraError::Io(_) =>
                RecoveryStrategy::ExponentialBackoff { base_ms: 100 },
            ContextraError::LimitExceeded { .. } =>
                RecoveryStrategy::SkipSubsystem,
            _ => RecoveryStrategy::Propagate,
        }
    }
}
```

---

## E.12 FrictionlessPersonalization — Zero-Effort-Nutzeradaption

### E.12.1 IST / Problem

`RieGreedyPersonalizer` (🟢) und `ConformalCalibrator` (🟢 in `contextra-router`) sind vorhanden aber müssen explizit konfiguriert werden. FrictionlessPersonalization aktiviert beide automatisch ohne jegliche Konfiguration.

### E.12.2 SOLL — Schnittstelle

```rust
// crates/contextra-autopilot/src/personalization.rs

/// Zero-Effort-Personalisierung: läuft vollständig implizit.
/// Keine expliziten Feedback-Calls nötig — lernt aus Retrieval-Mustern.
pub struct FrictionlessPersonalization {
    rie_personalizer: contextra_adapt::rie_greedy::RieGreedyPersonalizer,
    temporal_decay: contextra_adapt::rie_greedy::TemporalDecayConfig,
    config: PersonalizationConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PersonalizationConfig {
    /// Wie stark Personalisierung in ACE-Ranking eingeht.
    /// Default: 0.3 (30% Personalisierungs-Boost)
    pub personalization_weight: f32,

    /// Ob implizites Feedback aus Retrieval-Nutzung abgeleitet wird.
    /// Mechanismus: Chunks die Agent länger im Kontext behält → positives Signal.
    /// Default: true
    pub implicit_feedback_from_context_retention: bool,

    /// Ob Session-übergreifendes Lernen aktiv ist.
    /// Default: true
    pub cross_session_learning: bool,
}

impl FrictionlessPersonalization {
    /// Passt Chunk-Scores basierend auf Nutzer-Profil an.
    /// Wird automatisch in ACE Phase 4 (Ranking) aufgerufen.
    /// Kein expliziter Call nötig.
    pub fn personalize_scores(
        &self,
        tenant: TenantId,
        chunks: &mut Vec<ScoredContextChunk>,
    );

    /// Lernt implizit aus welche Chunks der Agent tatsächlich genutzt hat.
    /// Wird automatisch nach jeder Agent-Antwort aufgerufen.
    pub fn observe_usage(
        &mut self,
        tenant: TenantId,
        chunks_used: &[DocId],
        chunks_discarded: &[DocId],
        context_embedding: &[f32],
    );
}
```


# Contextra → Baukasten: Entkopplungsanalyse und Architekturentscheidungen

**Rolle:** Principal Rust Engineer · **Stand:** 11.11.2026 · **Objekt:** `tfufuz1/contextra` (flacher Clone, HEAD)
**Methode:** ausschließlich statisch (Manifest-Parsing, Quelltext-Grep, Lesen). Es wurde **nichts kompiliert, getestet oder ausgeführt**.

Beleg-Marken: **[V]** direkt an Manifest/Quelltext gemessen · **[A]** daraus abgeleitet, nicht gebaut.

---

## 1. Urteil

Das Repo ist **zweigeteilt**:

- **Echte Bausteine (Leaf-Crates):** `simd`, `rank`, `adapt`, `privacy`, `prompt-guard`, `erasure`, `sandbox`, `ingest`, `sys`, `wire`, `durable-fs`, `ids`, `env`. Ihre zwingende interne Abhängigkeits-Closure ist leer, Quelltext und Manifest stimmen überein. [V]
- **Das Kernprodukt ist nur im Manifest entkoppelt:** `vector`, `graph`, `text`, `store`, `crypto`, `agent`, `cognition`, `mcp`. Dort sind `types`/`ports` als `optional` deklariert, der Quelltext importiert sie aber ungated oder die Default-Features schalten sie wieder ein. [V]

Bei den **Algorithmen** sieht es besser aus als bei den **Crates**: Der Kern von HNSW, PPR, WAND, GMM, Transitivity-Veto ist meist fachlich rein. Er scheitert an Typen (`ContextraError`, `DocId`), an Konfigurationen, die im falschen Crate liegen, und an Crate-Grenzen, die falsch geschnitten sind.

| Bewertung der 50 Einzelfunktionen aus Abschnitt 4 (Spalte „Entn.“) | Anzahl |
|---|---|
| Als Crate sofort entnehmbar (✅) | 24 |
| Entnehmbar nach kleinem, mechanischem Eingriff (⚠️) | 16 |
| Fest verdrahtet (❌) | 9 |
| Im Quelltext nicht auffindbar (Context Compaction) | 1 |

Die Verteilung nach Domäne: Rang/Routing/Datenschutz sind fast durchgehend ✅, Retrieval-Kern (HNSW, WAND) und Speicher (LSM, WAL) sowie Agent/Kognition tragen die ❌.

---

## 2. Methodik und Grenzen

1. Alle `Cargo.toml` per `tomllib` geparst: zwingende vs. `optional` Abhängigkeiten, Default-Features, daraus die **transitive Closure ohne Features**.
2. Quelltext: Importe von `contextra_types`/`contextra_ports`/anderen Crates je Datei, Abgleich mit `cfg(feature)` in `lib.rs`.
3. Algorithmen per Grep auf Dateien abgebildet, Imports Datei für Datei gelesen.
4. Repo-eigene Dokumente (`AGENTS.md`, `docs/…Kopplungsbericht.md`, `docs/generated/baukasten-status.md`) **als Gegenprobe** gelesen, nicht ungeprüft übernommen.

**Grenzen**

- Die Gating-Heuristik ist grob: Item-Level-`#[cfg]` auf einzelnen `impl`-Blöcken erkennt sie nicht. Ein Treffer in `text/types.rs` war nachweislich ein False Positive (`#[cfg(feature = "types-compat")]` vor dem Block). Zahlen sind Näherungen; der Repo-eigene Bericht zählt für `vector` z. B. 35 Dateien, meine Heuristik ≥ 24.
- „Kompiliert standalone nicht“ ist **[A]**: Es folgt aus ungated `use contextra_types` bei optionaler Dependency, wurde aber nicht gebaut.
- Laufzeit- und Performance-Effekte der empfohlenen Umbauten (z. B. HNSW-Hotpath) sind nicht gemessen.
- Das Repo besitzt bereits Gate-Werkzeuge (`tools/isolation_proof.py`, `gate_layering.py`, `export_crate.py`). Sie wurden **nicht ausgeführt** und sind der erste Schritt der Roadmap.

---

## 3. Querschnittsbefunde

### F1 · Fehlertyp-Monolith `ContextraError` [V]
`contextra-types/src/error.rs` hat 1177 Zeilen und enthält domänenspezifische Varianten wie `HnswConnectivityDegraded`, `WalCorruption`, `ChecksumMismatch`, `KvQuantization`, `SandboxTimeout`, `EmbeddingDimensionMismatch`, `OrphanedVectorReference`. Jeder Algorithmus, der `ContextraError` importiert, zieht damit die Fehlerwelt des Gesamtsystems nach sich. Ausgerechnet die Crates, die es am wenigsten bräuchten, nutzen ihn am stärksten: `vector` 77×, `graph` 16×, `text` 17×.

Gleichzeitig hat fast jedes Crate bereits einen eigenen Fehlertyp (`RankError`, `TextError`, `SandboxError` …). Das richtige Muster existiert also schon.

### F2 · Manifest widerspricht Quelltext [V]
`vector/src/types.rs` behauptet im Kopf, der Baustein arbeite „ohne interne Abhängigkeiten zu `contextra-ports`, `contextra-types` oder `contextra-mvcc`“. Dagegen stehen ungated:

- `vector/src/hnsw/types.rs:11` → `use contextra_types::{ContextraError, DocId, Result}`
- `vector/src/hnsw/types.rs:231` → Feld `RwLock<contextra_mvcc::SequenceLog>`
- `vector/src/hnsw/types.rs:236-237` → `Arc<dyn contextra_ports::{Rng, Clock}>`
- `vector/src/quantize/mod.rs:19` → `use contextra_types::DistanceMetric`
- `graph/src/hyperedge.rs:16`, `graph/src/ppr/forward_push.rs:17`, `graph/src/csr/types.rs:13`, `graph/src/community.rs:22`

`pub mod hnsw;` (`vector/lib.rs:69`) und `pub mod ppr/csr/community` (`graph/lib.rs`) sind nicht gegated. **[A]** Ohne die Features `ports-compat`/`workspace` bauen beide Crates nicht.

### F3 · Default-Features schalten die Bindung wieder ein [V]
`vector`: `default = ["std","ports-compat"]` aktiviert `types`, `ports`, `mvcc`, `sys`. `graph`, `text`, `rank`: `default` enthält `workspace`. „Optional“ ist damit nur auf dem Papier optional; ein Konsument bekommt den Workspace-Stack per Default.

### F4 · Seam-Duplikate trotz vorhandener Besitzer [V]
`env` (Clock, Rng, MetricsSink, IdGen) und `ids` (DocId, TxId, TenantId, EntityId) existieren als Leaf-Crates, die Kopien sind aber nicht abgelöst:

| Seam | Eigene Definitionen in |
|---|---|
| `trait Clock` | vector (2×), text, rank, prompt-guard, privacy, license, graph, erasure, env, adapt (10 Crates) |
| `trait Rng` | vector (2×), text, graph, env, adapt |
| `struct DocId` | vector (2×), text (2×), ids (2× cfg), rank, ingest, graph, erasure (7 Crates) |
| `enum DistanceMetric` | vector, types, simd |
| `TokenBudget` | `agent::budget` **und** `contextra_types` |

Besonders inkonsistent: `text` hängt **zwingend** an `env` und `ids`, definiert `Clock`, `Rng` und `DocId` aber trotzdem selbst.

### F5 · Algorithmus-Konfiguration liegt im falschen Crate [V]
`PprConfig` und `TlHfdParams` (graph), `DistanceMetric` (vector/simd), `MAX_SEARCH_K`, `TOMBSTONE_BIT` (text/store) liegen in `contextra-types`. Die „gemeinsame Typen“-Schicht kennt damit Parameter einzelner Algorithmen: eine umgekehrte Abhängigkeit, die jeden Baustein an `types` fesselt.

### F6 · Shims verschleiern Kopplung [V]
`cognition/src/engine_local.rs:10` (`pub use contextra_engine as engine;`) und `graph_local.rs:10` leiten das konkrete `engine::collection::Collection`, `AdaptiveDecayController`, `graph::detect_communities` und `CsrGraph` durch. Grep auf `contextra_engine::` zeigt fast nichts, tatsächlich hängt `cognition` am Integrations-Crate (Closure 14).

### F7 · Closure ohne Features (nur zwingende Kanten) [V]

| Crate | Closure | Crate | Closure |
|---|---|---|---|
| simd, rank, adapt, privacy, erasure, ingest, sandbox | **0** | store | 7 |
| vector, graph (nur Manifest) | 0 (⚠️ siehe F2/F3) | router | 5 |
| text, mvcc, crypto | 2 (`env`, `ids`) | infer-candle | 5 |
| cognition | 14 | agent | **18** |
| mcp | **22** | | |

`agent` zieht über `db` → `engine` den kompletten Speicher-/Index-Stack samt `vault`.

### F8 · Weitere Hygienepunkte [V]
- `contextra-vault` steht in `exclude` des Workspaces (`Cargo.toml:44-46`), ist aber zwingende Abhängigkeit von `db`. Es wird von Workspace-weiten CI-Läufen nicht erfasst.
- Drei unabhängige BLAKE3-Hash-Ketten: `privacy/context_edit_audit.rs`, `crypto/audit_chain.rs` (374 LOC), `agent/audit.rs`.
- Zwei „Vault“-Konzepte mit gleichem Namen: `privacy/egress_vault.rs` (Surrogate Vaulting) und `contextra-vault` (RAM-Locking). Der Schnitt `sys` (Primitive) → `vault` (Wrapper, lokale `VaultDocId`) ist dagegen sauber.
- Veraltete Erasure-Typen stehen noch deprecated in `types`/`ports` (`ErasureId`, `GateId` mit `MIGRATION(S-70)`-Hinweis).
- `AGENTS.md` nennt „Klassen A/B/C“ dekommissioniert, `docs/` und Tools verwenden sie weiter.

---

## 4. Bewertung je Funktion

**Kern rein** = Algorithmus ohne Systembindung. **Entnehmbar** = Crate lässt sich ohne Workspace-Stack einsetzen.

### Domäne 1: Hybrid Retrieval

| Funktion | Ort | Kern | Entn. | Beleg / Blocker |
|---|---|---|---|---|
| SIMD AVX2/512/NEON/Skalar | `simd/kernels/*`, `dispatch.rs` | ✅ | ✅ | Closure 0, eigene `DistanceMetric`, Brücke nur in `compat` |
| HNSW | `vector/hnsw/*` | ⚠️ | ❌ | F2: `ContextraError`, `DocId`, `mvcc::SequenceLog`, Ports `Clock/Rng/MetricsSink`; `vector_index_impl.rs:140` wirft `HnswConnectivityDegraded` |
| SQ8 | `vector/quantize/mod.rs`, `hnsw/sq8_bias.rs` | ✅ | ⚠️ | nur `DistanceMetric` + `ContextraError` |
| RaBitQ | `vector/quantize_rabitq.rs` | ✅ | ⚠️ | nur `ContextraError, Result` (:24) |
| DiskANN | `vector/diskann/*` | ⚠️ | ⚠️ | implementiert `ports::VectorIndex` direkt; experimentell gegated |
| Acorn | `vector/acorn/*`, `hnsw/acorn_filtered.rs` | ✅ | ⚠️ | `DocId`/`DistanceMetric` aus `types` |
| BM25 | `text/bm25.rs` | ✅ | ✅ | lokale `TextError`, `LocalDocId` |
| Block-Max WAND | `text/wand.rs` (891) | ⚠️ | ❌ | hinter `types-compat` (`lib.rs:66`); importiert `ContextraError, DocId, Result, MAX_SEARCH_K` |
| Deutsche Morphologie | `text/morphology/*`, `tokenizer.rs` | ✅ | ✅ | 0 `contextra_*`-Referenzen |
| Inverted Index (Snapshots) | `text/inverted/*`, `memory_index.rs` | ⚠️ | ⚠️ | produktive Variante an `ports::{TextIndex,…}` + `types-compat` gebunden; lokaler `MemoryTextIndex` existiert. CoW-Snapshot-Semantik habe ich statisch nur teilweise lokalisiert |
| Personalized PageRank | `graph/ppr/forward_push.rs` (278) | ✅ | ⚠️ | `{EntityId, PprConfig}` aus `types` (F5) |
| CSR | `graph/csr/*` | ✅ | ⚠️ | `{DocId, EntityId, ResourceTracker, TxId}` |
| K-Path / APPRH | `graph/path_rag/k_path.rs`, `apprh/` | ✅ | ⚠️ | nur `EntityId`, `Result`; feature-gegated |
| Community / Perkolation / Edge-RL | `graph/community.rs` (640), `percolation`, `edge_reinforcement` | ✅ | ⚠️ | `community.rs:22` nur `EntityId, Result` |
| wRRF / mRRF / Resonanz | `rank/fusion/{rrf,mrrf}.rs` | ✅ | ✅ | `local_types` (DocId, FusionStrategy), `RankError` |
| PAVA / Platt / Conformal | `rank/calibration/*` | ✅ | ✅ | nur `local_types` |
| Drift Detection | `rank/drift.rs` | ✅ | ✅ | keine Abhängigkeiten |

### Domäne 2: Agenten & Kognition

| Funktion | Ort | Kern | Entn. | Beleg / Blocker |
|---|---|---|---|---|
| StateGraph-Schleife | `agent/engine.rs` | ✅ | ❌ | Kern ist portbasiert (`Arc<dyn AgentCheckpointStore>`, `Arc<dyn Clock>`, `Box<dyn AgentTool>`), aber `engine.rs:89,112` `try_from_db(&contextra_db::Contextra)`, `context.rs:114` `Arc<contextra_router::DefaultRouterEngine>`; Closure 18 |
| RAII-Checkpoint / Rollback | `agent/engine.rs`, `audit.rs`, `dlq.rs` | ⚠️ | ❌ | `audit.rs:20` an `db::Collection` |
| TokenBudget | `agent/budget`, `types/budget.rs` | ⚠️ | ⚠️ | zwei Typen (F4) |
| Memory Consolidation (strukturell) | `cognition/memory_consolidation.rs`, `consolidation_executor.rs` | ✅ | ❌ | `detect_near_duplicates(&[(DocId, Vec<f32>)], f32)` (:280) ist rein, der Executor hängt an `engine::collection::Collection` (F6) |
| Context Compaction (Truncate/Summary/Selective) | – | ? | ? | **als Strategien nicht auffindbar**; nur Kommentar `cognition/context.rs:114`. Lücke zwischen Spezifikation und Code |
| LeanRAG-GMM | `cognition/aggregation_phase.rs` (928) | ✅ | ❌ | `run_deterministic_diagonal_gmm` (:613), SplitMix64, deterministisch, aber im Integrations-Crate |
| Transitivity Veto | `cognition/transitivity_veto.rs` (177) | ✅ | ⚠️ | arbeitet auf `types_local::{DocId,EntityId}`, hängt an `aggregation_phase` im selben Crate |

### Domäne 3: Routing & Regelung

| Funktion | Ort | Kern | Entn. | Beleg / Blocker |
|---|---|---|---|---|
| LinUCB + Sherman-Morrison | `adapt/bandit.rs` (1218) | ✅ | ✅ | Closure 0, eigene `Clock`/`Rng` |
| FC-TS | `adapt/flow_thompson.rs` (918) | ✅ | ✅ | |
| RIE-Greedy | `adapt/rie_greedy.rs` | ✅ | ✅ | |
| PID / Homeostat / Decay | `adapt/pid*.rs`, `homeostat.rs`, `decay_controller.rs` | ✅ | ✅ | |
| Router | `router/*` | ✅ | ⚠️ | zwingend `types, ports, adapt` (Closure 5); `adapt` selbst ist rein |

### Domäne 4: Datenschutz & Compliance

| Funktion | Ort | Kern | Entn. | Beleg / Blocker |
|---|---|---|---|---|
| PII/DLP (IBAN, E-Mail) | `privacy/entity_recognizer.rs` (1398) | ✅ | ✅ | 0 interne Importe |
| Surrogate Vaulting | `privacy/egress_vault.rs`, `egress_gateway.rs` | ✅ | ✅ | |
| k-NN Egress Guard | `privacy/egress_guard.rs` | ✅ | ✅ | keine Abhängigkeit auf `vector` → Ähnlichkeitsquelle muss injiziert sein **[A]** |
| Bulk-Exfiltration | `privacy/bulk_exfiltration_detector.rs` | ✅ | ✅ | `mcp` re-exportiert nur (4 LOC, keine Duplikation) |
| Prompt-Injection + Audit | `prompt-guard/guard.rs`, `audit.rs` | ✅ | ✅ | `audit.rs` ist ein `std::fs`-Logger, nicht verkettet |
| Context-Edit-Audit (BLAKE3) | `privacy/context_edit_audit.rs` | ✅ | ✅ | liegt in `privacy`, nicht in `prompt-guard`; F8 |
| Crypto Shredding | `erasure/kv_shredding.rs` | ✅ | ✅ | Closure 0, eigenes `crypto_local.rs` |
| Ed25519-Löschnachweise | `erasure/ed25519_proof.rs`, `report/signing.rs` | ✅ | ✅ | Feature `signing` |
| AES-GCM-SIV / Argon2 / HKDF | `crypto/kv_cipher.rs`, `wal_crypto.rs` | ✅ | ⚠️ | Primitive rein, aber Crate bindet `TenantId/TenantScoped`, `kv_segment`, `audit_chain`; ein Repo-ADR benennt das selbst als Verstoß |
| mlock / VirtualLock | `sys/mlock.rs`, `sys/vault.rs` | ✅ | ✅ | |
| ZeroizeOnDrop-Vault | `vault/vault.rs` | ✅ | ⚠️ | aus Workspace ausgeschlossen (F8), lokale `VaultDocId/TxId/TenantId` |

### Domäne 5: Speicher & Isolation

| Funktion | Ort | Kern | Entn. | Beleg / Blocker |
|---|---|---|---|---|
| LSM (Memtable/SSTable/Compaction) | `store/*` (31 kLOC) | ⚠️ | ❌ | zwingend `types, ports, sys, mvcc, crypto` (Closure 7), 249 `types`/`ports`-Importe |
| Bloom-Filter | `store/sstable/bloom.rs` (177) | ✅ | ⚠️ | importiert `ContextraError` nur für `Result` (:7) |
| WAL + HMAC-Kette | `store/wal/*` | ⚠️ | ❌ | `wal/mod.rs`, `encode.rs` importieren `crypto::wal_crypto::{WalHmac, IntegrityVerifier}`, `KeyManager` |
| SSI / SequenceLog / ReadSet | `mvcc/ssi.rs` (1237), `seq_log.rs`, `tx_buffer.rs` | ✅ | ✅ | zwingend nur `env`, `ids`; `types` nur im Fehlerpfad |
| WASM-Fuel | `sandbox/executor/mod.rs:38,127` | ✅ | ✅ | `consume_fuel(true)` + `set_fuel(max_fuel)`, Closure 0 |

### Domäne 6: Ingest, Inferenz, MCP

| Funktion | Ort | Kern | Entn. | Beleg / Blocker |
|---|---|---|---|---|
| Chunking / Extraktion | `ingest/*` (21 kLOC) | ✅ | ✅ | Closure 0, `compat/types.rs` kapselt die ID-Konversion, Formate per `fmt-*` |
| Candle / GGUF | `infer-candle/gguf_loader.rs` | ✅ | ⚠️ | zwingend `types, ports, rank`; `rank` nur für `compat/calibrator.rs:4` |
| ONNX | `infer-onnx` | ✅ | ✅ | 1 Referenz (`ports::EmbeddingProvider`) |
| Ollama + Circuit-Breaker | `infer-ollama/client/circuit_breaker.rs` | ✅ | ✅ | `default = []`, `reqwest` hinter `client` |
| MCP JSON-RPC / STDIO | `mcp`, `wire` | ⚠️ | ❌ | JSON-RPC-Typen sauber in `wire` (Closure 0). `mcp` hängt zwingend an `contextra`-Fassade, `db` … (Closure 22); `io.rs`, `server_dispatch.rs`, `validation.rs` haben aber 0 Workspace-Referenzen → trennbar |

---

## 5. Architekturentscheidungen

Jede Entscheidung ist final formuliert. Die Begründung nennt den Befund, der sie erzwingt.

### AD-1 · Vier Ebenen mit hartem Abhängigkeitsrichtung-Gesetz
| Ebene | Inhalt | Darf zwingend abhängen von |
|---|---|---|
| **E0a Seams** | `ids`, `env`, `sys`, `wire`, `durable-fs` | nichts Internes |
| **E0b Bausteine** | `simd`, `rank`, `adapt`, `privacy`, `prompt-guard`, `erasure`, `sandbox`, `ingest`, `vault`, `license` | nur E0a (`ids`, `env`) |
| **E1 Capability** | `vector`, `text`, `graph`, `mvcc`, `crypto`, `store`, `checkpoint`, `kvcache`, `infer-*`, `router` | E0a, E0b (explizite Whitelist je Crate) |
| **E2 Bridge** | `contextra-bridge`, `engine`, `db`, `cognition`, `agent` | beliebig E0/E1 |
| **E3 Anwendung** | `contextra`, `mcp`, `py` | E2 |

*Begründung:* F7. Heute stehen `agent` (18) und `mcp` (22) auf E2/E3, ziehen aber niemals nutzbare Teilmengen heraus.

### AD-2 · `ids` und `env` sind die einzigen universellen Seams
`DocId`, `TxId`, `TenantId`, `EntityId` kommen ausschließlich aus `ids`; `Clock`, `Rng`, `MetricsSink`, `IdGen` ausschließlich aus `env`. **Alle lokalen Kopien werden gelöscht** (10× `Clock`, 5× `Rng`, 7× `DocId`).
*Begründung:* F4. Beide Crates sind Leafs mit minimalen Abhängigkeiten; die Kopie ist keine Autarkie, sondern Drift-Risiko. Ein Konsument mit eigenen IDs mappt via `From<u64>` am Rand. Keine Generics über die ID (HNSW-Hotpath, einfache Signaturen).
*Ausnahme:* `erasure`, `privacy`, `prompt-guard` dürfen eigene Typen behalten, weil sie bewusst vollständig null-abhängig bleiben sollen (Sicherheitsbausteine, auditierbare Closure).

### AD-3 · Ports gehören dem Konsumenten, nicht einer zentralen Mega-Crate
Jeder E0/E1-Baustein definiert **seinen minimalen Vertrag selbst** (`contract.rs`: `PostingStore`, `NeighborSource`, `GraphSource` …). Das Muster existiert bereits (`vector/contract`, `text/contract`, `text/storage`). `contextra-ports` (6,7 kLOC) schrumpft zu **Anwendungsports** (`StorageEngine`, `VectorIndex`, `TextIndex`, `LicenseGate`), die nur E2/E3 nutzen. Kein E0/E1-Crate importiert `ports`.
*Begründung:* F1/F3. Eine zentrale Trait-Crate erzwingt Fan-in (21 Verwender) und macht jeden Baustein von ihrer Evolution abhängig.

### AD-4 · Ein Fehlertyp pro Crate, Übersetzung im Bridge, kein `ContextraError` unterhalb E2
- Jedes Crate: `#[derive(thiserror::Error)] #[non_exhaustive] pub enum <Crate>Error`.
- `ContextraError` verliert alle algorithmusspezifischen Varianten und wird zu `{ domain: Domain, kind: ErrorKind, source: Box<dyn Error + Send + Sync> }`.
- **Orphan-Regel beachten:** `impl From<VectorError> for ContextraError` darf weder in `types` (würde `vector` importieren) noch in `contextra-bridge` liegen (beides fremde Typen). Deshalb: Konstruktor `ContextraError::wrap(Domain::Index, e)` plus Extension-Trait `trait IntoContextra` im Bridge. Das ist der eigentliche Grund, warum bisher `compat`-Features *im* Crate liegen, und der Grund, warum sie zu Bindung führten.
*Begründung:* F1, F2.

### AD-5 · Konfiguration gehört dem Algorithmus
`PprConfig`, `TlHfdParams` → `graph`; `DistanceMetric` → **`simd`** (einziger Besitzer; `vector` re-exportiert); `MAX_SEARCH_K` → je Index-Crate als `const`/Config-Feld; `TOMBSTONE_BIT` → `store`/`mvcc`. `types` behält nur anwendungsweite DTOs (`SearchQuery`, `Filter`, `ScoredDocument`, `Schema`).
*Begründung:* F5.

### AD-6 · Kein `compat` im Baustein, ein `contextra-bridge` auf E2
Alle Konversionen und Port-Implementierungen (`impl VectorIndex for HnswIndex`, `impl From<LocalDocId>`, `HnswPurger`, `HnswScanner`) wandern in **ein** Crate `contextra-bridge` mit additiven Features pro Baustein (`vector`, `text`, `graph`, `store` …). Features in E0/E1 sind ausschließlich **additiv und fachlich** (`avx512`, `diskann`, `fmt-pdf`), nie struktureller Natur.
*Entscheidung statt „ein Adapter-Crate pro Baustein“:* ein Ort für den gesamten Klebstoff, E0/E1 bleiben frei davon. Der Preis (Bridge als Kopplungsmagnet) ist akzeptiert, weil er genau dort liegt, wo Kopplung hingehört.

### AD-7 · `default = []` für alle E0/E1, nur `std` erlaubt
Kein `default`, das Workspace-Abhängigkeiten aktiviert (F3).

### AD-8 · Crate-Schnitte
Aufteilen, wo ein Crate Schichten mischt oder >~15 kLOC ist:

| Heute | Neu | Inhalt |
|---|---|---|
| `graph` (23 kLOC) | `graph-core` | `csr`, `ppr`, `path_rag`, `apprh`, `community` |
| | `graph-rag` (E2) | `hyperedge`, `cascade`, `entity_extraction`, `session_dag`, `provenance`, `purger` |
| `store` (31 kLOC) | `store-lsm` | Memtable, SSTable, Bloom, Compaction, Manifest |
| | `store-wal` | WAL, HMAC-Kette (generisch über `ChainSigner`-Trait) |
| `crypto` | `crypto` | nur Primitive: Cipher, KDF, HMAC, `wal_crypto` |
| | `audit-chain` | **einzige** BLAKE3-Kette; `privacy`, `agent`, `crypto` nutzen sie |
| | `kv-segment` | Domänenlogik (`TenantIsolatedKvStore`, `kv_segment`) |
| `agent` | `agent-core` (E1) | StateGraph, RAII-Checkpoint, TokenBudget, Traits |
| | `agent-contextra` (E2) | `db`-Bindung, `DefaultRouterEngine`, Scratchpad |
| `cognition` | `cognition-core` (E1) | `detect_near_duplicates`, GMM, Transitivity-Veto, Konsolidierungs-Pass auf Slices/Traits |
| | `cognition` (E2) | `Collection`-/`engine`-Bindung, Executor |
| `mcp` | `mcp-core` (E1) | `io`, `server_dispatch`, `validation`, `protocol` auf `wire` |
| | `mcp` (E3) | Tool-Bindung an `contextra`, Egress/Prompt-Guard-Verdrahtung |
| `vector` | `vector` | HNSW, SQ8, Acorn; RaBitQ/DiskANN bleiben feature-gegated |

*Begründung:* F6, F7. Die Kerne von `cognition` und `agent` sind fachlich rein, die Crate-Grenze ist falsch.

### AD-9 · Reine Kerne, I/O an den Rändern
E0/E1-Kerne sind synchron, deterministisch (`Clock`/`Rng` injiziert) und arbeiten auf Slices bzw. Traits. `async` und `async_trait` nur in E2/E3 und in I/O-Adaptern (`infer-ollama`). `candle`/`ort`/`wasmtime`/`reqwest` bleiben hinter Features.

### AD-10 · Schnittstellen-Vertrag je Baustein
Jedes Crate liefert: (1) ein `contract`-Modul (Traits + Fehler), (2) eine **Referenzimplementierung ohne I/O** (`MemoryTextIndex`-Muster), (3) ein `examples/standalone.rs` mit weniger als 30 Zeilen. Das ist die Messlatte für „wenig Implementierung beim Einsatz“.

---

## 6. Durchsetzung (CI-Gates)

| Gate | Prüfung | Zielzustand |
|---|---|---|
| G-1 | `cargo check -p X --no-default-features` für **jedes** Crate (E0/E1) | grün |
| G-2 | `cargo tree -p X -e normal --no-default-features`: Closure ⊆ Whitelist (E0a/E0b) | E0b: ⊆ {ids, env} |
| G-3 | `cargo hack check --each-feature -p X` | grün, Features additiv |
| G-4 | `grep`/Clippy `disallowed-types`: `ContextraError`, `contextra_types::`, `contextra_ports::` in E0/E1 | 0 Treffer |
| G-5 | `trait Clock/Rng`, `struct DocId` außerhalb `env`/`ids` | 0 Treffer (Ausnahmen AD-2) |
| G-6 | `tools/export_crate.py` (Standalone-Export + Check) pro E0/E1-Crate | grün |
| G-7 | `cargo-semver-checks` auf Baustein-Crates | keine ungeplanten Brüche |
| G-8 | `cargo deny` mit `bans` für E0→E2-Kanten | 0 Verstöße |

Die Gates G-1, G-2, G-6 existieren in Teilen bereits als Skripte im Repo; sie müssen **blockierend** werden, und `contextra-vault` muss zurück in die Workspace-Members.

---

## 7. Roadmap

| Prio | Maßnahme | Abnahme |
|---|---|---|
| **P0** | Gates G-1/G-2/G-6 ausführen und blockierend schalten; `vault` in Members; Ist-Zustand als Baseline fixieren | Baseline-Report, CI rot für alle Verstöße aus F2 |
| **P0** | AD-4: `ContextraError` entkernen, `wrap`/`IntoContextra` einführen | keine algorithmusspezifische Variante mehr in `types` |
| **P1** | `vector`: `hnsw/*`, `quantize/*` auf lokale `VectorError`, `ids::DocId`, `env::{Clock,Rng,MetricsSink}`; `SequenceLog`-Bindung hinter Trait `VisibilityGate` | G-1 für `vector` grün; HNSW-Benchmarks unverändert (Hotpath-Regression prüfen) |
| **P1** | `graph-core` abspalten; `PprConfig`/`TlHfdParams` verschieben | `graph-core` Closure ⊆ {ids} |
| **P1** | `text`: `wand.rs`/`inverted/*` auf `contract::PostingStore`; lokale `Clock/Rng/DocId` löschen (AD-2) | WAND ohne `types-compat` baubar |
| **P1** | `cognition-core` und `agent-core` abspalten (AD-8) | Closure ≤ 4 |
| **P2** | `store`: Bloom/SSTable/Memtable → `store-lsm`; WAL/HMAC → `store-wal` mit `ChainSigner`-Trait | `store-lsm` ohne `crypto` |
| **P2** | `crypto` aufteilen; **eine** Audit-Kette (`audit-chain`) | 3 → 1 Implementierung |
| **P2** | `mcp-core` abspalten | `mcp-core` ohne `contextra`/`db` |
| **P3** | `infer-candle`: `rank` → optional; `Clock/Rng`-Reste entfernen; deprecated Erasure-Typen aus `types`/`ports` entfernen; Begriffe „Klasse A/B/C“ in Docs bereinigen | G-4/G-5 = 0 |
| **P3** | Context-Compaction-Strategien (Truncate/Summary/Selective) implementieren oder aus der Spezifikation streichen | Spezifikation = Code |

**Reihenfolge begründet:** P0 schafft Messbarkeit und löst den größten Hebel (F1). P1 bedient die vier Crates, die den Produktwert tragen (Suche, Graph, Gedächtnis, Agent). P2 ist aufwändiger und riskanter (Storage-Format, Krypto) und kommt erst, wenn die Gates stehen.

---

## 8. Offene Punkte (nicht belegbar)

1. **Kompilierbarkeit ohne Features** ist abgeleitet, nicht bewiesen (F2).
2. **CoW-Snapshot-Isolation** des Inverted Index: Mechanik nur teilweise statisch lokalisiert.
3. **Context Compaction** (Truncate/Summary/Selective) und **Sliding-Window-Clustering** der Konsolidierung: als eigenständige Strategien im Quelltext nicht gefunden.
4. **k-NN Egress Guard:** Herkunft der Vektorähnlichkeit ohne `vector`-Abhängigkeit nicht geklärt.
5. **LinUCB „Sketched Projection“** nicht gesondert verifiziert.
6. Performance-Auswirkung der Umbauten auf HNSW/WAND-Hotpaths ist ungemessen und muss durch die Benchmarks in `benchmarks/contextra-bench` abgesichert werden.

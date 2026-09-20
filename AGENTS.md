# MemFuse — AI-Assistenten-Kontext

<!-- Anker-Index (für §N-Referenzen in anderen Dokumenten) -->
<!-- §1 = Verifizierter Codestand -->
<!-- §2 = Crate-Topologie (Ring-0–4-Modell) -->
<!-- §3 = Was TATSÄCHLICH implementiert ist vs. FEHLT (verifiziert) -->
<!-- §4 = Bekannte offene Risiken -->
<!-- §5 = ⚠️ Frischhaltungspflicht dieser Datei -->
<!-- §6 = Phasenmodell & Entwicklungsprozess -->
<!-- §7 = Non-Obvious Decisions (would cause wrong code without this knowledge) -->

<a id="1"></a>
## Verifizierter Codestand · HEAD `6eb0c782145c65989718e86c058d6e859028933d` · Stand 2026-09-20 00:47:35 +0200

> **Für AI-Assistenten:** Diese Datei beschreibt was TATSÄCHLICH implementiert ist,
> nicht was Spezifikationen oder ungeprüfte Dokumente behaupten. Bei Widerspruch gilt:
> Code-Befund & `cargo metadata` > diese Datei > Spezifikationen (§0.1 Quellenhierarchie).
> **Basis-Commit dieser Fassung:** `6eb0c782145c65989718e86c058d6e859028933d`.

---

<a id="2"></a>
## Crate-Topologie (Ring-0–4-Modell, IST + SOLL)

MemFuse wird von der historischen Schichtarchitektur (Layer 0–8) auf das verifizierte **Ring-0–4-Modell** (27 Fach-Crates + 3 Tooling-Crates) umgestellt (`docs/GESAMTSPEZIFIKATION.md` §A2, §4, §20).
Die Mitglieder im Hauptworkspace plus isolierte/exkludierte Members (`memfuse-py`, `xtask`, `memfuse-bench`) bilden folgenden Stand (IST-Status per `cargo metadata` am HEAD und SOLL-Zielphase):

### Ring-Matrix & Crate-Status (30 Crates)

| Crate | Ring | Status | Kurzbeschreibung & Kriterium (P30: I/U/C/S/D) |
|---|---|---|---|
| `memfuse-types` | Ring 0 | 🔜 geplant (Phase 1b) | IDs (`DocId`, `DocIdx`, `TxId`, `TenantId`), Filter-AST, Budgets, Schema-Versionen (C, D) | <!-- doc-ref-ignore -->
| `memfuse-ports` | Ring 0 | 🔜 geplant (Phase 1b) | `dyn`-kompatible Traits: `StorageRead`, `VectorIndex`, `TextIndex`, `GraphIndex`, `Clock`, `Rng` (D) | <!-- doc-ref-ignore -->
| `memfuse-mvcc` | Ring 0 | 🔜 geplant (Phase 1b) | `SeqLog`, `SnapshotRegistry`, `TxBuffer` (loom-getestet) (C, D) | <!-- doc-ref-ignore -->
| `memfuse-wire` | Ring 0 | ✅ vorhanden (Phase 0R) | FlatBuffers-Generat (`memfuse.fbs`) + IPC-Adapter (U — Unsafe-Insel) |
| `memfuse-sys` | Ring 0 | ✅ vorhanden (Phase 0R) | Unsafe-Insel: `ReadOnlyMap` (mmap), Win32-ACL (`crates/memfuse-store/src/wal/io.rs`) (U — Unsafe-Insel) |
| `memfuse-simd` | Ring 0 | ✅ vorhanden (Phase 0R) | Unsafe-Insel: SIMD-Distanzkernel, Laufzeit-Dispatch (AVX2/NEON) (U — Unsafe-Insel) |
| `memfuse-crypto` | Ring 0 | ✅ vorhanden (Package: `memfuse-security`) | Schlüsselhierarchie, AEAD, WAL-HMAC-Kette, `DeletionProof`, Zeroize (C) |
| `memfuse-vector` | Ring 0 | 🔄 Legacy (`memfuse-index`, Phase 1c) | HNSW, SQ8-Quantisierung, DiskANN (S, C) | <!-- doc-ref-ignore -->
| `memfuse-text` | Ring 0 | ✅ vorhanden | BM25/BM25F Volltextsuche, deutsche Morphologie & Komposita (C) |
| `memfuse-graph` | Ring 0 | ✅ vorhanden | CSR-Graph, PPR, Leiden, Hyperkanten (`HyperEdge`, `RoleBinding`) (S, C) |
| `memfuse-rank` | Ring 0 | 🔄 Legacy (`memfuse-calibration` + `memfuse-db::fusion`, Phase 1b) | 4-Signal-Fusion (RRF), Platt/Isotonic-Kalibrierung, Drift (C) | <!-- doc-ref-ignore -->
| `memfuse-adapt` | Ring 0 | ✅ vorhanden (Phase 1b) | LinUCB-Bandit, Lyapunov, PID, Off-Policy; `#[forbid(unsafe_code)]` (C) | <!-- doc-ref-ignore -->
| `memfuse-store` | Ring 1 | ✅ vorhanden | LSM-Tree, WAL (Group Commit, HMAC), MVCC-Pin (S, C) |
| `memfuse-kvcache` | Ring 1 | ✅ vorhanden (Phase 1) | In-Memory LRU-Cache, Eviction-Worker, Tenant-Isolation, Tiering (C) |
| `memfuse-checkpoint` | Ring 1 | ✅ vorhanden | RAII-Checkpoint & Persistent Store Management, time-travel snapshots (C, D) |
| `memfuse-infer-candle` | Ring 2 | 🔄 Legacy (`memfuse-candle`, Phase 1a) | Native GGUF ML Inferenz (Candle), KV-State (I) | <!-- doc-ref-ignore -->
| `memfuse-infer-ollama` | Ring 2 | 🔄 Legacy (`memfuse-ollama`, Phase 1a) | HTTP Ollama Client & Context Prefix Engine (I) | <!-- doc-ref-ignore -->
| `memfuse-infer-onnx` | Ring 2 | 🔄 Legacy (`memfuse-embed`, Phase 1a) | `ort` Cross-Encoder, ONNX Embedding Backend; exkludiert aus `default-members` (I) | <!-- doc-ref-ignore -->
| `memfuse-sandbox` | Ring 2 | ✅ vorhanden | WASM Execution Boundary, Fuel + Wall-Clock Budgets, `ToolSandbox` (I) |
| `memfuse-engine` | Ring 3 | 🔄 Legacy (`memfuse-db` Datenebene, Phase 3b) | Collection, Transaktionen, `RetrievalPlanner`, Ingestion, ComputePool (S, C) | <!-- doc-ref-ignore -->
| `memfuse-cognition` | Ring 3 | 🔄 Legacy (`memfuse-db` Kontrollebene, Phase 3b) | `ConsolidationEngine`, Context Compaction, Scheduler (C) | <!-- doc-ref-ignore -->
| `memfuse-privacy` | Ring 3 | 🔄 Legacy (`memfuse-crypto::egress_vault` + `memfuse-mcp::egress_gateway`, Phase 1a) | Cloud-Egress Gateway, DLP, Surrogat-Tokenisierung, `GuardedPayload` (C) | <!-- doc-ref-ignore -->
| `memfuse-router` | Ring 3 | ✅ vorhanden (Schlankungs-Ziel Phase 1b) | SLM-Profil-Routing, MCP-Dispatch (keine Numerik mehr) (C) |
| `memfuse-agent` | Ring 3 | ✅ vorhanden | Multi-Step Persistent Agent Workflow Loop, Audit, DLQ (C) |
| `memfuse` | Ring 4 | 🔜 geplant (Phase 1a) | Hauptfassade, Builder, einzige Composition Root (D) | <!-- doc-ref-ignore -->
| `memfuse-mcp` | Ring 4 | ✅ vorhanden | Model Context Protocol stdio JSON-RPC 2.0 Server (C) |
| `memfuse-py` | Ring 4 | ✅ vorhanden (Isolierter Workspace) | PyO3 Python-Bindings, FFI catch_unwind (I) |
| `memfuse-testkit` | Tooling | ✅ vorhanden (Phase 0R) | Fault-VFS, `ManualClock`, In-Memory-`StorageEngine` (P28) |
| `memfuse-bench` | Tooling | ✅ vorhanden | Reproduzierbare Benchmark-Harness (`benchmarks/memfuse-bench`) |
| `xtask` | Tooling | ✅ vorhanden (Exkludiert aus Root-`members`) | Custom CI/CD Tasks, Preflight, Drift Gates, Layering Checks |

### Die 3 Unsafe-Inseln (§0.4 & §4.1)

Jeder Nicht-Insel-Crate setzt zwingend `#-[#![forbid(unsafe_code)]` in seiner `lib.rs`. `unsafe` Rust ist exklusiv auf folgende **drei Unsafe-Inseln** beschränkt: <!-- doc-ref-ignore -->
1. `memfuse-simd`: SIMD Hardware-Optimierungen (AVX2, AVX-512, NEON) mit Laufzeit-Dispatch.
2. `memfuse-sys`: Systemprimitiven — `ReadOnlyMap` (mmap), `LockedBuf` (`mlock`/`munlock`), Win32 ACLs (`crates/memfuse-store/src/wal/io.rs`).
3. `memfuse-wire`: Auto-generierter FlatBuffers-Code (`memfuse.fbs`).

*(Befristete Ausnahmeliste bis Phase 1c wird per CI-Prüfung kontrolliert).* <!-- doc-ref-ignore -->

### Ring-Matrix & Abhängigkeitsregeln (§4.3)

```
Ring 0  → Ring 0 (Typen/Ports/Sys/Simd/Wire/Crypto -> Fachkerne Vector/Text/Graph/Rank/Adapt)
          Kerne (vector, text, graph, rank, adapt) kennen einander nicht!
Ring 1  → Ring 0. Kein Ring-1-Crate hängt von einem anderen Ring-1-Crate ab.
Ring 2  → types, ports, crypto. Niemals Ring 1 oder Ring 3.
Ring 3  → Ring 0, Ring 1, Ports von Ring 2 (nie deren konkrete Crates).
Ring 4  → Composition Root / Fassaden.
```

---

<a id="3"></a>
## Was TATSÄCHLICH implementiert ist vs. FEHLT (verifiziert)

### Implementiert ✅

| Komponente / Typ | File:Line Reference | Beschreibung / Anmerkung |
|---|---|---|
| `memfuse-testkit` | `crates/memfuse-testkit/` | Neu in Welle 1 (Phase 0R). Determinismus-Infrastruktur (`ManualClock`, `FaultVfs`, `InMemoryStore`) |
| `memfuse-kvcache` | `crates/memfuse-kvcache/` | Neu in Phase 1 (Cache & Inferenz). KV-Cache LRU-Store, Eviction-Worker, Tenant-Isolation |
| `memfuse-wire` | `crates/memfuse-wire/` | Neu in Welle 1 (Phase 0R). Auto-generated FlatBuffers IPC code (`memfuse.fbs`) |
| `memfuse-sys` | `crates/memfuse-sys/` | Neu in Welle 1 (Phase 0R). Unsafe-Insel für mmap/mlock/Win32-ACLs |
| `memfuse-simd` | `crates/memfuse-simd/` | Neu in Welle 1 (Phase 0R). Unsafe-Insel für SIMD-Distanzkernel |
| `TenantId` | `crates/memfuse-core/src/types/domain.rs` | Mandanten-Identifikator |
| `ConfigFingerprint` | `crates/memfuse-core/src/types/domain.rs` | Invalidation-Fingerprint für Kalibrierung & Profile |
| `DeletionProof` & `LayerCleanupProof` | `crates/memfuse-crypto/src/deletion_proof.rs` | Kryptographischer Löschnachweis mit typsystemischer Absicherung |
| `memfuse-security` | `crates/memfuse-crypto/` | Package-Name `memfuse-security` in Cargo.toml. Encryption-at-Rest & KV-Segment-Security |
| `EdgeProvenance` | `crates/memfuse-graph/src/provenance.rs` | Herkunftsnachweis für Graph-Kanten (`DocEdgeIndex`) |
| `PathRAGEngine` | `crates/memfuse-graph/src/path_rag.rs` | Bidirektionale Graph-Retrieval Search Engine |
| `ConsistencyEnforcer` | `crates/memfuse-graph/src/consistency_enforcement.rs` | Widerspruchserkennung & Edge-Suppression |
| `ConsolidationSession` | `crates/memfuse-db/src/context_compaction.rs` | Context Compaction mit Transaktionssicherheit |
| `AdaptiveDecayController` | `crates/memfuse-db/src/decay_controller.rs` | Thermodynamisches Adaptive-Decay hinter `adaptive-decay` |
| `MarkdownChunker` | `crates/memfuse-db/src/chunker.rs` | Strukturiertes Dokumentsplitting vor Vektor-Embedding |
| `MultiStepEngine` & RRF | `crates/memfuse-db/src/multistep.rs` | Iterative Search Engine mit RRF-Signal-Fusion |
| `scan_bounded` | `crates/memfuse-core/src/traits/storage.rs` | Speicherbeschränkter Range-Scan zur OOM-Vermeidung |
| `CheckpointGuard` | `crates/memfuse-checkpoint/src/lib.rs` | RAII-Checkpoint & Persistent Store Management |
| `CrossEncoderReranker` | `crates/memfuse-embed/src/reranker.rs` | Cross-Encoder Reranking für High-Precision Retrieval |
| `ContextPrefixEngine` | `crates/memfuse-ollama/src/context_prefixer.rs` | Context Prefix Compression Engine |
| `CSRGraph` & PPR | `crates/memfuse-graph/src/csr.rs` | Compressed Sparse Row Graph mit Forward-Push PageRank |
| `PersistentAgentWorkflow` | `crates/memfuse-agent/src/lib.rs` | Multi-Step Agent Execution Loop mit State Graph |
| `BanditRouter` | `crates/memfuse-router/src/bandit.rs` | LinUCB Diagonal & Sherman-Morrison Bandit Routing |
| `check-bandit-latency-budget` | `xtask/src/check_bandit_latency_budget.rs` | Latenz-Budget-Prüfung für LinUCB-Bandit |
| Gate `check-module-reachability` | `xtask/src/check_orphan_modules.rs` | Neu in Welle 1 (Phase 0R). Prüft unerreichbare `.rs`-Dateien |
| Gate `check-dag` | `xtask/src/main.rs` | Neu in Welle 1 (Phase 0R). Layering-Prüfung nach Ring-Modell |
| Gate `check-agents-integrity` | `xtask/src/check_agents_integrity.rs` | Neu in Welle 1 (Phase 0R). Prüft Integrität von AGENTS.md |
| `memfuse-adapt` | `crates/memfuse-adapt/` | Neu in Phase 1b. LinUCB Bandit, Lyapunov Drift, PID Controller, OffPolicy |

### Fehlt / In Arbeit 🔴 / 🔄

| Komponente / Feature | Status | Ziel-Phase / Beschreibung |
|---|---|---|
| `memfuse` Fassade | 🔜 GEPLANT | Phase 1a. Neue Composition Root & Builder | <!-- doc-ref-ignore -->
| `memfuse-types`, `ports`, `mvcc` | 🔜 GEPLANT | Phase 1b. Zerlegung von `memfuse-core` | <!-- doc-ref-ignore -->
| `memfuse-engine`, `cognition`, `rank`, `privacy` | 🔜 GEPLANT | Phase 1b–3b. Zerlegung von `memfuse-db` & `memfuse-router` | <!-- doc-ref-ignore -->
| KV-Cache-Bridge (echter Prefill) | 🔴 SOLL | Phase 4. Derzeitiger Status ist Stub in `memfuse-candle` (§9.2), Ausbau in Stufen A/B/C |
| N-äre Hyperkanten (`relate_n_ary`, `HyperEdge`) | 🔴 SOLL | Phase 1. Bipartite Stern-Expansion (§6) |

---

<a id="4"></a>
## Bekannte offene Risiken

1. **`rebuild_region()` ohne Recall-Tests (F-02, `crates/memfuse-index/src/hnsw.rs`)**:
   `rebuild_region()` führt reines Tombstone-Pruning durch, ohne dass wissenschaftliche Recall-Tests vorliegen. `partial-rebuild-pruning` MUSS deaktiviert bleiben.
2. **KV-Cache-Bridge ist ein Stub (🔴, §9.2)**:
   `generate_with_context` speichert nur Platzhalter-Strings und Zähler (`prefill_skip_count`), ohne echten Inferenz-Prefill einzusparen. Echter Ausbau erfolgt in Stufen A/B/C (Phase 4).
3. **`memfuse-sandbox` Waisen-Status (F2)**:
   `memfuse-sandbox` existiert als funktionierendes WASM-Isolation-Crate, ist jedoch noch von keinem Konsumenten (`memfuse-mcp` / `memfuse-agent`) verdrahtet.
4. **LSM Compaction & MANIFEST TOCTOU / Lock Window (H-1, H-2, §5.6)**:
   TOCTOU-Fenster bei Candidate Selection in `crates/memfuse-store/src/compaction.rs` und Latency-Flaschenhals bei `LsmStorage::commit` unter Write-Lock.
5. **Hyperkanten Cascade-Invalidierungs-Fanout (H-5, §6.6)**:
   Synchrone Kaskaden-Löschung bei großem Fan-Out blockiert den Main-Thread. Geforderte Lösung: Persistente idempotente Background-Queue für $N > \theta$.

---

<a id="5"></a>
## ⚠️ Frischhaltungspflicht dieser Datei

Diese Datei MUSS bei jedem PR und Phasen-Merge aktualisiert werden (§5 Frischhaltungspflicht, `docs/GESAMTSPEZIFIKATION.md` §5):
- HEAD-Commit Hash und Datum gegen `git log -1 --format='%H %ci'` prüfen und aktualisieren.
- Neue Crates oder Statusänderungen von `🔜 geplant` / `🔄 Legacy` zu `✅ vorhanden` eintragen.
- Bei Umsetzung bisher fehlender Komponenten die Tabellen in §2 und §3 anpassen.

**Prüfpflicht vor jedem Commit-Merge:** Diff dieser Datei gegen `git log --oneline -20` prüfen — wurde etwas implementiert, das hier noch als "Fehlt" oder "Geplant" steht?

Eine veraltete AGENTS.md ist schlimmer als keine — sie führt Agenten aktiv in die Irre.

---

<a id="6"></a>
## Phasenmodell & Entwicklungsprozess (§20.2)

Die Migration auf das Ring-Modell folgt dem verbindlichen Phasenplan aus `docs/GESAMTSPEZIFIKATION.md` §20.2:

- **Phase 0R (Welle 1 — UMGESETZT):** `memfuse-testkit` angelegt; `xtask` aus Root-Workspace exkludiert; Unwrap-Ratchet durch `clippy-panic-lints` ersetzt; Lints workspace-weit aktiviert; neue Gates `check-orphan-modules`, `check-dag`, `check-agents-integrity`; `deny.toml` mit Ring-Bans; `loom-tests` als eigener Job.
- **Phase 1a (Aufwärtskanten auflösen):** Fassade `memfuse` anlegen; `memfuse-db` entkoppeln (`Arc<dyn Embedder>`); Layering-Test scharf für Ring 3. <!-- doc-ref-ignore -->
- **Phase 1b (`core`-Zerlegung & `router`-Schlankung):** `memfuse-core` → `types`/`ports`/`mvcc`; `memfuse-router` → `adapt` (Numerik) & `router` (Routing/MCP). <!-- doc-ref-ignore -->
- **Phase 1c (Unsafe-Inseln erzwingen):** `memfuse-sys`, `memfuse-simd`, `memfuse-wire` vollständig isolieren; `#![forbid(unsafe_code)]` in allen anderen Crates.
- **Phase 2 (Sync-Kerne):** Synchronen Kern von I/O trennen (`StorageRead` sync / `StorageWrite` async); `tokio` aus Ring 0 verbannen.
- **Phase 3a (Konsistenz-Spike):** Crash-Injektion via `memfuse-testkit` Fault-VFS; WAL-Recovery validieren.
- **Phase 3b (`db`-Zerlegung):** `memfuse-db` in `engine`, `cognition`, `rank`, `adapt`, `router`, `privacy` zerlegen; `memfuse` Fassade als primäre API etablieren. <!-- doc-ref-ignore -->
- **Phase 4 (KV-Cache echtschalten):** Stufen A (RAM-Prefix) → B (eigenes Llama-Modell mit `KvState`) → C (Segment-Spill).
- **Phase 5 (Hygiene & Doku-Generierung):** God-Files zerlegen; Reifegrad-Marker aus Capability-Manifest generieren. <!-- doc-ref-ignore -->

---

<a id="7"></a>
## Non-Obvious Decisions (would cause wrong code without this knowledge)

- **P26 Sync-Kern, async-Schale:** Ring 0 (`memfuse-types` … `memfuse-adapt`) enthält kein `tokio`. Kerne sind synchron; I/O und Async liegen in Ring 1+ (`memfuse-store`, `memfuse-engine`). <!-- doc-ref-ignore -->
- **P27 `dyn`-kompatible Ports:** Traits in `memfuse-ports` are synchron where mmap/pread blocks, or return `BoxFuture` instead of AFIT, to allow runtime interchangeability via `Arc<dyn Trait>`. <!-- doc-ref-ignore -->
- **P28 Injizierter Nichtdeterminismus:** `Clock`, `Rng`, `IdGen` sind injizierte Ports. Kein direkter Aufruf von `SystemTime::now()` oder `rand::thread_rng()` in Kern- oder Persistenzcode. `memfuse-testkit` stellt `ManualClock` bereit.
- **P29 Kein globaler veränderlicher Zustand:** Keine mutable `static` Variablenspeicher oder globale `OnceLock`-Registries. Zustand gehört Instanzen.
- **P30 Crate-Zuschnitt-Kriterien (I/U/C/S/D):** Jeder Crate-Zuschnitt begründet sich durch Isolation (I), Unsafe-Insel (U), Bounded Context (C), Größe > 8k LOC (S) oder Richtungserzwingung (D).
- **Strangler-Regel:** Legacy-Crates (`memfuse-core`, `memfuse-db`, `memfuse-candle`, etc.) re-exportieren neue Typen mit `#[deprecated]` während der Migrationsphasen, um schrittweisen Umbau ohne Big-Bang-Breakage zu garantieren.
- **ADR-Verweise:** Nur real existierende ADRs in `docs/decisions/` zitieren (`ADR-027-community-detection.md`, `ADR-082-docid-width.md`, `ADR-083-hnsw-diskann-stufenmodell.md`, `ADR-084-flatbuffers-drift-gate.md`, `ADR-0XX-memfuse-sandbox.md`). Die geplanten Refactoring-ADRs N01–N10 sind ausdrücklich als `🔜 geplant` zu kennzeichnen.
- **TxId generation**: ALWAYS `collection.allocate_tx()` — NEVER `SystemTime::as_nanos()`
- **fsync errors**: ALWAYS propagate with `?` — NEVER `let _ = dir.sync_all()`
- **Document chunking**: ALWAYS use `MarkdownChunker` — NEVER embed entire text as 1 vector
- **MCP transport**: stdio JSON-RPC 2.0 ONLY — axum was removed (ADR-010)
- **WAL HMAC key**: ALWAYS via `load_or_create_integrity_key()` — NEVER hardcoded
- **TOMBSTONE_BIT-Disziplin (ADR-041)**: Bit 63 strikt maskieren (`seq & !TOMBSTONE_BIT`) vor `max_seq` Vergleichen.
- **SSTable Flush-Sichtbarkeit (ADR-043)**: `last_committed_tx` vor `sstables.push()` in `LsmStorage::flush` aktualisieren.
- **MCP Write-Authorization & Sandbox Policy (ADR-044)**: DB-Schreibzugriffe im MCP Server sind standardmäßig GESPERRT (Read-Only Policy).

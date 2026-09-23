---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "00"
---
## 0. Meta: Workspace-Layout und Build-Konfiguration

### 0.1 Verzeichnisstruktur

**Jetzt (verbindlich ab dieser Fassung, Ring-Modell — Zielzustand, wird strangler-artig gemäß §20 erreicht):**

```
contextra/
├── Cargo.toml                      # [workspace], resolver = "2", default-members ohne infer-onnx
├── xtask/                          # CI-Tooling (Drift-Gates, Layering, Benchmarks; Ziel < 3k LOC)
│   └── src/
│       ├── main.rs
│       ├── check_flatbuffers_drift.rs
│       └── check_bandit_latency_budget.rs
├── schemas/
│   └── contextra.fbs                 # §12
├── crates/
│   ├── contextra-types/              # Ring 0 — IDs, TxId, TenantId, Fingerprint, Filter-AST, Budgets
│   ├── contextra-ports/              # Ring 0 — Traits: StorageRead/Write, VectorIndex, TextIndex, GraphIndex, Embedder, Clock, Rng, IdGen, MetricsSink
│   ├── contextra-mvcc/               # Ring 0 — SeqLog, SnapshotRegistry, TxBuffer (loom-getestet)
│   ├── contextra-wire/               # Ring 0 — FlatBuffers-Generat + Adapter (vormals core-ipc-gen), Unsafe-Insel
│   ├── contextra-sys/                # Ring 0 — mmap, mlock, Win32-ACL, Unsafe-Insel (neu, §A2 D1)
│   ├── contextra-simd/               # Ring 0 — Distanzkernel, Laufzeit-Dispatch, Unsafe-Insel
│   ├── contextra-crypto/             # Ring 0 — Schlüsselhierarchie, AEAD, WAL-HMAC-Kette, Deletion-Proof, Zeroize
│   ├── contextra-vector/             # Ring 0 — HNSW, DiskANN, Quantisierung (vormals contextra-index)
│   ├── contextra-text/               # Ring 0 — BM25/BM25F, deutsche Morphologie
│   ├── contextra-graph/              # Ring 0 — CSR, PPR, Leiden, Hyperkanten
│   ├── contextra-rank/               # Ring 0 — 4-Signal-Fusion, Isotonic/Platt-Kalibrierung, Drift
│   ├── contextra-adapt/              # Ring 0 — Bandit, Lyapunov, PID, Homeostat, Decay (Clock/Rng injiziert)
│   ├── contextra-store/              # Ring 1 — LSM-Tree, WAL (Group-Commit, HMAC), MVCC-Pin
│   ├── contextra-kvcache/            # Ring 1 — Prefix-Radix-Baum, KV-Blöcke, Tiering, AEAD, Segmentdateien
│   ├── contextra-checkpoint/         # Ring 1 — Time-Travel-Registry gegen Port StorageEngine, ohne Global-State
│   ├── contextra-infer-candle/       # Ring 2 — GGUF, eigenes Llama-Modell mit KvState (Stufe B)
│   ├── contextra-infer-ollama/       # Ring 2 — HTTP-Backend, Contextual-Chunk-Prefixing
│   ├── contextra-infer-onnx/         # Ring 2 — ort, Cross-Encoder; NICHT in default-members
│   ├── contextra-sandbox/            # Ring 2 — WASM-Isolation, Fuel + Wall-Clock; implementiert ToolSandbox
│   ├── contextra-engine/             # Ring 3 — Collection, Transaktionen, RetrievalPlanner, Ingestion, ComputePool
│   ├── contextra-cognition/          # Ring 3 — Consolidation, Synthese, Kompaktierung, Scheduler
│   ├── contextra-privacy/            # Ring 3 — Egress-Gateway, PII-Vault, DLP, GuardedPayload
│   ├── contextra-router/             # Ring 3 — SLM-Profil-Routing, MCP-Dispatch (schlank, keine Numerik mehr)
│   ├── contextra-agent/              # Ring 3 — Workflow-Engine, Audit, DLQ
│   ├── contextra/                    # Ring 4 — Fassade, Builder, einzige Composition Root
│   ├── contextra-mcp/                # Ring 4 — stdio-JSON-RPC, Protokoll, Tool-Wiring
│   ├── contextra-py/                 # Ring 4 — PyO3, eigene Runtime, catch_unwind
│   ├── contextra-testkit/            # Tooling — Fault-VFS, ManualClock, In-Memory-StorageEngine
│   └── contextra-bench/              # Tooling — Benchmark-Harness
├── benchmarks/
│   └── contextra-bench/
├── .github/workflows/
│   └── merge-gate.yml              # §15.4
└── docs/decisions/                 # ADR-0NN-*.md, siehe §20.3 für N01–N10
```

**Vorher (Fassung bis zur Vorgängerversion dieser Spec, Layer-0–5-Modell — nicht mehr normativ, siehe §4 „Vorher"):**

```
contextra/
├── crates/
│   ├── contextra-core-ipc-gen/       # Layer 0 — FlatBuffers-generierter Code
│   ├── contextra-core/               # Layer 0 — Kerntypen, Traits, Fehlerbehandlung
│   ├── contextra-store/              # Layer 1 — LSM-Tree, WAL, Block-Cache
│   ├── contextra-crypto/             # Layer 1 — AES-256-GCM-SIV, DeletionProof, KV-Segment-Security
│   ├── contextra-text/               # Layer 1 — BM25/BM25F-Volltextindex, deutsche Morphologie
│   ├── contextra-index/              # Layer 1 — HNSW/DiskANN-Vektorindex, SIMD-Distanz
│   ├── contextra-graph/              # Layer 1 — CSR-Graph, PPR, Leiden, Hyperkanten
│   ├── contextra-checkpoint/         # Layer 1 — Snapshotting
│   ├── contextra-calibration/        # Layer 1 — Score-Kalibrierung, Drift-Erkennung
│   ├── contextra-db/                 # Layer 2 — Collection-API, 4-Signal-Fusion, Provenance
│   ├── contextra-router/             # Layer 3 — Contextual-Bandit-Routing
│   ├── contextra-candle/             # Layer 3 — Natives GGUF-Inferenz-Backend, KV-Cache-Bridge
│   ├── contextra-ollama/             # Layer 3 — Ollama-Client, Contextual-Chunk-Prefixing
│   ├── contextra-embed/              # Layer 3 — ONNX-Embeddings, Cross-Encoder (optional)
│   ├── contextra-agent/              # Layer 3 — Persistente Agent-Workflow-Engine
│   ├── contextra-py/                 # Layer 3 — Python-FFI via PyO3 (eigener Workspace, war real nur Member)
│   ├── contextra-sandbox/            # Layer 6.5 — WASM Execution Boundary
│   ├── contextra-mcp/                # Layer 4 — MCP-Server, Egress-Gateway
│   └── contextra-bench/              # Layer 5 — Benchmark-Harness
```

Grund der Ablösung, Zuordnung alt→neu je Crate und Migrationsreihenfolge: §A2.2, §4, §20.

### 0.2 Root-`Cargo.toml` (normativ)

**Jetzt:** Werte werden **aus dem realen Manifest generiert** (P12/§A2.3), nicht mehr handgepflegt — Grund:
die Vorfassung enthielt vier unbelegte Werte (`rust-version`, `license`, `flatbuffers`, `thiserror`), die am
Code widerlegt sind (§A2.1, D8-Nachbarbefund). Bis der Generator (§20, Phase 5) steht, gelten die verifizierten
Ist-Werte als normativ, ergänzt um die neu beschlossenen Workspace-Lints und das `release-abort`-Profil:

```toml
[workspace]
resolver = "2"
members = ["crates/*"]
default-members = [ "crates/*" ]     # ohne contextra-infer-onnx / --features onnx-bench (§A2.1 D8)
exclude = ["xtask"]

[workspace.package]
edition = "2021"
rust-version = "1.89"                # vorher fälschlich als 1.79 spezifiziert
license = "MIT OR Apache-2.0"        # vorher fälschlich als "Apache-2.0" spezifiziert

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
flatbuffers = "24.3"                 # vorher fälschlich als "23" spezifiziert
crossbeam-epoch = "0.9"
arc-swap = "1"
ahash = "0.8"
scc = "2"
quick_cache = "0.5"
zerocopy = "0.7"
thiserror = "2"                      # vorher fälschlich als "1" spezifiziert
tokio = { version = "1", features = ["rt-multi-thread", "sync", "time", "macros"] }
aes-gcm-siv = "0.11"
blake3 = "1"
wasmtime = "25"

# Neu ab dieser Fassung (§A2, §4.2 der Zielarchitektur-Quelle):
[workspace.lints.rust]
unsafe_code = "deny"                 # bewusst "deny", nicht "forbid" — s. Begründung unten
unsafe_op_in_unsafe_fn = "deny"

[workspace.lints.clippy]
undocumented_unsafe_blocks = "deny"
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"

[profile.release]
panic = "unwind"                     # Root bleibt unwind — vorher "abort" (widersprach contextra-py, §A2.1)

[profile.release-abort]
inherits = "release"
panic = "abort"                      # nur für Binaries ohne FFI, per Paket ausgewählt
```

**Warum `deny` statt `forbid` für `unsafe_code` (verbindliche Korrektur, §A2.1 D2):** `forbid` ist in Rust
nicht lokal überschreibbar (Compiler-Fehler E0453); ein Versuch, `#![forbid(unsafe_code)]` workspace-weit zu
setzen und in den drei Unsafe-Inseln (§0.4) per `#[allow(unsafe_code)]` zu durchbrechen, baut nicht. Stattdessen
gilt: Workspace-Lint ist `deny`; jeder Nicht-Insel-Crate setzt zusätzlich in seiner `lib.rs` explizit
`#![forbid(unsafe_code)]` (das ist zulässig, weil die Workspace-Stufe nur `deny` ist); jede Insel setzt
`#![allow(unsafe_code)]`. Ein Inventar-Test erzwingt, dass kein weiterer Crate `allow(unsafe_code)` trägt
(§0.4, `tests/unsafe_islands.rs`).

**Vorher (nicht mehr normativ):**

```toml
[workspace]
resolver = "2"
members = ["crates/*", "xtask"]

[workspace.package]
edition = "2021"
rust-version = "1.79"
license = "Apache-2.0"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
flatbuffers = "23"
thiserror = "1"
wasmtime = "23"
# ... (gekürzt, siehe Git-Historie dieses Dokuments)
```

`xtask` ist ab dieser Fassung kein Workspace-Member mehr (`exclude`), sondern ein eigenständiges Cargo-Projekt,
damit CI-Tooling-Abhängigkeiten nicht in `cargo tree --workspace` erscheinen (Grundlage für den
`cargo tree`-basierten Netzfreiheits- und Bans-Test, §20 Phase 0R).

### 0.3 Cargo-Feature-Katalog (crateübergreifend normativ)

**Politik ab dieser Fassung (P5/§A2.3):** Additive Features nur noch für **schwere optionale Abhängigkeiten**
(onnx, cuda, metal, wasmtime). **Keine typverändernden Features mehr** — `docid-128` entfällt ersatzlos
(ADR-N05, §20.3); Verhalten wird über Laufzeitkonfiguration statt Compile-Time-Flags gesteuert, wo immer das
ohne Typenwechsel möglich ist. Der Katalog wird perspektivisch aus den Crate-Manifesten generiert (P12); bis
dahin gilt die folgende, an den Ring-Zuschnitt angepasste Tabelle als normativ:

| Feature | Definierender Crate (jetzt) | Definierender Crate (vorher) | Default | Wirkung |
|---|---|---|---|---|
| ~~`docid-128`~~ | — (entfällt) | `contextra-core` | — | **Entfällt** (ADR-N05): typverändernde Features sind verboten; externes 128-Bit-`DocId` mit internem dichten `DocIdx(u32)` ist eine offene Entscheidung (§A2.4 Nr. 3), keine Compile-Time-Option |
| `block-cache-v2` | `contextra-store` | `contextra-store` | aus | `QuickCacheBlockCacheBackend` statt `LruBlockCacheBackend` (§5.4) |
| `egress-sherman-morrison` | `contextra-adapt` | `contextra-router` | aus | `ShermanMorrisonBandit` statt `DiagonalApproximation` (§8.2) |
| `experimental-diskann` | `contextra-vector` | `contextra-index` | aus | `DiskAnnIndex` über `VectorIndexTier::DiskAnn` wählbar (§7.5) |
| `bandit-routing` | `contextra-adapt` | `contextra-router` | an | Aktiviert den Bandit-Router überhaupt |
| `cloud-egress-guard` | `contextra-privacy` | `contextra-mcp` | an | Aktiviert `egress_gateway`-Modul (im Manifest der Vorfassung real nicht vorhanden, wird mit dem Umzug nach `privacy` nachgezogen) |
| `wasm-sandbox` | `contextra-sandbox` | `contextra-mcp` | an | Aktiviert die Sandbox; `contextra-sandbox` ist bis zur Anbindung an einen Konsumenten aus `default-members` ausgeschlossen (§A2.2, Waisen-Crate) |
| `kv-bridge` | `contextra-kvcache` | `contextra-candle` | **aus** | Aktiviert Stufe B/C (§9.2); Stufe A ist kein Feature, sondern Default-Verhalten sobald `contextra-infer-candle` aktiv ist. War in der Vorfassung fälschlich als „an" mit 🟢-Status dokumentiert, obwohl der Code ein Stub war (§A2.1) |
| `edge-reinforcement-learning` | `contextra-graph` | `contextra-graph` | aus | Aktiviert `SignalKind::EdgeReinforcement`-Pfad |
| `fault-injection` | `contextra-testkit` | `contextra-store` | nur `dev-dependencies` | Deterministische I/O-Fehlerinjektion; wandert in den neuen Tooling-Crate `contextra-testkit` |
| `loom` | `contextra-mvcc`, `contextra-graph` | `contextra-store`, `contextra-graph` | nur `dev-dependencies` | `loom::sync::*` statt `std::sync::*` hinter `#[cfg(loom)]` |
| `bm25f` | `contextra-text` | `contextra-text` | an | Feldgewichtete BM25-Bewertung |
| `adaptive-decay` / `-control` | `contextra-cognition` | `contextra-db` | an | Kalibrierungs-Feintuning |
| `partial-index-rebuild` | `contextra-vector` | `contextra-index` | an | Inkrementeller Indexaufbau |
| `onnx-bench` | `contextra-bench` | — (v1 erzwang `onnx` hart) | aus | Einzige Stelle, die `contextra-infer-onnx`/`ort` in einen Benchmark-Build zieht; bereits umgesetzt (§A2.1 D8) |

### 0.4 Globale Compile-Time-Regeln

**Jetzt (verbindlich, §A2.1 D1/D2 — löst den Mechanismus der Vorfassung vollständig ab):** Es gibt **genau
drei** Unsafe-Inseln, nicht sechs. Jeder Nicht-Insel-Crate erhält in `lib.rs`:

```rust
#![forbid(unsafe_code)]
```

Das ist zulässig, weil die Workspace-Lint-Stufe (§0.2) nur `deny` ist, nicht `forbid` — ein pauschaler
Workspace-`forbid` mit lokalem `allow` in den Inseln, wie er in der Vorfassung beschrieben war, kompiliert
wegen E0453 nicht und wurde deshalb verworfen.

| Insel (jetzt) | Begründung | Herkunft / Vorher |
|---|---|---|
| `contextra-simd` | Distanzkernel mit Laufzeit-Dispatch (AVX2, AVX-512, NEON), Längenprüfung im safe Wrapper, `OnceLock<fn>`-Dispatch, Proptest gegen skalares Orakel je Stufe, Miri nur für den skalaren Pfad | vorher: `contextra-index` (SIMD **und** mmap in einem Topf) |
| `contextra-sys` (**neu**, §A2.1 D1) | `ReadOnlyMap` (mmap), `LockedBuf` (`mlock`/`munlock`/`VirtualLock`), Owner-only-ACL (Win32); Verträge safe gekapselt (`Deref<Target=[u8]>` für Mmap; Zeroize-vor-`munlock`-Drop-Guard für `LockedBuf`) | vorher verteilt und **unvollständig erfasst** in `contextra-store` (Win32-ACL), `contextra-index` (mmap), `contextra-db` (`mlock`, `volatile_vault`) — die Vorfassung nannte hierfür fälschlich `contextra-store`, `contextra-db` und `contextra-index` als *drei separate* Ausnahmen statt einer gemeinsamen Insel |
| `contextra-wire` | FlatBuffers-Generat: `#![allow(unsafe_code, clippy::unwrap_used)]` auf Crate-Ebene, weil flatc-generierter Code `unwrap` für Felder mit Default erzeugt; bestehendes Drift-Gate `check_flatbuffers_drift` bleibt | vorher: `contextra-core-ipc-gen` |

**Entfallen als Unsafe-Ausnahme (0 `unsafe` verifiziert, §A2.1 D1):** `contextra-router`/`contextra-adapt`
(Sherman-Morrison-Arithmetik ist in Safe Rust implementiert, keine Intrinsics) und `contextra-embed`/
`contextra-infer-onnx` (die C-FFI-Grenze zu `ort` liegt außerhalb des Crates in der `ort`-Bibliothek selbst).
`contextra-db` entfällt als Ausnahme-Crate, weil `mlock` in die neue Insel `contextra-sys` wandert.

**Erzwingung:** `tests/unsafe_islands.rs` (Workspace-Root) prüft (a) das Schlüsselwort `unsafe` kommt nur in
`contextra-sys`, `contextra-simd`, `contextra-wire` vor, (b) jede Nicht-Insel-`lib.rs` enthält `forbid(unsafe_code)`,
(c) `allow(unsafe_code)` steht nur in den drei Inseln. Bis Migrationsphase 1c (§20) gilt eine explizit
benannte Übergangsliste mit Datei:Zeile-Angaben für `contextra-vector`, `contextra-store`, `contextra-db` (Feature
`volatile-vault`) und `contextra-wire` — diese Ausnahme ist befristet, nicht dauerhaft.

Jeder Crate erhält zusätzlich workspace-weit (§0.2, `[workspace.lints.clippy]`):

```rust
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::todo, clippy::unimplemented)]
```

**Vorher:** eine versionierte Ausnahmeliste `.unwrap-baseline.json` im Crate-Root plus CI-Ratchet
(`check-unwrap-baseline`), der nur schrumpfen durfte. **Jetzt:** Der Ratchet entfällt ersatzlos (§20 Phase
0R: `xtask/{unwrap_ratchet, check_unwrap_ratchet, check_unwrap_baseline_trend}.rs` werden gelöscht); die
verifizierte Zahl produktiver `unwrap`/`expect`/`panic`-Stellen liegt bei ca. 11 (Store 1, DB/Engine 1,
Crypto 3, Candle/Infer 1, Core/Types 1, Text 1, Py 3), diese werden im selben Umbau-PR behoben, in dem der
Lint scharf geschaltet wird — kein Ratchet, kein Übergangszustand mit offener Ausnahmeliste. `xtask` selbst
erhält befristet ein `#![allow]`, bis es in Phase 5 unter 3.000 Zeilen reduziert ist.

**`indexing_slicing` (neu, §A2.1 D6):** entgegen einem ursprünglich erwogenen workspace-weiten `deny` (das an
≈ 640 Hot-Path-Ausdrücken gescheitert wäre) gilt `deny` nur an klar benannten Parsing-/Decode-Grenzen
(`store/wal/{replay,encode}`, `store/sstable`-Decode, `contextra-wire`, `contextra-mcp/protocol`,
`contextra-text/tokenizer`); in den rechenintensiven Kernen gilt `warn` plus `debug_assert!` am
Schleifeneintritt, `get_unchecked` ist ausschließlich in `contextra-simd` zulässig.

### 0.5 Non-Obvious Decisions (systemweit bindend)

- **TxId-Generation:** IMMER `collection.allocate_tx()` — NIEMALS `SystemTime::as_nanos()`
- **fsync-Fehler:** IMMER mit `?` propagieren — NIEMALS `let _ = dir.sync_all()`
- **Dokument-Chunking:** IMMER `MarkdownChunker` — NIEMALS gesamten Text als 1 Vektor embedden
- **MCP-Transport:** stdio JSON-RPC 2.0 ONLY — axum wurde entfernt (ADR-010)
- **WAL-HMAC-Key:** IMMER via `load_or_create_integrity_key()` — NIEMALS hardcoded
- **TOMBSTONE_BIT-Disziplin (ADR-041):** Bit 63 strikt maskieren (`seq & !TOMBSTONE_BIT`) vor `max_seq`-Vergleichen
- **SSTable-Flush-Sichtbarkeit (ADR-043):** `last_committed_tx` vor `sstables.push()` aktualisieren

---

<a id="1-kernthese"></a>

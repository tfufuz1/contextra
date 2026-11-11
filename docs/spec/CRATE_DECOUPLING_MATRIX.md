# Contextra Baukasten: Crate-Entkopplungs- und Entscheidungsmatrix

> **Dokument-Status:** Normativ & Verbindlich  
> **Rolle:** Principal Rust Systems Engineer  
> **Gültigkeit:** Workspace `tfufuz1/contextra`

---

## 1. Systemübersicht des Baukastens (E0 bis E3)

Diese Matrix definiert für jedes Crate die architektonische Ebene, die erlaubte Kern-Closure, die Kapselung von Schnittstellen und Fehlern, sowie die Port- und Feature-Strategie.

| Crate | Ebene | Kern-Closure (Erlaubt) | Eigener Fehlertyp | Port-Abstraktion (Kern) | Feature `workspace` Brücke | Status / Zielzustand |
|---|---|---|---|---|---|---|
| `contextra-ids` | **E0** | $\emptyset$ | `IdError` | Keine | Nicht nötig | Autarker Leaf |
| `contextra-env` | **E0** | $\emptyset$ | Minimal | `Clock`, `Rng`, `IdGen`, `MetricsSink` | Nicht nötig | Autarker Leaf |
| `contextra-sys` | **E0** | $\emptyset$ | `SysError` | OS-Primitives (`mlock`, `DurableFile`) | Nicht nötig | Unsafe-Insel (POSIX) |
| `contextra-durable-fs` | **E0** | $\emptyset$ | `DfsError` | Block-I/O Traits | Nicht nötig | Autarker Leaf |
| `contextra-simd` | **E0** | $\emptyset$ | `SimdError` | `DistanceFn`, `MetricDistanceFn` | `compat/types.rs` | Unsafe-Insel (SIMD) |
| `contextra-wire` | **E0** | $\emptyset$ | `WireError` | Framed Codec Traits | Nicht nötig | Unsafe-Insel (IPC) |
| `contextra-adapt` | **E0** | $\emptyset$ | `AdaptError` | Lokale Bandit- & Regler-Traits | `compat/ports.rs` | Autarker Baustein |
| `contextra-privacy` | **E0** | $\emptyset$ | `PrivacyError` | DLP- & Vault-Schnittstellen | Nicht nötig | Autarker Baustein |
| `contextra-sandbox` | **E0** | $\emptyset$ | `SandboxError` | Plugin-Execution Interface | Nicht nötig | Autarker Baustein |
| `contextra-prompt-guard` | **E0** | $\emptyset$ | `GuardError` | Prompt-Prüfungs-Traits | Nicht nötig | Autarker Baustein |
| `contextra-erasure` | **E0** | $\emptyset$ | `ErasureError` | Deletion-Proof, Crypto-Shred | `compat/ports.rs` | Autarker Baustein (Compliance) |
| `contextra-license` | **E0** | $\emptyset$ | `LicenseError` | `LicenseValidator` | `compat/` | Autarker Baustein |
| `contextra-avv-generator` | **E0** | $\emptyset$ | `AvvError` | Dokumenten-Template Traits | Nicht nötig | Autarker Baustein |
| `contextra-audit-export` | **E0** | $\emptyset$ | `ExportError` | Export-Format-Writer | `compat/` | Autarker Baustein |
| `contextra-ingest` | **E0** | $\emptyset$ | `IngestError` | Parser- & Chunking-Traits | `compat/` | Autarker Baustein |
| `contextra-mvcc` | **E1a** | `ids`, `env` | `MvccError` | `SequenceLog`, `TxBuffer` | `compat/` | Algorithmus-Kern |
| `contextra-crypto` | **E1a** | `ids`, `env` | `CryptoError` | KDF-, AEAD- und HMAC-Primitives | `compat/` | Algorithmus-Kern |
| `contextra-text` | **E1a** | `ids`, `env` | `TextError` | Lokales `PostingStore`-Trait | `compat/` | Algorithmus-Kern |
| `contextra-graph` | **E1a** | `ids`, `env` | `GraphError` | CSR-Graph, PPR-Traits | `compat/` | Algorithmus-Kern |
| `contextra-rank` | **E1a** | `ids`, `env` | `RankError` | Generische Signal-Fusion & Calibrator | `compat/` | Algorithmus-Kern |
| `contextra-checkpoint` | **E1a** | `ids`, `env` | `CheckpointError`| Snapshot-Manager | `compat/` | Algorithmus-Kern |
| `contextra-kvcache` | **E1a** | `env` | `KvCacheError` | Prefix-Radix Cache | `compat/` | Algorithmus-Kern |
| `contextra-vault` | **E1a** | `sys` | `VaultError` | RAM-Locking Memory-Buffer | `compat/` | Algorithmus-Kern |
| `contextra-infer-candle` | **E1a** | `env` | `CandleError` | Generische Embedder-/Generator-Traits | `compat/` | Inferenz-Kern (Local) |
| `contextra-infer-ollama` | **E1a** | `env` | `OllamaError` | REST Client & Circuit Breaker | `compat/` | Inferenz-Kern (Remote) |
| `contextra-infer-onnx` | **E1a** | `env` | `OnnxError` | ONNX Runtime Wrapper | `compat/` | Inferenz-Kern (Local) |
| `contextra-store` | **E1b** | `ids`, `env`, `sys`, `mvcc`, `crypto` | `StoreError` | LSM-Tree, MemTable, SSTable, WAL | `compat/` | State-Kern |
| `contextra-vector` | **E1b** | `ids`, `env`, `mvcc` (+ `simd`) | `VectorError` | Lokales `Index`-Trait | `compat/` | Index-Kern |
| `contextra-types` | **E2** | `ids` | `ContextraError` | DTOs, Filter, Schemata | Core-Integrator | Integration / Types |
| `contextra-ports` | **E2** | `types`, `ids`, `env` | — | Zentrale Integrations-Traits | Core-Integrator | Integration / Ports |
| `contextra-router` | **E2** | `types`, `ports`, `adapt` | `RouterError` | Cascade & Bandit Router | Integration | Integration / Routing |
| `contextra-agent` | **E2** | `types`, `ports` | `AgentError` | StateGraph FSM & Execution Loop | Integration | Integration / Agent |
| `contextra-cognition` | **E2** | `types`, `ports` | `CognitionError` | Memory Consolidation & Compaction | Integration | Integration / Kognition |
| `contextra-engine` | **E2** | E0 + E1 (max 9 direkt) | `ContextraError` | Hybrid-Retrieval Orchestrator | Core-Engine | Integration / Engine |
| `contextra-db` | **E2** | `engine`, `types`, `ports`, `vault` | — | Collection- und Datenbank-API | Core-DB | Integration / DB |
| `contextra` | **E2** | `db`, `types`, `ports` | — | High-Level Fassade | Top-Level | Composition Root |
| `contextra-mcp` | **E3** | E0 + E1 + E2 | `McpError` | JSON-RPC 2.0 / STDIO Protocol | Server-Applikation | Applikation |
| `contextra-py` | **E3** | `contextra`, `types`, `wire` | PyO3-Errors | Python Bindings | Client-Library | Applikation |
| `contextra-testkit` | **E3** | `ports`, `types`, `env` | — | In-Memory Fakes & Test-Harness | Test-Harnisch | Dev-Tooling |
| `contextra-bench` | **E3** | `store`, `vector`, `text`, `db` | — | Criterion Performance & Recall Benchmarks| Benchmark-Suite | Dev-Tooling |

---

## 2. Brücken- und Adapterarchitektur (`src/compat/`)

Um die Autarkie der E0- und E1-Bausteine zu garantieren und gleichzeitig die nahtlose Zusammensetzbarkeit im Gesamtsystem zu ermöglichen, gilt folgendes standardisiertes Brückenmuster:

```
crates/<baustein>/
├── Cargo.toml          -> default = [], optional = ["contextra-types", "contextra-ports"]
├── src/
│   ├── lib.rs          -> Re-Exportiert den autarken Kern
│   ├── contract.rs     -> Lokale Micro-Traits und domänenspezifische Datenstrukturen
│   ├── error.rs        -> #[derive(thiserror::Error)] pub enum <Baustein>Error
│   ├── core_logic.rs   -> Reiner Algorithmus ohne externe Ökosystem-Kopplung
│   └── compat/         -> #[cfg(feature = "workspace")]
│       ├── mod.rs
│       ├── error.rs    -> impl From<<Baustein>Error> for ContextraError
│       └── ports.rs    -> impl contextra_ports::<PortTrait> for <BausteinStruct>
```

Zukünftige Entwickler können diesen Crate nun in ein beliebiges Drittprojekt importieren:
* **Drittprojekt:** Nutzt `contextra-<baustein> = { version = "...", default-features = false }`. Keine Kopplung an `contextra-types`, `ports`, `engine` oder andere interne Module.
* **Contextra-Ökosystem:** Aktiviert `features = ["workspace"]`. Die Brücken-Implementierungen für `contextra-ports` und `ContextraError` stehen unmittelbar zur Verfügung.

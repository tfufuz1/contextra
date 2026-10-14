# AGENTS.md — contextra-ports
> Ring 1 · stable · Quelle: capabilities.toml · Spec: K.19, §A2, §0, §3, §4, §9, §20, EP1–EP6

## 1. Zweck
`contextra-ports` definiert die rein abstrakten, `dyn`-kompatiblen Port-Schnittstellen (`StorageEngine`, `VectorIndex`, `TextIndex`, `GraphIndex`, `ErasureBackend`, `Clock`, `Rng`, `IdGenerator` u. a.) zur Entkopplung aller Subsysteme.
Er erzwingt `#![forbid(unsafe_code)]` und dient als zentraler Ring-1-Vertrag ohne konkrete Backend-Implementierungen.

## 2. Modul-Karte
| Datei / Verzeichnis | Verantwortung |
| :--- | :--- |
| `src/lib.rs` | Re-Exporte aller Port-Traits (`#![forbid(unsafe_code)]`). |
| `src/erasure.rs` | `ErasureBackend`, `LogicalScope`, `PhysicalScope`, `PurgeOutput`, `ErasureMode` (EP1 / EP2). |
| `src/attention.rs` | `AttentionExporter` Trait für 1D-Prefill-Attention-Gewichte (KV-Cache-Eviktion, EP4). |
| `src/checkpoint.rs` | `CheckpointStore` Trait für transaktionale Snapshot-Integrität. |
| `src/clock.rs` | `Clock` Trait für injizierten Determinismus (P28). |
| `src/embedding.rs` | `EmbeddingProvider` Trait für Vektoreinbettungen (MockEmbedder hinter `testkit` gegatet). |
| `src/graph.rs` | `GraphStore` Trait für Knoten- und Kantenmanipulationen. |
| `src/graph_index.rs` | `GraphIndex` Trait für CSR-Graph-Traversierung und GraphRAG (EP3). |
| `src/id_gen.rs` | `IdGenerator` Trait für injizierte ID-Erzeugung. |
| `src/kv.rs` | `TenantPrefixKvStore`, `KvLifecycleHooks` Traits für isolierten Mandanten-KV-Zugriff. |
| `src/kv_bridge_port.rs` | `KvBridgePort` Trait zur Anbindung von KV-Tiers. |
| `src/license.rs` | `LicenseVerifier` Trait für Lizenzvalidierung. |
| `src/lifecycle.rs` | `SubsystemLifecycle` Trait für geordnetes Anfahren und Herunterfahren. |
| `src/metrics.rs` | `MetricsSink` und `NoopMetricsSink` für Telemetrie-Sammelstellen. |
| `src/observability.rs` | Observability-Traits (`SpanRecorder`, `DriftStatusProvider`). |
| `src/plugin.rs` | `ToolSandbox` Trait für isolierte Plugin-Ausführung. |
| `src/query_rewriter.rs` | `QueryRewriter` Trait für Transformationen von Suchanfragen. |
| `src/reranker.rs` | `Reranker` Trait für Cross-Encoder-Reranking (EP3 / EP4). |
| `src/rng.rs` | `Rng` Trait für injizierten Zufall. |
| `src/storage.rs` | `StorageEngine`, `StorageRead` und `StorageWrite` Traits (inkl. `as_erasure_backend()`). |
| `src/text_index.rs` | `TextIndex` Trait für BM25-Volltextsuche. |
| `src/vector_index.rs` | `VectorIndex` Trait für HNSW- / DiskANN-Vektorsuche. |

## 3. Invarianten
- **INV-P28-DETERMINISM:** Zeit (`Clock`), Zufall (`Rng`) und IDs (`IdGenerator`) dürfen in produktiven Subsystemen niemals direkt über `SystemTime::now()` oder `rand::thread_rng()` bezogen werden, sondern ausschließlich über die injizierten Ports.
- **INV-CSPRNG-EXEMPTION:** Kryptographische Schlüssel und Salts (z. B. in WAL HMAC, Vaults) MÜSSEN echten CSPRNG (`OsRng`) nutzen und dürfen NICHT über den `Rng`-Port bezogen werden (Sicherheitsausnahme zu P28).
- **INV-TOCTOU-DEFAULTS:** Standardmethoden in Ports dürfen keine TOCTOU-Races induzieren. Prüfung: `cargo xtask check-toctou-defaults`.
- **INV-TXID-ALLOCATION:** `TxId` darf nicht über `IdGenerator` erzeugt werden, sondern strikt über `collection.allocate_tx()`.
- **INV-ERASURE-3:** Konstruktion von `PhysicalScope` ist ausschließlich über `ErasureBackend::resolve_physical_scope` erlaubt, um Key-Mismatch (N-K) zur Compile-Zeit zu verhindern.

## 4. Verboten / Anti-Patterns
- **VERBOTEN:** Konkrete Speicher- oder Indeximplementierungen in `contextra-ports` ablegen.
- **VERBOTEN:** `MockEmbedder` ohne `#[cfg(any(test, feature = "testkit"))]` im Release-Build exportieren.
- **VERBOTEN:** Trait-Methoden definieren, die `dyn`-Inkompatibilität erzwingen (z. B. Methoden mit generischen Parametern ohne `where Self: Sized`), sofern der Port dyn-kompatibel sein muss.
- **VERBOTEN:** Runtime-Abhängigkeiten zu Async-Runtimes (`tokio`) in synchronen Port-Traits.

## 5. Nebenläufigkeit, Async- und Lock-Regeln
- Ring 1 Reinheit (P26): Keine Async-Runtime (`tokio`) in `dependencies`.
- Synchrone Trait-Methoden (z. B. `StorageRead`, `Clock`, `Rng`) tragen keine Async-Signaturen.
- Async-Methoden in I/O-Port-Traits (wie `StorageWrite`, `ErasureBackend::purge`) nutzen `BoxFuture` für Thread-Sicherheit (`Send + Sync + 'static`).

## 6. Verifikation (xtask Cheat-Sheet)

```bash
cargo xtask crate-context contextra-ports
cargo test -p contextra-ports
cargo xtask check-ring0-async-purity
cargo xtask unwired-ports --only-unwired
cargo xtask check-toctou-defaults
cargo xtask doctrine-scan --crate contextra-ports
cargo xtask check-unsafe-islands
cargo xtask check-agents-integrity
```

## 7. Bekannte Lücken / SOLL
- Behebung N-B: `StorageEngine` wird um `fn as_erasure_backend(&self) -> Option<&dyn ErasureBackend>` erweitert, um `Any`-Downcasts in `contextra-engine::collection::crud::delete` zu eliminieren.
- Behebung N-K: `PhysicalScope` kapselt die durch `TenantScopedStorage` kodierten mandantenspezifischen Schlüssel und Präfixe.

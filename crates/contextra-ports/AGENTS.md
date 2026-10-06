# AGENTS.md — contextra-ports
> Ring 0 · stable · Quelle: capabilities.toml · Spec: K.19, §A2, §0, §3, §4, §9, §20

## 1. Zweck
`contextra-ports` definiert die abstrahierenden, dyn-kompatiblen Trait-Schnittstellen (`StorageEngine`, `VectorIndex`, `TextIndex`, `GraphIndex`, `Clock`, `Rng`, `IdGenerator` u. a.) zur Entkopplung aller Subsysteme.
Er erzwingt `#![forbid(unsafe_code)]` und dient als zentraler Ring-0-Vertrag ohne konkrete Implementierungen.

## 2. Modul-Karte
| Datei / Verzeichnis | Verantwortung |
| :--- | :--- |
| `src/lib.rs` | Re-Exporte aller Port-Traits. |
| `src/attention.rs` | `AttentionExporter` Trait für 1D-Prefill-Attention-Gewichte (KV-Cache-Eviktion). |
| `src/checkpoint.rs` | `CheckpointStore` Trait für transaktionale Snapshot-Integrität. |
| `src/clock.rs` | `Clock` Trait für injizierten Determinismus (P28). |
| `src/embedding.rs` | `Embedder` Trait für Vektoreinbettungen. |
| `src/graph.rs` | `GraphStore` Trait für Knoten- und Kantenmanipulationen. |
| `src/graph_index.rs` | `GraphIndex` Trait für CSR-Graph-Traversierung und GraphRAG. |
| `src/id_gen.rs` | `IdGenerator` Trait für injizierte ID-Erzeugung. |
| `src/kv.rs` | `TenantPrefixKvStore` Trait für isolierten Mandanten-KV-Zugriff. |
| `src/kv_bridge_port.rs` | `KvBridgePort` Trait zur Anbindung von KV-Tiers. |
| `src/license.rs` | `LicenseVerifier` Trait für Lizenzvalidierung. |
| `src/lifecycle.rs` | `SubsystemLifecycle` Trait für geordnetes Anfahren und Herunterfahren. |
| `src/metrics.rs` | `MetricsSink` und `NoopMetricsSink` für Telemetrie-Sammelstellen. |
| `src/observability.rs` | Observability-Traits (`SpanRecorder`). |
| `src/plugin.rs` | `ToolSandbox` Trait für isolierte Plugin-Ausführung. |
| `src/query_rewriter.rs` | `QueryRewriter` Trait für Transformationen von Suchanfragen. |
| `src/reranker.rs` | `Reranker` Trait für Cross-Encoder-Reranking. |
| `src/rng.rs` | `Rng` Trait für injizierten Zufall. |
| `src/storage.rs` | `StorageEngine`, `StorageRead` und `StorageWrite` Traits. |
| `src/text_index.rs` | `TextIndex` Trait für BM25-Volltextsuche. |
| `src/vector_index.rs` | `VectorIndex` Trait für HNSW- / DiskANN-Vektorsuche. |

## 3. Invarianten
- **INV-P28-DETERMINISM:** Zeit (`Clock`), Zufall (`Rng`) und IDs (`IdGenerator`) dürfen in produktiven Subsystemen niemals direkt über `SystemTime::now()` oder `rand::thread_rng()` bezogen werden, sondern ausschließlich über die injizierten Ports.
- **INV-CSPRNG-EXEMPTION:** Kryptographische Schlüssel und Salts (z. B. in WAL HMAC, Vaults) MÜSSEN echten CSPRNG nutzen und dürfen NICHT über den `Rng`-Port bezogen werden (Sicherheitsausnahme zu P28).
- **INV-TOCTOU-DEFAULTS:** Standardmethoden in Ports dürfen keine TOCTOU-Races induzieren. Prüfung: `cargo xtask check-toctou-defaults`.
- **INV-TXID-ALLOCATION:** `TxId` darf nicht über `IdGenerator` erzeugt werden, sondern strikt über `collection.allocate_tx()`.

## 4. Verboten / Anti-Patterns
- **VERBOTEN:** Konkrete Speicher- oder Indeximplementierungen in `contextra-ports` ablegen.
- **VERBOTEN:** Trait-Methoden definieren, die `dyn`-Inkompatibilität erzwingen (z. B. Methoden mit generischen Parametern ohne `where Self: Sized`), sofern der Port dyn-kompatibel sein muss.
- **VERBOTEN:** Runtime-Abhängigkeiten zu Async-Runtimes in synchronen Port-Traits.

## 5. Nebenläufigkeit, Async- und Lock-Regeln
- Synchrone Trait-Methoden (z. B. `StorageRead`, `Clock`, `Rng`) dürfen keine Async-Signaturen tragen.
- Async-Methoden in I/O-Port-Traits (wie `StorageWrite` oder `Embedder`) nutzen `BoxFuture` für Thread-Sicherheit (`Send + Sync`).

## 6. Verifikation

```bash
cargo test -p contextra-ports
cargo xtask check-toctou-defaults
cargo xtask check-agents-integrity
cargo xtask doctrine-scan --crate contextra-ports
cargo xtask check-ring0-async-purity
cargo xtask check-unsafe-islands
```

## 7. Bekannte Lücken / SOLL
- Einige Trait-Standardimplementierungen werden fortlaufend auf strikte TOCTOU-Sicherheit geprüft.

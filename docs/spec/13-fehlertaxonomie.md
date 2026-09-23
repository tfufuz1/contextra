---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "13"
---
## 13. Fehlertaxonomie (crateübergreifend)

| Crate | Fehler-Enum | Einbettet |
|---|---|---|
| `contextra-core` | `CoreError` | — |
| `contextra-store` | `StoreError` | `WalError`, `LockError`, `CoreError` |
| `contextra-crypto` | `CryptoError` | — |
| `contextra-index` | `IndexError` | `CoreError` |
| `contextra-graph` | `GraphMutationError`, `GraphError` | `LockError` |
| `contextra-router` | `BanditError` | — |
| `contextra-candle` | `KvBridgeError` | `CryptoError` |
| `contextra-mcp` | `SandboxError`, `EgressError` | `wasmtime::Error` |
| `contextra-db` | `DbError` | alle Layer-1-Fehler per `#[from]` |

**Regel (verbindlich):** Kein öffentlicher Funktionsrückgabetyp ist `Box<dyn std::error::Error>`. Jeder Crate
exportiert genau einen (oder wenige, klar abgegrenzte) `thiserror`-Fehlertyp(en); `contextra-db` als oberste
Konsumentenschicht bündelt alle Unterfehler verlustfrei per `#[from]`/`#[error(transparent)]`.

```rust
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error(transparent)] Store(#[from] contextra_store::StoreError),
    #[error(transparent)] Graph(#[from] contextra_graph::GraphMutationError),
    #[error(transparent)] Index(#[from] contextra_index::IndexError),
    #[error("provenance builder missing required field: {0}")]
    ProvenanceIncomplete(&'static str),
}
```

---

<a id="14-features"></a>

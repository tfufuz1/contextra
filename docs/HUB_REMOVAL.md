# Hub-Entfernbarkeit: types, ports, engine, db

Generiert am 2026-11-12 durch `hub_analysis.py`. Quellen: Cargo-Manifeste, `capabilities.toml` und die Schnittstellendokumente aus `interfaces/`. Statisch, nicht gebaut.

## Regel

Ein Hub ist **entfernbar**, wenn (a) kein Crate eine nicht optionale Kante auf ihn hat, (b) kein E0/E1-Baustein ihn im Kern-Quellcode außerhalb von `compat/` nutzt und (c) die verbleibenden Verwendungen in E2/E3 durch andere Crates ersetzt sind.

## 1 Übersicht

| Hub | Ebene | LOC (Produktion) | Harte Verbraucher | Optionale Verbraucher | dev-Verbraucher | Hub hängt ab von (inkl. dev) |
|---|---|---|---|---|---|---|
| `contextra-types` | E2 | 4540 | 11 | 18 | 6 | `contextra-ids` |
| `contextra-ports` | E2 | 3899 | 8 | 14 | 6 | `contextra-env`, `contextra-types` |
| `contextra-engine` | E2 | 13317 | 2 | 0 | 0 | `contextra-adapt`, `contextra-checkpoint`, `contextra-graph`, `contextra-infer-onnx`, `contextra-ingest`, `contextra-mvcc`, `contextra-ports`, `contextra-rank`, `contextra-sandbox`, `contextra-store`, `contextra-testkit`, `contextra-text`, `contextra-types`, `contextra-vector` |
| `contextra-db` | E2 | 1446 | 3 | 0 | 0 | `contextra-adapt`, `contextra-checkpoint`, `contextra-cognition`, `contextra-engine`, `contextra-graph`, `contextra-ports`, `contextra-store`, `contextra-testkit`, `contextra-text`, `contextra-types`, `contextra-vault`, `contextra-vector` |

## 2.1 `contextra-types`

- Ebene E2, Produktions-LOC 4540.
- Hängt ab von: `contextra-ids`.
- Modulbaum: 25 Quelldateien.

### Verbraucher

| Verbraucher | Ebene | Art | optional | default-features | Verwendungen (src / Brücke / Test) | Einstufung |
|---|---|---|---|---|---|---|
| `contextra` | E2 | normal | nein | ja | 13 / 0 / 24 | E2/E3, harte Kante |
| `contextra-adapt` | E0 | normal | ja | **nein** | 0 / 8 / 0 | E0/E1, optionale Kante (nur Feature) |
| `contextra-agent` | E2 | normal | nein | ja | 86 / 2 / 64 | E2/E3, harte Kante |
| `contextra-audit-export` | E0 | normal | ja | ja | 0 / 4 / 0 | E0/E1, optionale Kante (nur Feature) |
| `contextra-bench` | E3 | normal | nein | ja | 11 / 0 / 1 | E2/E3, harte Kante |
| `contextra-checkpoint` | E1 | normal | ja | ja | 0 / 5 / 38 | E0/E1, optionale Kante (nur Feature) |
| `contextra-cognition` | E2 | normal | nein | ja | 40 / 0 / 47 | E2/E3, harte Kante |
| `contextra-crypto` | E1 | dev, normal | ja | ja | 0 / 2 / 22 | E0/E1, optionale Kante (nur Feature) |
| `contextra-db` | E2 | normal | nein | ja | 15 / 0 / 98 | E2/E3, harte Kante |
| `contextra-engine` | E2 | normal | nein | ja | 395 / 0 / 264 | E2/E3, harte Kante |
| `contextra-graph` | E1 | normal | ja | ja | 167 / 39 / 225 | E0/E1, VERLETZT: 167 Verwendungen im Kern außerhalb compat/ |
| `contextra-infer-candle` | E1 | dev, normal | ja | **nein** | 6 / 47 / 25 | E0/E1, VERLETZT: 6 Verwendungen im Kern außerhalb compat/ |
| `contextra-infer-ollama` | E1 | dev, normal | ja | ja | 0 / 52 / 0 | E0/E1, optionale Kante (nur Feature) |
| `contextra-infer-onnx` | E0 | normal | ja | ja | 0 / 0 / 0 | E0/E1, optionale Kante (nur Feature) |
| `contextra-ingest` | E0 | normal | ja | ja | 0 / 19 / 3 | E0/E1, optionale Kante (nur Feature) |
| `contextra-kvcache` | E1 | dev, normal | ja | ja | 0 / 32 / 34 | E0/E1, optionale Kante (nur Feature) |
| `contextra-license` | E0 | normal | ja | ja | 0 / 4 / 0 | E0/E1, optionale Kante (nur Feature) |
| `contextra-mcp` | E3 | normal | nein | ja | 16 / 0 / 4 | E2/E3, harte Kante |
| `contextra-mvcc` | E1 | dev, normal | ja | ja | 0 / 4 / 3 | E0/E1, optionale Kante (nur Feature) |
| `contextra-ports` | E2 | normal | nein | ja | 46 / 0 / 0 | E2/E3, harte Kante |
| `contextra-py` | E3 | normal | nein | ja | 10 / 0 / 0 | E2/E3, harte Kante |
| `contextra-rank` | E1 | dev, normal | ja | **nein** | 0 / 53 / 3 | E0/E1, optionale Kante (nur Feature) |
| `contextra-router` | E2 | normal | nein | ja | 21 / 0 / 123 | E2/E3, harte Kante |
| `contextra-simd` | E0 | normal | ja | ja | 0 / 28 / 0 | E0/E1, optionale Kante (nur Feature) |
| `contextra-store` | E1 | normal | ja | ja | 0 / 41 / 189 | E0/E1, optionale Kante (nur Feature) |
| `contextra-testkit` | E3 | normal | nein | ja | 51 / 0 / 26 | E2/E3, harte Kante |
| `contextra-text` | E1 | normal | ja | ja | 0 / 62 / 187 | E0/E1, optionale Kante (nur Feature) |
| `contextra-vault` | E1 | normal | ja | ja | 0 / 4 / 6 | E0/E1, optionale Kante (nur Feature) |
| `contextra-vector` | E1 | normal | ja | ja | 24 / 54 / 193 | E0/E1, VERLETZT: 24 Verwendungen im Kern außerhalb compat/ |

### Blocker für die Entfernung

- Bausteine mit Kante auf den Hub: `contextra-adapt`, `contextra-audit-export`, `contextra-checkpoint`, `contextra-crypto`, `contextra-graph`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-ingest`, `contextra-kvcache`, `contextra-license`, `contextra-mvcc`, `contextra-rank`, `contextra-simd`, `contextra-store`, `contextra-text`, `contextra-vault`, `contextra-vector`. Ihre Brücke (`compat/`) muss umgekehrt werden oder die Kante fällt weg.
- Integrations-Verbraucher (E2/E3), die ersetzt werden müssen: `contextra`, `contextra-agent`, `contextra-bench`, `contextra-cognition`, `contextra-db`, `contextra-engine`, `contextra-mcp`, `contextra-ports`, `contextra-py`, `contextra-router`, `contextra-testkit`.

### Modulbaum (für Abgrenzung)

```text
erasure.rs
error.rs
error_dto.rs
ids.rs
lib.rs
model_fingerprint.rs
refs.rs
retrieval_strategy.rs
schema.rs
tenant_scope.rs
tombstone.rs
types.rs
types/budget.rs
types/domain/document.rs
types/domain/ids.rs
types/domain/misc.rs
types/domain/mod.rs
types/domain/tests/doc_tests.rs
types/domain/tests/ids_tests.rs
types/domain/tests/misc_tests.rs
types/domain/tests/mod.rs
types/filter.rs
types/importance.rs
types/saos.rs
witness.rs
```

## 2.2 `contextra-ports`

- Ebene E2, Produktions-LOC 3899.
- Hängt ab von: `contextra-env`, `contextra-types`.
- Modulbaum: 42 Quelldateien.

### Verbraucher

| Verbraucher | Ebene | Art | optional | default-features | Verwendungen (src / Brücke / Test) | Einstufung |
|---|---|---|---|---|---|---|
| `contextra` | E2 | normal | nein | ja | 16 / 0 / 6 | E2/E3, harte Kante |
| `contextra-adapt` | E0 | normal | ja | **nein** | 0 / 1 / 0 | E0/E1, optionale Kante (nur Feature) |
| `contextra-agent` | E2 | normal | nein | ja | 28 / 1 / 26 | E2/E3, harte Kante |
| `contextra-checkpoint` | E1 | normal | ja | ja | 0 / 6 / 32 | E0/E1, optionale Kante (nur Feature) |
| `contextra-cognition` | E2 | normal | nein | ja | 10 / 0 / 29 | E2/E3, harte Kante |
| `contextra-crypto` | E1 | dev, normal | ja | ja | 0 / 0 / 2 | E0/E1, optionale Kante (nur Feature) |
| `contextra-db` | E2 | normal | nein | ja | 11 / 0 / 38 | E2/E3, harte Kante |
| `contextra-engine` | E2 | normal | nein | ja | 126 / 0 / 65 | E2/E3, harte Kante |
| `contextra-graph` | E1 | normal | ja | ja | 27 / 17 / 69 | E0/E1, VERLETZT: 27 Verwendungen im Kern außerhalb compat/ |
| `contextra-infer-candle` | E1 | dev, normal | ja | **nein** | 4 / 28 / 20 | E0/E1, VERLETZT: 4 Verwendungen im Kern außerhalb compat/ |
| `contextra-infer-ollama` | E1 | dev, normal | ja | ja | 0 / 30 / 0 | E0/E1, optionale Kante (nur Feature) |
| `contextra-infer-onnx` | E0 | normal | ja | ja | 0 / 3 / 0 | E0/E1, optionale Kante (nur Feature) |
| `contextra-kvcache` | E1 | dev, normal | ja | ja | 0 / 40 / 26 | E0/E1, optionale Kante (nur Feature) |
| `contextra-license` | E0 | normal | ja | ja | 0 / 26 / 0 | E0/E1, optionale Kante (nur Feature) |
| `contextra-mcp` | E3 | normal | nein | ja | 24 / 0 / 26 | E2/E3, harte Kante |
| `contextra-mvcc` | E1 | dev, normal | ja | ja | 0 / 0 / 0 | E0/E1, optionale Kante (nur Feature) |
| `contextra-rank` | E1 | dev, normal | ja | **nein** | 0 / 4 / 1 | E0/E1, optionale Kante (nur Feature) |
| `contextra-router` | E2 | normal | nein | ja | 26 / 0 / 42 | E2/E3, harte Kante |
| `contextra-store` | E1 | normal | ja | ja | 0 / 31 / 114 | E0/E1, optionale Kante (nur Feature) |
| `contextra-testkit` | E3 | normal | nein | ja | 74 / 0 / 64 | E2/E3, harte Kante |
| `contextra-text` | E1 | normal | ja | ja | 0 / 22 / 146 | E0/E1, optionale Kante (nur Feature) |
| `contextra-vector` | E1 | normal | ja | ja | 12 / 11 / 83 | E0/E1, VERLETZT: 12 Verwendungen im Kern außerhalb compat/ |

### Blocker für die Entfernung

- Bausteine mit Kante auf den Hub: `contextra-adapt`, `contextra-checkpoint`, `contextra-crypto`, `contextra-graph`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-kvcache`, `contextra-license`, `contextra-mvcc`, `contextra-rank`, `contextra-store`, `contextra-text`, `contextra-vector`. Ihre Brücke (`compat/`) muss umgekehrt werden oder die Kante fällt weg.
- Integrations-Verbraucher (E2/E3), die ersetzt werden müssen: `contextra`, `contextra-agent`, `contextra-cognition`, `contextra-db`, `contextra-engine`, `contextra-mcp`, `contextra-router`, `contextra-testkit`.

### Modulbaum (für Abgrenzung)

```text
attention.rs
barrier.rs
checkpoint.rs
clock.rs
conformance.rs
contract_tests.rs
embedding.rs
env.rs
erasure.rs
erasure_legacy.rs
fs.rs
gate.rs
graph.rs
graph_index.rs
id_gen.rs
index.rs
inference.rs
key_material.rs
keys.rs
kv.rs
kv_bridge_port.rs
ledger.rs
lib.rs
license.rs
lifecycle.rs
metrics.rs
observability.rs
observer.rs
plugin.rs
policy.rs
query_rewriter.rs
report_signer.rs
reranker.rs
rng.rs
scanner/layer.rs
scanner/mod.rs
scanner/session.rs
scanner/witness.rs
storage.rs
text_index.rs
vector_index.rs
vfs.rs
```

## 2.3 `contextra-engine`

- Ebene E2, Produktions-LOC 13317.
- Hängt ab von: `contextra-adapt`, `contextra-checkpoint`, `contextra-graph`, `contextra-infer-onnx`, `contextra-ingest`, `contextra-mvcc`, `contextra-ports`, `contextra-rank`, `contextra-sandbox`, `contextra-store`, `contextra-testkit`, `contextra-text`, `contextra-types`, `contextra-vector`.
- Modulbaum: 88 Quelldateien.

### Verbraucher

| Verbraucher | Ebene | Art | optional | default-features | Verwendungen (src / Brücke / Test) | Einstufung |
|---|---|---|---|---|---|---|
| `contextra-cognition` | E2 | normal | nein | ja | 24 / 0 / 12 | E2/E3, harte Kante |
| `contextra-db` | E2 | normal | nein | ja | 37 / 0 / 0 | E2/E3, harte Kante |

### Blocker für die Entfernung

- Keine E0/E1-Kante auf diesen Hub.
- Integrations-Verbraucher (E2/E3), die ersetzt werden müssen: `contextra-cognition`, `contextra-db`.

### Modulbaum (für Abgrenzung)

```text
adapters/mod.rs
adapters/ollama.rs
adapters/privacy.rs
adapters/text.rs
backends.rs
background_workers/config.rs
background_workers/expiry_workers.rs
background_workers/hyperedge_worker.rs
background_workers/mod.rs
background_workers/orphan_workers.rs
background_workers/tests.rs
chunker/blocks.rs
chunker/config.rs
chunker/estimate.rs
chunker/mod.rs
chunker/sections.rs
chunker/split.rs
chunker/unit_tests.rs
chunker/windows.rs
collection/crud/auto_extraction.rs
collection/crud/delete.rs
collection/crud/insert.rs
collection/crud/internal.rs
collection/crud/kv.rs
collection/crud/links.rs
collection/crud/mod.rs
collection/crud/read.rs
collection/crud/tests.rs
collection/crud/update.rs
collection/kv_lock.rs
collection/maintenance.rs
collection/mod.rs
collection/query_builder/builder.rs
collection/query_builder/builder_exec.rs
collection/query_builder/mod.rs
collection/query_builder/scope.rs
collection/query_builder/strategy.rs
collection/query_builder/tests.rs
collection/query_builder/weights.rs
collection/relate.rs
collection/search/basic.rs
collection/search/checkpoint.rs
collection/search/filtered.rs
collection/search/hybrid/mod.rs
collection/search/hybrid/query.rs
collection/search/hydrate.rs
collection/search/mod.rs
collection/tests/concurrency_tests.rs
collection/tests/fixtures.rs
collection/tests/hybrid_search_tests.rs
collection/tests/insert_tests.rs
collection/tests/link_tests.rs
collection/tests/misc_tests.rs
collection/tests/mod.rs
collection/tests/read_tests.rs
collection/tests/search_tests.rs
collection/tests/update_delete_tests.rs
collection/tx.rs
contextra_impl/collections.rs
contextra_impl/crud.rs
contextra_impl/diagnostics.rs
contextra_impl/lifecycle.rs
contextra_impl/mod.rs
contextra_impl/wiring.rs
decay_controller.rs
export.rs
extraction/mod.rs
extraction/open_ie.rs
extraction/rule_based.rs
extraction/tests.rs
extraction/types.rs
filter.rs
fusion.rs
import.rs
lib.rs
license_seam.rs
merge_operator_adapter.rs
metrics_names.rs
orchestration/mod.rs
temporal_filter.rs
transaction/cleanup.rs
transaction/compensating_actions.rs
transaction/db_transaction.rs
transaction/db_transaction_commit.rs
transaction/db_transaction_rollback.rs
transaction/intent.rs
transaction/mod.rs
transaction/tests.rs
```

## 2.4 `contextra-db`

- Ebene E2, Produktions-LOC 1446.
- Hängt ab von: `contextra-adapt`, `contextra-checkpoint`, `contextra-cognition`, `contextra-engine`, `contextra-graph`, `contextra-ports`, `contextra-store`, `contextra-testkit`, `contextra-text`, `contextra-types`, `contextra-vault`, `contextra-vector`.
- Modulbaum: 7 Quelldateien.

### Verbraucher

| Verbraucher | Ebene | Art | optional | default-features | Verwendungen (src / Brücke / Test) | Einstufung |
|---|---|---|---|---|---|---|
| `contextra` | E2 | normal | nein | ja | 28 / 0 / 1 | E2/E3, harte Kante |
| `contextra-agent` | E2 | normal | nein | ja | 26 / 2 / 78 | E2/E3, harte Kante |
| `contextra-bench` | E3 | normal | nein | ja | 9 / 0 / 0 | E2/E3, harte Kante |

### Blocker für die Entfernung

- Keine E0/E1-Kante auf diesen Hub.
- Integrations-Verbraucher (E2/E3), die ersetzt werden müssen: `contextra`, `contextra-agent`, `contextra-bench`.

### Modulbaum (für Abgrenzung)

```text
cognition/bridge.rs
cognition/mod.rs
homeostat.rs
lib.rs
multistep.rs
pid_latency_controller.rs
volatile_vault.rs
```

## 3 Empfohlene Reihenfolge

1. **Brücken umkehren** (E0/E1 → E2). Jede `compat/`-Brücke eines Bausteins wandert in den Verbraucher oder in einen Adapter. Danach hat kein Baustein eine Kante auf `types` oder `ports`. Betrifft 18 Bausteine (E0/E1) mit Kante auf `types` oder `ports`.
2. **`ports` auflösen.** Port-Traits gehören dem Verbraucher, der sie braucht (S-63 zeigt das Muster für `cognition` und `agent`). `erasure`, `erasure_legacy`, `license`, `ledger`, `conformance` gehen an ihre Besitzer (S-50).
3. **`types` aufteilen.** `erasure.rs` → `erasure`, `tombstone.rs` entfällt (S-30B), `PprConfig` und `TlHfdParams` nur bei `graph` (S-51). Der Rest (Fehlerbasis, `RequestContext`, Budget, Filter) braucht einen Zielort je Typ.
4. **`engine` auflösen.** Die Module gehen nach `db` oder in einen neuen E2-Orchestrator. `cognition` darf `engine` nicht mehr brauchen (S-63).
5. **`db` auflösen.** Die Fassade zieht in `contextra` (Root). `agent` braucht `Contextra` heute als konkreten Typ; er muss auf Ports umgestellt werden, bevor `db` fällt.

## 4 Grenzen

- Verwendungszahlen stammen aus den Schnittstellendokumenten (Zählung ohne Kommentare, Klassen src/compat/test). Sie sind nicht identisch mit der Gate-Textzählung.
- Feature-Gating einzelner Verwendungen wird nicht ausgewertet.
- Nichts wurde gebaut.

# contextra-testkit

Test Fixtures, Mock Engines, In-Memory Referenz-Implementierungen und Port-Konformitätssuiten für Contextra.

## Zweck
`contextra-testkit` stellt Test-Mocks, In-Memory Reference Stores, Injektions-Fixtures und wiederverwendbare Vertragstests (Konformitätssuiten) für Port-Traits bereit. Es dient ausschließlich als Testhilfe in `[dev-dependencies]`.

## Reifegrad
- **Status:** Stabil (P1–P6 erfüllt)
- **Rolle:** Test-Hilfsbibliothek (`[dev-dependencies]`)
- **Architektur-Invariante (SPEC R2 / INV-EMBED-1):** Keine produktive Nutzung; Kerne dürfen Testkit-Fakes und Mocks NIEMALS in Nicht-Test-Code einbinden. Enthält deterministische Doubles für Storage, Vector, Text, Graph, KV-Cache, Routing, VFS (`FaultVfs`), Clock (`ManualClock`) und Metrics.

## Interne Abhängigkeiten
`contextra-testkit` konsumiert ausschließlich Fundament-Typen und Port-Schnittstellen (keine Abhängigkeit zu `contextra-kvcache` oder `contextra-router`):
- `contextra-types`
- `contextra-ports` (mit Feature `testkit`)

## Features
- `default = ["testkit", "erasure-conformance"]`
- `testkit`: Aktiviert Testkit-Mocks und Konformitätssuiten.
- `erasure-conformance`: Übergangs-Feature für die Bereitstellung synthetischer Witness-Hilfsfunktionen (`mint_witness_for_tests`) für bestehende Testsuiten (wird in Phase 2 durch den Integrator entfernt, siehe `docs/handoff/fragments/P09/PENDING_API_CHANGES.md`).

## Einbindung in Fremdprojekte (Cargo-Snippet)
```toml
[dev-dependencies]
contextra-testkit = { version = "0.1", default-features = false, features = ["testkit"] }
```

## Anwendungsbeispiel: Eigene `KvPrefixStore`-Implementierung prüfen
In ca. 15 Zeilen lässt sich ein eigener KV-Cache-Adapter gegen die standardisierte Vertragssuite prüfen:

```rust
use contextra_testkit::conformance::run_kv_prefix_store_suite;
use contextra_testkit::mocks::InMemoryKvPrefixStore;

#[test]
fn test_my_kv_prefix_store_conformance() {
    let store = InMemoryKvPrefixStore::new();
    let res = run_kv_prefix_store_suite(&store);
    assert!(res.is_ok(), "Conformance failure: {res:?}");
}
```

## Zuordnungstabelle: Fake -> Port -> Suite

| Fake-Typ (`contextra_testkit::mocks::*`) | Implementierter Port (`contextra_ports::*`) | Konformitätssuite (`conformance::*`) |
|---|---|---|
| `InMemoryKvPrefixStore` | `KvPrefixStore` | `run_kv_prefix_store_suite` |
| `InMemoryKvBridgeStorage` | `KvBridgeStorage` | `run_kv_bridge_storage_suite` |
| `ScriptedAttentionExporter` | `AttentionExporter` | `run_attention_exporter_suite` |
| `ScriptedHybridSearchProvider` | `HybridSearchProvider` | `run_hybrid_search_provider_suite` |
| `StaticCommunityResolver` | `CommunityResolver` | `run_community_resolver_suite` |
| `BudgetTruncatingContextPreparer` | `ContextPreparer` | `run_context_preparer_suite` |
| `StaticDriftStatusProvider` | `DriftStatusProvider` | `run_drift_status_provider_suite` |
| `ScriptedEgressPolicy` | `EgressPolicy` | `run_egress_policy_suite` |
| `InMemoryStorageEngine` | `StorageEngine` | `run_storage_engine_suite` |
| `InMemoryVectorIndex` | `VectorIndex` | `run_vector_index_suite` |
| `InMemoryTextIndex` | `TextIndex` | `run_text_index_suite` |
| `InMemoryGraphIndex` | `GraphIndex` | `run_graph_index_suite` |
| `ManualClock` | `Clock` | `run_clock_suite` |
| `FaultVfs` | `Vfs` | `run_vfs_suite` |
| `TestMetricsSink` | `MetricsSink` | `run_metrics_sink_suite` |

## Bekannte Grenzen
- In-Memory Doubles sind nicht für Produktivlasten gedacht und halten alle Daten unkomprimiert im flüchtigen RAM.
- `FaultVfs` simuliert Fehler deterministisch anhand von Zählern und Fehlerplänen für Testzwecke.

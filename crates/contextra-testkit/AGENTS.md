# AGENTS.md — `contextra-testkit`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-testkit` · **Ring:** tooling · **Version:** 0.1.0
- **Beschreibung:** Deterministic test utilities, ManualClock, InMemoryStorageEngine, and FaultVfs for Contextra
- **src:** 22 Dateien, 3745 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 2):** `contextra-ports`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 3):** `bytes`, `parking_lot`, `tracing`
- **Reverse-Abhängigkeiten (0):** –
- **Kopplung an Hub-Crates:** ausgehend `contextra-ports`, `contextra-types`; eingehend –
- **Features (3):** `default`, `erasure-conformance`, `testkit`

### Öffentliche API (grob, aus src/)
- struct (19): `BudgetTruncatingContextPreparer`, `ConformanceFailure`, `FaultConfig`, `FaultVfs`, `InMemoryGraphIndex`, `InMemoryKvBridgeStorage`, `InMemoryKvPrefixStore`, `InMemoryStorageEngine`, `InMemoryTextIndex`, `InMemoryVectorIndex`, `ManualClock`, `MockEmbedder`, `ReferenceModel`, `ScriptedAttentionExporter`, `ScriptedEgressPolicy` … (+4)
- enum (2): `MetricEvent`, `RefOp`
- fn (65): `add_deny_pattern`, `add_redact_pattern`, `advance`, `assert_attention_exporter_conformance`, `assert_community_resolver_conformance`, `assert_context_preparer_conformance`, `assert_drift_status_provider_conformance`, `assert_egress_policy_conformance`, `assert_hybrid_search_provider_conformance`, `assert_kv_bridge_storage_conformance`, `assert_kv_prefix_store_conformance`, `clear`, `commit`, `commit_batch`, `delete` … (+50)
- type (1): `ConformanceResult`
- mod (24): `clock`, `conformance`, `doubles`, `failure`, `fault`, `fault_vfs`, `graph`, `in_memory_graph`, `in_memory_store`, `in_memory_text`, `in_memory_vector`, `kv_ports`, `macros`, `manual_clock`, `metrics` … (+9)
- pub use: `clock::run_clock_suite`, `crate::fault_vfs::{FaultConfig, FaultVfs}`, `crate::in_memory_graph::InMemoryGraphIndex`, `crate::in_memory_store::InMemoryStorageEngine`, `crate::in_memory_text::InMemoryTextIndex`, `crate::in_memory_vector::InMemoryVectorIndex`, `crate::manual_clock::ManualClock`, `crate::mocks::{ BudgetTruncatingContextPreparer, InMemoryKvBridgeStorage, InMemoryKvPrefixStore, MockEmbedder, ScriptedAttentionExporter, ScriptedEgressPolicy, ScriptedHybridSearchProvider, StaticCommunityResolver, StaticDriftStatusProvider, TestMetricsSink, }`, `crate::reference_model::{RefOp, ReferenceModel}`, `failure::{ConformanceFailure, ConformanceResult}` … (+17)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->`.

<!-- END GENERATED: crate_inventory.py -->

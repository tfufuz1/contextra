# AGENTS.md — contextra-ports — Port-Schnittstellen / Traits

**Status:** Normativ  
**Crate-Spezifikation:** [`docs/crates/contextra-ports.md`](file:///home/freddy/Projekte/contextra/docs/crates/contextra-ports.md)  
**Gesamtspezifikation:** [`docs/SPEC.md`](file:///home/freddy/Projekte/contextra/docs/SPEC.md) — Bindung: docs/SPEC.md §2 (Architektur), §5 (Löschkette), §6 (ReadGate), §7 (ErasureLayer), §10 (Barrieren)  
**Workspace-Richtlinie:** [`AGENTS.md`](file:///home/freddy/Projekte/contextra/AGENTS.md) (Wurzel)  
**Ebene:** E2 (Integration & Schnittstellenverträge)  
**Kern-Closure:** Darf E0, E1, E2 nutzen, nie E3  
**Unsafe-Status:** Verboten (`#![forbid(unsafe_code)]`)  

---

## 1. Zweck & Systemrolle

Hexagonale Architektur: Trait-Schnittstellen zur Schichtenentkopplung (ErasureLayer, ReadGate, Barrieren).

---

## 2. Kern-Richtlinien für Agenten

- **Schichten-Entkopplung:** Definiert `ErasureLayer`, `ReadGate`, `CommitBarrier`, `PurgeBarrier`, `SnapshotRevoker`.
- **Positivkontrolle & Messung (INV-WITNESS-MINT-1):** `LayerWitness` und `Scanner<Armed>` sind kanonisch hier definiert. `residue_matches == 0` ist Bedingung für den erfolgreichen Abschluss einer Löschsaga.
- **Keine konkreten Speicher-Imports:** `contextra-ports` importiert weder `store`, `vector`, `text`, `graph` noch `db`.
- **Alle Traits sind `Send + Sync`.**

---

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-ports` · **Ring:** 0 · **Version:** 0.1.0
- **Beschreibung:** Canonical dyn-compatible port traits for Contextra subsystems
- **src:** 42 Dateien, 4251 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 2):** `contextra-env`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 7):** `ahash`, `blake3`, `bytes`, `futures-util`, `serde`, `serde_json`, `thiserror`
- **Reverse-Abhängigkeiten (12):** `contextra`, `contextra-checkpoint`, `contextra-crypto`, `contextra-db`, `contextra-graph`, `contextra-mcp`, `contextra-mvcc`, `contextra-rank`, `contextra-store`, `contextra-testkit`, `contextra-text`, `contextra-vector`
- **Kopplung an Hub-Crates:** ausgehend `contextra-types`; eingehend `contextra`, `contextra-db`
- **Features (4):** `conformance`, `contract-tests`, `default`, `testkit`

### Öffentliche API (grob, aus src/)
- struct (47): `Armed`, `AuthorizedRing`, `BarrierExclusive`, `BarrierShared`, `CompactionPermit`, `ContextSegment`, `Envelope`, `ErasureEvent`, `FileId`, `FileKey`, `GraphIndexStats`, `GroundingAssessment`, `HyperEdgeId`, `KeyBuf`, `KeyRange` … (+32)
- enum (20): `EgressDecision`, `EmbeddingError`, `EraseTarget`, `FeatureRing`, `FileKind`, `GateDecision`, `GraphMutationError`, `KeyBound`, `KeyCoverage`, `LicenseError`, `ObservedPhase`, `ObserverError`, `PluginError`, `RangeGate`, `ScanTechnique` … (+5)
- trait (43): `AttentionExporter`, `Checkpoint`, `CheckpointCoordinator`, `CommitBarrier`, `CommunityResolver`, `ContextPreparer`, `DriftStatusProvider`, `EgressPolicy`, `EmbeddingProvider`, `ErasureLayer`, `ErasureObserver`, `ErasureTarget`, `FileKeyring`, `GraphCollectionMutation`, `GraphIndex` … (+28)
- fn (64): `activate_all`, `affected_units`, `as_bytes`, `as_str`, `as_u64`, `assert_attention_exporter_contract`, `assert_checkpoint_contract`, `assert_checkpoint_coordinator_contract`, `assert_egress_policy_contract`, `assert_embedding_contract`, `assert_erasure_layer_contract`, `assert_graph_index_contract`, `assert_keys_contract`, `assert_ledger_contract`, `assert_license_contract` … (+49)
- type (4): `BoxFuture`, `BoxStream`, `SignerId`, `Timestamp`
- const (1): `MAX_SCAN_MERGE_ACCUMULATOR`
- mod (43): `attention`, `barrier`, `check`, `checkpoint`, `clock`, `conformance`, `contract_tests`, `embedding`, `env`, `erasure`, `erasure_legacy`, `fs`, `gate`, `graph`, `graph_index` … (+28)
- pub use: `attention::*`, `barrier::{ BarrierExclusive, BarrierShared, CommitBarrier, CompactionPermit, PurgeBarrier, PurgePermit, }`, `checkpoint::*`, `clock::*`, `conformance::*`, `contextra_env::*`, `contextra_env::clock::*`, `contextra_env::id_gen::*`, `contextra_env::metrics::*`, `contextra_env::rng::*` … (+59)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->

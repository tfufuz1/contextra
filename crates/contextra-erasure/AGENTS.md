# AGENTS.md — contextra-erasure — DSGVO-Löschkern & Saga-Orchestrator

**Status:** Normativ  
**Crate-Spezifikation:** [`docs/crates/contextra-erasure.md`](file:///home/freddy/Projekte/contextra/docs/crates/contextra-erasure.md)  
**Gesamtspezifikation:** [`docs/SPEC.md`](file:///home/freddy/Projekte/contextra/docs/SPEC.md) — Bindung: docs/SPEC.md §4 (Schlüsselhierarchie), §5 (Löschkette), §6 (ReadGate), §7 (ErasureLayer), §10 (Barrieren), §11 (Fehlerverhalten), §12 (Testkatalog), §13 (Invarianten)  
**Workspace-Richtlinie:** [`AGENTS.md`](file:///home/freddy/Projekte/contextra/AGENTS.md) (Wurzel)  
**Ebene:** E0 (Autarkes Fundament / Ring 0)  
**Kern-Closure:** Leer (0 interne Workspace-Abhängigkeiten)  
**Unsafe-Status:** Verboten (`#![forbid(unsafe_code)]`)  

---

## 1. Zweck & Systemrolle

Zentraler Löschkern: 6-Schritte-Löschsaga, Crypto-Shredding, In-Memory ReadGate, Purge-Orchestrierung und beweisbare Messung.

---

## 2. Kern-Richtlinien für Agenten

- **Ring-Isolation (E0):** Keine direkten Abhängigkeiten auf konkrete Speicher-Crates (`store`, `vector`, etc.); Steuerung erfolgt strikt über Port-Traits aus `contextra-ports`.
- **Löschkette:** Ausschließlich `chain::Chain` (SPEC §5.3). Schritte: 1 Intent (fsync #1), 2 Gate unter `CommitBarrier` (fsync #2), 3 `on_gate` → Schlüsselvernichtung → `verify_absent` → `KeyRevoked`, 4 Purge in Prioritätsreihenfolge, 5 Messung mit Positivkontrolle und Lesepfad-Test, 6 Bericht über `ReportSink` → `Completed`.
- **Normative Quelle:** `docs/SPEC.md` §5–§7. Die Crate-Spec `docs/crates/contextra-erasure.md` leitet sich davon ab. Bei Widerspruch gilt die SPEC.
- **ReadGate-Garantie:** `SagaGate::check` ist O(1). Gate vor Schlüsselvernichtung, Gate nie zurücknehmen (INV-10).
- **Verbotene Altlasten:** Keine Saga-Orchestrierung außerhalb von `chain/`, kein `KeyDestroyed`/`KeyRange`, keine eigenen Krypto-Primitive ausserhalb des Tresor-Kerns (R4, Restarbeit siehe Crate-Spec §10).


---

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-erasure` · **Ring:** 0 · **Version:** 0.1.0
- **Beschreibung:** Löschkern: Löschkette (Saga), ReadGate, Tresor, Messung, Schichtschnittstelle.
- **src:** 23 Dateien, 4359 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 3):** `contextra-durable-fs`, `contextra-env`, `contextra-ids`
- **Interne Dev/Build-Abhängigkeiten (1):** `contextra-env`
- **Externe Abhängigkeiten (normal, 11):** `blake3`, `bytes`, `hmac`, `parking_lot`, `serde`, `serde_json`, `sha2`, `subtle`, `thiserror`, `tracing`, `zeroize`
- **Reverse-Abhängigkeiten (0):** –
- **Kopplung an Hub-Crates:** ausgehend –; eingehend –
- **Features (1):** `default`

### Öffentliche API (grob, aus src/)
- struct (38): `AncestorSalts`, `Armed`, `BarrierExclusive`, `Chain`, `ChainDeps`, `CommitBarrier`, `ControlEvidence`, `Dek`, `Destroyed`, `DurableVaultShard`, `FileJournal`, `FileVault`, `FileVaultKeys`, `JournalRecord`, `LayerId` … (+23)
- enum (15): `EraseTarget`, `ErasureError`, `GateDecision`, `GateMode`, `HkdfError`, `KeyCoverage`, `MeasureClass`, `PendingReason`, `RangeGate`, `RetentionCode`, `SagaState`, `ShardBackend`, `SlotType`, `TargetKind`, `VerifiedVia`
- trait (12): `Clock`, `DiskReserve`, `ErasureLayer`, `GateSink`, `Journal`, `KeyVault`, `PositiveControl`, `ReadGate`, `ReportSink`, `VaultScrubContext`, `Vfs`, `VfsFile`
- fn (136): `activate_key`, `activate_prefix`, `activate_restricted_key`, `activate_restricted_prefix`, `arm`, `armed`, `as_bytes`, `barrier`, `blocks_erasure`, `blocks_reads`, `bytes`, `can_transition_to`, `capacity`, `class`, `collect_active_slots` … (+121)
- type (2): `BoxFuture`, `LayerFuture`
- const (19): `CTX_FILE_MAGIC`, `CTX_FORMAT_VERSION`, `GEN`, `MAC`, `MAX_DISTINCT_PREFIX_LENGTHS`, `PAYLOAD`, `SLOT_TYPE`, `STATUS`, `STATUS_ACTIVE`, `STATUS_EMPTY`, `STATUS_TOMBSTONE`, `TAG`, `VAULT_ALG_IDS`, `VAULT_FILE_KIND`, `VAULT_HEADER_SIZE` … (+4)
- mod (23): `adapters`, `chain`, `crypto_local`, `durable`, `error`, `gate`, `io`, `journal`, `layer`, `off`, `open`, `ports`, `rehash`, `retention`, `saga` … (+8)
- pub use: `crate::types_local::VerifiedVia`, `durable::VAULT_SLOT_BASE`, `gate::saga_gate::{GateDecision, RangeGate, ReadGate, SagaGate}`, `journal::{FileJournal, JournalRecord}`, `layer::{ErasureLayer, LayerFuture, LayerScope, PurgeCx, PurgeOutcome}`, `open::{ compute_metablock_mac, encode_metablock, open_vault, open_vault_shard, MetablockParams, VaultOpenOutcome, VAULT_METABLOCK_SIZE, }`, `ports::{ CommitBarrier, DiskReserve, GateMode, GateSink, Journal, KeyVault, PositiveControl, ReportSink, }`, `rehash::{rehash, rehash_durable_shard, rehash_shard, RehashOutcome}`, `retention::{RetentionAttestation, RetentionCode}`, `saga::{Chain, ChainDeps, Receipt}` … (+6)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->

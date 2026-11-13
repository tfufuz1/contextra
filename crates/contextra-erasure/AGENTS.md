# AGENTS.md — `contextra-erasure`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-erasure` · **Ring:** 3 · **Version:** 0.1.0
- **Beschreibung:** Löschkern: Saga, Gate, Journal, Tresor, Messung, signierter Bericht.
- **src:** 83 Dateien, 16726 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 0):** –
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 18):** `aes-gcm-siv`, `ahash`, `bincode`, `blake3`, `bytes`, `ed25519-dalek`, `futures-util`, `hmac`, `parking_lot`, `rand`, `serde`, `serde_json`, `sha2`, `subtle`, `thiserror`, `tokio`, `tracing`, `zeroize`
- **Reverse-Abhängigkeiten (0):** –
- **Kopplung an Hub-Crates:** ausgehend –; eingehend –
- **Features (4):** `async`, `default`, `signing`, `vault`

### Öffentliche API (grob, aus src/)
- struct (186): `ActiveScopeFlight`, `AncestorSalts`, `AppendLog`, `Armed`, `ArmedScanner`, `BarrierExclusive`, `BarrierShared`, `BatchGateOutcome`, `BatchGateRequest`, `BatchRecordItem`, `BlobRewriter`, `CollectionId`, `CollectionRef`, `CompactionPermit`, `ControlEvidence` … (+171)
- enum (86): `AdmissionFailure`, `CollectionStatus`, `CustodyLevel`, `DeletionLayer`, `DeletionProofError`, `DeletionScope`, `DisposeFailure`, `EraseMode`, `EraseTarget`, `ErasureError`, `ErasureStatus`, `EvidenceKind`, `ExcludedScope`, `FenceFailure`, `FenceGateError` … (+71)
- trait (35): `Clock`, `CommitBarrier`, `EpochKeys`, `ErasureLayer`, `ErasureObserver`, `ErasureTarget`, `FenceGate`, `FileKeyring`, `IdGen`, `KeyStore`, `LayerScanner`, `LayerWitnessExt`, `LedgerSink`, `LedgerStorage`, `ProofKeyProvider` … (+20)
- fn (447): `acquire`, `acquire_with_lease`, `add_batch`, `add_entry`, `add_erased`, `add_erased_with_bounds`, `add_fenced_key`, `add_finding`, `add_prefix_seq`, `add_restricted`, `add_restricted_with_bounds`, `advance_to_next`, `advance_to_r2_intent`, `advance_to_r3_gate_old`, `advance_to_r4_key_destroyed` … (+432)
- type (7): `BoxFuture`, `CryptoError`, `InMemoryKeyring`, `Result`, `ScopeLayerId`, `SignerId`, `WitnessEntry`
- const (34): `CACHE_EVICT`, `CTRALEDG_MAGIC`, `CTRALEDG_VERSION`, `CTX_FILE_MAGIC`, `CTX_FORMAT_VERSION`, `DEFAULT_EXCLUDED_SCOPES`, `DEFAULT_SHRED_KEY_GROUP_SIZE`, `FIXED_HEADER_LEN`, `GEN`, `GENESIS_HASH`, `MAC`, `MIN_FRAME_LEN`, `PAYLOAD`, `SIZE`, `SLOT_CLEANUP` … (+19)
- mod (79): `appendlog`, `backend`, `batch`, `binding`, `build`, `canonical`, `codec`, `core`, `crypto_local`, `cx`, `deletion_proof`, `driver`, `durable`, `ed25519_proof`, `entry` … (+64)
- pub use: `appendlog::{AppendLog, DurableJournal}`, `binding::{compute_witnesses_chain_hash, verify_witnesses_binding}`, `build::{ build_report, build_report_with_options, canonicalize_report, ErasureReport, FieldMeasurementSection, LayerKeyCoverage, LayerWitnessReport, ProcessHardeningStatus, ReachStatus, ReportBuildOptions, WitnessEntry, }`, `canonical::{ canonicalize_json, canonicalize_value, to_canonical_json, to_canonical_json_string, verify_canonical_nfc, }`, `codec::{decode_scope, encode_scope}`, `core::{EraseExecutionRequest, ErasureConfig, ErasureCore, ErasureCoreStatus, ErasureDeps}`, `crate::saga::SagaState`, `crate::types_local::VerifiedVia`, `crate::types_local::{ Armed, ControlEvidence, LayerWitness, MeasureClass, Scanner, Unarmed, }`, `crate::types_local::{ EraseMode, EraseTarget, PendingReason, RetentionAttestation, RetentionCode, Timing, }` … (+53)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->

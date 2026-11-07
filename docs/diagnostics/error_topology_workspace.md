# Error Topology & Conversion Architecture Report

> **Werkzeug:** `error_topology_mapper.py`
> **Erstellungszeit:** `2026-11-07T23:15:42.238646+00:00`
> **Status/Befunde:** `85 Befunde erfasst`

## Beschreibung
Maps strongly-typed error definitions (thiserror), variants, error codes, and cross-boundary error conversion implementations (impl From<E>).

## Metriken-Übersicht
| Metrik | Wert |
|---|---|
| `total_error_enums` | `43` |
| `total_error_variants` | `312` |
| `total_conversions_from` | `42` |

## Erfasste Error-Klassen
| Crate | Error-Typ | Zeile | Varianten | Datei |
|---|---|---|---|---|
| `contextra` | `CollectionProfileError` | 92 | 2 | `crates/contextra/src/collection_profile.rs` |
| `contextra` | `DurabilityConfigError` | 51 | 4 | `crates/contextra/src/performance_profile.rs` |
| `contextra` | `PerformanceProfileError` | 126 | 3 | `crates/contextra/src/performance_profile.rs` |
| `contextra-adapt` | `BanditError` | 36 | 8 | `crates/contextra-adapt/src/bandit.rs` |
| `contextra-adapt` | `FcTsError` | 29 | 5 | `crates/contextra-adapt/src/flow_thompson.rs` |
| `contextra-adapt` | `OffPolicyError` | 713 | 1 | `crates/contextra-adapt/src/flow_thompson.rs` |
| `contextra-adapt` | `RieGreedyError` | 12 | 5 | `crates/contextra-adapt/src/rie_greedy.rs` |
| `contextra-audit-export` | `AuditExportError` | 7 | 5 | `crates/contextra-audit-export/src/error.rs` |
| `contextra-avv-generator` | `AvvError` | 32 | 11 | `crates/contextra-avv-generator/src/error.rs` |
| `contextra-crypto` | `DeletionProofError` | 29 | 3 | `crates/contextra-crypto/src/ed25519_proof.rs` |
| `contextra-crypto` | `CryptoError` | 21 | 18 | `crates/contextra-crypto/src/error.rs` |
| `contextra-crypto` | `VerificationError` | 14 | 16 | `crates/contextra-crypto/src/deletion_proof/verifier.rs` |
| `contextra-db` | `VaultError` | 154 | 5 | `crates/contextra-db/src/volatile_vault.rs` |
| `contextra-erasure` | `ErasureError` | 7 | 21 | `crates/contextra-erasure/src/error.rs` |
| `contextra-erasure` | `SignerError` | 1073 | 3 | `crates/contextra-erasure/src/types_local.rs` |
| `contextra-erasure` | `ObserverError` | 1153 | 3 | `crates/contextra-erasure/src/types_local.rs` |
| `contextra-graph` | `HyperedgeSuggestError` | 45 | 1 | `crates/contextra-graph/src/hyperedge_suggest.rs` |
| `contextra-graph` | `TlHfdError` | 8 | 2 | `crates/contextra-graph/src/tl_hfd/error.rs` |
| `contextra-graph` | `ApprhError` | 8 | 3 | `crates/contextra-graph/src/apprh/error.rs` |
| `contextra-ingest` | `IngestError` | 52 | 14 | `crates/contextra-ingest/src/error.rs` |
| `contextra-license` | `StepVerificationError` | 87 | 7 | `crates/contextra-license/src/signed_gate.rs` |
| `contextra-mcp` | `ProofKeyError` | 33 | 1 | `crates/contextra-mcp/src/proof_key.rs` |
| `contextra-mcp` | `McpError` | 15 | 5 | `crates/contextra-mcp/src/protocol.rs` |
| `contextra-ports` | `EmbeddingError` | 65 | 5 | `crates/contextra-ports/src/embedding.rs` |
| `contextra-ports` | `GraphMutationError` | 93 | 12 | `crates/contextra-ports/src/graph.rs` |
| `contextra-ports` | `LicenseError` | 55 | 3 | `crates/contextra-ports/src/license.rs` |
| `contextra-ports` | `ObserverError` | 62 | 3 | `crates/contextra-ports/src/observer.rs` |
| `contextra-ports` | `PluginError` | 68 | 3 | `crates/contextra-ports/src/plugin.rs` |
| `contextra-ports` | `SignerError` | 30 | 3 | `crates/contextra-ports/src/report_signer.rs` |
| `contextra-privacy` | `ContextEditAuditError` | 40 | 1 | `crates/contextra-privacy/src/context_edit_audit.rs` |
| `contextra-privacy` | `EgressVaultError` | 237 | 7 | `crates/contextra-privacy/src/egress_vault.rs` |
| `contextra-privacy` | `EgressError` | 17 | 7 | `crates/contextra-privacy/src/error.rs` |
| `contextra-rank` | `ConformalError` | 15 | 5 | `crates/contextra-rank/src/calibration/conformal.rs` |
| `contextra-router` | `ArmRegistryError` | 34 | 1 | `crates/contextra-router/src/arm_registry.rs` |
| `contextra-router` | `FcTsDispatchError` | 21 | 2 | `crates/contextra-router/src/fc_ts_dispatch.rs` |
| `contextra-sandbox` | `ApprovalTransitionError` | 61 | 7 | `crates/contextra-sandbox/src/approval.rs` |
| `contextra-sandbox` | `SandboxError` | 7 | 10 | `crates/contextra-sandbox/src/error.rs` |
| `contextra-store` | `DurabilityConfigError` | 25 | 4 | `crates/contextra-store/src/lsm/config.rs` |
| `contextra-store` | `WalParseError` | 12 | 9 | `crates/contextra-store/src/wal/encode.rs` |
| `contextra-types` | `HnswDeletionError` | 38 | 4 | `crates/contextra-types/src/error.rs` |
| `contextra-types` | `ContextraError` | 129 | 69 | `crates/contextra-types/src/error.rs` |
| `contextra-vector` | `AcornError` | 30 | 7 | `crates/contextra-vector/src/acorn/mod.rs` |
| `contextra-wire` | `FlatBufferError` | 34 | 4 | `crates/contextra-wire/src/verification.rs` |

## Detaillierte Befunde
| Crate | Datei | Zeile | Art / Beschreibung |
|---|---|---|---|
| `contextra` | `crates/contextra/src/agent_memory.rs` | `84` | impl From<String> for MemoryId |
| `contextra` | `crates/contextra/src/agent_memory.rs` | `137` | impl From<SearchResult> for Memory |
| `contextra` | `crates/contextra/src/collection_profile.rs` | `92` | Error enum `CollectionProfileError` with 2 variants |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `51` | Error enum `DurabilityConfigError` with 4 variants |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `126` | Error enum `PerformanceProfileError` with 3 variants |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `60` | impl From<DurabilityConfigError> for contextra_types::ContextraError |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `332` | impl From<DurabilityMode> for contextra_db::DurabilityMode |
| `contextra` | `crates/contextra/src/performance_profile.rs` | `342` | impl From<contextra_db::DurabilityMode> for DurabilityMode |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `36` | Error enum `BanditError` with 8 variants |
| `contextra-adapt` | `crates/contextra-adapt/src/bandit.rs` | `61` | impl From<BanditError> for contextra_types::ContextraError |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `29` | Error enum `FcTsError` with 5 variants |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `713` | Error enum `OffPolicyError` with 1 variants |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `46` | impl From<FcTsError> for contextra_types::ContextraError |
| `contextra-adapt` | `crates/contextra-adapt/src/flow_thompson.rs` | `719` | impl From<OffPolicyError> for contextra_types::ContextraError |
| `contextra-adapt` | `crates/contextra-adapt/src/rie_greedy.rs` | `12` | Error enum `RieGreedyError` with 5 variants |
| `contextra-adapt` | `crates/contextra-adapt/src/rie_greedy.rs` | `29` | impl From<RieGreedyError> for contextra_types::ContextraError |
| `contextra-audit-export` | `crates/contextra-audit-export/src/error.rs` | `7` | Error enum `AuditExportError` with 5 variants |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/error.rs` | `32` | Error enum `AvvError` with 11 variants |
| `contextra-crypto` | `crates/contextra-crypto/src/ed25519_proof.rs` | `29` | Error enum `DeletionProofError` with 3 variants |
| `contextra-crypto` | `crates/contextra-crypto/src/ed25519_proof.rs` | `157` | impl From<SignatureVersion> for u8 |
| `contextra-crypto` | `crates/contextra-crypto/src/error.rs` | `21` | Error enum `CryptoError` with 18 variants |
| `contextra-crypto` | `crates/contextra-crypto/src/error.rs` | `85` | impl From<CryptoError> for contextra_types::ContextraError |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/verifier.rs` | `14` | Error enum `VerificationError` with 16 variants |
| `contextra-db` | `crates/contextra-db/src/volatile_vault.rs` | `154` | Error enum `VaultError` with 5 variants |
| `contextra-engine` | `crates/contextra-engine/src/collection/query_builder/strategy.rs` | `71` | impl From<GraphTraversalStrategy> for SearchStrategy |
| `contextra-engine` | `crates/contextra-engine/src/collection/query_builder/weights.rs` | `33` | impl From<SignalWeights> for FusionWeights |
| `contextra-engine` | `crates/contextra-engine/src/collection/query_builder/weights.rs` | `39` | impl From<FusionWeights> for SignalWeights |
| `contextra-erasure` | `crates/contextra-erasure/src/error.rs` | `7` | Error enum `ErasureError` with 21 variants |
| `contextra-erasure` | `crates/contextra-erasure/src/types_local.rs` | `1073` | Error enum `SignerError` with 3 variants |
| `contextra-erasure` | `crates/contextra-erasure/src/types_local.rs` | `1153` | Error enum `ObserverError` with 3 variants |
| `contextra-graph` | `crates/contextra-graph/src/arc_slice.rs` | `123` | impl From<Arc> for ArcSlice |
| `contextra-graph` | `crates/contextra-graph/src/arc_slice.rs` | `130` | impl From<Vec> for ArcSlice |
| `contextra-graph` | `crates/contextra-graph/src/hyperedge_suggest.rs` | `45` | Error enum `HyperedgeSuggestError` with 1 variants |
| `contextra-graph` | `crates/contextra-graph/src/hyperedge_suggest.rs` | `51` | impl From<HyperedgeSuggestError> for GraphMutationError |
| `contextra-graph` | `crates/contextra-graph/src/tl_hfd/error.rs` | `8` | Error enum `TlHfdError` with 2 variants |
| `contextra-graph` | `crates/contextra-graph/src/tl_hfd/error.rs` | `18` | impl From<TlHfdError> for contextra_types::ContextraError |
| `contextra-graph` | `crates/contextra-graph/src/apprh/error.rs` | `8` | Error enum `ApprhError` with 3 variants |
| `contextra-graph` | `crates/contextra-graph/src/apprh/error.rs` | `22` | impl From<ApprhError> for contextra_types::ContextraError |
| `contextra-ingest` | `crates/contextra-ingest/src/error.rs` | `52` | Error enum `IngestError` with 14 variants |
| `contextra-ingest` | `crates/contextra-ingest/src/error.rs` | `85` | impl From<IngestError> for ContextraError |
| `contextra-license` | `crates/contextra-license/src/signed_gate.rs` | `87` | Error enum `StepVerificationError` with 7 variants |
| `contextra-license` | `crates/contextra-license/src/signed_gate.rs` | `288` | impl From<LegacyLicensePayload> for LicensePayload |
| `contextra-mcp` | `crates/contextra-mcp/src/egress_gateway.rs` | `11` | impl From<EgressError> for McpError |
| `contextra-mcp` | `crates/contextra-mcp/src/proof_key.rs` | `33` | Error enum `ProofKeyError` with 1 variants |
| `contextra-mcp` | `crates/contextra-mcp/src/protocol.rs` | `15` | Error enum `McpError` with 5 variants |
| `contextra-mcp` | `crates/contextra-mcp/src/protocol.rs` | `60` | impl From<contextra_types::ContextraError> for McpError |
| `contextra-mcp` | `crates/contextra-mcp/src/protocol.rs` | `70` | impl From<String> for McpError |
| `contextra-ports` | `crates/contextra-ports/src/embedding.rs` | `65` | Error enum `EmbeddingError` with 5 variants |
| `contextra-ports` | `crates/contextra-ports/src/graph.rs` | `93` | Error enum `GraphMutationError` with 12 variants |
| `contextra-ports` | `crates/contextra-ports/src/graph.rs` | `30` | impl From<u32> for RoleId |
| `contextra-ports` | `crates/contextra-ports/src/graph.rs` | `79` | impl From<u64> for HyperEdgeId |
| `contextra-ports` | `crates/contextra-ports/src/graph.rs` | `139` | impl From<GraphMutationError> for ContextraError |
| `contextra-ports` | `crates/contextra-ports/src/license.rs` | `55` | Error enum `LicenseError` with 3 variants |
| `contextra-ports` | `crates/contextra-ports/src/observer.rs` | `62` | Error enum `ObserverError` with 3 variants |
| `contextra-ports` | `crates/contextra-ports/src/plugin.rs` | `68` | Error enum `PluginError` with 3 variants |
| `contextra-ports` | `crates/contextra-ports/src/plugin.rs` | `82` | impl From<PluginError> for crate::ContextraError |
| `contextra-ports` | `crates/contextra-ports/src/report_signer.rs` | `30` | Error enum `SignerError` with 3 variants |
| `contextra-ports` | `crates/contextra-ports/src/scanner/layer.rs` | `63` | impl From<String> for WitnessLayer |
| `contextra-privacy` | `crates/contextra-privacy/src/bulk_exfiltration_detector.rs` | `30` | impl From<String> for SessionId |
| `contextra-privacy` | `crates/contextra-privacy/src/context_edit_audit.rs` | `40` | Error enum `ContextEditAuditError` with 1 variants |
| `contextra-privacy` | `crates/contextra-privacy/src/egress_vault.rs` | `237` | Error enum `EgressVaultError` with 7 variants |
| `contextra-privacy` | `crates/contextra-privacy/src/error.rs` | `17` | Error enum `EgressError` with 7 variants |
| `contextra-privacy` | `crates/contextra-privacy/src/error.rs` | `45` | impl From<crate::egress_vault::EgressVaultError> for EgressError |
| `contextra-rank` | `crates/contextra-rank/src/calibration/conformal.rs` | `15` | Error enum `ConformalError` with 5 variants |
| `contextra-router` | `crates/contextra-router/src/arm_registry.rs` | `34` | Error enum `ArmRegistryError` with 1 variants |
| `contextra-router` | `crates/contextra-router/src/fc_ts_dispatch.rs` | `21` | Error enum `FcTsDispatchError` with 2 variants |
| `contextra-sandbox` | `crates/contextra-sandbox/src/approval.rs` | `61` | Error enum `ApprovalTransitionError` with 7 variants |
| `contextra-sandbox` | `crates/contextra-sandbox/src/error.rs` | `7` | Error enum `SandboxError` with 10 variants |
| `contextra-store` | `crates/contextra-store/src/lsm/config.rs` | `25` | Error enum `DurabilityConfigError` with 4 variants |
| `contextra-store` | `crates/contextra-store/src/lsm/config.rs` | `34` | impl From<DurabilityConfigError> for contextra_types::ContextraError |
| `contextra-store` | `crates/contextra-store/src/wal/encode.rs` | `12` | Error enum `WalParseError` with 9 variants |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `38` | Error enum `HnswDeletionError` with 4 variants |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `129` | Error enum `ContextraError` with 69 variants |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `81` | impl From<u64> for StepId |
| `contextra-types` | `crates/contextra-types/src/error.rs` | `499` | impl From<std::array::TryFromSliceError> for ContextraError |
| `contextra-types` | `crates/contextra-types/src/error_dto.rs` | `75` | impl From<ContextraError> for ContextraErrorDto |
| `contextra-types` | `crates/contextra-types/src/error_dto.rs` | `81` | impl From<String> for ContextraErrorDto |
| `contextra-types` | `crates/contextra-types/src/types/saos.rs` | `62` | impl From<FusionStrategy> for SignalFusionStrategies |
| `contextra-types` | `crates/contextra-types/src/types/domain/ids.rs` | `152` | impl From<u64> for CollectionId |
| `contextra-types` | `crates/contextra-types/src/types/domain/ids.rs` | `277` | impl From<u64> for DocId |
| `contextra-types` | `crates/contextra-types/src/types/domain/ids.rs` | `284` | impl From<u128> for DocId |
| `contextra-types` | `crates/contextra-types/src/types/domain/ids.rs` | `366` | impl From<u64> for EntityId |
| `contextra-types` | `crates/contextra-types/src/types/domain/ids.rs` | `401` | impl From<String> for EntityId |
| `contextra-vector` | `crates/contextra-vector/src/acorn/mod.rs` | `30` | Error enum `AcornError` with 7 variants |
| `contextra-wire` | `crates/contextra-wire/src/verification.rs` | `34` | Error enum `FlatBufferError` with 4 variants |

---
*Automatisch generiert durch `error_topology_mapper.py` für Agentic Engineering & Context Auditing.*
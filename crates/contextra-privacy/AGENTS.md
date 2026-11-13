# AGENTS.md — `contextra-privacy`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-privacy` · **Ring:** – · **Version:** 0.1.0
- **Beschreibung:** Cloud Egress Security, DLP and Exfiltration Protection for Contextra
- **src:** 13 Dateien, 4151 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 0):** –
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 11):** `blake3`, `parking_lot`, `rand`, `regex`, `scc`, `serde`, `serde_json`, `thiserror`, `tokio`, `tracing`, `zeroize`
- **Reverse-Abhängigkeiten (1):** `contextra-mcp`
- **Kopplung an Hub-Crates:** ausgehend –; eingehend –
- **Features (4):** `default`, `serde`, `std`, `tracing`

### Öffentliche API (grob, aus src/)
- struct (30): `BulkExfiltrationDetector`, `CloudQueryRequest`, `CloudQueryResponse`, `CloudResponseRehydrator`, `CompiledPattern`, `ContextEditAuditRecord`, `ContextEditInput`, `DefaultEgressGuard`, `DeterministicBlake3Hasher`, `DeterministicBuildHasher`, `EgressGateway`, `EgressGatewayBuilder`, `EgressGuard`, `EgressVault`, `EntityMatch` … (+15)
- enum (11): `BlockReason`, `BulkExfiltrationOutcome`, `ContextEditAuditError`, `ContextEditKind`, `EgressClassification`, `EgressDecision`, `EgressVaultError`, `PolicyCategory`, `PrivacyError`, `ProcessorRole`, `TenantScopeViolation`
- trait (8): `Clock`, `EgressClassifier`, `EgressClassifierTrace`, `EgressGuardCheck`, `EgressPolicy`, `EntityRecognizer`, `InjectionDetector`, `TextSearchEngine`
- fn (91): `add_gazetteer_term`, `add_term`, `append_record`, `as_str`, `as_u64`, `build`, `build_context_edit_audit_record`, `builder`, `capacity_exceeded`, `category`, `check`, `check_bulk_exfiltration`, `check_scoped`, `classify_layer1`, `classify_layer1_arc` … (+76)
- type (6): `BoxFuture`, `DefaultEgressClassifier`, `EgressError`, `Error`, `PatternRecognizer`, `Result`
- const (8): `DEFAULT_EGRESS_GUARD_MIN_BYTES`, `DEFAULT_EGRESS_GUARD_THRESHOLD`, `DEFAULT_EGRESS_GUARD_TIMEOUT`, `DEFAULT_MAX_INPUT_LENGTH`, `DEFAULT_TIMEOUT`, `MAX_CLASSIFY_PAYLOAD_BYTES`, `MAX_SEARCH_QUERY_BYTES`, `SYSTEM`
- mod (12): `audit_trace`, `bulk_exfiltration_detector`, `context_edit_audit`, `contract`, `egress_gateway`, `egress_guard`, `egress_vault`, `entity_recognizer`, `error`, `guarded_payload`, `processing_registry`, `types`
- pub use: `audit_trace::{compute_audit_trace, extract_rule_id, EgressClassifierTrace}`, `bulk_exfiltration_detector::{ BulkExfiltrationDetector, BulkExfiltrationOutcome, SessionId, }`, `context_edit_audit::{ build_context_edit_audit_record, compute_record_hash, render_audit_line, verify_audit_chain, ContextEditAuditError, ContextEditAuditRecord, ContextEditKind, }`, `contract::{Clock, EgressDecision, EgressPolicy}`, `crate::egress_guard::TextSearchEngine`, `crate::egress_vault::EntityRecognizer`, `crate::types::{Clock, EgressDecision, EgressPolicy}`, `egress_gateway::{ handle_cloud_query, handle_cloud_query_with_bulk_detector, handle_cloud_query_with_guard, pii_vault_forces_crypto_shred, pii_vault_requires_isolated_retention, process_cloud_response, resolve_effective_kv_delete_mode, resolve_effective_retention_mode, CloudQueryRequest, CloudQueryResponse, CloudResponseRehydrator, DefaultEgressClassifier, EgressGateway, EgressGatewayBuilder, EgressGuardCheck, InjectionDetector, MAX_SEARCH_QUERY_BYTES, }`, `egress_guard::{ EgressGuard, TextSearchEngine, TextSearchResult, DEFAULT_EGRESS_GUARD_MIN_BYTES, DEFAULT_EGRESS_GUARD_THRESHOLD, DEFAULT_EGRESS_GUARD_TIMEOUT, }`, `egress_vault::NoOpRecognizer` … (+6)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->`.

<!-- END GENERATED: crate_inventory.py -->

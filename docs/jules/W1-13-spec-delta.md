# SPEC-DELTA: SESSION W1-13 — Erasure-Ledger, Restore-Guard und Checkpoint-Scrub (Anti-Resurrection)

## 1. GEÄNDERTE APIS & NEUE TYPEN

### `contextra-store`
- `ErasureLedger::open_or_fail_closed_with_instance_id(dir, key, db_has_data, expected_head_mac, db_instance_id)`:
  Additive Öffnungsmethode, die ein injiziertes `db_instance_id: [u8; 16]` entgegennimmt und gegen den `ledger.head`-Marker abgleicht.
  `open_or_fail_closed` ist als `#[deprecated]` markiert.
- `ErasureLedger::append`: Hardening mit idempotenter `keyed_hash`-Deduplizierung (neben `proof_id`).
- `AntiResurrection`: Enum `{ Guaranteed { tombstones_reapplied: usize }, NotGuaranteed { reason: &'static str } }`.
- `ErasureLedger::reapply_tombstones_from_ledger(&self, storage_scope, truncation_anchor_mac)` & `ErasureLedgerHandle::reapply_tombstones_from_ledger`:
  Liefert Beschleuniger über `keyed_hash` und schlägt mit `AntiResurrection::NotGuaranteed` fehl, falls der Truncation-Anchor unvollständig/ungültig ist.
- `LayerScan` Implementation für `ErasureLedger`: Layer ID 12, führt Scans über den `PhysicalScope` aus und prüft ausschließlich keyed hashes (kein Klartext-Zugriff).

### `contextra-checkpoint`
- `RestorePolicy`: Enum `{ Filter(Arc<dyn CpyRestoreFilter>), ForbidAfterErasure }`.
- `NoOpRestoreFilter`: Nur unter `cfg(any(test, feature = "test-utils"))` verfügbar.
- `PersistentCheckpointStore`: Feature-gating `feature = "checkpoint"` für Scrubbing-APIs (`scrub_collection_checkpoints`, `scrub_tenant_checkpoints`, `purge_checkpoint_by_name`), die bei deaktiviertem Feature `ContextraError::CapabilityUnsupported` zurückgeben.

## 2. GEHÄRTETE INVARIANTEN
- **INV-ERA-1 / E-4**: Abschneiden (Truncation) ohne verifizierten Anchor garantiert keine Wiederherstellungssicherheit (`NotGuaranteed`).
- **INV-ERA-3 / X-16**: Checkpoints/Kopien gelöschter Daten werden über die Scrubbing-API scrubbbar gemacht.
- **INV-PRIVACY**: Klartext-Schlüssel werden zu keinem Zeitpunkt im ErasureLedger auf der Disk abgelegt.
- **FAIL-CLOSED RESTORE**: Ein Restore nach Löschungen ohne `CpyRestoreFilter` scheitert fail-closed.

## 3. SPEZIELL BENANNTE TESTS
- `ledger_append_idempotent_same_keyed_hash`
- `ledger_privacy_no_plain_keys_on_disk`
- `ledger_append_dir_sync_error_propagates`
- `reapply_tombstones_accelerated_matches_naive`
- `truncation_without_anchor_reports_not_guaranteed`
- `ledger_source_both_none_unrepresentable` (trybuild compile-fail)
- `checkpoint_scrub_disabled_reports_unsupported`
- `checkpoint_scrub_removes_copies_raw_sweep`
- `restore_without_filter_after_erasure_fails_closed`
- `delete_then_restore_keeps_deleted_deleted`
- `restore_rejects_key_erased_after_checkpoint_tx`
- `ledger_scanner_uses_only_keyed_hashes`

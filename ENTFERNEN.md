# Entfernungsliste: Altpfade und SPEC-widrige Implementierungen

> Erzeugt aus dem Quellcode am 2026-11-25 (HEAD `ce4579a56`), per Skript, nicht von Hand. Jede Zeile ist ein Element (Funktion, Typ, Trait, Impl oder Modulebene), das mindestens einen Verstoß enthält. Testcode (`tests`, `testing`, `test_doubles`) ist ausgeschlossen. Nicht kompiliert.

Bezug: SPEC R14 §2.6, §2.7, §13.3, DEC-41 bis DEC-53.

## 1. Vollständig zu löschende Dateien und Verzeichnisse

Diese Einheiten enthalten ausschließlich Altpfade. Ihre Verstöße sind in Abschnitt 2 nicht aufgeführt. Vor dem Löschen: Verwender prüfen (`grep -rn`), siehe Spalte „Verwender“.

| Crate | Datei/Verzeichnis | Zeilen | Quelle der Entscheidung |
|---|---|---:|---|
| contextra-checkpoint | `src/contextra_ports.rs` | 185 | Stub (R14 §13.2 Nr. 4) |
| contextra-checkpoint | `src/contextra_types.rs` | 120 | Stub (R14 §13.2 Nr. 4) |
| contextra-checkpoint | `src/hardlink_cloner.rs` | 292 | Altpfad §13.3 |
| contextra-mvcc | `src/contextra_ports.rs` | 91 | Stub (R14 §13.2 Nr. 4) |
| contextra-mvcc | `src/contextra_types.rs` | 273 | Stub (R14 §13.2 Nr. 4) |
| contextra-mvcc | `src/seq_log.rs` | 836 | Altpfad §13.3 (TxId/SequenceLog) |
| contextra-mvcc | `src/ssi.rs` | 1206 | NZ-3, §13.3 |
| contextra-mvcc | `src/tx_buffer.rs` | 1435 | Altpfad §13.3 |
| contextra-ports | `src/kv.rs` | 261 | NZ-18, DEC-38 |
| contextra-ports | `src/kv_bridge_port.rs` | 23 | NZ-18 |
| contextra-ports | `src/attention.rs` | 21 | NZ-18 |
| contextra-ports | `src/inference.rs` | 17 | NZ-23 |
| contextra-ports | `src/vfs.rs` | 404 | Ersatz durch Fs (§13.3) |
| contextra-ports | `src/erasure.rs` | 272 | Altpfad §13.3 (StorageBackend/ErasureLayer) |
| contextra-ports | `src/scanner/layer.rs` | 123 | §13.3 |
| contextra-ports | `src/scanner/mod.rs` | 210 | §13.3 |
| contextra-crypto | `src/kv_cipher.rs` | 405 | §13.3, DEC-04 |
| contextra-crypto | `src/kv_shredding.rs` | 613 | §13.3 |
| contextra-graph | `src/session_dag.rs` | 1008 | NZ-22, §13.3 |
| contextra-graph | `src/edge_reinforcement.rs` | 170 | NZ-22, §18 |
| contextra-graph | `src/edge_reinforcement_buffer.rs` | 218 | NZ-22, §18 |
| contextra-graph | `src/purger.rs` | 125 | §13.2 Nr. 9 (contextra-erasure-spi) |
| contextra-graph | `src/entity_extraction.rs` | 318 | NZ-23 (Extraktion nicht im Graph) |
| contextra-engine | `src/decay_controller.rs` | 192 | NZ-22 |
| contextra-engine | `src/adapters/ollama.rs` | 10 | NZ-23 |
| contextra-engine | `src/collection/kv_lock.rs` | 278 | §13.2 Nr. 7 |
| contextra-crypto | `src/kv_segment` | 1671 | §13.3 (kv_segment) |
| contextra-engine | `src/orchestration` | 330 | NZ-21, §13.2 Nr. 7 |
| contextra-engine | `src/transaction` | 1652 | DEC-19, §13.3 (2PC) |
| contextra-engine | `src/contextra_impl` | 1434 | Parallel-Fassade, Inhalt verteilen (Plan engine) |

## 2. Funktionen, Typen und Impl-Blöcke mit Verstoß (Entfernen oder Ändern)

Spalte „Aktion“: **Entfernen** = Element ist ein Altpfad oder verbotene Implementierung ohne Ersatz im Ziel-Zustand. **Ändern** = Element bleibt, verliert aber den Verstoß (Import, Abhängigkeit, Konstante, Adapter).

### contextra-checkpoint (133 Einträge)

| Datei | Element | Art | Kategorie | Treffer | Erste Zeile | Aktion |
|---|---|---|---|---:|---:|---|
| `src/guard.rs` | `—` | (Modulebene) | Altpfad-Checkpoint | 1 | 20 | Entfernen |
| `src/guard.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 23 | Entfernen/Ändern |
| `src/guard.rs` | `PinGuard` | struct | Altpfad-Port | 1 | 41 | Entfernen (legacy) |
| `src/guard.rs` | `PinGuard<S>` | impl | Altpfad-Port | 1 | 48 | Entfernen (legacy) |
| `src/guard.rs` | `pin_with_clock` | fn | Altpfad-Checkpoint | 1 | 81 | Entfernen |
| `src/guard.rs` | `Drop` | impl | Altpfad-Port | 2 | 119 | Entfernen (legacy) |
| `src/guard.rs` | `CheckpointGuard` | struct | Altpfad-Port | 1 | 152 | Entfernen (legacy) |
| `src/guard.rs` | `CheckpointGuard` | struct | Altpfad-Checkpoint | 1 | 153 | Entfernen |
| `src/guard.rs` | `CheckpointGuard<S>` | impl | Altpfad-Port | 1 | 163 | Entfernen (legacy) |
| `src/guard.rs` | `new` | fn | Altpfad-Checkpoint | 1 | 165 | Entfernen |
| `src/guard.rs` | `with_clock` | fn | Altpfad-Checkpoint | 1 | 172 | Entfernen |
| `src/guard.rs` | `with_registry` | fn | Altpfad-Checkpoint | 1 | 193 | Entfernen |
| `src/guard.rs` | `with_registry_and_counter` | fn | Altpfad-Checkpoint | 1 | 212 | Entfernen |
| `src/guard.rs` | `with_registry_counter_and_clock` | fn | Altpfad-Checkpoint | 1 | 231 | Entfernen |
| `src/guard.rs` | `for_agent_step` | fn | Altpfad-Typ | 1 | 273 | Entfernen/Ändern |
| `src/guard.rs` | `for_agent_step_with_registry` | fn | Altpfad-Typ | 1 | 281 | Entfernen/Ändern |
| `src/guard.rs` | `for_agent_step_with_registry` | fn | Altpfad-Checkpoint | 1 | 285 | Entfernen |
| `src/guard.rs` | `for_agent_step_with_clock` | fn | Altpfad-Typ | 1 | 301 | Entfernen/Ändern |
| `src/guard.rs` | `for_agent_step_with_clock` | fn | Altpfad-Checkpoint | 1 | 308 | Entfernen |
| `src/guard.rs` | `checkpoint` | fn | Altpfad-Checkpoint | 1 | 317 | Entfernen |
| `src/guard.rs` | `commit` | fn | Altpfad-Checkpoint | 1 | 331 | Entfernen |
| `src/guard.rs` | `commit` | fn | Verboten-Laufzeit | 1 | 350 | Ändern (kein Async/Lock im L2) |
| `src/guard.rs` | `rollback` | fn | Altpfad-Typ | 2 | 392 | Entfernen/Ändern |
| `src/guard.rs` | `rollback_blocking` | fn | Verboten-Laufzeit | 3 | 419 | Ändern (kein Async/Lock im L2) |
| `src/guard.rs` | `rollback_blocking` | fn | Altpfad-Typ | 1 | 457 | Entfernen/Ändern |
| `src/guard.rs` | `tests` | mod | Altpfad-Port | 1 | 514 | Entfernen (legacy) |
| `src/guard.rs` | `tests` | mod | Verboten-Laufzeit | 1 | 515 | Ändern (kein Async/Lock im L2) |
| `src/guard.rs` | `MockStorage` | struct | Altpfad-Typ | 1 | 521 | Entfernen/Ändern |
| `src/guard.rs` | `StorageEngine` | impl | Altpfad-Port | 1 | 534 | Entfernen (legacy) |
| `src/guard.rs` | `put` | fn | Altpfad-Typ | 1 | 547 | Entfernen/Ändern |
| `src/guard.rs` | `delete` | fn | Altpfad-Typ | 1 | 556 | Entfernen/Ändern |
| `src/guard.rs` | `commit` | fn | Altpfad-Typ | 1 | 562 | Entfernen/Ändern |
| `src/guard.rs` | `rollback_to_tx` | fn | Altpfad-Typ | 1 | 568 | Entfernen/Ändern |
| `src/guard.rs` | `last_tx_id` | fn | Altpfad-Typ | 2 | 577 | Entfernen/Ändern |
| `src/guard.rs` | `pin_checkpoint` | fn | Altpfad-Checkpoint | 1 | 592 | Entfernen |
| `src/guard.rs` | `test_orphan_registry_persists_across_drop` | fn | Verboten-Laufzeit | 2 | 634 | Ändern (kein Async/Lock im L2) |
| `src/guard.rs` | `test_rollback_blocking_in_sync_context` | fn | Altpfad-Checkpoint | 1 | 662 | Entfernen |
| `src/guard.rs` | `test_rollback_blocking_in_sync_context` | fn | Altpfad-Typ | 2 | 663 | Entfernen/Ändern |
| `src/guard.rs` | `test_rollback_blocking_in_sync_context` | fn | Verboten-Laufzeit | 1 | 679 | Ändern (kein Async/Lock im L2) |
| `src/guard.rs` | `test_rollback_blocking_in_async_context_returns_error` | fn | Altpfad-Checkpoint | 1 | 682 | Entfernen |
| `src/guard.rs` | `test_rollback_blocking_in_async_context_returns_error` | fn | Altpfad-Typ | 1 | 683 | Entfernen/Ändern |
| `src/guard.rs` | `test_rollback_blocking_in_async_context_returns_error` | fn | Verboten-Laufzeit | 2 | 690 | Ändern (kein Async/Lock im L2) |
| `src/guard.rs` | `test_checkpoint_guard_for_agent_step` | fn | Altpfad-Typ | 3 | 714 | Entfernen/Ändern |
| `src/guard.rs` | `test_checkpoint_guard_for_agent_step` | fn | Verboten-Laufzeit | 1 | 728 | Ändern (kein Async/Lock im L2) |
| `src/guard.rs` | `checkpoint_guard_CASE_uncommitted_guard_holds_state` | fn | Altpfad-Checkpoint | 1 | 731 | Entfernen |
| `src/guard.rs` | `checkpoint_guard_CASE_uncommitted_guard_holds_state` | fn | Altpfad-Typ | 3 | 732 | Entfernen/Ändern |
| `src/guard.rs` | `checkpoint_guard_CASE_uncommitted_guard_holds_state` | fn | Verboten-Laufzeit | 1 | 747 | Ändern (kein Async/Lock im L2) |
| `src/guard.rs` | `checkpoint_guard_CASE_commit_moves_ownership` | fn | Altpfad-Checkpoint | 1 | 751 | Entfernen |
| `src/guard.rs` | `checkpoint_guard_CASE_commit_moves_ownership` | fn | Altpfad-Typ | 2 | 752 | Entfernen/Ändern |
| `src/guard.rs` | `checkpoint_guard_CASE_commit_moves_ownership` | fn | Verboten-Laufzeit | 1 | 769 | Ändern (kein Async/Lock im L2) |
| `src/guard.rs` | `checkpoint_guard_CASE_rollback_consumed_returns_err` | fn | Verboten-Laufzeit | 1 | 818 | Ändern (kein Async/Lock im L2) |
| `src/guard.rs` | `test_checkpoint_metrics_commit_duration` | fn | Altpfad-Checkpoint | 1 | 825 | Entfernen |
| `src/guard.rs` | `test_checkpoint_metrics_commit_duration` | fn | Altpfad-Typ | 2 | 826 | Entfernen/Ändern |
| `src/guard.rs` | `test_checkpoint_metrics_commit_duration` | fn | Verboten-Laufzeit | 1 | 853 | Ändern (kein Async/Lock im L2) |
| `src/guard.rs` | `test_checkpoint_metrics_drop_rollback_duration` | fn | Altpfad-Checkpoint | 1 | 860 | Entfernen |
| `src/guard.rs` | `test_checkpoint_metrics_drop_rollback_duration` | fn | Altpfad-Typ | 1 | 861 | Entfernen/Ändern |
| `src/lib.rs` | `contextra_types` | mod | Altpfad-Typ | 1 | 38 | Entfernen/Ändern |
| `src/lib.rs` | `contextra_types` | mod | Altpfad-Checkpoint | 1 | 38 | Entfernen |
| `src/lib.rs` | `hardlink_cloner` | mod | Altpfad-Checkpoint | 1 | 43 | Entfernen |
| `src/lib.rs` | `store` | mod | Altpfad-Checkpoint | 2 | 51 | Entfernen |
| `src/manifest.rs` | `new` | fn | Verboten-Hash | 1 | 69 | Ändern (Ersatz CRC32C/Mac) |
| `src/manifest.rs` | `verify` | fn | Verboten-Hash | 1 | 91 | Ändern (Ersatz CRC32C/Mac) |
| `src/manifest.rs` | `tests` | mod | Altpfad-Typ | 1 | 107 | Entfernen/Ändern |
| `src/manifest.rs` | `test_manifest_validation_blank_component` | fn | Altpfad-Typ | 1 | 116 | Entfernen/Ändern |
| `src/manifest.rs` | `test_checkpoint_manifest_CASE_whitespace_component_rejected` | fn | Altpfad-Typ | 1 | 133 | Entfernen/Ändern |
| `src/manifest.rs` | `test_checkpoint_manifest_CASE_tampered_manifest_fails_verify` | fn | Altpfad-Typ | 1 | 149 | Entfernen/Ändern |
| `src/manifest.rs` | `test_manifest_creation_invalid_meta_name` | fn | Altpfad-Typ | 1 | 167 | Entfernen/Ändern |
| `src/manifest.rs` | `test_prop_manifest_roundtrip` | fn | Altpfad-Typ | 3 | 195 | Entfernen/Ändern |
| `src/meta.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 14 | Entfernen/Ändern |
| `src/meta.rs` | `—` | (Modulebene) | Altpfad-Checkpoint | 1 | 14 | Entfernen |
| `src/meta.rs` | `CheckpointMeta` | struct | Altpfad-Typ | 1 | 68 | Entfernen/Ändern |
| `src/meta.rs` | `into_workflow_state` | fn | Altpfad-Checkpoint | 1 | 79 | Entfernen |
| `src/meta.rs` | `to_workflow_state` | fn | Altpfad-Checkpoint | 2 | 90 | Entfernen |
| `src/meta.rs` | `to_workflow_state` | fn | Verboten-Hash | 1 | 94 | Ändern (Ersatz CRC32C/Mac) |
| `src/meta.rs` | `StateCheckpoint` | struct | Altpfad-Checkpoint | 1 | 111 | Entfernen |
| `src/meta.rs` | `StateCheckpoint` | struct | Altpfad-Typ | 1 | 112 | Entfernen/Ändern |
| `src/meta.rs` | `checkpoint_meta_CASE_serialization_roundtrip` | fn | Altpfad-Typ | 2 | 138 | Entfernen/Ändern |
| `src/meta.rs` | `state_checkpoint_CASE_serialization_roundtrip` | fn | Altpfad-Checkpoint | 2 | 160 | Entfernen |
| `src/meta.rs` | `state_checkpoint_CASE_serialization_roundtrip` | fn | Altpfad-Typ | 2 | 161 | Entfernen/Ändern |
| `src/meta.rs` | `state_checkpoint_deserializes_legacy_json_without_namespace` | fn | Altpfad-Checkpoint | 1 | 179 | Entfernen |
| `src/meta.rs` | `state_checkpoint_deserializes_legacy_json_without_namespace` | fn | Altpfad-Typ | 1 | 182 | Entfernen/Ändern |
| `src/meta.rs` | `into_workflow_state_CASE_valid_conversion` | fn | Altpfad-Typ | 2 | 195 | Entfernen/Ändern |
| `src/orphan.rs` | `—` | (Modulebene) | Altpfad-Checkpoint | 1 | 18 | Entfernen |
| `src/orphan.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 20 | Entfernen/Ändern |
| `src/orphan.rs` | `—` | (Modulebene) | Verboten-Laufzeit | 1 | 21 | Ändern (kein Async/Lock im L2) |
| `src/orphan.rs` | `RecoveryReport` | struct | Altpfad-Typ | 1 | 41 | Entfernen/Ändern |
| `src/orphan.rs` | `recover_and_clean` | fn | Altpfad-Port | 1 | 179 | Entfernen (legacy) |
| `src/orphan.rs` | `OrphanState` | struct | Altpfad-Checkpoint | 1 | 239 | Entfernen |
| `src/orphan.rs` | `persist_sync` | fn | Verboten-FS | 2 | 251 | Ändern (über sys/durable-fs) |
| `src/orphan.rs` | `load_sync` | fn | Verboten-FS | 2 | 266 | Ändern (über sys/durable-fs) |
| `src/orphan.rs` | `InstanceOrphanRegistry` | struct | Altpfad-Checkpoint | 1 | 303 | Entfernen |
| `src/orphan.rs` | `register_checkpoint_sync` | fn | Altpfad-Checkpoint | 1 | 457 | Entfernen |
| `src/orphan.rs` | `flush_orphan_registry` | fn | Verboten-Laufzeit | 2 | 492 | Ändern (kein Async/Lock im L2) |
| `src/orphan.rs` | `drain_orphaned_checkpoints` | fn | Altpfad-Checkpoint | 1 | 546 | Entfernen |
| `src/orphan.rs` | `get_orphaned_checkpoints` | fn | Altpfad-Checkpoint | 2 | 563 | Entfernen |
| `src/orphan.rs` | `clear_orphaned_checkpoint` | fn | Altpfad-Typ | 2 | 568 | Entfernen/Ändern |
| `src/orphan.rs` | `register_orphaned_checkpoint` | fn | Altpfad-Checkpoint | 1 | 599 | Entfernen |
| `src/orphan.rs` | `get_orphaned_checkpoints_for_namespace` | fn | Altpfad-Checkpoint | 1 | 618 | Entfernen |
| `src/orphan.rs` | `test_instance_orphan_registry_drain_pins_and_checkpoints` | fn | Altpfad-Checkpoint | 1 | 700 | Entfernen |
| `src/orphan.rs` | `test_instance_orphan_registry_drain_pins_and_checkpoints` | fn | Altpfad-Typ | 2 | 701 | Entfernen/Ändern |
| `src/orphan.rs` | `test_instance_orphan_registry_clear_nonexistent` | fn | Altpfad-Checkpoint | 1 | 728 | Entfernen |
| `src/orphan.rs` | `test_instance_orphan_registry_clear_nonexistent` | fn | Altpfad-Typ | 2 | 729 | Entfernen/Ändern |
| `src/orphan.rs` | `test_two_instance_registries_independent` | fn | Altpfad-Checkpoint | 1 | 792 | Entfernen |
| `src/orphan.rs` | `test_two_instance_registries_independent` | fn | Altpfad-Typ | 1 | 793 | Entfernen/Ändern |
| `src/store.rs` | `—` | (Modulebene) | Altpfad-Checkpoint | 4 | 22 | Entfernen |
| `src/store.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 32 | Entfernen/Ändern |
| `src/store.rs` | `—` | (Modulebene) | Verboten-Laufzeit | 1 | 33 | Ändern (kein Async/Lock im L2) |
| `src/store.rs` | `recover_orphaned_checkpoints` | fn | Altpfad-Typ | 3 | 63 | Entfernen/Ändern |
| `src/store.rs` | `determine_tx_counter_bounds` | fn | Altpfad-Port | 1 | 143 | Entfernen (legacy) |
| `src/store.rs` | `determine_tx_counter_bounds` | fn | Altpfad-Typ | 5 | 162 | Entfernen/Ändern |
| `src/store.rs` | `persist_hwm_internal` | fn | Altpfad-Port | 1 | 203 | Entfernen (legacy) |
| `src/store.rs` | `persist_hwm_internal` | fn | Altpfad-Typ | 1 | 214 | Entfernen/Ändern |
| `src/store.rs` | `PersistentCheckpointStore` | struct | Altpfad-Port | 1 | 232 | Entfernen (legacy) |
| `src/store.rs` | `PersistentCheckpointStore` | struct | Verboten-Laufzeit | 2 | 239 | Ändern (kein Async/Lock im L2) |
| `src/store.rs` | `PersistentCheckpointStore<S>` | impl | Altpfad-Port | 1 | 258 | Entfernen (legacy) |
| `src/store.rs` | `open_with_orphan_registry` | fn | Verboten-Laufzeit | 2 | 319 | Ändern (kein Async/Lock im L2) |
| `src/store.rs` | `new_with_orphan_registry` | fn | Verboten-Laufzeit | 8 | 356 | Ändern (kein Async/Lock im L2) |
| `src/store.rs` | `allocate_tx` | fn | Altpfad-Typ | 3 | 417 | Entfernen/Ändern |
| `src/store.rs` | `create_guard` | fn | Altpfad-Typ | 1 | 494 | Entfernen/Ändern |
| `src/store.rs` | `create_guard` | fn | Altpfad-Checkpoint | 1 | 496 | Entfernen |
| `src/store.rs` | `create_checkpoint` | fn | Altpfad-Typ | 1 | 526 | Entfernen/Ändern |
| `src/store.rs` | `restore_checkpoint` | fn | Altpfad-Typ | 1 | 774 | Entfernen/Ändern |
| `src/store.rs` | `register_orphaned_checkpoint` | fn | Altpfad-Checkpoint | 1 | 819 | Entfernen |
| `src/store.rs` | `get_orphaned_checkpoints` | fn | Altpfad-Checkpoint | 1 | 824 | Entfernen |
| `src/store.rs` | `clear_orphaned_checkpoint` | fn | Altpfad-Typ | 1 | 829 | Entfernen/Ändern |
| `src/store.rs` | `CheckpointRegistry` | impl | Altpfad-Port | 1 | 915 | Entfernen (legacy) |
| `src/store.rs` | `crate::contextra_ports::CheckpointCoordinator` | impl | Altpfad-Port | 1 | 955 | Entfernen (legacy) |
| `src/store.rs` | `create_named_checkpoint` | fn | Altpfad-Typ | 1 | 965 | Entfernen/Ändern |
| `src/store.rs` | `crate::contextra_ports::Checkpoint` | impl | Altpfad-Port | 1 | 985 | Entfernen (legacy) |
| `src/store.rs` | `take_snapshot` | fn | Altpfad-Typ | 1 | 988 | Entfernen/Ändern |
| `src/store.rs` | `take_snapshot` | fn | Altpfad-Checkpoint | 2 | 988 | Entfernen |
| `src/store.rs` | `take_snapshot` | fn | Verboten-Hash | 1 | 993 | Ändern (Ersatz CRC32C/Mac) |
| `src/store.rs` | `restore` | fn | Altpfad-Checkpoint | 1 | 998 | Entfernen |

### contextra-crypto (58 Einträge)

| Datei | Element | Art | Kategorie | Treffer | Erste Zeile | Aktion |
|---|---|---|---|---:|---:|---|
| `src/aead.rs` | `—` | (Modulebene) | Verboten-Crypto | 2 | 9 | Ändern (XChaCha20-Poly1305) |
| `src/aead.rs` | `seal_record_with_nonce` | fn | Verboten-Crypto | 1 | 25 | Ändern (XChaCha20-Poly1305) |
| `src/aead.rs` | `open_record` | fn | Verboten-Crypto | 1 | 92 | Ändern (XChaCha20-Poly1305) |
| `src/anti_tamper.rs` | `extend` | fn | Verboten-Hash | 1 | 112 | Ändern (Ersatz CRC32C/Mac) |
| `src/audit_chain.rs` | `—` | (Modulebene) | Altpfad-KV | 1 | 30 | Entfernen |
| `src/audit_chain.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 31 | Entfernen/Ändern |
| `src/audit_chain.rs` | `AuditChainEntry` | struct | Altpfad-Typ | 1 | 68 | Entfernen/Ändern |
| `src/audit_chain.rs` | `compute_hash` | fn | Altpfad-Typ | 1 | 86 | Entfernen/Ändern |
| `src/audit_chain.rs` | `compute_hash` | fn | Verboten-Hash | 1 | 90 | Ändern (Ersatz CRC32C/Mac) |
| `src/audit_chain.rs` | `compute_hash` | fn | Verboten-Serial | 1 | 96 | Ändern (feature-gate oder Ersatz) |
| `src/audit_chain.rs` | `compute_record_commitment` | fn | Verboten-Hash | 1 | 200 | Ändern (Ersatz CRC32C/Mac) |
| `src/audit_chain.rs` | `append` | fn | Altpfad-Typ | 1 | 233 | Entfernen/Ändern |
| `src/crypto.rs` | `—` | (Modulebene) | Verboten-Crypto | 2 | 48 | Ändern (XChaCha20-Poly1305) |
| `src/crypto.rs` | `KeyManager` | struct | Verboten-Crypto | 1 | 67 | Ändern (XChaCha20-Poly1305) |
| `src/crypto.rs` | `try_new` | fn | Verboten-Crypto | 2 | 110 | Ändern (XChaCha20-Poly1305) |
| `src/crypto.rs` | `try_new_with_kdf` | fn | Verboten-Crypto | 2 | 134 | Ändern (XChaCha20-Poly1305) |
| `src/crypto.rs` | `derive_file_key` | fn | Verboten-Crypto | 2 | 189 | Ändern (XChaCha20-Poly1305) |
| `src/crypto.rs` | `derive_segment_key` | fn | Verboten-Crypto | 2 | 215 | Ändern (XChaCha20-Poly1305) |
| `src/crypto.rs` | `derive_kv_key` | fn | Altpfad-KV | 1 | 241 | Entfernen |
| `src/crypto.rs` | `derive_kv_key` | fn | Verboten-Crypto | 2 | 273 | Ändern (XChaCha20-Poly1305) |
| `src/crypto.rs` | `derive_kv_key_scoped` | fn | Altpfad-KV | 1 | 295 | Entfernen |
| `src/crypto.rs` | `cipher_for` | fn | Altpfad-KV | 1 | 306 | Entfernen |
| `src/crypto.rs` | `cipher` | fn | Verboten-Crypto | 4 | 350 | Ändern (XChaCha20-Poly1305) |
| `src/crypto.rs` | `test_derive_kv_key_length_prefix_prevents_collision` | fn | Altpfad-KV | 1 | 711 | Entfernen |
| `src/crypto.rs` | `test_derive_kv_key_same_fingerprint_produces_same_key` | fn | Altpfad-KV | 1 | 743 | Entfernen |
| `src/crypto.rs` | `test_derive_kv_key_scoped_enforces_tenant_isolation` | fn | Altpfad-KV | 1 | 765 | Entfernen |
| `src/crypto.rs` | `test_befund_1_nonce_uniqueness_and_envelope_format` | fn | Verboten-Laufzeit | 1 | 882 | Ändern (kein Async/Lock im L2) |
| `src/crypto.rs` | `test_parallel_nonce_uniqueness` | fn | Verboten-Laufzeit | 1 | 896 | Ändern (kein Async/Lock im L2) |
| `src/dek.rs` | `—` | (Modulebene) | Verboten-Crypto | 2 | 8 | Ändern (XChaCha20-Poly1305) |
| `src/dek.rs` | `wrap_dek` | fn | Verboten-Crypto | 1 | 82 | Ändern (XChaCha20-Poly1305) |
| `src/dek.rs` | `unwrap_dek` | fn | Verboten-Crypto | 1 | 103 | Ändern (XChaCha20-Poly1305) |
| `src/deletion_proof/creation.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 13 | Entfernen/Ändern |
| `src/deletion_proof/creation.rs` | `create` | fn | Altpfad-Typ | 1 | 218 | Entfernen/Ändern |
| `src/deletion_proof/creation.rs` | `create_with_wal_receipt` | fn | Altpfad-Typ | 1 | 238 | Entfernen/Ändern |
| `src/deletion_proof/creation.rs` | `create_with_wal_receipt` | fn | Verboten-Serial | 3 | 249 | Ändern (feature-gate oder Ersatz) |
| `src/deletion_proof/creation.rs` | `create_v3` | fn | Altpfad-Typ | 1 | 303 | Entfernen/Ändern |
| `src/deletion_proof/creation.rs` | `create_v3_with_audit_position` | fn | Altpfad-Typ | 1 | 328 | Entfernen/Ändern |
| `src/deletion_proof/creation.rs` | `create_with_wal_receipt_v3` | fn | Altpfad-Typ | 1 | 355 | Entfernen/Ändern |
| `src/deletion_proof/creation.rs` | `create_full_v3` | fn | Altpfad-Typ | 1 | 382 | Entfernen/Ändern |
| `src/deletion_proof/creation.rs` | `construct_v5_payload` | fn | Verboten-Serial | 6 | 446 | Ändern (feature-gate oder Ersatz) |
| `src/deletion_proof/creation.rs` | `construct_v3_payload` | fn | Verboten-Serial | 4 | 490 | Ändern (feature-gate oder Ersatz) |
| `src/deletion_proof/mod.rs` | `tests` | mod | Altpfad-Typ | 1 | 41 | Entfernen/Ändern |
| `src/deletion_proof/mod.rs` | `test_v3_create_and_verify_external_roundtrip` | fn | Altpfad-Typ | 1 | 54 | Entfernen/Ändern |
| `src/deletion_proof/types.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 15 | Entfernen/Ändern |
| `src/deletion_proof/types.rs` | `hash_deleted_keys_length_prefixed` | fn | Verboten-Hash | 1 | 87 | Ändern (Ersatz CRC32C/Mac) |
| `src/deletion_proof/types.rs` | `DeletionProof` | struct | Altpfad-Typ | 1 | 330 | Entfernen/Ändern |
| `src/deletion_proof/verification.rs` | `verify` | fn | Verboten-Serial | 4 | 43 | Ändern (feature-gate oder Ersatz) |
| `src/lib.rs` | `kv_cipher` | mod | Altpfad-KV | 1 | 62 | Entfernen |
| `src/lib.rs` | `kv_segment` | mod | Altpfad-KV | 1 | 63 | Entfernen |
| `src/lib.rs` | `kv_shredding` | mod | Altpfad-KV | 1 | 69 | Entfernen |
| `src/lib.rs` | `wal_crypto` | mod | Altpfad-KV | 5 | 103 | Entfernen |
| `src/revocation_log.rs` | `—` | (Modulebene) | Verboten-Laufzeit | 1 | 36 | Ändern (kein Async/Lock im L2) |
| `src/revocation_log.rs` | `open_or_create` | fn | Verboten-Serial | 1 | 173 | Ändern (feature-gate oder Ersatz) |
| `src/revocation_log.rs` | `append` | fn | Verboten-Serial | 1 | 276 | Ändern (feature-gate oder Ersatz) |
| `src/revocation_log.rs` | `tests` | mod | Altpfad-KV | 1 | 327 | Entfernen |
| `src/revocation_log.rs` | `test_vfs_conformance` | fn | Verlorener-Fehler | 1 | 529 | Ändern (Fehler propagieren) |
| `src/wal_crypto.rs` | `test_integrity_verifier_chain` | fn | Verboten-Laufzeit | 1 | 621 | Ändern (kein Async/Lock im L2) |
| `src/wal_crypto.rs` | `test_encrypted_wal_roundtrip` | fn | Verboten-Laufzeit | 1 | 635 | Ändern (kein Async/Lock im L2) |

### contextra-durable-fs (14 Einträge)

| Datei | Element | Art | Kategorie | Treffer | Erste Zeile | Aktion |
|---|---|---|---|---:|---:|---|
| `src/journal.rs` | `—` | (Modulebene) | Verboten-FS | 1 | 4 | Ändern (über sys/durable-fs) |
| `src/journal.rs` | `calculate_crc32` | fn | Verboten-Hash | 1 | 48 | Ändern (Ersatz CRC32C/Mac) |
| `src/journal.rs` | `open_with_health` | fn | Verboten-FS | 1 | 328 | Ändern (über sys/durable-fs) |
| `src/lib.rs` | `vfs` | mod | Verboten-FS | 1 | 47 | Ändern (über sys/durable-fs) |
| `src/lib.rs` | `rename_durable` | fn | Verboten-FS | 1 | 118 | Ändern (über sys/durable-fs) |
| `src/lib.rs` | `drop` | fn | Verboten-FS | 1 | 313 | Ändern (über sys/durable-fs) |
| `src/lib.rs` | `atomic_replace_with_injector` | fn | Verboten-FS | 1 | 371 | Ändern (über sys/durable-fs) |
| `src/lib.rs` | `durable_remove_with_injector` | fn | Verboten-FS | 1 | 392 | Ändern (über sys/durable-fs) |
| `src/platform.rs` | `—` | (Modulebene) | Verboten-FS | 1 | 6 | Ändern (über sys/durable-fs) |
| `src/scrub.rs` | `—` | (Modulebene) | Verboten-FS | 1 | 14 | Ändern (über sys/durable-fs) |
| `src/scrub.rs` | `verify_zero_readback` | fn | Verboten-FS | 1 | 225 | Ändern (über sys/durable-fs) |
| `src/scrub.rs` | `check_cow_filesystem` | fn | Verboten-FS | 1 | 253 | Ändern (über sys/durable-fs) |
| `src/vfs.rs` | `—` | (Modulebene) | Verboten-FS | 1 | 3 | Ändern (über sys/durable-fs) |
| `src/vfs.rs` | `drop` | fn | Verboten-FS | 1 | 127 | Ändern (über sys/durable-fs) |

### contextra-engine (194 Einträge)

| Datei | Element | Art | Kategorie | Treffer | Erste Zeile | Aktion |
|---|---|---|---|---:|---:|---|
| `src/adapters/mod.rs` | `ollama` | mod | Altpfad-Inferenz | 1 | 8 | Entfernen |
| `src/adapters/text.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 3 | Entfernen (legacy) |
| `src/adapters/text.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 5 | Entfernen/Ändern |
| `src/adapters/text.rs` | `Bm25Scorer` | struct | Altpfad-Port | 1 | 9 | Entfernen (legacy) |
| `src/adapters/text.rs` | `Bm25Scorer<S>` | impl | Altpfad-Port | 1 | 13 | Entfernen (legacy) |
| `src/adapters/text.rs` | `TextIndex` | impl | Altpfad-Port | 1 | 22 | Entfernen (legacy) |
| `src/adapters/text.rs` | `insert` | fn | Altpfad-Typ | 1 | 31 | Entfernen/Ändern |
| `src/adapters/text.rs` | `delete_document` | fn | Altpfad-Typ | 1 | 35 | Entfernen/Ändern |
| `src/adapters/text.rs` | `commit` | fn | Altpfad-Typ | 1 | 39 | Entfernen/Ändern |
| `src/adapters/text.rs` | `rollback` | fn | Altpfad-Typ | 1 | 43 | Entfernen/Ändern |
| `src/backends.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 8 | Entfernen (legacy) |
| `src/backends.rs` | `Storage` | type | Altpfad-Port | 1 | 14 | Entfernen (legacy) |
| `src/backends.rs` | `Vector` | type | Altpfad-Store | 1 | 20 | Entfernen (nach G-CUTOVER) |
| `src/backends.rs` | `Storage` | type | Altpfad-Store | 1 | 31 | Entfernen (nach G-CUTOVER) |
| `src/background_workers/expiry_workers.rs` | `—` | (Modulebene) | Altpfad-Inferenz | 1 | 14 | Entfernen |
| `src/background_workers/expiry_workers.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 15 | Entfernen (legacy) |
| `src/background_workers/expiry_workers.rs` | `start_expiry_cleanup_worker` | fn | Altpfad-Port | 1 | 25 | Entfernen (legacy) |
| `src/background_workers/expiry_workers.rs` | `start_expiry_cleanup_worker` | fn | Verboten-Laufzeit | 5 | 29 | Ändern (kein Async/Lock im L2) |
| `src/background_workers/expiry_workers.rs` | `start_expiry_reaper` | fn | Altpfad-Port | 1 | 75 | Entfernen (legacy) |
| `src/background_workers/expiry_workers.rs` | `start_expiry_reaper` | fn | Verboten-Laufzeit | 1 | 79 | Ändern (kein Async/Lock im L2) |
| `src/background_workers/expiry_workers.rs` | `start_decay_cleanup_worker` | fn | Altpfad-Port | 1 | 86 | Entfernen (legacy) |
| `src/background_workers/expiry_workers.rs` | `start_decay_cleanup_worker` | fn | Verboten-Laufzeit | 5 | 91 | Ändern (kein Async/Lock im L2) |
| `src/background_workers/expiry_workers.rs` | `start_decay_cleanup_worker` | fn | Altpfad-Inferenz | 2 | 92 | Entfernen |
| `src/background_workers/expiry_workers.rs` | `start_thermostat_reaper` | fn | Altpfad-Port | 1 | 114 | Entfernen (legacy) |
| `src/background_workers/expiry_workers.rs` | `start_thermostat_reaper` | fn | Verboten-Laufzeit | 1 | 119 | Ändern (kein Async/Lock im L2) |
| `src/background_workers/hyperedge_worker.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 14 | Entfernen (legacy) |
| `src/background_workers/hyperedge_worker.rs` | `—` | (Modulebene) | Verboten-Laufzeit | 1 | 18 | Ändern (kein Async/Lock im L2) |
| `src/background_workers/hyperedge_worker.rs` | `start_hyperedge_cascade_deferred_worker` | fn | Altpfad-Port | 1 | 53 | Entfernen (legacy) |
| `src/background_workers/hyperedge_worker.rs` | `start_hyperedge_cascade_deferred_worker` | fn | Verboten-Laufzeit | 5 | 58 | Ändern (kein Async/Lock im L2) |
| `src/background_workers/hyperedge_worker.rs` | `start_hyperedge_cascade_deferred_worker` | fn | Altpfad-Typ | 1 | 81 | Entfernen/Ändern |
| `src/background_workers/orphan_workers.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 13 | Entfernen/Ändern |
| `src/background_workers/orphan_workers.rs` | `reap_orphans_step` | fn | Altpfad-Typ | 1 | 25 | Entfernen/Ändern |
| `src/background_workers/orphan_workers.rs` | `start_orphan_cleanup_worker_with_config` | fn | Altpfad-Typ | 1 | 36 | Entfernen/Ändern |
| `src/background_workers/orphan_workers.rs` | `start_orphan_cleanup_worker_with_config` | fn | Verboten-Laufzeit | 7 | 41 | Ändern (kein Async/Lock im L2) |
| `src/background_workers/orphan_workers.rs` | `start_orphan_cleanup_worker` | fn | Altpfad-Typ | 1 | 172 | Entfernen/Ändern |
| `src/background_workers/orphan_workers.rs` | `start_orphan_cleanup_worker` | fn | Verboten-Laufzeit | 1 | 176 | Ändern (kein Async/Lock im L2) |
| `src/background_workers/orphan_workers.rs` | `start_orphan_reaper` | fn | Altpfad-Typ | 1 | 190 | Entfernen/Ändern |
| `src/background_workers/orphan_workers.rs` | `start_orphan_reaper` | fn | Verboten-Laufzeit | 1 | 194 | Ändern (kein Async/Lock im L2) |
| `src/collection/crud/auto_extraction.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 17 | Entfernen (legacy) |
| `src/collection/crud/auto_extraction.rs` | `LlmTextGenerator` | impl | Altpfad-Port | 1 | 69 | Entfernen (legacy) |
| `src/collection/crud/auto_extraction.rs` | `AUTO_EXTRACTION_REGISTRY` | static | Verboten-Laufzeit | 2 | 156 | Ändern (kein Async/Lock im L2) |
| `src/collection/crud/auto_extraction.rs` | `AUTO_EXTRACTION_REGISTRY` | static | Altpfad-Port | 1 | 160 | Entfernen (legacy) |
| `src/collection/crud/auto_extraction.rs` | `Collection<S` | impl | Altpfad-Port | 1 | 168 | Entfernen (legacy) |
| `src/collection/crud/auto_extraction.rs` | `with_auto_extraction` | fn | Altpfad-Port | 1 | 172 | Entfernen (legacy) |
| `src/collection/crud/auto_extraction.rs` | `set_auto_extraction` | fn | Altpfad-Port | 1 | 182 | Entfernen (legacy) |
| `src/collection/crud/auto_extraction.rs` | `set_auto_extraction_with_mode` | fn | Altpfad-Port | 1 | 191 | Entfernen (legacy) |
| `src/collection/crud/auto_extraction.rs` | `set_auto_extraction_config` | fn | Altpfad-Port | 1 | 203 | Entfernen (legacy) |
| `src/collection/crud/auto_extraction.rs` | `auto_extraction_config` | fn | Altpfad-Port | 1 | 210 | Entfernen (legacy) |
| `src/collection/crud/auto_extraction.rs` | `auto_extraction_config_with_extractor` | fn | Altpfad-Port | 1 | 222 | Entfernen (legacy) |
| `src/collection/crud/auto_extraction.rs` | `auto_extract_and_relate` | fn | Altpfad-Port | 2 | 240 | Entfernen (legacy) |
| `src/collection/crud/auto_extraction.rs` | `auto_extraction_tests` | mod | Verboten-Laufzeit | 1 | 321 | Ändern (kein Async/Lock im L2) |
| `src/collection/crud/delete.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 9 | Entfernen (legacy) |
| `src/collection/crud/delete.rs` | `Collection<S` | impl | Altpfad-Port | 1 | 12 | Entfernen (legacy) |
| `src/collection/crud/delete.rs` | `delete_op` | fn | Altpfad-Typ | 1 | 40 | Entfernen/Ändern |
| `src/collection/crud/insert.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 14 | Entfernen (legacy) |
| `src/collection/crud/insert.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 15 | Entfernen/Ändern |
| `src/collection/crud/insert.rs` | `Collection<S` | impl | Altpfad-Port | 1 | 17 | Entfernen (legacy) |
| `src/collection/crud/insert.rs` | `check_doc_id_collision_tracked` | fn | Altpfad-Typ | 1 | 278 | Entfernen/Ändern |
| `src/collection/crud/insert.rs` | `insert_op` | fn | Altpfad-Typ | 1 | 307 | Entfernen/Ändern |
| `src/collection/crud/internal.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 9 | Entfernen (legacy) |
| `src/collection/crud/internal.rs` | `Collection<S` | impl | Altpfad-Port | 1 | 54 | Entfernen (legacy) |
| `src/collection/crud/internal.rs` | `apply_insert_backpressure` | fn | Altpfad-Store | 4 | 63 | Entfernen (nach G-CUTOVER) |
| `src/collection/crud/internal.rs` | `apply_insert_backpressure` | fn | Verboten-Laufzeit | 1 | 90 | Ändern (kein Async/Lock im L2) |
| `src/collection/crud/internal.rs` | `lock_keys_sorted` | fn | Altpfad-KV | 1 | 101 | Entfernen |
| `src/collection/crud/kv.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 10 | Entfernen (legacy) |
| `src/collection/crud/kv.rs` | `Collection<S` | impl | Altpfad-Port | 1 | 13 | Entfernen (legacy) |
| `src/collection/crud/links.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 9 | Entfernen (legacy) |
| `src/collection/crud/links.rs` | `Collection<S` | impl | Altpfad-Port | 1 | 12 | Entfernen (legacy) |
| `src/collection/crud/links.rs` | `update_metadata_links` | fn | Altpfad-Typ | 1 | 143 | Entfernen/Ändern |
| `src/collection/crud/read.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 10 | Entfernen (legacy) |
| `src/collection/crud/read.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 11 | Entfernen/Ändern |
| `src/collection/crud/read.rs` | `Collection<S` | impl | Altpfad-Port | 1 | 24 | Entfernen (legacy) |
| `src/collection/crud/read.rs` | `get_doc_tx` | fn | Altpfad-Typ | 3 | 62 | Entfernen/Ändern |
| `src/collection/crud/read.rs` | `get_tracked` | fn | Altpfad-Typ | 1 | 291 | Entfernen/Ändern |
| `src/collection/crud/read.rs` | `get_at_seq_tracked` | fn | Altpfad-Typ | 1 | 304 | Entfernen/Ändern |
| `src/collection/crud/update.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 12 | Entfernen (legacy) |
| `src/collection/crud/update.rs` | `Collection<S` | impl | Altpfad-Port | 1 | 15 | Entfernen (legacy) |
| `src/collection/crud/update.rs` | `update_op` | fn | Altpfad-Typ | 1 | 56 | Entfernen/Ändern |
| `src/collection/maintenance.rs` | `—` | (Modulebene) | Altpfad-Inferenz | 1 | 15 | Entfernen |
| `src/collection/maintenance.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 20 | Entfernen (legacy) |
| `src/collection/maintenance.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 21 | Entfernen/Ändern |
| `src/collection/maintenance.rs` | `Collection<S` | impl | Altpfad-Port | 2 | 25 | Entfernen (legacy) |
| `src/collection/maintenance.rs` | `Collection<S` | impl | Altpfad-Inferenz | 1 | 29 | Entfernen |
| `src/collection/maintenance.rs` | `evict_decayed_chunks` | fn | Altpfad-Inferenz | 1 | 32 | Entfernen |
| `src/collection/maintenance.rs` | `BATCH_SIZE` | const | Altpfad-Inferenz | 1 | 108 | Entfernen |
| `src/collection/maintenance.rs` | `BATCH_SIZE` | const | Verboten-Laufzeit | 4 | 120 | Ändern (kein Async/Lock im L2) |
| `src/collection/maintenance.rs` | `BATCH_SIZE` | const | Altpfad-Typ | 1 | 629 | Entfernen/Ändern |
| `src/collection/mod.rs` | `kv_lock` | mod | Altpfad-KV | 1 | 18 | Entfernen |
| `src/collection/mod.rs` | `tests` | mod | Altpfad-Port | 1 | 42 | Entfernen (legacy) |
| `src/collection/mod.rs` | `tests` | mod | Altpfad-Store | 1 | 46 | Entfernen (nach G-CUTOVER) |
| `src/collection/mod.rs` | `tests` | mod | Altpfad-Typ | 1 | 51 | Entfernen/Ändern |
| `src/collection/mod.rs` | `ensure_importance_metadata` | fn | Altpfad-Typ | 1 | 143 | Entfernen/Ändern |
| `src/collection/mod.rs` | `extract_effective_importance` | fn | Altpfad-Typ | 1 | 211 | Entfernen/Ändern |
| `src/collection/mod.rs` | `Collection` | struct | Altpfad-Port | 2 | 273 | Entfernen (legacy) |
| `src/collection/mod.rs` | `Collection` | struct | Altpfad-Store | 2 | 273 | Entfernen (nach G-CUTOVER) |
| `src/collection/mod.rs` | `Collection` | struct | Verboten-Laufzeit | 6 | 282 | Ändern (kein Async/Lock im L2) |
| `src/collection/mod.rs` | `Collection` | struct | Altpfad-KV | 1 | 284 | Entfernen |
| `src/collection/mod.rs` | `Clone` | impl | Altpfad-Port | 1 | 303 | Entfernen (legacy) |
| `src/collection/mod.rs` | `clone` | fn | Verboten-Laufzeit | 5 | 314 | Ändern (kein Async/Lock im L2) |
| `src/collection/mod.rs` | `Collection<S` | impl | Altpfad-Port | 2 | 331 | Entfernen (legacy) |
| `src/collection/mod.rs` | `new` | fn | Altpfad-Store | 1 | 382 | Entfernen (nach G-CUTOVER) |
| `src/collection/mod.rs` | `new` | fn | Verboten-Laufzeit | 6 | 394 | Ändern (kein Async/Lock im L2) |
| `src/collection/mod.rs` | `new` | fn | Altpfad-KV | 1 | 396 | Entfernen |
| `src/collection/mod.rs` | `with_pressure_receiver` | fn | Altpfad-Store | 1 | 458 | Entfernen (nach G-CUTOVER) |
| `src/collection/mod.rs` | `with_pressure_receiver` | fn | Verboten-Laufzeit | 1 | 458 | Ändern (kein Async/Lock im L2) |
| `src/collection/mod.rs` | `set_pressure_receiver` | fn | Altpfad-Store | 1 | 467 | Entfernen (nach G-CUTOVER) |
| `src/collection/mod.rs` | `set_pressure_receiver` | fn | Verboten-Laufzeit | 1 | 467 | Ändern (kein Async/Lock im L2) |
| `src/collection/mod.rs` | `embedder` | fn | Altpfad-Port | 1 | 488 | Entfernen (legacy) |
| `src/collection/mod.rs` | `with_embedder` | fn | Altpfad-Port | 1 | 494 | Entfernen (legacy) |
| `src/collection/mod.rs` | `set_embedder` | fn | Altpfad-Port | 1 | 504 | Entfernen (legacy) |
| `src/collection/mod.rs` | `check_and_trigger_community_detection` | fn | Verboten-Laufzeit | 1 | 701 | Ändern (kein Async/Lock im L2) |
| `src/collection/query_builder/builder.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 13 | Entfernen (legacy) |
| `src/collection/query_builder/builder.rs` | `HybridQueryBuilder` | struct | Altpfad-Port | 1 | 31 | Entfernen (legacy) |
| `src/collection/query_builder/builder.rs` | `HybridQueryBuilder` | struct | Verboten-Laufzeit | 1 | 52 | Ändern (kein Async/Lock im L2) |
| `src/collection/query_builder/builder.rs` | `HybridQueryBuilder` | struct | Altpfad-Typ | 1 | 56 | Entfernen/Ändern |
| `src/collection/query_builder/builder.rs` | `HybridQueryBuilder<` | impl | Altpfad-Port | 1 | 60 | Entfernen (legacy) |
| `src/collection/query_builder/builder.rs` | `pid_controller` | fn | Verboten-Laufzeit | 1 | 233 | Ändern (kein Async/Lock im L2) |
| `src/collection/query_builder/builder.rs` | `current_tx` | fn | Altpfad-Typ | 1 | 258 | Entfernen/Ändern |
| `src/collection/query_builder/builder.rs` | `Collection<S` | impl | Altpfad-Port | 1 | 307 | Entfernen (legacy) |
| `src/collection/query_builder/builder_exec.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 9 | Entfernen (legacy) |
| `src/collection/query_builder/builder_exec.rs` | `HybridQueryBuilder<` | impl | Altpfad-Port | 2 | 12 | Entfernen (legacy) |
| `src/collection/query_builder/builder_exec.rs` | `apply_post_rrf_filters` | fn | Altpfad-Typ | 1 | 180 | Entfernen/Ändern |
| `src/collection/query_builder/builder_exec.rs` | `apply_reranker_stage` | fn | Verboten-Laufzeit | 1 | 241 | Ändern (kein Async/Lock im L2) |
| `src/collection/query_builder/scope.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 13 | Entfernen (legacy) |
| `src/collection/query_builder/scope.rs` | `HybridQueryBuilder<` | impl | Altpfad-Port | 1 | 56 | Entfernen (legacy) |
| `src/collection/relate.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 9 | Entfernen (legacy) |
| `src/collection/relate.rs` | `Collection<S` | impl | Altpfad-Port | 1 | 12 | Entfernen (legacy) |
| `src/collection/relate.rs` | `test_db` | fn | Verboten-Laufzeit | 1 | 229 | Ändern (kein Async/Lock im L2) |
| `src/collection/relate.rs` | `test_relate_n_ary_basic` | fn | Verboten-Laufzeit | 1 | 271 | Ändern (kein Async/Lock im L2) |
| `src/collection/search/basic.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 15 | Entfernen (legacy) |
| `src/collection/search/basic.rs` | `Collection<S` | impl | Altpfad-Port | 1 | 18 | Entfernen (legacy) |
| `src/collection/search/checkpoint.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 12 | Entfernen (legacy) |
| `src/collection/search/checkpoint.rs` | `CheckpointPinGuard` | struct | Altpfad-Port | 1 | 22 | Entfernen (legacy) |
| `src/collection/search/checkpoint.rs` | `CheckpointPinGuard<` | impl | Altpfad-Port | 1 | 28 | Entfernen (legacy) |
| `src/collection/search/checkpoint.rs` | `new` | fn | Altpfad-Checkpoint | 1 | 31 | Entfernen |
| `src/collection/search/checkpoint.rs` | `new_at_latest` | fn | Altpfad-Checkpoint | 1 | 43 | Entfernen |
| `src/collection/search/checkpoint.rs` | `Drop` | impl | Altpfad-Port | 1 | 62 | Entfernen (legacy) |
| `src/collection/search/checkpoint.rs` | `with_pinned_checkpoint` | fn | Altpfad-Port | 1 | 78 | Entfernen (legacy) |
| `src/collection/search/checkpoint.rs` | `with_pinned_checkpoint_at_latest` | fn | Altpfad-Port | 1 | 96 | Entfernen (legacy) |
| `src/collection/search/filtered.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 13 | Entfernen (legacy) |
| `src/collection/search/filtered.rs` | `Collection<S` | impl | Altpfad-Port | 1 | 22 | Entfernen (legacy) |
| `src/collection/search/hybrid/mod.rs` | `query` | mod | Altpfad-Port | 1 | 15 | Entfernen (legacy) |
| `src/collection/search/hybrid/mod.rs` | `query` | mod | Altpfad-Typ | 1 | 16 | Entfernen/Ändern |
| `src/collection/search/hybrid/mod.rs` | `Collection<S` | impl | Altpfad-Port | 1 | 18 | Entfernen (legacy) |
| `src/collection/search/hybrid/mod.rs` | `filter_by_importance` | fn | Altpfad-Typ | 1 | 388 | Entfernen/Ändern |
| `src/collection/search/hybrid/query.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 20 | Entfernen (legacy) |
| `src/collection/search/hybrid/query.rs` | `calc_hybrid_candidate_k` | fn | Altpfad-Store | 2 | 26 | Entfernen (nach G-CUTOVER) |
| `src/collection/search/hybrid/query.rs` | `Collection<S` | impl | Altpfad-Port | 1 | 37 | Entfernen (legacy) |
| `src/collection/search/hydrate.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 12 | Entfernen (legacy) |
| `src/collection/search/hydrate.rs` | `Collection<S` | impl | Altpfad-Port | 1 | 15 | Entfernen (legacy) |
| `src/collection/tx.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 9 | Entfernen (legacy) |
| `src/collection/tx.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 10 | Entfernen/Ändern |
| `src/collection/tx.rs` | `Collection<S` | impl | Altpfad-Port | 1 | 13 | Entfernen (legacy) |
| `src/collection/tx.rs` | `next_tx` | fn | Altpfad-Typ | 1 | 19 | Entfernen/Ändern |
| `src/collection/tx.rs` | `allocate_tx` | fn | Altpfad-Typ | 4 | 26 | Entfernen/Ändern |
| `src/collection/tx.rs` | `begin_transaction` | fn | Altpfad-Typ | 2 | 38 | Entfernen/Ändern |
| `src/export.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 16 | Entfernen (legacy) |
| `src/export.rs` | `Collection<S` | impl | Altpfad-Port | 1 | 87 | Entfernen (legacy) |
| `src/export.rs` | `BATCH_SIZE` | const | Altpfad-Typ | 1 | 151 | Entfernen/Ändern |
| `src/export.rs` | `tests` | mod | Verboten-Laufzeit | 1 | 232 | Ändern (kein Async/Lock im L2) |
| `src/export.rs` | `test_export_import_roundtrip` | fn | Verboten-Laufzeit | 1 | 349 | Ändern (kein Async/Lock im L2) |
| `src/extraction/mod.rs` | `types` | mod | Altpfad-Port | 1 | 27 | Entfernen (legacy) |
| `src/extraction/mod.rs` | `extract_triples` | fn | Altpfad-Port | 4 | 36 | Entfernen (legacy) |
| `src/extraction/mod.rs` | `extract_triples_for_mode` | fn | Altpfad-Port | 1 | 86 | Entfernen (legacy) |
| `src/extraction/mod.rs` | `extract_triples_for_config` | fn | Altpfad-Port | 1 | 96 | Entfernen (legacy) |
| `src/extraction/open_ie.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 16 | Entfernen (legacy) |
| `src/extraction/open_ie.rs` | `extract_triples` | fn | Altpfad-Port | 1 | 82 | Entfernen (legacy) |
| `src/import.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 16 | Entfernen (legacy) |
| `src/import.rs` | `Collection<S` | impl | Altpfad-Port | 1 | 33 | Entfernen (legacy) |
| `src/lib.rs` | `orchestration` | mod | Altpfad-Inferenz | 1 | 15 | Entfernen |
| `src/lib.rs` | `orchestration` | mod | Altpfad-Port | 2 | 20 | Entfernen (legacy) |
| `src/lib.rs` | `orchestration` | mod | Altpfad-Store | 2 | 24 | Entfernen (nach G-CUTOVER) |
| `src/lib.rs` | `decay_controller` | mod | Altpfad-Inferenz | 1 | 42 | Entfernen |
| `src/lib.rs` | `transaction` | mod | Altpfad-Inferenz | 1 | 62 | Entfernen |
| `src/lib.rs` | `ConsolidationLauncher` | type | Altpfad-Store | 1 | 103 | Entfernen (nach G-CUTOVER) |
| `src/lib.rs` | `ConsolidationLauncher` | type | Verboten-Laufzeit | 1 | 107 | Ändern (kein Async/Lock im L2) |
| `src/lib.rs` | `register_consolidation_launcher` | fn | Altpfad-Store | 1 | 121 | Entfernen (nach G-CUTOVER) |
| `src/lib.rs` | `register_consolidation_launcher` | fn | Verboten-Laufzeit | 1 | 125 | Ändern (kein Async/Lock im L2) |
| `src/lib.rs` | `ContextraConfig` | struct | Altpfad-Store | 1 | 233 | Entfernen (nach G-CUTOVER) |
| `src/lib.rs` | `default` | fn | Altpfad-Store | 1 | 334 | Entfernen (nach G-CUTOVER) |
| `src/lib.rs` | `TenantCollectionMap` | type | Altpfad-Store | 1 | 357 | Entfernen (nach G-CUTOVER) |
| `src/lib.rs` | `Contextra` | struct | Altpfad-Store | 2 | 364 | Entfernen (nach G-CUTOVER) |
| `src/lib.rs` | `Contextra` | struct | Verboten-Laufzeit | 7 | 371 | Ändern (kein Async/Lock im L2) |
| `src/lib.rs` | `Contextra` | struct | Altpfad-Port | 1 | 375 | Entfernen (legacy) |
| `src/lib.rs` | `inner_storage` | fn | Altpfad-Store | 1 | 395 | Entfernen (nach G-CUTOVER) |
| `src/temporal_filter.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 18 | Entfernen/Ändern |
| `src/temporal_filter.rs` | `ValidityWindow` | struct | Altpfad-Typ | 2 | 28 | Entfernen/Ändern |
| `src/temporal_filter.rs` | `is_valid_bitemporal` | fn | Altpfad-Typ | 1 | 53 | Entfernen/Ändern |
| `src/temporal_filter.rs` | `extract_validity_window` | fn | Altpfad-Typ | 2 | 132 | Entfernen/Ändern |
| `src/temporal_filter.rs` | `apply_temporal_validity_filter_with_clock` | fn | Altpfad-Typ | 1 | 175 | Entfernen/Ändern |
| `src/temporal_filter.rs` | `apply_temporal_validity_filter` | fn | Altpfad-Typ | 1 | 189 | Entfernen/Ändern |
| `src/temporal_filter.rs` | `apply_temporal_validity_filter_at` | fn | Altpfad-Typ | 1 | 211 | Entfernen/Ändern |
| `src/temporal_filter.rs` | `test_temporal_filter_valid_until_in_past_filtered_out` | fn | Altpfad-Typ | 1 | 255 | Entfernen/Ändern |
| `src/temporal_filter.rs` | `test_temporal_filter_fail_open_without_metadata` | fn | Altpfad-Typ | 1 | 272 | Entfernen/Ändern |

### contextra-graph (186 Einträge)

| Datei | Element | Art | Kategorie | Treffer | Erste Zeile | Aktion |
|---|---|---|---|---:|---:|---|
| `src/apprh/gate_monitor.rs` | `—` | (Modulebene) | Verboten-Laufzeit | 1 | 14 | Ändern (kein Async/Lock im L2) |
| `src/apprh/mod.rs` | `tests` | mod | Verboten-Laufzeit | 1 | 110 | Ändern (kein Async/Lock im L2) |
| `src/apprh/selector.rs` | `—` | (Modulebene) | Verboten-Laufzeit | 1 | 18 | Ändern (kein Async/Lock im L2) |
| `src/cascade.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 13 | Entfernen/Ändern |
| `src/cascade.rs` | `cascade_invalidate_edges_for_superseded_doc` | fn | Altpfad-Typ | 1 | 45 | Entfernen/Ändern |
| `src/cascade.rs` | `sweep_orphans` | fn | Altpfad-Typ | 2 | 116 | Entfernen/Ändern |
| `src/cascade.rs` | `cascade_invalidate_hyperedges_for_superseded_doc` | fn | Altpfad-Typ | 1 | 265 | Entfernen/Ändern |
| `src/cascade.rs` | `tests` | mod | Verboten-Laufzeit | 1 | 446 | Ändern (kein Async/Lock im L2) |
| `src/cascade.rs` | `test_cascade_invalidation_tombstones_edges_of_superseded_doc` | fn | Altpfad-Typ | 1 | 449 | Entfernen/Ändern |
| `src/cascade.rs` | `test_cascade_invalidation_tombstones_edges_of_superseded_doc` | fn | Verboten-Laufzeit | 1 | 511 | Ändern (kein Async/Lock im L2) |
| `src/cascade.rs` | `test_cascade_invalidation_is_idempotent` | fn | Altpfad-Typ | 1 | 514 | Entfernen/Ändern |
| `src/cascade.rs` | `test_cascade_invalidation_is_idempotent` | fn | Verboten-Laufzeit | 1 | 556 | Ändern (kein Async/Lock im L2) |
| `src/cascade.rs` | `test_pathrag_sufficiency_excludes_cascaded_tombstones` | fn | Altpfad-Typ | 1 | 559 | Entfernen/Ändern |
| `src/cascade.rs` | `test_pathrag_sufficiency_excludes_cascaded_tombstones` | fn | Verboten-Laufzeit | 1 | 603 | Ändern (kein Async/Lock im L2) |
| `src/cascade.rs` | `test_cascade_invalidation_nonexistent_doc_returns_zero_count` | fn | Verboten-Laufzeit | 1 | 617 | Ändern (kein Async/Lock im L2) |
| `src/cascade.rs` | `test_cascade_invalidation_max_wal_seq` | fn | Altpfad-Typ | 1 | 620 | Entfernen/Ändern |
| `src/cascade.rs` | `test_cascade_invalidation_max_wal_seq` | fn | Verboten-Laufzeit | 1 | 656 | Ändern (kein Async/Lock im L2) |
| `src/cascade.rs` | `test_cascade_invalidation_multiple_edges_same_doc` | fn | Altpfad-Typ | 1 | 659 | Entfernen/Ändern |
| `src/cascade.rs` | `test_cascade_invalidation_multiple_edges_same_doc` | fn | Verboten-Laufzeit | 1 | 695 | Ändern (kein Async/Lock im L2) |
| `src/cascade.rs` | `ROLE_OBJECT` | const | Verboten-Laufzeit | 1 | 727 | Ändern (kein Async/Lock im L2) |
| `src/cascade.rs` | `ROLE_2` | const | Verboten-Laufzeit | 1 | 768 | Ändern (kein Async/Lock im L2) |
| `src/cascade.rs` | `ROLE_1` | const | Verboten-Laufzeit | 1 | 799 | Ändern (kein Async/Lock im L2) |
| `src/cascade.rs` | `ROLE_ESCROW` | const | Verboten-Laufzeit | 1 | 844 | Ändern (kein Async/Lock im L2) |
| `src/cascade.rs` | `test_cascade_invalidate_hyperedges_nonexistent_doc` | fn | Verboten-Laufzeit | 1 | 858 | Ändern (kein Async/Lock im L2) |
| `src/cascade.rs` | `ROLE_R` | const | Verboten-Laufzeit | 1 | 910 | Ändern (kein Async/Lock im L2) |
| `src/cascade.rs` | `test_sweep_orphans_sweeps_degenerated_hyperedges` | fn | Altpfad-Typ | 1 | 913 | Entfernen/Ändern |
| `src/cascade.rs` | `ROLE` | const | Altpfad-Typ | 1 | 962 | Entfernen/Ändern |
| `src/cascade.rs` | `ROLE` | const | Verboten-Laufzeit | 1 | 974 | Ändern (kein Async/Lock im L2) |
| `src/community.rs` | `StarExpansionIterator` | struct | Verboten-Laufzeit | 1 | 67 | Ändern (kein Async/Lock im L2) |
| `src/consistency_enforcement.rs` | `—` | (Modulebene) | Altpfad-Graph | 1 | 19 | Entfernen |
| `src/consistency_enforcement.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 20 | Entfernen/Ändern |
| `src/consistency_enforcement.rs` | `ConflictPattern` | struct | Altpfad-Typ | 2 | 37 | Entfernen/Ändern |
| `src/consistency_enforcement.rs` | `pattern_hash` | fn | Verboten-Hash | 1 | 60 | Ändern (Ersatz CRC32C/Mac) |
| `src/consistency_enforcement.rs` | `check_before_insert` | fn | Altpfad-Typ | 1 | 117 | Entfernen/Ändern |
| `src/consistency_enforcement.rs` | `record_contradiction` | fn | Altpfad-Typ | 1 | 141 | Entfernen/Ändern |
| `src/consistency_enforcement.rs` | `test_first_contradiction` | fn | Altpfad-Typ | 1 | 221 | Entfernen/Ändern |
| `src/consistency_enforcement.rs` | `test_three_repeated_contradictions_suppresses_pattern` | fn | Altpfad-Typ | 5 | 237 | Entfernen/Ändern |
| `src/consistency_enforcement.rs` | `test_independent_pattern_hash_counting_no_cross_contamination` | fn | Altpfad-Typ | 4 | 258 | Entfernen/Ändern |
| `src/consistency_enforcement.rs` | `test_active_patterns_and_suggest_tombstones` | fn | Altpfad-Typ | 2 | 334 | Entfernen/Ändern |
| `src/consistency_enforcement.rs` | `test_zero_threshold_suppresses_immediately` | fn | Altpfad-Typ | 1 | 375 | Entfernen/Ändern |
| `src/consistency_enforcement.rs` | `test_conflict_pattern_and_edge_assertion_serde_roundtrip` | fn | Altpfad-Typ | 2 | 419 | Entfernen/Ändern |
| `src/consistency_enforcement.rs` | `test_consistency_enforcer_bincode_serde_roundtrip` | fn | Altpfad-Typ | 1 | 468 | Entfernen/Ändern |
| `src/consistency_enforcement.rs` | `test_consistency_enforcer_bincode_serde_roundtrip` | fn | Verboten-Serial | 2 | 470 | Ändern (feature-gate oder Ersatz) |
| `src/csr/graph_index/mod.rs` | `traversal` | mod | Altpfad-Typ | 1 | 13 | Entfernen/Ändern |
| `src/csr/graph_index/mod.rs` | `add_entity` | fn | Altpfad-Typ | 1 | 96 | Entfernen/Ändern |
| `src/csr/graph_index/mod.rs` | `add_edge` | fn | Altpfad-Typ | 1 | 100 | Entfernen/Ändern |
| `src/csr/graph_index/mod.rs` | `personalized_page_rank_at` | fn | Altpfad-Typ | 2 | 134 | Entfernen/Ändern |
| `src/csr/graph_index/mod.rs` | `traverse_at` | fn | Altpfad-Typ | 1 | 176 | Entfernen/Ändern |
| `src/csr/graph_index/mod.rs` | `traverse_at_time` | fn | Altpfad-Typ | 1 | 185 | Entfernen/Ändern |
| `src/csr/graph_index/mod.rs` | `traverse_at_bitemporal` | fn | Altpfad-Typ | 1 | 197 | Entfernen/Ändern |
| `src/csr/graph_index/mod.rs` | `commit` | fn | Altpfad-Typ | 1 | 211 | Entfernen/Ändern |
| `src/csr/graph_index/mod.rs` | `remove_edge` | fn | Altpfad-Typ | 1 | 217 | Entfernen/Ändern |
| `src/csr/graph_index/mod.rs` | `add_bidirectional` | fn | Altpfad-Typ | 1 | 226 | Entfernen/Ändern |
| `src/csr/graph_index/mod.rs` | `rollback` | fn | Altpfad-Typ | 1 | 244 | Entfernen/Ändern |
| `src/csr/graph_index/mod.rs` | `rollback_to_tx` | fn | Altpfad-Typ | 1 | 254 | Entfernen/Ändern |
| `src/csr/graph_index/mod.rs` | `last_tx_id` | fn | Altpfad-Typ | 2 | 258 | Entfernen/Ändern |
| `src/csr/graph_index/mutations.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 12 | Entfernen/Ändern |
| `src/csr/graph_index/mutations.rs` | `validate_tx_id_heuristic` | fn | Altpfad-Typ | 5 | 23 | Entfernen/Ändern |
| `src/csr/graph_index/mutations.rs` | `remove_entity` | fn | Altpfad-Typ | 1 | 41 | Entfernen/Ändern |
| `src/csr/graph_index/mutations.rs` | `add_entity_impl` | fn | Altpfad-Typ | 1 | 89 | Entfernen/Ändern |
| `src/csr/graph_index/mutations.rs` | `add_edge_impl` | fn | Altpfad-Typ | 1 | 110 | Entfernen/Ändern |
| `src/csr/graph_index/mutations.rs` | `add_edge_impl` | fn | Verboten-Hash | 1 | 123 | Ändern (Ersatz CRC32C/Mac) |
| `src/csr/graph_index/mutations.rs` | `commit_impl` | fn | Altpfad-Typ | 1 | 186 | Entfernen/Ändern |
| `src/csr/graph_index/mutations.rs` | `remove_edge_impl` | fn | Altpfad-Typ | 1 | 206 | Entfernen/Ändern |
| `src/csr/graph_index/mutations.rs` | `commit_staged_entities` | fn | Altpfad-Typ | 1 | 229 | Entfernen/Ändern |
| `src/csr/graph_index/mutations.rs` | `commit_staged_edges` | fn | Altpfad-Typ | 1 | 252 | Entfernen/Ändern |
| `src/csr/graph_index/mutations.rs` | `commit_staged_removals` | fn | Altpfad-Typ | 1 | 301 | Entfernen/Ändern |
| `src/csr/graph_index/traversal.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 10 | Entfernen/Ändern |
| `src/csr/graph_index/traversal.rs` | `traverse_at_bitemporal_impl` | fn | Altpfad-Typ | 1 | 21 | Entfernen/Ändern |
| `src/csr/graph_index/traversal.rs` | `BitemporalBfsArgs` | struct | Altpfad-Typ | 1 | 87 | Entfernen/Ändern |
| `src/csr/graph_index/traversal.rs` | `execute_bfs_bitemporal` | fn | Altpfad-Typ | 1 | 165 | Entfernen/Ändern |
| `src/csr/graph_persist.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 13 | Entfernen (legacy) |
| `src/csr/graph_persist.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 14 | Entfernen/Ändern |
| `src/csr/graph_persist.rs` | `persist_entity` | fn | Altpfad-Port | 1 | 24 | Entfernen (legacy) |
| `src/csr/graph_persist.rs` | `persist_entity` | fn | Altpfad-Typ | 1 | 27 | Entfernen/Ändern |
| `src/csr/graph_persist.rs` | `persist_entity` | fn | Verboten-Serial | 1 | 31 | Ändern (feature-gate oder Ersatz) |
| `src/csr/graph_persist.rs` | `persist_edge` | fn | Altpfad-Port | 1 | 37 | Entfernen (legacy) |
| `src/csr/graph_persist.rs` | `persist_edge` | fn | Altpfad-Typ | 1 | 40 | Entfernen/Ändern |
| `src/csr/graph_persist.rs` | `persist_edge` | fn | Verboten-Serial | 1 | 52 | Ändern (feature-gate oder Ersatz) |
| `src/csr/graph_persist.rs` | `delete_edge_persistence` | fn | Altpfad-Port | 1 | 58 | Entfernen (legacy) |
| `src/csr/graph_persist.rs` | `delete_edge_persistence` | fn | Altpfad-Typ | 1 | 61 | Entfernen/Ändern |
| `src/csr/graph_persist.rs` | `load_from_storage` | fn | Altpfad-Port | 1 | 76 | Entfernen (legacy) |
| `src/csr/graph_persist.rs` | `load_from_storage` | fn | Verboten-Serial | 2 | 94 | Ändern (feature-gate oder Ersatz) |
| `src/csr/graph_persist.rs` | `LegacyPersistedEdgePayloadV2` | struct | Altpfad-Typ | 2 | 128 | Entfernen/Ändern |
| `src/csr/graph_persist.rs` | `LegacyPersistedEdgePayloadV2` | struct | Verboten-Serial | 1 | 135 | Ändern (feature-gate oder Ersatz) |
| `src/csr/graph_persist.rs` | `LegacyPersistedEdgePayloadV1` | struct | Altpfad-Typ | 3 | 150 | Entfernen/Ändern |
| `src/csr/graph_persist.rs` | `LegacyPersistedEdgePayloadV1` | struct | Verboten-Serial | 2 | 155 | Ändern (feature-gate oder Ersatz) |
| `src/csr/graph_read/mod.rs` | `path_rag` | mod | Altpfad-Typ | 1 | 11 | Entfernen/Ändern |
| `src/csr/graph_read/mod.rs` | `remove_entity` | fn | Altpfad-Typ | 1 | 163 | Entfernen/Ändern |
| `src/csr/graph_read/path_rag.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 8 | Entfernen/Ändern |
| `src/csr/graph_read/path_rag.rs` | `path_rag_at` | fn | Altpfad-Typ | 3 | 19 | Entfernen/Ändern |
| `src/csr/graph_write/edge_mutation.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 9 | Entfernen/Ändern |
| `src/csr/graph_write/edge_mutation.rs` | `add_edge` | fn | Altpfad-Typ | 2 | 52 | Entfernen/Ändern |
| `src/csr/graph_write/edge_mutation.rs` | `insert_edge_direct_with_validity` | fn | Altpfad-Typ | 2 | 124 | Entfernen/Ändern |
| `src/csr/graph_write/edge_mutation.rs` | `insert_edge_direct_with_bitemporal_validity` | fn | Altpfad-Typ | 2 | 148 | Entfernen/Ändern |
| `src/csr/graph_write/edge_mutation.rs` | `load_edge_direct` | fn | Altpfad-Typ | 2 | 176 | Entfernen/Ändern |
| `src/csr/graph_write/hyperedge.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 8 | Entfernen/Ändern |
| `src/csr/graph_write/hyperedge.rs` | `process_cascade_queue` | fn | Altpfad-Typ | 1 | 58 | Entfernen/Ändern |
| `src/csr/graph_write/hyperedge.rs` | `persist_hyperedge` | fn | Altpfad-Typ | 1 | 130 | Entfernen/Ändern |
| `src/csr/graph_write/hyperedge.rs` | `apply_batch_tombstones` | fn | Altpfad-Typ | 1 | 276 | Entfernen/Ändern |
| `src/csr/graph_write/hyperedge.rs` | `commit_super_edge_batch` | fn | Altpfad-Typ | 1 | 361 | Entfernen/Ändern |
| `src/csr/graph_write/mod.rs` | `tombstone` | mod | Verboten-Laufzeit | 2 | 11 | Ändern (kein Async/Lock im L2) |
| `src/csr/graph_write/mod.rs` | `tombstone` | mod | Altpfad-Port | 1 | 19 | Entfernen (legacy) |
| `src/csr/graph_write/mod.rs` | `CsrGraph` | struct | Verboten-Laufzeit | 1 | 28 | Ändern (kein Async/Lock im L2) |
| `src/csr/graph_write/mod.rs` | `CsrGraph` | struct | Altpfad-Port | 1 | 30 | Entfernen (legacy) |
| `src/csr/graph_write/mod.rs` | `with_config` | fn | Verboten-Laufzeit | 1 | 48 | Ändern (kein Async/Lock im L2) |
| `src/csr/graph_write/mod.rs` | `with_storage` | fn | Altpfad-Port | 1 | 59 | Entfernen (legacy) |
| `src/csr/graph_write/mod.rs` | `with_config_and_storage` | fn | Altpfad-Port | 1 | 66 | Entfernen (legacy) |
| `src/csr/graph_write/mod.rs` | `with_config_and_storage` | fn | Verboten-Laufzeit | 1 | 71 | Ändern (kein Async/Lock im L2) |
| `src/csr/graph_write/mod.rs` | `with_consistency_enforcer` | fn | Verboten-Laufzeit | 1 | 86 | Ändern (kein Async/Lock im L2) |
| `src/csr/graph_write/mod.rs` | `rollback_to_tx` | fn | Altpfad-Typ | 1 | 115 | Entfernen/Ändern |
| `src/csr/graph_write/mod.rs` | `inner_read` | fn | Verboten-Laufzeit | 1 | 120 | Ändern (kein Async/Lock im L2) |
| `src/csr/graph_write/mod.rs` | `inner_write` | fn | Verboten-Laufzeit | 1 | 127 | Ändern (kein Async/Lock im L2) |
| `src/csr/graph_write/mod.rs` | `set_storage` | fn | Altpfad-Port | 1 | 132 | Entfernen (legacy) |
| `src/csr/graph_write/mod.rs` | `storage` | fn | Altpfad-Port | 1 | 137 | Entfernen (legacy) |
| `src/csr/graph_write/tombstone.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 8 | Entfernen/Ändern |
| `src/csr/graph_write/tombstone.rs` | `tombstone_edge` | fn | Altpfad-Typ | 1 | 15 | Entfernen/Ändern |
| `src/csr/graph_write/tombstone.rs` | `tombstone_edges_direct` | fn | Altpfad-Typ | 1 | 27 | Entfernen/Ändern |
| `src/csr/graph_write/tombstone.rs` | `tombstone_hyperedge` | fn | Altpfad-Typ | 1 | 72 | Entfernen/Ändern |
| `src/csr/inner/compaction.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 6 | Entfernen/Ändern |
| `src/csr/inner/compaction.rs` | `apply_compacted_arrays` | fn | Altpfad-Typ | 2 | 135 | Entfernen/Ändern |
| `src/csr/inner/compaction.rs` | `compact` | fn | Altpfad-Typ | 2 | 220 | Entfernen/Ändern |
| `src/csr/inner/estimate.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 9 | Entfernen/Ändern |
| `src/csr/inner/estimate.rs` | `estimate_primary_tables_memory` | fn | Altpfad-Typ | 3 | 62 | Entfernen/Ändern |
| `src/csr/inner/estimate.rs` | `estimate_csr_vectors_memory` | fn | Altpfad-Typ | 2 | 115 | Entfernen/Ändern |
| `src/csr/inner/mod.rs` | `reinforcement` | mod | Verboten-Laufzeit | 1 | 10 | Ändern (kein Async/Lock im L2) |
| `src/csr/inner/mod.rs` | `reinforcement` | mod | Altpfad-Typ | 1 | 14 | Entfernen/Ändern |
| `src/csr/inner/mod.rs` | `GraphInner` | struct | Altpfad-Typ | 5 | 34 | Entfernen/Ändern |
| `src/csr/inner/mod.rs` | `tx_valid_from_at` | fn | Altpfad-Typ | 2 | 112 | Entfernen/Ändern |
| `src/csr/inner/mod.rs` | `tx_valid_to_at` | fn | Altpfad-Typ | 2 | 120 | Entfernen/Ändern |
| `src/csr/inner/mod.rs` | `InnerWriteGuard` | struct | Verboten-Laufzeit | 2 | 153 | Ändern (kein Async/Lock im L2) |
| `src/csr/inner/mod.rs` | `drop` | fn | Verboten-Laufzeit | 1 | 172 | Ändern (kein Async/Lock im L2) |
| `src/csr/inner/reinforcement.rs` | `sync_edge_reinforcement_weights` | fn | Altpfad-Graph | 2 | 118 | Entfernen |
| `src/csr/types.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 6 | Entfernen/Ändern |
| `src/csr/types.rs` | `PersistedEdgePayload` | struct | Altpfad-Typ | 2 | 58 | Entfernen/Ändern |
| `src/csr/types.rs` | `EdgePayload` | struct | Altpfad-Typ | 2 | 105 | Entfernen/Ändern |
| `src/csr/types.rs` | `StagedEdgePayload` | struct | Altpfad-Typ | 2 | 116 | Entfernen/Ändern |
| `src/csr/visibility.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 6 | Entfernen/Ändern |
| `src/csr/visibility.rs` | `is_suspicious_tx_id` | fn | Altpfad-Typ | 2 | 38 | Entfernen/Ändern |
| `src/csr/visibility.rs` | `is_edge_visible` | fn | Altpfad-Typ | 3 | 47 | Entfernen/Ändern |
| `src/csr/visibility.rs` | `is_edge_visible_bitemporal` | fn | Altpfad-Typ | 3 | 73 | Entfernen/Ändern |
| `src/hyperedge.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 9 | Entfernen/Ändern |
| `src/hyperedge.rs` | `—` | (Modulebene) | Verboten-Laufzeit | 1 | 10 | Ändern (kein Async/Lock im L2) |
| `src/hyperedge.rs` | `HyperEdge` | struct | Altpfad-Typ | 2 | 58 | Entfernen/Ändern |
| `src/hyperedge.rs` | `LegacyHyperEdgeV1` | struct | Altpfad-Typ | 2 | 83 | Entfernen/Ändern |
| `src/hyperedge.rs` | `with_tx_validity` | fn | Altpfad-Typ | 1 | 119 | Entfernen/Ändern |
| `src/hyperedge.rs` | `serialize` | fn | Verboten-Serial | 1 | 193 | Ändern (feature-gate oder Ersatz) |
| `src/hyperedge.rs` | `deserialize` | fn | Verboten-Serial | 2 | 201 | Ändern (feature-gate oder Ersatz) |
| `src/hyperedge.rs` | `test_hyperedge_bincode_serialization_roundtrip` | fn | Altpfad-Typ | 1 | 470 | Entfernen/Ändern |
| `src/hyperedge.rs` | `test_hyperedge_legacy_deserialization_fallback` | fn | Altpfad-Typ | 1 | 494 | Entfernen/Ändern |
| `src/hyperedge.rs` | `test_hyperedge_legacy_deserialization_fallback` | fn | Verboten-Serial | 1 | 501 | Ändern (feature-gate oder Ersatz) |
| `src/hyperedge_suggest.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 7 | Entfernen (legacy) |
| `src/hyperedge_suggest.rs` | `validate_candidates_with_llm` | fn | Altpfad-Port | 1 | 121 | Entfernen (legacy) |
| `src/lib.rs` | `edge_reinforcement` | mod | Altpfad-Graph | 1 | 42 | Entfernen |
| `src/lib.rs` | `edge_reinforcement_buffer` | mod | Altpfad-Graph | 1 | 44 | Entfernen |
| `src/lib.rs` | `session_dag` | mod | Altpfad-Graph | 1 | 58 | Entfernen |
| `src/lib.rs` | `tl_hfd` | mod | Altpfad-Graph | 5 | 84 | Entfernen |
| `src/lib.rs` | `remove_entity` | fn | Altpfad-Typ | 1 | 136 | Entfernen/Ändern |
| `src/path_rag/snapshot.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 12 | Entfernen/Ändern |
| `src/path_rag/snapshot.rs` | `SnapshotPathGraph` | struct | Altpfad-Typ | 1 | 23 | Entfernen/Ändern |
| `src/path_rag/snapshot.rs` | `new` | fn | Altpfad-Typ | 1 | 29 | Entfernen/Ändern |
| `src/percolation.rs` | `tests` | mod | Altpfad-Typ | 1 | 152 | Entfernen/Ändern |
| `src/percolation.rs` | `test_percolation_skips_too_small_graphs` | fn | Verboten-Laufzeit | 1 | 205 | Ändern (kein Async/Lock im L2) |
| `src/percolation.rs` | `test_find_rebonding_candidates` | fn | Altpfad-Typ | 1 | 208 | Entfernen/Ändern |
| `src/percolation.rs` | `test_cosine_similarity_edge_cases` | fn | Verboten-Laufzeit | 1 | 270 | Ändern (kein Async/Lock im L2) |
| `src/percolation.rs` | `test_find_rebonding_candidates_empty_or_single_node` | fn | Verboten-Laufzeit | 1 | 288 | Ändern (kein Async/Lock im L2) |
| `src/ppr/cost.rs` | `—` | (Modulebene) | Verboten-Laufzeit | 1 | 4 | Ändern (kein Async/Lock im L2) |
| `src/ppr/snapshot.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 11 | Entfernen/Ändern |
| `src/ppr/snapshot.rs` | `filter_node_visible_edges` | fn | Altpfad-Typ | 1 | 14 | Entfernen/Ändern |
| `src/ppr/snapshot.rs` | `populate_snapshot_hyperedges` | fn | Altpfad-Typ | 1 | 69 | Entfernen/Ändern |
| `src/ppr/snapshot.rs` | `compute_ppr_at` | fn | Altpfad-Typ | 1 | 103 | Entfernen/Ändern |
| `src/provenance.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 14 | Entfernen/Ändern |
| `src/provenance.rs` | `—` | (Modulebene) | Verboten-Laufzeit | 1 | 15 | Ändern (kein Async/Lock im L2) |
| `src/provenance.rs` | `EdgeProvenance` | struct | Altpfad-Typ | 1 | 29 | Entfernen/Ändern |
| `src/provenance.rs` | `new` | fn | Altpfad-Typ | 1 | 34 | Entfernen/Ändern |
| `src/provenance.rs` | `test_record_provenance` | fn | Altpfad-Typ | 1 | 133 | Entfernen/Ändern |
| `src/provenance.rs` | `test_record_provenance_empty_docs` | fn | Altpfad-Typ | 1 | 145 | Entfernen/Ändern |
| `src/provenance.rs` | `test_edge_provenance_serde_roundtrip` | fn | Altpfad-Typ | 1 | 183 | Entfernen/Ändern |
| `src/storage.rs` | `InMemoryGraphStorage` | struct | Verboten-Laufzeit | 1 | 30 | Ändern (kein Async/Lock im L2) |
| `src/tl_hfd/mod.rs` | `tests` | mod | Verboten-Laufzeit | 1 | 64 | Ändern (kein Async/Lock im L2) |
| `src/types.rs` | `from_key` | fn | Verboten-Hash | 2 | 31 | Ändern (Ersatz CRC32C/Mac) |
| `src/types.rs` | `TxId` | struct | Altpfad-Typ | 1 | 104 | Entfernen/Ändern |
| `src/types.rs` | `TxId` | impl | Altpfad-Typ | 1 | 106 | Entfernen/Ändern |
| `src/types.rs` | `fmt` | impl | Altpfad-Typ | 1 | 118 | Entfernen/Ändern |
| `src/types.rs` | `From` | impl | Altpfad-Typ | 2 | 124 | Entfernen/Ändern |
| `src/types.rs` | `from` | fn | Altpfad-Typ | 1 | 131 | Entfernen/Ändern |

### contextra-mvcc (3 Einträge)

| Datei | Element | Art | Kategorie | Treffer | Erste Zeile | Aktion |
|---|---|---|---|---:|---:|---|
| `src/lib.rs` | `contextra_types` | mod | Altpfad-Typ | 1 | 86 | Entfernen/Ändern |
| `src/lib.rs` | `tx_buffer` | mod | Altpfad-Typ | 2 | 94 | Entfernen/Ändern |
| `src/snapshot.rs` | `—` | (Modulebene) | Verboten-Laufzeit | 1 | 36 | Ändern (kein Async/Lock im L2) |

### contextra-ports (101 Einträge)

| Datei | Element | Art | Kategorie | Treffer | Erste Zeile | Aktion |
|---|---|---|---|---:|---:|---|
| `src/barrier.rs` | `CommitBarrier` | trait | Altpfad-Port | 1 | 91 | Entfernen (legacy) |
| `src/barrier.rs` | `PurgeBarrier` | trait | Altpfad-Port | 1 | 112 | Entfernen (legacy) |
| `src/barrier.rs` | `CommitBarrier` | impl | Altpfad-Port | 1 | 143 | Entfernen (legacy) |
| `src/barrier.rs` | `PurgeBarrier` | impl | Altpfad-Port | 1 | 153 | Entfernen (legacy) |
| `src/checkpoint.rs` | `take_snapshot` | fn | Altpfad-Typ | 1 | 11 | Entfernen/Ändern |
| `src/checkpoint.rs` | `take_snapshot` | fn | Altpfad-Checkpoint | 1 | 11 | Entfernen |
| `src/checkpoint.rs` | `restore` | fn | Altpfad-Checkpoint | 1 | 14 | Entfernen |
| `src/checkpoint.rs` | `create_named_checkpoint` | fn | Altpfad-Typ | 1 | 36 | Entfernen/Ändern |
| `src/conformance.rs` | `check` | mod | Altpfad-Port | 5 | 11 | Entfernen (legacy) |
| `src/conformance.rs` | `check` | mod | Altpfad-Typ | 1 | 32 | Entfernen/Ändern |
| `src/conformance.rs` | `check` | mod | Altpfad-Checkpoint | 1 | 32 | Entfernen |
| `src/conformance.rs` | `assert_checkpoint_contract` | fn | Altpfad-Typ | 1 | 38 | Entfernen/Ändern |
| `src/conformance.rs` | `assert_checkpoint_contract` | fn | Altpfad-Checkpoint | 2 | 38 | Entfernen |
| `src/conformance.rs` | `assert_checkpoint_coordinator_contract` | fn | Altpfad-Typ | 1 | 48 | Entfernen/Ändern |
| `src/conformance.rs` | `assert_vector_index_contract` | fn | Altpfad-Typ | 1 | 68 | Entfernen/Ändern |
| `src/conformance.rs` | `assert_attention_exporter_contract` | fn | Altpfad-Port | 1 | 72 | Entfernen (legacy) |
| `src/conformance.rs` | `assert_graph_index_contract` | fn | Altpfad-Typ | 1 | 79 | Entfernen/Ändern |
| `src/conformance.rs` | `assert_embedding_contract` | fn | Altpfad-Port | 3 | 109 | Entfernen (legacy) |
| `src/conformance.rs` | `assert_storage_engine_contract` | fn | Altpfad-Port | 1 | 153 | Entfernen (legacy) |
| `src/conformance.rs` | `assert_storage_engine_contract` | fn | Altpfad-Typ | 1 | 157 | Entfernen/Ändern |
| `src/conformance.rs` | `assert_storage_engine_contract` | fn | Altpfad-Checkpoint | 1 | 167 | Entfernen |
| `src/conformance.rs` | `assert_purger_contract` | fn | Altpfad-Port | 1 | 175 | Entfernen (legacy) |
| `src/conformance.rs` | `assert_vfs_file_contract` | fn | Altpfad-Port | 1 | 180 | Entfernen (legacy) |
| `src/conformance.rs` | `assert_erasure_layer_contract` | fn | Altpfad-Port | 1 | 223 | Entfernen (legacy) |
| `src/embedding.rs` | `TextEmbeddingEngine` | trait | Altpfad-Port | 1 | 43 | Entfernen (legacy) |
| `src/embedding.rs` | `embed_batch` | fn | Altpfad-Port | 1 | 120 | Entfernen (legacy) |
| `src/embedding.rs` | `LlmTextGenerator` | trait | Altpfad-Port | 1 | 128 | Entfernen (legacy) |
| `src/embedding.rs` | `LlmTextGeneratorStreaming` | trait | Altpfad-Port | 1 | 153 | Entfernen (legacy) |
| `src/embedding.rs` | `TextEmbeddingEngine` | impl | Altpfad-Port | 1 | 163 | Entfernen (legacy) |
| `src/embedding.rs` | `tests` | mod | Verboten-Laufzeit | 1 | 187 | Ändern (kein Async/Lock im L2) |
| `src/embedding.rs` | `LlmTextGenerator` | impl | Altpfad-Port | 1 | 200 | Entfernen (legacy) |
| `src/embedding.rs` | `generate` | fn | Verboten-Laufzeit | 1 | 207 | Ändern (kein Async/Lock im L2) |
| `src/embedding.rs` | `test_context_segment_and_generate_with_context_default` | fn | Verboten-Laufzeit | 1 | 234 | Ändern (kein Async/Lock im L2) |
| `src/embedding.rs` | `test_generate_with_context_default_matches_concatenation` | fn | Verboten-Laufzeit | 1 | 247 | Ändern (kein Async/Lock im L2) |
| `src/fs.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 6 | Entfernen (legacy) |
| `src/fs.rs` | `prelude` | mod | Altpfad-Port | 1 | 10 | Entfernen (legacy) |
| `src/graph_index.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 8 | Entfernen/Ändern |
| `src/graph_index.rs` | `remove_edge` | fn | Altpfad-Typ | 1 | 69 | Entfernen/Ändern |
| `src/graph_index.rs` | `add_bidirectional` | fn | Altpfad-Typ | 1 | 82 | Entfernen/Ändern |
| `src/graph_index.rs` | `traverse_at_time` | fn | Altpfad-Typ | 2 | 218 | Entfernen/Ändern |
| `src/graph_index.rs` | `traverse_at_bitemporal` | fn | Altpfad-Typ | 2 | 237 | Entfernen/Ändern |
| `src/graph_index.rs` | `add_entity` | fn | Altpfad-Typ | 3 | 273 | Entfernen/Ändern |
| `src/graph_index.rs` | `add_edge` | fn | Altpfad-Typ | 3 | 276 | Entfernen/Ändern |
| `src/graph_index.rs` | `commit` | fn | Altpfad-Typ | 3 | 279 | Entfernen/Ändern |
| `src/graph_index.rs` | `rollback` | fn | Altpfad-Typ | 3 | 282 | Entfernen/Ändern |
| `src/graph_index.rs` | `rollback_to_tx` | fn | Altpfad-Typ | 3 | 285 | Entfernen/Ändern |
| `src/graph_index.rs` | `last_tx_id` | fn | Altpfad-Typ | 5 | 288 | Entfernen/Ändern |
| `src/graph_index.rs` | `tests` | mod | Verboten-Laufzeit | 1 | 324 | Ändern (kein Async/Lock im L2) |
| `src/graph_index.rs` | `stats` | fn | Altpfad-Typ | 3 | 402 | Entfernen/Ändern |
| `src/graph_index.rs` | `stats` | fn | Verboten-Laufzeit | 1 | 424 | Ändern (kein Async/Lock im L2) |
| `src/ledger.rs` | `compute_chained` | fn | Verboten-Hash | 1 | 103 | Ändern (Ersatz CRC32C/Mac) |
| `src/ledger.rs` | `verify_chain_hash` | fn | Verboten-Hash | 1 | 126 | Ändern (Ersatz CRC32C/Mac) |
| `src/lib.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 29 | Entfernen/Ändern |
| `src/lib.rs` | `—` | (Modulebene) | Altpfad-Checkpoint | 1 | 29 | Entfernen |
| `src/lib.rs` | `policy` | mod | Altpfad-Port | 1 | 115 | Entfernen (legacy) |
| `src/lib.rs` | `_assert_dyn_storage` | fn | Altpfad-Port | 1 | 142 | Entfernen (legacy) |
| `src/lib.rs` | `_assert_dyn_kv_bridge_storage` | fn | Altpfad-Port | 1 | 144 | Entfernen (legacy) |
| `src/lib.rs` | `_assert_dyn_kv_prefix_store` | fn | Altpfad-Port | 1 | 145 | Entfernen (legacy) |
| `src/lib.rs` | `_assert_dyn_embedding` | fn | Altpfad-Port | 1 | 148 | Entfernen (legacy) |
| `src/lib.rs` | `_assert_dyn_attention_exporter` | fn | Altpfad-Port | 1 | 154 | Entfernen (legacy) |
| `src/lib.rs` | `_assert_dyn_layer_scanner` | fn | Altpfad-Port | 1 | 157 | Entfernen (legacy) |
| `src/lib.rs` | `_assert_dyn_purger` | fn | Altpfad-Port | 1 | 158 | Entfernen (legacy) |
| `src/lib.rs` | `report_signer` | mod | Altpfad-Port | 3 | 194 | Entfernen (legacy) |
| `src/scanner/session.rs` | `finish` | fn | Verboten-Hash | 1 | 86 | Ändern (Ersatz CRC32C/Mac) |
| `src/storage.rs` | `StorageEngine` | trait | Altpfad-Port | 1 | 35 | Entfernen (legacy) |
| `src/storage.rs` | `get_tracked` | fn | Altpfad-Typ | 1 | 50 | Entfernen/Ändern |
| `src/storage.rs` | `get_at_seq_tracked` | fn | Altpfad-Typ | 1 | 63 | Entfernen/Ändern |
| `src/storage.rs` | `scan_prefix_tracked` | fn | Altpfad-Typ | 1 | 78 | Entfernen/Ändern |
| `src/storage.rs` | `put` | fn | Altpfad-Typ | 1 | 97 | Entfernen/Ändern |
| `src/storage.rs` | `put_if_absent` | fn | Altpfad-Typ | 1 | 102 | Entfernen/Ändern |
| `src/storage.rs` | `put_batch` | fn | Altpfad-Typ | 1 | 117 | Entfernen/Ändern |
| `src/storage.rs` | `delete` | fn | Altpfad-Typ | 1 | 129 | Entfernen/Ändern |
| `src/storage.rs` | `delete_many` | fn | Altpfad-Typ | 1 | 132 | Entfernen/Ändern |
| `src/storage.rs` | `delete_prefix` | fn | Altpfad-Typ | 1 | 145 | Entfernen/Ändern |
| `src/storage.rs` | `commit` | fn | Altpfad-Typ | 1 | 158 | Entfernen/Ändern |
| `src/storage.rs` | `rollback` | fn | Altpfad-Typ | 1 | 161 | Entfernen/Ändern |
| `src/storage.rs` | `rollback_to_tx` | fn | Altpfad-Typ | 1 | 164 | Entfernen/Ändern |
| `src/storage.rs` | `last_tx_id` | fn | Altpfad-Typ | 1 | 176 | Entfernen/Ändern |
| `src/storage.rs` | `pin_checkpoint` | fn | Altpfad-Checkpoint | 1 | 179 | Entfernen |
| `src/storage.rs` | `prelude` | mod | Altpfad-Port | 2 | 269 | Entfernen (legacy) |
| `src/text_index.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 8 | Entfernen/Ändern |
| `src/text_index.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 13 | Entfernen (legacy) |
| `src/text_index.rs` | `insert` | fn | Altpfad-Typ | 3 | 81 | Entfernen/Ändern |
| `src/text_index.rs` | `delete` | fn | Altpfad-Typ | 3 | 89 | Entfernen/Ändern |
| `src/text_index.rs` | `commit` | fn | Altpfad-Typ | 3 | 97 | Entfernen/Ändern |
| `src/text_index.rs` | `rollback` | fn | Altpfad-Typ | 3 | 105 | Entfernen/Ändern |
| `src/text_index.rs` | `rollback_to_tx` | fn | Altpfad-Typ | 3 | 113 | Entfernen/Ändern |
| `src/text_index.rs` | `last_tx_id` | fn | Altpfad-Typ | 5 | 121 | Entfernen/Ändern |
| `src/text_index.rs` | `test_text_index_stats_serialization` | fn | Verboten-Laufzeit | 1 | 166 | Ändern (kein Async/Lock im L2) |
| `src/text_index.rs` | `stats` | fn | Verboten-Laufzeit | 1 | 216 | Ändern (kein Async/Lock im L2) |
| `src/vector_index.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 8 | Entfernen/Ändern |
| `src/vector_index.rs` | `insert` | fn | Altpfad-Typ | 3 | 41 | Entfernen/Ändern |
| `src/vector_index.rs` | `insert_batch` | fn | Altpfad-Typ | 1 | 59 | Entfernen/Ändern |
| `src/vector_index.rs` | `delete` | fn | Altpfad-Typ | 3 | 134 | Entfernen/Ändern |
| `src/vector_index.rs` | `commit` | fn | Altpfad-Typ | 3 | 142 | Entfernen/Ändern |
| `src/vector_index.rs` | `rollback` | fn | Altpfad-Typ | 3 | 150 | Entfernen/Ändern |
| `src/vector_index.rs` | `rollback_to_tx` | fn | Altpfad-Typ | 3 | 158 | Entfernen/Ändern |
| `src/vector_index.rs` | `last_tx_id` | fn | Altpfad-Typ | 5 | 166 | Entfernen/Ändern |
| `src/vector_index.rs` | `test_vector_index_stats_serialization` | fn | Verboten-Laufzeit | 1 | 243 | Ändern (kein Async/Lock im L2) |
| `src/vector_index.rs` | `stats` | fn | Verboten-Laufzeit | 1 | 297 | Ändern (kein Async/Lock im L2) |
| `src/vector_index.rs` | `stats` | fn | Altpfad-Typ | 1 | 344 | Entfernen/Ändern |

### contextra-rank (15 Einträge)

| Datei | Element | Art | Kategorie | Treffer | Erste Zeile | Aktion |
|---|---|---|---|---:|---:|---|
| `src/dibud/driver.rs` | `fuse_exact_prefix` | fn | Altpfad-Graph | 2 | 22 | Entfernen |
| `src/dibud/driver.rs` | `fuse_exact_prefix_async` | fn | Altpfad-Graph | 2 | 53 | Entfernen |
| `src/dibud/state.rs` | `compute_f_minus` | fn | Altpfad-Graph | 1 | 185 | Entfernen |
| `src/dibud/state.rs` | `compute_f_plus_unseen` | fn | Altpfad-Graph | 1 | 211 | Entfernen |
| `src/dibud/types.rs` | `FusionBudget` | struct | Altpfad-Graph | 2 | 55 | Entfernen |
| `src/dibud/types.rs` | `default` | fn | Altpfad-Graph | 2 | 67 | Entfernen |
| `src/dibud/types.rs` | `validate` | fn | Altpfad-Graph | 7 | 97 | Entfernen |
| `src/explain.rs` | `extract_raw_signal_entries` | fn | Altpfad-Graph | 1 | 140 | Entfernen |
| `src/explain.rs` | `test_four_signals_exact_contributions` | fn | Altpfad-Graph | 1 | 339 | Entfernen |
| `src/explain.rs` | `test_signal_kind_golden_serde_and_display` | fn | Altpfad-Graph | 1 | 404 | Entfernen |
| `src/fusion/rrf.rs` | `update_entry_kind_scores` | fn | Altpfad-Graph | 1 | 312 | Entfernen |
| `src/fusion/signal.rs` | `SignalKind` | enum | Altpfad-Graph | 1 | 102 | Entfernen |
| `src/fusion/signal.rs` | `from_name` | fn | Altpfad-Graph | 1 | 129 | Entfernen |
| `src/fusion/signal.rs` | `as_str` | fn | Altpfad-Graph | 1 | 140 | Entfernen |
| `src/fusion/signal.rs` | `name_de` | fn | Altpfad-Graph | 1 | 150 | Entfernen |

### contextra-storage (120 Einträge)

| Datei | Element | Art | Kategorie | Treffer | Erste Zeile | Aktion |
|---|---|---|---|---:|---:|---|
| `src/catalog/edit.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 4 | Ändern (Import/Abhängigkeit) |
| `src/catalog/file.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 5 | Ändern (Import/Abhängigkeit) |
| `src/catalog/ops.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 3 | Ändern (Import/Abhängigkeit) |
| `src/catalog/record.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 4 | Ändern (Import/Abhängigkeit) |
| `src/catalog/reserve.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 4 | Ändern (Import/Abhängigkeit) |
| `src/catalog/rollover.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 3 | Ändern (Import/Abhängigkeit) |
| `src/catalog/state.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 5 | Ändern (Import/Abhängigkeit) |
| `src/checkpoint/create.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 4 | Ändern (Import/Abhängigkeit) |
| `src/checkpoint/create.rs` | `write_manifest` | fn | Verlorener-Fehler | 2 | 193 | Ändern (Fehler propagieren) |
| `src/checkpoint/restore.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 5 | Ändern (Import/Abhängigkeit) |
| `src/codec/comparator.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 3 | Ändern (Import/Abhängigkeit) |
| `src/codec/format.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 3 | Ändern (Import/Abhängigkeit) |
| `src/codec/header.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 3 | Ändern (Import/Abhängigkeit) |
| `src/codec/keys_codec.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 3 | Ändern (Import/Abhängigkeit) |
| `src/codec/seal.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 4 | Ändern (Import/Abhängigkeit) |
| `src/codec/tags.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 3 | Ändern (Import/Abhängigkeit) |
| `src/config.rs` | `—` | (Modulebene) | Phantom-Crate | 3 | 2 | Ändern (Import/Abhängigkeit) |
| `src/engine/branches.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 7 | Ändern (Import/Abhängigkeit) |
| `src/engine/cache.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 4 | Ändern (Import/Abhängigkeit) |
| `src/engine/committer.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 8 | Ändern (Import/Abhängigkeit) |
| `src/engine/cursor.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 5 | Ändern (Import/Abhängigkeit) |
| `src/engine/extension.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 5 | Ändern (Import/Abhängigkeit) |
| `src/engine/feed.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 5 | Ändern (Import/Abhängigkeit) |
| `src/engine/guard_window.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 8 | Ändern (Import/Abhängigkeit) |
| `src/engine/idem.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 4 | Ändern (Import/Abhängigkeit) |
| `src/engine/memtable.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 4 | Ändern (Import/Abhängigkeit) |
| `src/engine/packgroups.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 4 | Ändern (Import/Abhängigkeit) |
| `src/engine/page.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 4 | Ändern (Import/Abhängigkeit) |
| `src/engine/registry.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 4 | Ändern (Import/Abhängigkeit) |
| `src/engine/segments.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 4 | Ändern (Import/Abhängigkeit) |
| `src/engine/version.rs` | `—` | (Modulebene) | Phantom-Crate | 3 | 4 | Ändern (Import/Abhängigkeit) |
| `src/engine/views.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 12 | Ändern (Import/Abhängigkeit) |
| `src/engine/volatile.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 9 | Ändern (Import/Abhängigkeit) |
| `src/engine/volatile.rs` | `validate_change` | fn | Phantom-Crate | 3 | 34 | Ändern (Import/Abhängigkeit) |
| `src/env/anchor.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 4 | Ändern (Import/Abhängigkeit) |
| `src/env/anchor.rs` | `write_anchor_atomic` | fn | Verlorener-Fehler | 2 | 210 | Ändern (Fehler propagieren) |
| `src/env/budget.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 3 | Ändern (Import/Abhängigkeit) |
| `src/env/faultfs.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 3 | Ändern (Import/Abhängigkeit) |
| `src/env/memfs.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 3 | Ändern (Import/Abhängigkeit) |
| `src/env/sim.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 4 | Ändern (Import/Abhängigkeit) |
| `src/erase/cluster_table.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 3 | Ändern (Import/Abhängigkeit) |
| `src/erase/gate.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 10 | Ändern (Import/Abhängigkeit) |
| `src/erase/ledger.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 7 | Ändern (Import/Abhängigkeit) |
| `src/erase/links.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 8 | Ändern (Import/Abhängigkeit) |
| `src/erase/protocol.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 16 | Ändern (Import/Abhängigkeit) |
| `src/erase/protocol.rs` | `recover_from_record` | fn | Feste-Löschevidenz | 4 | 197 | Ändern (gemessene Evidenz) |
| `src/erase/protocol.rs` | `finalize` | fn | Phantom-Crate | 1 | 561 | Ändern (Import/Abhängigkeit) |
| `src/erase/receipt.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 7 | Ändern (Import/Abhängigkeit) |
| `src/erase/restrictions.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 7 | Ändern (Import/Abhängigkeit) |
| `src/erase/tickets.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 7 | Ändern (Import/Abhängigkeit) |
| `src/keys/cache.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 4 | Ändern (Import/Abhängigkeit) |
| `src/keys/cache.rs` | `get` | fn | Phantom-Crate | 1 | 103 | Ändern (Import/Abhängigkeit) |
| `src/keys/wrapper.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 3 | Ändern (Import/Abhängigkeit) |
| `src/lib.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 8 | Ändern (Import/Abhängigkeit) |
| `src/lib.rs` | `testing` | mod | Phantom-Crate | 2 | 68 | Ändern (Import/Abhängigkeit) |
| `src/lib.rs` | `CoreChecksum` | struct | Sim-als-Produktion | 1 | 88 | Ändern (cfg(testing)) |
| `src/lib.rs` | `Checksum` | impl | Sim-als-Produktion | 1 | 90 | Ändern (cfg(testing)) |
| `src/lib.rs` | `PlainNoopCipher` | struct | Sim-als-Produktion | 1 | 97 | Ändern (cfg(testing)) |
| `src/lib.rs` | `Cipher` | impl | Sim-als-Produktion | 1 | 99 | Ändern (cfg(testing)) |
| `src/lib.rs` | `CoreMac` | struct | Sim-als-Produktion | 1 | 126 | Ändern (cfg(testing)) |
| `src/lib.rs` | `Mac` | impl | Sim-als-Produktion | 1 | 128 | Ändern (cfg(testing)) |
| `src/lib.rs` | `open` | fn | Sim-als-Produktion | 3 | 214 | Ändern (cfg(testing)) |
| `src/lib.rs` | `apply` | fn | Phantom-Crate | 3 | 509 | Ändern (Import/Abhängigkeit) |
| `src/lib.rs` | `flush_cluster` | fn | Sim-als-Produktion | 3 | 774 | Ändern (cfg(testing)) |
| `src/lib.rs` | `erase_cluster` | fn | Sim-als-Produktion | 1 | 886 | Ändern (cfg(testing)) |
| `src/lib.rs` | `erase_cluster` | fn | Verlorener-Fehler | 1 | 939 | Ändern (Fehler propagieren) |
| `src/lib.rs` | `erase_cluster` | fn | Phantom-Crate | 1 | 942 | Ändern (Import/Abhängigkeit) |
| `src/lib.rs` | `erase_cluster` | fn | Feste-Löschevidenz | 4 | 995 | Ändern (gemessene Evidenz) |
| `src/lib.rs` | `get` | fn | Sim-als-Produktion | 3 | 1155 | Ändern (cfg(testing)) |
| `src/lib.rs` | `op_target` | fn | Phantom-Crate | 4 | 1281 | Ändern (Import/Abhängigkeit) |
| `src/lib.rs` | `change_digest` | fn | Phantom-Crate | 3 | 1299 | Ändern (Import/Abhängigkeit) |
| `src/maint/blob_gc.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 5 | Ändern (Import/Abhängigkeit) |
| `src/maint/compaction.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 4 | Ändern (Import/Abhängigkeit) |
| `src/maint/compaction.rs` | `install_compacted_run` | fn | Verlorener-Fehler | 2 | 43 | Ändern (Fehler propagieren) |
| `src/maint/flush.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 4 | Ändern (Import/Abhängigkeit) |
| `src/maint/flush.rs` | `install_run` | fn | Verlorener-Fehler | 2 | 46 | Ändern (Fehler propagieren) |
| `src/maint/purge.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 4 | Ändern (Import/Abhängigkeit) |
| `src/maint/reserve.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 3 | Ändern (Import/Abhängigkeit) |
| `src/maint/reserve.rs` | `open` | fn | Phantom-Crate | 1 | 33 | Ändern (Import/Abhängigkeit) |
| `src/maint/scheduler.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 4 | Ändern (Import/Abhängigkeit) |
| `src/maint/scrub.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 5 | Ändern (Import/Abhängigkeit) |
| `src/metrics.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 3 | Ändern (Import/Abhängigkeit) |
| `src/recovery/integrity.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 3 | Ändern (Import/Abhängigkeit) |
| `src/recovery/open.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 13 | Ändern (Import/Abhängigkeit) |
| `src/recovery/open.rs` | `OpenedInstance` | struct | Phantom-Crate | 1 | 33 | Ändern (Import/Abhängigkeit) |
| `src/recovery/open.rs` | `open_instance` | fn | Phantom-Crate | 1 | 41 | Ändern (Import/Abhängigkeit) |
| `src/recovery/replay.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 9 | Ändern (Import/Abhängigkeit) |
| `src/recovery/scan.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 6 | Ändern (Import/Abhängigkeit) |
| `src/spaces.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 4 | Ändern (Import/Abhängigkeit) |
| `src/store/blob/gc_index.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 6 | Ändern (Import/Abhängigkeit) |
| `src/store/blob/reader.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 10 | Ändern (Import/Abhängigkeit) |
| `src/store/blob/record.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 3 | Ändern (Import/Abhängigkeit) |
| `src/store/blob/writer.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 7 | Ändern (Import/Abhängigkeit) |
| `src/store/blob/writer.rs` | `create` | fn | Verlorener-Fehler | 2 | 74 | Ändern (Fehler propagieren) |
| `src/store/redo/cut.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 6 | Ändern (Import/Abhängigkeit) |
| `src/store/redo/frame.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 3 | Ändern (Import/Abhängigkeit) |
| `src/store/redo/frame.rs` | `encode_frame` | fn | Phantom-Crate | 1 | 109 | Ändern (Import/Abhängigkeit) |
| `src/store/redo/frame.rs` | `decode_frame` | fn | Phantom-Crate | 1 | 127 | Ändern (Import/Abhängigkeit) |
| `src/store/redo/reader.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 6 | Ändern (Import/Abhängigkeit) |
| `src/store/redo/reader.rs` | `scan` | fn | Phantom-Crate | 1 | 72 | Ändern (Import/Abhängigkeit) |
| `src/store/redo/reserve.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 3 | Ändern (Import/Abhängigkeit) |
| `src/store/redo/segment.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 3 | Ändern (Import/Abhängigkeit) |
| `src/store/redo/writer.rs` | `—` | (Modulebene) | Phantom-Crate | 3 | 5 | Ändern (Import/Abhängigkeit) |
| `src/store/run/block.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 9 | Ändern (Import/Abhängigkeit) |
| `src/store/run/builder.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 11 | Ändern (Import/Abhängigkeit) |
| `src/store/run/copy.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 5 | Ändern (Import/Abhängigkeit) |
| `src/store/run/directory.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 6 | Ändern (Import/Abhängigkeit) |
| `src/store/run/extension.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 3 | Ändern (Import/Abhängigkeit) |
| `src/store/run/filter.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 3 | Ändern (Import/Abhängigkeit) |
| `src/store/run/format.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 3 | Ändern (Import/Abhängigkeit) |
| `src/store/run/pack_meta.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 5 | Ändern (Import/Abhängigkeit) |
| `src/store/run/rangedel.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 4 | Ändern (Import/Abhängigkeit) |
| `src/store/run/reader.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 8 | Ändern (Import/Abhängigkeit) |
| `src/store/run/reader.rs` | `get_entry` | fn | Phantom-Crate | 2 | 99 | Ändern (Import/Abhängigkeit) |
| `src/store/run/reader.rs` | `scan_entries` | fn | Phantom-Crate | 2 | 114 | Ändern (Import/Abhängigkeit) |
| `src/store/run/section.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 13 | Ändern (Import/Abhängigkeit) |
| `src/store/run/skip_scan.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 4 | Ändern (Import/Abhängigkeit) |
| `src/store/run/subindex.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 3 | Ändern (Import/Abhängigkeit) |
| `src/store/run/table_cache.rs` | `—` | (Modulebene) | Phantom-Crate | 2 | 3 | Ändern (Import/Abhängigkeit) |
| `src/sync/ranked.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 3 | Ändern (Import/Abhängigkeit) |

### contextra-sys (8 Einträge)

| Datei | Element | Art | Kategorie | Treffer | Erste Zeile | Aktion |
|---|---|---|---|---:|---:|---|
| `src/lib.rs` | `map` | mod | Verboten-Speicherabbild | 2 | 56 | Ändern (hinter Feature oder entfernen) |
| `src/mmap.rs` | `mmap_readonly` | fn | Verboten-Speicherabbild | 2 | 30 | Ändern (hinter Feature oder entfernen) |
| `src/mmap.rs` | `SealedMap` | struct | Verboten-Speicherabbild | 1 | 95 | Ändern (hinter Feature oder entfernen) |
| `src/mmap.rs` | `map_sealed` | fn | Verboten-Speicherabbild | 1 | 101 | Ändern (hinter Feature oder entfernen) |
| `src/mmap.rs` | `test_mmap_readonly` | fn | Verboten-Speicherabbild | 1 | 174 | Ändern (hinter Feature oder entfernen) |
| `src/mmap.rs` | `test_mmap_readonly_empty_file` | fn | Verboten-Speicherabbild | 1 | 191 | Ändern (hinter Feature oder entfernen) |
| `src/mmap.rs` | `test_mmap_readonly_large_file` | fn | Verboten-Speicherabbild | 1 | 210 | Ändern (hinter Feature oder entfernen) |
| `src/mmap.rs` | `test_mmap_readonly_access_boundary` | fn | Verboten-Speicherabbild | 1 | 229 | Ändern (hinter Feature oder entfernen) |

### contextra-text (89 Einträge)

| Datei | Element | Art | Kategorie | Treffer | Erste Zeile | Aktion |
|---|---|---|---|---:|---:|---|
| `src/contract.rs` | `Epoch` | struct | Altpfad-Typ | 1 | 39 | Entfernen/Ändern |
| `src/inverted/index_struct.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 8 | Entfernen (legacy) |
| `src/inverted/index_struct.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 9 | Entfernen/Ändern |
| `src/inverted/index_struct.rs` | `InvertedIndex` | struct | Altpfad-Port | 1 | 18 | Entfernen (legacy) |
| `src/inverted/index_struct.rs` | `InvertedIndex` | struct | Altpfad-Typ | 2 | 25 | Entfernen/Ändern |
| `src/inverted/index_struct.rs` | `InvertedIndex` | struct | Verboten-Laufzeit | 3 | 25 | Ändern (kein Async/Lock im L2) |
| `src/inverted/index_struct.rs` | `Clone` | impl | Altpfad-Port | 1 | 31 | Entfernen (legacy) |
| `src/inverted/index_struct.rs` | `InvertedIndex<S>` | impl | Altpfad-Port | 1 | 48 | Entfernen (legacy) |
| `src/inverted/index_struct.rs` | `new_with_language` | fn | Verboten-Laufzeit | 3 | 84 | Ändern (kein Async/Lock im L2) |
| `src/inverted/index_struct.rs` | `commit_lock` | fn | Verboten-Laufzeit | 1 | 115 | Ändern (kein Async/Lock im L2) |
| `src/inverted/index_struct.rs` | `load_stats` | fn | Verboten-Serial | 2 | 124 | Ändern (feature-gate oder Ersatz) |
| `src/inverted/index_struct.rs` | `upsert_document` | fn | Altpfad-Typ | 1 | 271 | Entfernen/Ändern |
| `src/inverted/index_struct.rs` | `upsert_document` | fn | Verboten-Serial | 4 | 303 | Ändern (feature-gate oder Ersatz) |
| `src/inverted/index_struct.rs` | `rewrite_posting_lists` | fn | Altpfad-Typ | 1 | 363 | Entfernen/Ändern |
| `src/inverted/index_struct.rs` | `compact_terms` | fn | Altpfad-Typ | 1 | 387 | Entfernen/Ändern |
| `src/inverted/index_struct.rs` | `resolve_tombstones` | fn | Altpfad-Typ | 1 | 391 | Entfernen/Ändern |
| `src/inverted/index_struct.rs` | `resolve_tombstones` | fn | Verboten-Serial | 1 | 432 | Ändern (feature-gate oder Ersatz) |
| `src/inverted/index_struct.rs` | `delete_document` | fn | Altpfad-Typ | 1 | 469 | Entfernen/Ändern |
| `src/inverted/index_struct.rs` | `delete_document` | fn | Verboten-Serial | 1 | 490 | Ändern (feature-gate oder Ersatz) |
| `src/inverted/index_struct.rs` | `stage_stats_change` | fn | Altpfad-Typ | 1 | 523 | Entfernen/Ändern |
| `src/inverted/index_struct.rs` | `commit_stats` | fn | Altpfad-Typ | 1 | 539 | Entfernen/Ändern |
| `src/inverted/index_struct.rs` | `commit_stats` | fn | Verboten-Serial | 2 | 588 | Ändern (feature-gate oder Ersatz) |
| `src/inverted/index_struct.rs` | `rollback_stats` | fn | Altpfad-Typ | 1 | 598 | Entfernen/Ändern |
| `src/inverted/index_struct.rs` | `TextIndex` | impl | Altpfad-Port | 1 | 672 | Entfernen (legacy) |
| `src/inverted/index_struct.rs` | `insert` | fn | Altpfad-Typ | 1 | 693 | Entfernen/Ändern |
| `src/inverted/index_struct.rs` | `delete` | fn | Altpfad-Typ | 1 | 697 | Entfernen/Ändern |
| `src/inverted/index_struct.rs` | `commit` | fn | Altpfad-Typ | 1 | 701 | Entfernen/Ändern |
| `src/inverted/index_struct.rs` | `rollback` | fn | Altpfad-Typ | 1 | 706 | Entfernen/Ändern |
| `src/inverted/index_struct.rs` | `rollback_to_tx` | fn | Altpfad-Typ | 1 | 711 | Entfernen/Ändern |
| `src/inverted/index_struct.rs` | `last_tx_id` | fn | Altpfad-Typ | 1 | 715 | Entfernen/Ändern |
| `src/inverted/index_struct.rs` | `stats` | fn | Verboten-Serial | 2 | 726 | Ändern (feature-gate oder Ersatz) |
| `src/inverted/morph_index.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 5 | Entfernen (legacy) |
| `src/inverted/morph_index.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 6 | Entfernen/Ändern |
| `src/inverted/morph_index.rs` | `BM25MorphIndex` | struct | Altpfad-Port | 1 | 11 | Entfernen (legacy) |
| `src/inverted/morph_index.rs` | `BM25MorphIndex<S>` | impl | Altpfad-Port | 1 | 16 | Entfernen (legacy) |
| `src/inverted/morph_index.rs` | `TextIndex` | impl | Altpfad-Port | 1 | 41 | Entfernen (legacy) |
| `src/inverted/morph_index.rs` | `insert` | fn | Altpfad-Typ | 1 | 57 | Entfernen/Ändern |
| `src/inverted/morph_index.rs` | `delete` | fn | Altpfad-Typ | 1 | 61 | Entfernen/Ändern |
| `src/inverted/morph_index.rs` | `commit` | fn | Altpfad-Typ | 1 | 65 | Entfernen/Ändern |
| `src/inverted/morph_index.rs` | `rollback` | fn | Altpfad-Typ | 1 | 69 | Entfernen/Ändern |
| `src/inverted/morph_index.rs` | `rollback_to_tx` | fn | Altpfad-Typ | 1 | 73 | Entfernen/Ändern |
| `src/inverted/morph_index.rs` | `last_tx_id` | fn | Altpfad-Typ | 1 | 77 | Entfernen/Ändern |
| `src/inverted/purge.rs` | `—` | (Modulebene) | Altpfad-Port | 2 | 22 | Entfernen (legacy) |
| `src/inverted/purge.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 24 | Entfernen/Ändern |
| `src/inverted/purge.rs` | `block_on_async` | fn | Verboten-Laufzeit | 1 | 34 | Ändern (kein Async/Lock im L2) |
| `src/inverted/purge.rs` | `TextPurger` | struct | Altpfad-Port | 1 | 102 | Entfernen (legacy) |
| `src/inverted/purge.rs` | `TextPurger<S>` | impl | Altpfad-Port | 1 | 106 | Entfernen (legacy) |
| `src/inverted/purge.rs` | `purge_document_report` | fn | Altpfad-Typ | 1 | 114 | Entfernen/Ändern |
| `src/inverted/purge.rs` | `purge_document` | fn | Altpfad-Typ | 1 | 120 | Entfernen/Ändern |
| `src/inverted/purge.rs` | `purge_document_internal_report` | fn | Altpfad-Typ | 1 | 126 | Entfernen/Ändern |
| `src/inverted/purge.rs` | `purge_document_internal_report` | fn | Verboten-Serial | 2 | 194 | Ändern (feature-gate oder Ersatz) |
| `src/inverted/purge.rs` | `purge_collection_report` | fn | Altpfad-Typ | 1 | 420 | Entfernen/Ändern |
| `src/inverted/purge.rs` | `purge_collection` | fn | Altpfad-Typ | 1 | 482 | Entfernen/Ändern |
| `src/inverted/purge.rs` | `Purger` | impl | Altpfad-Port | 1 | 488 | Entfernen (legacy) |
| `src/inverted/purge.rs` | `text_purger` | fn | Altpfad-Port | 1 | 505 | Entfernen (legacy) |
| `src/inverted/scanner.rs` | `—` | (Modulebene) | Altpfad-Port | 2 | 21 | Entfernen (legacy) |
| `src/inverted/scanner.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 23 | Entfernen/Ändern |
| `src/inverted/scanner.rs` | `block_on_async` | fn | Verboten-Laufzeit | 1 | 85 | Ändern (kein Async/Lock im L2) |
| `src/inverted/scanner.rs` | `TextScanner` | struct | Altpfad-Port | 1 | 94 | Entfernen (legacy) |
| `src/inverted/scanner.rs` | `TextScanner` | struct | Verboten-Laufzeit | 1 | 96 | Ändern (kein Async/Lock im L2) |
| `src/inverted/scanner.rs` | `TextScanner<S>` | impl | Altpfad-Port | 1 | 99 | Entfernen (legacy) |
| `src/inverted/scanner.rs` | `new` | fn | Verboten-Laufzeit | 1 | 105 | Ändern (kein Async/Lock im L2) |
| `src/inverted/scanner.rs` | `new_with_target` | fn | Verboten-Laufzeit | 1 | 114 | Ändern (kein Async/Lock im L2) |
| `src/inverted/scanner.rs` | `run_positive_control` | fn | Altpfad-Typ | 2 | 138 | Entfernen/Ändern |
| `src/inverted/scanner.rs` | `LayerScanner` | impl | Altpfad-Port | 1 | 193 | Entfernen (legacy) |
| `src/inverted/scanner.rs` | `scan_units` | fn | Verboten-Hash | 4 | 203 | Ändern (Ersatz CRC32C/Mac) |
| `src/inverted/scanner.rs` | `text_scanner` | fn | Altpfad-Port | 2 | 303 | Entfernen (legacy) |
| `src/memory_index.rs` | `—` | (Modulebene) | Verboten-Laufzeit | 1 | 20 | Ändern (kein Async/Lock im L2) |
| `src/posting_list.rs` | `—` | (Modulebene) | Verboten-Laufzeit | 1 | 23 | Ändern (kein Async/Lock im L2) |
| `src/posting_list.rs` | `decode_compact` | fn | Verboten-Serial | 2 | 297 | Ändern (feature-gate oder Ersatz) |
| `src/posting_list.rs` | `test_delta_varint_legacy_bincode_fallback` | fn | Verboten-Serial | 1 | 567 | Ändern (feature-gate oder Ersatz) |
| `src/stream.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 17 | Entfernen (legacy) |
| `src/stream.rs` | `Bm25CandidateStream` | struct | Altpfad-Port | 1 | 42 | Entfernen (legacy) |
| `src/stream.rs` | `Bm25CandidateStream<` | impl | Altpfad-Port | 1 | 54 | Entfernen (legacy) |
| `src/tokenizer.rs` | `fingerprint` | fn | Verboten-Hash | 1 | 681 | Ändern (Ersatz CRC32C/Mac) |
| `src/wand.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 18 | Entfernen (legacy) |
| `src/wand.rs` | `block_max_wand_search` | fn | Altpfad-Port | 1 | 262 | Entfernen (legacy) |
| `src/wand.rs` | `tests` | mod | Verboten-Laufzeit | 1 | 627 | Ändern (kein Async/Lock im L2) |
| `src/wand.rs` | `MockStorage` | struct | Altpfad-Typ | 1 | 633 | Entfernen/Ändern |
| `src/wand.rs` | `StorageEngine` | impl | Altpfad-Port | 1 | 647 | Entfernen (legacy) |
| `src/wand.rs` | `put` | fn | Altpfad-Typ | 1 | 653 | Entfernen/Ändern |
| `src/wand.rs` | `delete` | fn | Altpfad-Typ | 1 | 676 | Entfernen/Ändern |
| `src/wand.rs` | `commit` | fn | Altpfad-Typ | 1 | 700 | Entfernen/Ändern |
| `src/wand.rs` | `rollback` | fn | Altpfad-Typ | 1 | 706 | Entfernen/Ändern |
| `src/wand.rs` | `rollback_to_tx` | fn | Altpfad-Typ | 1 | 723 | Entfernen/Ändern |
| `src/wand.rs` | `last_tx_id` | fn | Altpfad-Typ | 2 | 756 | Entfernen/Ändern |
| `src/wand.rs` | `pin_checkpoint` | fn | Altpfad-Checkpoint | 1 | 771 | Entfernen |
| `src/wand.rs` | `scan_prefix_at` | fn | Verboten-Laufzeit | 1 | 819 | Ändern (kein Async/Lock im L2) |
| `src/wand.rs` | `test_block_max_wand_equivalence_and_pruning_performance` | fn | Altpfad-Typ | 1 | 823 | Entfernen/Ändern |

### contextra-types (34 Einträge)

| Datei | Element | Art | Kategorie | Treffer | Erste Zeile | Aktion |
|---|---|---|---|---:|---:|---|
| `src/error.rs` | `—` | (Modulebene) | Phantom-Crate | 1 | 36 | Ändern (Import/Abhängigkeit) |
| `src/error.rs` | `ContextraError` | enum | Verboten-Serial | 1 | 285 | Ändern (feature-gate oder Ersatz) |
| `src/error.rs` | `test_error_display_all_variants` | fn | Verboten-Serial | 1 | 754 | Ändern (feature-gate oder Ersatz) |
| `src/error.rs` | `test_from_conversions` | fn | Verboten-Serial | 2 | 780 | Ändern (feature-gate oder Ersatz) |
| `src/error_dto.rs` | `test_dto_exhaustive_match_coverage` | fn | Verboten-Serial | 1 | 484 | Ändern (feature-gate oder Ersatz) |
| `src/types/domain/ids.rs` | `hash_key_u64` | fn | Verboten-Hash | 1 | 374 | Ändern (Ersatz CRC32C/Mac) |
| `src/types/domain/ids.rs` | `hash_key_u128` | fn | Verboten-Hash | 1 | 383 | Ändern (Ersatz CRC32C/Mac) |
| `src/types/domain/ids.rs` | `TxId` | struct | Altpfad-Typ | 1 | 417 | Entfernen/Ändern |
| `src/types/domain/ids.rs` | `TxId` | impl | Altpfad-Typ | 1 | 419 | Entfernen/Ändern |
| `src/types/domain/ids.rs` | `try_from_internal_offset` | fn | Altpfad-Typ | 1 | 489 | Entfernen/Ändern |
| `src/types/domain/ids.rs` | `_` | const | Altpfad-Typ | 2 | 507 | Entfernen/Ändern |
| `src/types/domain/ids.rs` | `std` | impl | Altpfad-Typ | 1 | 510 | Entfernen/Ändern |
| `src/types/domain/ids.rs` | `fmt` | fn | Altpfad-Typ | 1 | 512 | Entfernen/Ändern |
| `src/types/domain/misc.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 13 | Entfernen/Ändern |
| `src/types/domain/misc.rs` | `WorkflowState` | struct | Altpfad-Checkpoint | 1 | 61 | Entfernen |
| `src/types/domain/misc.rs` | `WorkflowState` | struct | Altpfad-Typ | 1 | 63 | Entfernen/Ändern |
| `src/types/domain/misc.rs` | `MemoryLink` | struct | Altpfad-Typ | 1 | 115 | Entfernen/Ändern |
| `src/types/domain/misc.rs` | `Edge` | struct | Altpfad-Typ | 2 | 195 | Entfernen/Ändern |
| `src/types/domain/misc.rs` | `with_tx_validity` | fn | Altpfad-Typ | 1 | 273 | Entfernen/Ändern |
| `src/types/domain/misc.rs` | `with_validity` | fn | Altpfad-Typ | 1 | 287 | Entfernen/Ändern |
| `src/types/importance.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 31 | Entfernen/Ändern |
| `src/types/importance.rs` | `decay_factor` | fn | Altpfad-Typ | 1 | 102 | Entfernen/Ändern |
| `src/types/importance.rs` | `decay_factor_wallclock` | fn | Altpfad-Typ | 2 | 159 | Entfernen/Ändern |
| `src/types/importance.rs` | `MemoryImportance` | struct | Altpfad-Typ | 1 | 192 | Entfernen/Ändern |
| `src/types/importance.rs` | `new` | fn | Altpfad-Typ | 1 | 197 | Entfernen/Ändern |
| `src/types/importance.rs` | `effective_score` | fn | Altpfad-Typ | 1 | 209 | Entfernen/Ändern |
| `src/types/importance.rs` | `test_decay_factor_none` | fn | Altpfad-Typ | 2 | 238 | Entfernen/Ändern |
| `src/types/importance.rs` | `test_decay_factor_exponential` | fn | Altpfad-Typ | 5 | 246 | Entfernen/Ändern |
| `src/types/importance.rs` | `test_decay_factor_step_floor` | fn | Altpfad-Typ | 5 | 265 | Entfernen/Ändern |
| `src/types/importance.rs` | `test_decay_factor_out_of_order_tx` | fn | Altpfad-Typ | 2 | 279 | Entfernen/Ändern |
| `src/types/importance.rs` | `test_memory_importance_effective_score` | fn | Altpfad-Typ | 2 | 290 | Entfernen/Ändern |
| `src/types/importance.rs` | `test_decay_factor_wall_clock_exponential` | fn | Altpfad-Typ | 2 | 304 | Entfernen/Ändern |
| `src/witness.rs` | `RequestContext` | struct | Altpfad-Typ | 1 | 56 | Entfernen/Ändern |
| `src/witness.rs` | `new` | fn | Altpfad-Typ | 1 | 66 | Entfernen/Ändern |

### contextra-vector (87 Einträge)

| Datei | Element | Art | Kategorie | Treffer | Erste Zeile | Aktion |
|---|---|---|---|---:|---:|---|
| `src/compute_pool.rs` | `PoolState` | struct | Verboten-Laufzeit | 2 | 28 | Ändern (kein Async/Lock im L2) |
| `src/compute_pool.rs` | `new` | fn | Verboten-Laufzeit | 3 | 46 | Ändern (kein Async/Lock im L2) |
| `src/diskann/build.rs` | `—` | (Modulebene) | Verboten-Schlüssel | 1 | 8 | Entfernen |
| `src/diskann/build.rs` | `append_to_tombstone_wal_blocking` | fn | Verboten-FS | 2 | 109 | Ändern (über sys/durable-fs) |
| `src/diskann/build.rs` | `append_to_tombstone_wal_blocking` | fn | Verboten-Schlüssel | 1 | 128 | Entfernen |
| `src/diskann/build.rs` | `read_tombstone_wal` | fn | Verboten-FS | 1 | 159 | Ändern (über sys/durable-fs) |
| `src/diskann/build.rs` | `read_tombstone_wal` | fn | Verboten-Schlüssel | 1 | 223 | Entfernen |
| `src/diskann/build.rs` | `append_to_pending_wal_blocking` | fn | Verboten-FS | 2 | 250 | Ändern (über sys/durable-fs) |
| `src/diskann/build.rs` | `append_to_pending_wal_blocking` | fn | Verboten-Schlüssel | 1 | 280 | Entfernen |
| `src/diskann/build.rs` | `read_pending_wal` | fn | Verboten-FS | 1 | 321 | Ändern (über sys/durable-fs) |
| `src/diskann/build.rs` | `read_pending_wal` | fn | Verboten-Schlüssel | 1 | 407 | Entfernen |
| `src/diskann/build.rs` | `recover_pending_delta_sync` | fn | Verboten-FS | 1 | 455 | Ändern (über sys/durable-fs) |
| `src/diskann/build.rs` | `build_sync` | fn | Verboten-FS | 4 | 722 | Ändern (über sys/durable-fs) |
| `src/diskann/build.rs` | `tests` | mod | Verboten-Laufzeit | 1 | 778 | Ändern (kein Async/Lock im L2) |
| `src/diskann/format.rs` | `DISKANN_INTEGRITY_KEY` | const | Verboten-Schlüssel | 1 | 11 | Entfernen |
| `src/diskann/integrity.rs` | `DiskAnnHmac` | struct | Verboten-Hash | 1 | 7 | Ändern (Ersatz CRC32C/Mac) |
| `src/diskann/integrity.rs` | `new` | fn | Verboten-Hash | 2 | 29 | Ändern (Ersatz CRC32C/Mac) |
| `src/diskann/persistence.rs` | `—` | (Modulebene) | Verboten-Schlüssel | 1 | 10 | Entfernen |
| `src/diskann/persistence.rs` | `persist_delta_sync` | fn | Verboten-FS | 5 | 50 | Ändern (über sys/durable-fs) |
| `src/diskann/persistence.rs` | `write_to_path_sync` | fn | Verboten-FS | 1 | 136 | Ändern (über sys/durable-fs) |
| `src/diskann/persistence.rs` | `write_to_path_sync` | fn | Verboten-Schlüssel | 1 | 176 | Entfernen |
| `src/diskann/persistence.rs` | `load_sync` | fn | Verboten-FS | 3 | 272 | Ändern (über sys/durable-fs) |
| `src/diskann/persistence.rs` | `load_sync` | fn | Verboten-Speicherabbild | 1 | 291 | Ändern (hinter Feature oder entfernen) |
| `src/diskann/persistence.rs` | `load_sync` | fn | Verboten-Schlüssel | 1 | 318 | Entfernen |
| `src/diskann/persistence.rs` | `tests` | mod | Verboten-Laufzeit | 1 | 639 | Ändern (kein Async/Lock im L2) |
| `src/diskann/predicate_augmented.rs` | `tests` | mod | Verboten-Laufzeit | 1 | 196 | Ändern (kein Async/Lock im L2) |
| `src/diskann/predicate_augmented.rs` | `test_gamma_zero_regression_reduction` | fn | Verboten-Laufzeit | 1 | 236 | Ändern (kein Async/Lock im L2) |
| `src/diskann/types.rs` | `—` | (Modulebene) | Verboten-Speicherabbild | 1 | 12 | Ändern (hinter Feature oder entfernen) |
| `src/diskann/types.rs` | `—` | (Modulebene) | Verboten-Laufzeit | 1 | 13 | Ändern (kein Async/Lock im L2) |
| `src/diskann/types.rs` | `test_try_new_corrupt_magic_fails_immediately` | fn | Verboten-FS | 1 | 83 | Ändern (über sys/durable-fs) |
| `src/diskann/types.rs` | `test_try_new_truncated_file_fails_immediately` | fn | Verboten-FS | 1 | 114 | Ändern (über sys/durable-fs) |
| `src/diskann/types.rs` | `try_new` | fn | Verboten-FS | 1 | 206 | Ändern (über sys/durable-fs) |
| `src/diskann/vector_index_impl.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 11 | Entfernen/Ändern |
| `src/diskann/vector_index_impl.rs` | `insert` | fn | Altpfad-Typ | 1 | 15 | Entfernen/Ändern |
| `src/diskann/vector_index_impl.rs` | `delete` | fn | Altpfad-Typ | 1 | 63 | Entfernen/Ändern |
| `src/diskann/vector_index_impl.rs` | `commit` | fn | Altpfad-Typ | 1 | 101 | Entfernen/Ändern |
| `src/diskann/vector_index_impl.rs` | `rollback` | fn | Altpfad-Typ | 1 | 109 | Entfernen/Ändern |
| `src/diskann/vector_index_impl.rs` | `rollback_to_tx` | fn | Altpfad-Typ | 1 | 117 | Entfernen/Ändern |
| `src/diskann/vector_index_impl.rs` | `last_tx_id` | fn | Altpfad-Typ | 2 | 147 | Entfernen/Ändern |
| `src/hnsw/arena.rs` | `—` | (Modulebene) | Verboten-Laufzeit | 1 | 8 | Ändern (kein Async/Lock im L2) |
| `src/hnsw/purge.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 9 | Entfernen (legacy) |
| `src/hnsw/purge.rs` | `hnsw_purger` | fn | Altpfad-Port | 1 | 38 | Entfernen (legacy) |
| `src/hnsw/purge.rs` | `Purger` | impl | Altpfad-Port | 1 | 43 | Entfernen (legacy) |
| `src/hnsw/purge.rs` | `purge` | fn | Verboten-Hash | 1 | 79 | Ändern (Ersatz CRC32C/Mac) |
| `src/hnsw/scanner.rs` | `—` | (Modulebene) | Altpfad-Port | 1 | 8 | Entfernen (legacy) |
| `src/hnsw/scanner.rs` | `run_positive_control` | fn | Verboten-Hash | 1 | 68 | Ändern (Ersatz CRC32C/Mac) |
| `src/hnsw/scanner.rs` | `scan_index_content` | fn | Verboten-Hash | 4 | 107 | Ändern (Ersatz CRC32C/Mac) |
| `src/hnsw/scanner.rs` | `hnsw_scanner` | fn | Altpfad-Port | 1 | 205 | Entfernen (legacy) |
| `src/hnsw/scanner.rs` | `LayerScanner` | impl | Altpfad-Port | 1 | 224 | Entfernen (legacy) |
| `src/hnsw/staging.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 5 | Entfernen/Ändern |
| `src/hnsw/staging.rs` | `—` | (Modulebene) | Verboten-Laufzeit | 1 | 6 | Ändern (kein Async/Lock im L2) |
| `src/hnsw/staging.rs` | `VectorTxBuffer` | struct | Altpfad-Typ | 1 | 39 | Entfernen/Ändern |
| `src/hnsw/staging.rs` | `stage` | fn | Altpfad-Typ | 1 | 53 | Entfernen/Ändern |
| `src/hnsw/staging.rs` | `drain` | fn | Altpfad-Typ | 1 | 84 | Entfernen/Ändern |
| `src/hnsw/staging.rs` | `discard` | fn | Altpfad-Typ | 1 | 103 | Entfernen/Ändern |
| `src/hnsw/staging.rs` | `SequenceLog` | struct | Altpfad-Typ | 1 | 117 | Entfernen/Ändern |
| `src/hnsw/staging.rs` | `SequenceLog` | impl | Altpfad-Typ | 1 | 122 | Entfernen/Ändern |
| `src/hnsw/types.rs` | `—` | (Modulebene) | Verboten-Laufzeit | 1 | 6 | Ändern (kein Async/Lock im L2) |
| `src/hnsw/types.rs` | `RebuildGuard` | struct | Verboten-Laufzeit | 1 | 65 | Ändern (kein Async/Lock im L2) |
| `src/hnsw/types.rs` | `HnswHotCore` | struct | Verboten-Laufzeit | 1 | 115 | Ändern (kein Async/Lock im L2) |
| `src/hnsw/types.rs` | `HnswColdCore` | struct | Altpfad-Typ | 1 | 203 | Entfernen/Ändern |
| `src/hnsw/types.rs` | `try_new_with_rng_and_clock` | fn | Verboten-Laufzeit | 1 | 265 | Ändern (kein Async/Lock im L2) |
| `src/hnsw/types.rs` | `try_new_with_rng_and_clock` | fn | Altpfad-Typ | 1 | 279 | Entfernen/Ändern |
| `src/hnsw/types.rs` | `new` | fn | Verboten-Laufzeit | 1 | 367 | Ändern (kein Async/Lock im L2) |
| `src/hnsw/types.rs` | `new` | fn | Altpfad-Typ | 1 | 381 | Entfernen/Ändern |
| `src/hnsw/types.rs` | `save_internal` | fn | Verboten-FS | 3 | 772 | Ändern (über sys/durable-fs) |
| `src/hnsw/vector_index_impl.rs` | `—` | (Modulebene) | Altpfad-Typ | 1 | 5 | Entfernen/Ändern |
| `src/hnsw/vector_index_impl.rs` | `insert` | fn | Altpfad-Typ | 1 | 13 | Entfernen/Ändern |
| `src/hnsw/vector_index_impl.rs` | `delete` | fn | Altpfad-Typ | 1 | 227 | Entfernen/Ändern |
| `src/hnsw/vector_index_impl.rs` | `commit` | fn | Altpfad-Typ | 1 | 240 | Entfernen/Ändern |
| `src/hnsw/vector_index_impl.rs` | `rollback` | fn | Altpfad-Typ | 1 | 380 | Entfernen/Ändern |
| `src/hnsw/vector_index_impl.rs` | `rollback_to_tx` | fn | Altpfad-Typ | 1 | 391 | Entfernen/Ändern |
| `src/hnsw/vector_index_impl.rs` | `last_tx_id` | fn | Altpfad-Typ | 2 | 459 | Entfernen/Ändern |
| `src/lib.rs` | `quantize_rabitq` | mod | Altpfad-Typ | 1 | 87 | Entfernen/Ändern |
| `src/persistence/atomic.rs` | `—` | (Modulebene) | Verboten-FS | 1 | 5 | Ändern (über sys/durable-fs) |
| `src/persistence/atomic.rs` | `write_atomic` | fn | Verboten-FS | 2 | 33 | Ändern (über sys/durable-fs) |
| `src/persistence/atomic.rs` | `write_atomic` | fn | Verlorener-Fehler | 1 | 40 | Ändern (Fehler propagieren) |
| `src/persistence/mmap.rs` | `MmapIndex` | struct | Verboten-Speicherabbild | 1 | 13 | Ändern (hinter Feature oder entfernen) |
| `src/persistence/mmap.rs` | `open` | fn | Verboten-FS | 1 | 21 | Ändern (über sys/durable-fs) |
| `src/persistence/mmap.rs` | `open` | fn | Verboten-Speicherabbild | 1 | 24 | Ändern (hinter Feature oder entfernen) |
| `src/types.rs` | `from_key` | fn | Verboten-Hash | 1 | 55 | Ändern (Ersatz CRC32C/Mac) |
| `src/types.rs` | `TxId` | struct | Altpfad-Typ | 1 | 97 | Entfernen/Ändern |
| `src/types.rs` | `TxId` | impl | Altpfad-Typ | 1 | 99 | Entfernen/Ändern |
| `src/types.rs` | `fmt` | impl | Altpfad-Typ | 1 | 114 | Entfernen/Ändern |
| `src/types.rs` | `fmt` | fn | Altpfad-Typ | 1 | 116 | Entfernen/Ändern |
| `src/types.rs` | `From` | impl | Altpfad-Typ | 3 | 120 | Entfernen/Ändern |
| `src/types.rs` | `from` | fn | Altpfad-Typ | 3 | 302 | Entfernen/Ändern |

## 3. Zusammenfassung

| Kategorie | Anzahl Einträge (Abschnitt 2) |
|---|---:|
| Altpfad-Typ | 378 |
| Altpfad-Port | 177 |
| Verboten-Laufzeit | 124 |
| Phantom-Crate | 103 |
| Altpfad-Checkpoint | 56 |
| Verboten-FS | 31 |
| Verboten-Serial | 29 |
| Verboten-Hash | 25 |
| Altpfad-Graph | 21 |
| Altpfad-Store | 17 |
| Altpfad-KV | 16 |
| Verboten-Crypto | 14 |
| Verboten-Speicherabbild | 12 |
| Altpfad-Inferenz | 10 |
| Sim-als-Produktion | 10 |
| Verboten-Schlüssel | 9 |
| Verlorener-Fehler | 8 |
| Feste-Löschevidenz | 2 |
| **Summe** | **1042** |

## 4. Grenzen dieser Liste

- **Erfasst** werden die in SPEC §13.3 und §2.7 benannten Altpfade und Verbote sowie die Phantom-Crate-Importe. Nicht erfasst sind allgemeine Stilfehler wie `unwrap()` oder `expect()` in Produktionscode. Deren Menge ist groß und gehört in einen separaten Lint-Lauf (`cargo clippy`, Workspace-Lints sind `deny`).
- **Konstanten-Literale** (`verified: true` usw.) sind nur auf die Löschevidenz gefiltert. Gewöhnliche `[0u8; 32]`-Arrays sind bewusst ausgenommen, weil sie in Krypto-Code legitim sind.
- **`std::fs` in `contextra-sys`** ist nicht gelistet: `sys` ist laut SPEC genau der Ort für OS-Zugriffe.
- **Abhängigkeiten in `Cargo.toml`** (z. B. `bincode`, `blake3`) sind nicht Teil dieser Tabelle, sondern stehen in den PLAN.md-Dateien.
- **Entfernen ≠ sofort löschbar.** Viele Altpfade werden noch von `engine` verwendet. Löschung ist erst nach dem Umbau des jeweiligen Konsumenten möglich (Reihenfolge in den PLAN.md).
- Nicht kompiliert. Ob die Liste vollständig ist, zeigt erst `cargo check` nach dem Entfernen (Fehlerliste = fehlende Verwender).

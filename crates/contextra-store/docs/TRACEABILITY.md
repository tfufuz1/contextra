# Traceability Matrix: Contextra Store (SPEC §9 / AGENTS.md §4)

Diese Matrix dokumentiert die Abdeckung aller in `crates/contextra-store/AGENTS.md` §4 genannten Systemverträge und Invarianten durch dedizierte Tests. Jeder Test prüft die Invariante aktiv und ist in der Lage, eine absichtliche Verletzung (Sabotage/Mutation) zu erkennen.

| Invariante-ID | Beschreibung | Testdatei | Testfunktion |
|---|---|---|---|
| `INV-FSYNC-1` | fsync-Fehler werden über `?` propagiert; nach Sync-Fehler wird Handle vergiftet. `fsync.rs` ist zentrale Sync-Stelle. | `tests/spec_invariants_agents_sec4.rs` | `test_inv_fsync_1_error_propagation_and_poisoning` |
| `INV-ATOMIC-WRITE` | Atomares Schreiben von Dateien: tmp-Datei → fsync → rename → Parent-fsync. | `tests/spec_invariants_agents_sec4.rs` | `test_inv_atomic_write_lifecycle` |
| `INV-TOMBSTONE-MASK` | Tombstone-Disziplin: Bit 63 maskieren (`seq & !TOMBSTONE_BIT`) vor jedem `max_seq`-Vergleich. | `tests/spec_invariants_agents_sec4.rs` | `test_inv_tombstone_bit_masking_before_comparison` |
| `INV-SINGLE-LOAD` | Snapshot-Isolation: `last_committed_tx` in `get_at_seq`/`scan_prefix_at` wird genau einmal atomar geladen. | `tests/spec_invariants_agents_sec4.rs` | `test_inv_single_load_snapshot_isolation` |
| `INV-FLUSH-BEFORE-VISIBLE` | Flush-before-Visible: `last_committed_tx` wird vor dem Sichtbarmachen der SSTable aktualisiert. | `tests/spec_invariants_agents_sec4.rs` | `test_inv_flush_before_visible_ordering` |
| `INV-WAL-HMAC-KEY` | WAL-HMAC-Schlüssel nur über `load_or_create_integrity_key`, nie hartcodiert. | `tests/spec_invariants_agents_sec4.rs` | `test_inv_wal_hmac_key_derivation_not_hardcoded` |
| `INV-TENANT-1` | Tenant-Präfix in jedem Schlüssel; strikte Mandantentrennung. | `tests/spec_invariants_agents_sec4.rs` | `test_inv_tenant_1_prefix_enforcement_and_isolation` |
| `INV-KEYSPACE-1` | Schlüssel- und Dateinamen dürfen nur opake IDs sein (kein Leaken von Klartext/Pfadnamen). | `tests/spec_invariants_agents_sec4.rs` | `test_inv_keyspace_1_opaque_keys_and_filenames` |
| `INV-FORMAT-1` | Jede Datei enthält Magic Header + Formatversion; Prüfsummen erkennen Bit-Rot deterministisch. | `tests/spec_invariants_agents_sec4.rs` | `test_inv_format_1_magic_and_version_headers` |

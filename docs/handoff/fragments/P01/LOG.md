# Commit- und Arbeitslog: Fragment P01

## Initialer Check
- Toolchain: `rustc 1.89.0`, `cargo 1.89.0`.
- DURABILITY_MATRIX.md mit 6 Vertragszeilen und Test-Mapping angelegt.

## WP-30: Crash-Konsistenz- und Fault-Injection-Härtung
- `tests/wal_every_byte_truncation.rs` implementiert: Schneidet WAL an jeder Byte-Grenze von 0 bis EOF ab und prüft saubere Wiederherstellung oder fail-closed Fehler ohne Panics.
- `tests/differential/redb_operation_sequence.rs`: Reopen-Semantik bei `Op::Restart` gegen `DirLock`-Freigabe gehärtet.
- `tests/fault_injection_durability_points.rs`: Deckt alle 5 definierten Fehlerpunkte ab (WAL-Append, fsync, Memtable-Flush, Manifest-Update, Compaction-Commit).
- Prüfbefehl: `cargo test -p contextra-store --test wal_every_byte_truncation --test fault_injection_durability_points` → grün.

## WP-31: Format-Versionen und Fixtures
- Versionsfelder von WAL (`WalVersion::V3`), SSTable (V4 / `SSTABLE_MAGIC_MFSX`) und Manifest (`CURRENT_MANIFEST_VERSION = 1` / `MFMN`) in `README.md` dokumentiert.
- Programmatische Fixtures unter `crates/contextra-store/tests/fixtures/` angelegt (`current_v3.wal`, `current_v4.sst`, `current_v1.manifest`).
- Test `tests/format_version_compat.rs` implementiert: Verifiziert Lesen von n-1 und saubere, panic-freie Ablehnung unbekannter Versionen.
- Prüfbefehl: `cargo test -p contextra-store --features test-hooks --test format_version_compat` → grün.

## WP-32: Shredding- und Proof-Altlasten bereinigen
- Crate-interne Shredding- und Proof-Vorkommnisse bereinigt (Scanner-Header auf neutrale Bezeichnungen umgestellt).
- Öffentliche API `DurabilityMode::validate_against_features(deletion_proof_active, ...)` unverändert gelassen und in `PENDING_API_CHANGES.md` dokumentiert.
- Audit-/Proof-spezifischer Test `campaign_j42_audit.rs` via `git mv` nach `archive/tests-legacy/contextra-store/` archiviert; Tombstone-/Purge-Tests (`scanner/purge.rs`, `tombstone_scan.rs`) verbleiben im Testbestand.
- Prüfbefehl: `cargo check -p contextra-store` → grün.

## WP-33: Scanner/Systemdruck prüfen und Dateien > 800 Zeilen aufteilen
- Nutzerprüfung von `system_pressure.rs`:
  - Genutzt in `contextra-store` (`CompactionEngine`, `LsmStorage`, `recovery.rs`) sowie extern in `contextra-engine` (`Collection::with_pressure_receiver`, `set_pressure_receiver`).
  - Empfehlung: Verbleibt unverändert in `contextra-store`, da `engine` diesen Druckindikator für Backpressure nutzt.
- Nutzerprüfung von `tenant_codec.rs`:
  - `TenantScopedStorage` implementiert den `StorageEngine`-Port für mandantenspezifische Schlüsselpräfixe (`t:{tenant}:{collection}:...`).
  - Empfehlung: Beibehalten als Standard-Adapter für Multi-Tenancy-Partitionierung.
- Mechanische Zerlegung von Großdateien (> 800 Zeilen) in Untermodule:
  1. `crates/contextra-store/src/tenant_codec.rs` (883 Zeilen) -> zerlegt in `tenant_codec.rs` (558 Zeilen), `tenant_codec/storage_engine.rs` (290 Zeilen), `tenant_codec/tests.rs` (44 Zeilen).
  2. `crates/contextra-store/src/memtable.rs` (1085 Zeilen) -> zerlegt in `memtable.rs` (612 Zeilen) und `memtable/tests.rs` (475 Zeilen).
  3. `crates/contextra-store/src/wal/flusher.rs` (829 Zeilen) -> zerlegt in `flusher.rs` (770 Zeilen) und `flusher/command.rs` (67 Zeilen).
- Prüfbefehl: `cargo test -p contextra-store --no-run` → grün.

## WP-34: README nach P6, API.md und Dokumentations-Härtung
- `README.md` nach P6 überarbeitet (Zweck, Reifegrad, Abhängigkeiten, Features, Minimalbeispiel, Entnahme-Rezept, Bekannte Grenzen).
- Commit-Semantik je `DurabilityMode` (`Full`, `WalNoHmac`, `MemoryOnly`) präzise formuliert und auf Tests gestützt.
- `crates/contextra-store/API.md` mit Spezifikation der öffentlichen Schnittstelle angelegt.
- `#![warn(missing_docs)]` in `crates/contextra-store/src/lib.rs` gesetzt.
- Prüfbefehl: `cargo test -p contextra-store --doc` → grün.

## Prüfungen P1/P2 & Feature-Matrix
- `cargo build -p contextra-store --no-default-features`: grün (erfolgreich kompiliert).
- `cargo build -p contextra-store --all-features`: grün (erfolgreich kompiliert).
- `cargo clippy -p contextra-store --lib -- -D warnings`: grün (keine Warnungen im Nicht-Test-Code).
- `cargo test -p contextra-store --test wal_every_byte_truncation --test format_version_compat --test fault_injection_durability_points`: grün.

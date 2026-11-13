# AGENTS.md — `contextra-audit-export`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-audit-export` · **Ring:** 0 · **Version:** 0.1.0
- **Beschreibung:** GDPR Article 30 Processing Register export generator for Contextra
- **src:** 8 Dateien, 747 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 1):** `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 3):** `serde`, `serde_json`, `thiserror`
- **Reverse-Abhängigkeiten (0):** –
- **Kopplung an Hub-Crates:** ausgehend `contextra-types`; eingehend –
- **Features (5):** `default`, `std`, `test-utils`, `types-compat`, `workspace`

### Öffentliche API (grob, aus src/)
- struct (9): `BsiMappingEntry`, `ControllerContact`, `DeletionProofSummary`, `EgressEventSummary`, `InMemoryProcessingRegisterSource`, `ProcessingRegisterEntry`, `RetentionPeriod`, `TenantId`, `ThirdCountryTransfer`
- enum (1): `AuditExportError`
- trait (2): `ProcessingRegisterSource`, `RecordSource`
- fn (17): `add_entry`, `as_u64`, `bsi_mapping_table`, `from_raw`, `inner`, `is_system`, `new`, `render_bsi_mapping_markdown`, `render_register_csv`, `render_register_json`, `render_register_markdown`, `sample_controller_contact`, `sanitize_markdown_cell`, `sort_internal_collections`, `try_new` … (+2)
- const (1): `SYSTEM`
- mod (7): `bsi_mapping`, `compat`, `csv_template`, `error`, `markdown_template`, `testkit`, `types`
- pub use: `bsi_mapping::{bsi_mapping_table, render_bsi_mapping_markdown, BsiMappingEntry}`, `csv_template::render_register_csv`, `error::AuditExportError`, `markdown_template::render_register_markdown`

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->

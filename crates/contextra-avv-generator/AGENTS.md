# AGENTS.md — `contextra-avv-generator`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-avv-generator` · **Ring:** 0 · **Version:** 0.1.0
- **Beschreibung:** AVV (Auftragsverarbeitungsvertrag) template generator referencing technical guarantees for Contextra
- **src:** 9 Dateien, 550 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 0):** –
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 2):** `serde`, `thiserror`
- **Reverse-Abhängigkeiten (0):** –
- **Kopplung an Hub-Crates:** ausgehend –; eingehend –
- **Features (3):** `default`, `serde`, `std`

### Öffentliche API (grob, aus src/)
- struct (3): `AvvContext`, `TechnicalMeasure`, `TenantId`
- enum (1): `AvvError`
- fn (14): `as_u64`, `default_technical_measures`, `escape_markdown`, `inner`, `new`, `render`, `render_avv_markdown`, `render_avv_markdown_with_date`, `render_section`, `render_with_date`, `try_new`, `validate`, `with_contract_date`, `with_subprocessors_allowed`
- type (1): `AvvGeneratorError`
- const (1): `SYSTEM`
- mod (8): `assistance`, `audit`, `confidentiality`, `data_subject_rights`, `error`, `instructions`, `template`, `termination`
- pub use: `error::{AvvError, AvvGeneratorError}`

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->`.

<!-- END GENERATED: crate_inventory.py -->

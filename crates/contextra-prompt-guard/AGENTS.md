# AGENTS.md — `contextra-prompt-guard`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-prompt-guard` · **Ring:** – · **Version:** 0.1.0
- **Beschreibung:** Signatur- und phrasenbasierte Prompt-Injection-Erkennung und Quarantäne-Policy
- **src:** 10 Dateien, 1085 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 0):** –
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 5):** `serde`, `serde_json`, `thiserror`, `tracing`, `unicode-normalization`
- **Reverse-Abhängigkeiten (1):** `contextra-mcp`
- **Kopplung an Hub-Crates:** ausgehend –; eingehend –
- **Features (3):** `default`, `file-audit`, `std`

### Öffentliche API (grob, aus src/)
- struct (6): `NormalizedPattern`, `PromptInjectionConfig`, `PromptInjectionGuard`, `SecurityAuditLogger`, `SecurityAuditRecord`, `SystemClock`
- enum (2): `GuardError`, `QuarantinePolicy`
- trait (1): `Clock`
- fn (23): `audit_logger`, `clock`, `collapse_whitespace`, `decode_base64`, `default_patterns`, `default_redaction_placeholder`, `detect`, `detect_recursive`, `error_message`, `extract_base64_candidates`, `from_env`, `get_recorded_events`, `is_zero_width`, `load_from_file`, `log_event` … (+8)
- const (2): `DEFAULT_REDACTION_PLACEHOLDER`, `MAX_RECURSION_DEPTH`
- mod (7): `audit`, `clock`, `compat`, `contract`, `error`, `guard`, `policy`
- pub use: `audit::{SecurityAuditLogger, SecurityAuditRecord}`, `clock::SystemClock`, `contract::Clock`, `crate::contract::Clock`, `error::GuardError`, `guard::{NormalizedPattern, PromptInjectionGuard}`, `policy::{PromptInjectionConfig, QuarantinePolicy, DEFAULT_REDACTION_PLACEHOLDER}`

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->`.

<!-- END GENERATED: crate_inventory.py -->

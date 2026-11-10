# Handoff MCP-2: contextra-prompt-guard (neu) aus contextra-mcp

Commit-Kennung: MC2
Branch: `jules/mcp-2-prompt-guard`
Stand: 2026-11-10

## 1. Status der Arbeitspakete

| Paket | Status | Prüfbefehl | Anmerkung |
|---|---|---|---|
| MC2.0 | Grün | `cargo check --workspace ...; tools/loc.py ...` | Baseline erfasst unter `docs/decoupling/baseline/mcp/` |
| MC2.1 | Grün | `cd crates/contextra-mcp/tests/golden && sha256sum -c MANIFEST.sha256` | Golden-Korpus für Injection-Erkennung & Audit-Logging |
| MC2.2 | Grün | `cargo check -p contextra-prompt-guard --all-targets` | Additive Anlage `crates/contextra-prompt-guard` |
| MC2.3 | Grün | `cargo test -p contextra-prompt-guard --test golden_characterization` | Umstellung & Re-Exports in `contextra-mcp` |
| MC2.4 | Grün | `grep -rn 'ports_local' crates/contextra-mcp/src` | Umstellung `ports_local`-Importe auf `contextra_ports` in `routing.rs` |
| MC2.5 | Grün | `cargo tree -p contextra-prompt-guard -e normal --depth 1` | Entkopplungs- und Autarkienachweis für `contextra-prompt-guard` (0 contextra-*) |
| MC2.6 | Grün | `cargo test -p contextra-prompt-guard --test golden_mutation_tests` | Härtung, Zero-Panic, Golden-Mutation-Tests |
| MC2.7 | Grün | `python3 tools/pattern_diagnostic.py --crate contextra-prompt-guard --category FORBIDDEN_WORDING` | Dokumentation (`AGENTS.md`, `README.md`, Fehlerabbildung) |

## 2. API-Freeze-Tabelle

| Typ / Funktion | Vorheriger Pfad | Neuer Pfad | Re-Export in `contextra-mcp` | Status |
|---|---|---|---|---|
| `PromptInjectionGuard` | `contextra_mcp::prompt_injection::PromptInjectionGuard` | `contextra_prompt_guard::PromptInjectionGuard` | Ja (`pub use ...`) | Identisch / Quellkompatibel |
| `QuarantinePolicy` | `contextra_mcp::prompt_injection::QuarantinePolicy` | `contextra_prompt_guard::QuarantinePolicy` | Ja (`pub use ...`) | Identisch / Quellkompatibel |
| `PromptInjectionConfig` | `contextra_mcp::prompt_injection::PromptInjectionConfig` | `contextra_prompt_guard::PromptInjectionConfig` | Ja (`pub use ...`) | Identisch / Quellkompatibel |
| `SecurityAuditLogger` | `contextra_mcp::prompt_injection::SecurityAuditLogger` | `contextra_prompt_guard::SecurityAuditLogger` | Ja (`pub use ...`) | Identisch / Quellkompatibel |
| `SecurityAuditRecord` | `contextra_mcp::prompt_injection::SecurityAuditRecord` | `contextra_prompt_guard::SecurityAuditRecord` | Ja (`pub use ...`) | Identisch / Quellkompatibel |
| `NormalizedPattern` | `contextra_mcp::prompt_injection::NormalizedPattern` | `contextra_prompt_guard::NormalizedPattern` | Ja (`pub use ...`) | Identisch / Quellkompatibel |
| `DEFAULT_REDACTION_PLACEHOLDER` | `contextra_mcp::prompt_injection::DEFAULT_REDACTION_PLACEHOLDER` | `contextra_prompt_guard::DEFAULT_REDACTION_PLACEHOLDER` | Ja (`pub use ...`) | Identisch / Quellkompatibel |
| `GuardError` | *(neu)* | `contextra_prompt_guard::GuardError` | Ja (`pub use ...`) | Neu / Ergänzung |

## 3. Messwerte vorher / nachher

| Metrik | Baseline (MC2.0) | Nach Abschluss (MC2.7) | Differenz |
|---|---|---|---|
| LOC `contextra-mcp` | 4.978 LOC (25 Dateien) | 4.041 LOC (21 Dateien) | -937 LOC (-4 Dateien) |
| LOC `contextra-prompt-guard` | 0 LOC (Crate existiert nicht) | 1.026 LOC (7 Dateien) | +1.026 LOC (+7 Dateien) |
| Interne Abhängigkeiten `mcp` | 11 | 11 (`contextra-prompt-guard` als Pfad; ports_local entkoppelt) | neutral |
| Interne Abhängigkeiten `prompt-guard` | n/a | 0 (`contextra-*`-frei, E0) | 0 (rein extern) |
| Prod Zero-Panic Verstöße `prompt-guard` | n/a | 0 | 0 |
| Forbidden Wording Befunde | n/a | 0 | 0 |

## 4. Koordinationsanforderungen (Folgeschritt Integration)

1. **Wurzel `Cargo.toml`:** `members`-Liste um `"crates/contextra-prompt-guard"` ergänzen; optional in `[workspace.dependencies]` als `contextra-prompt-guard = { path = "crates/contextra-prompt-guard" }` eintragen.
2. **`governance/capabilities.toml`:** Eintrag für `contextra-prompt-guard` (safety / E0, `allowed_internal = []`).
3. **CI / `.github/workflows/`:** Pfadfilter um `crates/contextra-prompt-guard/**` ergänzen.

## 5. Abweichungen von SPEC-v2

Keine.

## 6. Blockierte Punkte / Bekannte Upstream-Probleme

1. **`contextra-store`:** Vorbestehende Compile-Fehler auf `origin/main` (`block_cache.rs` doppelte Methodendefinitionen, `builder.rs` move out of drop, `scanner/mod.rs` fehlende Methoden). Hindert Workspace-weites `cargo check --workspace` und `cargo test --all`. Die Tests und Checks für isolierte Packages ohne `store`-Abhängigkeit (`contextra-prompt-guard`) laufen vollständig grün.

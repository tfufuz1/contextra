# Migration der Audit-Skripte auf Cargo Xtask

Die früheren eigenständigen Python- und Shell-Audit-Skripte unter `scripts/` wurden im Zuge der Systemkonsolidierung vollständig durch native, performante `cargo xtask`-Kommandos abgelöst.

## Zuordnung der ursprünglichen Skripte zu den `cargo xtask`-Kommandos

| Ursprüngliches Skript | Neues `cargo xtask`-Pendant | Beschreibung |
| :--- | :--- | :--- |
| `scripts/scan_orphan_symbols.py` | `cargo xtask orphan-symbols` | Identifiziert verwaiste/unbenutzte öffentliche Symbole über Crate-Grenzen hinweg. |
| `scripts/audit_unwired_ports.py` | `cargo xtask unwired-ports` | Prüft nicht angebundene Ports und Trait-Schnittstellen. |
| `scripts/scan_stub_implementations.py` | `cargo xtask stub-impls` | Findet unvollständige Stub-Implementierungen und Platzhalter. |
| `scripts/scan_doctrine_violations.py` | `cargo xtask doctrine-scan` | Überprüft Codebase-Integrität und Invarianten auf Doktrin-Verletzungen. |
| `scripts/audit_dependency_graph.py` | `cargo xtask dependency-graph-audit` | Auditiert den Crate-Abhängigkeitsgraphen (Zyklenerkennung & Ring-Distanzen). |
| `scripts/repo_doctor.sh` | `cargo xtask repo-doctor` | Repository-Integritätscheck und Diagnose. |
| `scripts/claim_guard.sh` | `cargo xtask claim-guard` | Validiert Claim-Gültigkeit und -Grenzen. |
| `scripts/bench_trend.py` | `cargo xtask bench-trend` | Analysiert Benchmark-Trends und Performance-Entwicklungen. |
| `scripts/fast_diff_lint.py` | `cargo xtask fast-diff-lint` | Führt Diff-basierte schnelle Lints und Sicherheitsprüfungen durch. |

## Vollständige Übersicht

Für eine vollständige Liste aller registrierten Harness-Kommandos und Qualitätsgates führe folgenden Befehl aus:

```bash
cargo xtask harness-list
```

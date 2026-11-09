# CI-Dokumentation (Zielzustand)

Dieses Dokument beschreibt den konsolidierten CI-Zielzustand im Repository `tfufuz1/contextra` gemäß Spezifikation. Sämtliche 35 ursprünglichen Workflows sind hier erfasst: 14 aktive Workflows unter `.github/workflows/` und 21 deaktivierte Workflows unter `.github/_disabled/`.

---

## 1. Aktive CI-Workflows (`.github/workflows/`)

| Workflow | Zweck | Trigger | Blockierend? | Bemerkung |
|---|---|---|---|---|
| `rust-ci.yml` | Kompilierung (`cargo check`), Formatierung (`cargo fmt`), Linting (`cargo clippy -D warnings`), Tests und Test-Coverage | `push` (main, master), `pull_request` | Ja (Pflicht-Jobs: check, fmt, clippy, test) / Nein (Coverage & Tiefenprüfung mit `continue-on-error: true`) | `xtask`-Aufrufe wurden vollständig durch native Cargo-Befehle ersetzt (`cargo check --workspace --exclude contextra-py`). |
| `deny.yml` | Supply-Chain-Prüfung: Lizenzen, Sicherheits-Advisories, verbotene Abhängigkeiten | `push` (main), `pull_request` | Ja | Verwendet `cargo-deny-action`. |
| `unwrap-gate.yml` | Ratchet-Zähler gegen ungesicherte `unwrap()`/`expect()` in Produktionscode | `pull_request` | Ja | Prüft gegen Baseline in `.github/unwrap_baseline.txt`. |
| `loom-gate.yml` | Nebenläufigkeitsprüfung für Concurrency-Kerne (`contextra-store`, `contextra-crypto`, `contextra-vector`) mit Loom | `schedule` (täglich 04:00 UTC), `workflow_dispatch`, `pull_request` | Ja | Läuft bei Änderungen in den Pfaden der Concurrency-Kerne. |
| `fuzz-extended.yml` | Fuzz-Testing auf Kern-Crates (`contextra-store`, `contextra-vector`, `contextra-db`, `contextra-text`) | `schedule` (nächtlich 01:00 UTC), `workflow_dispatch` | Nein (nächtlicher Lauf) | Fuzz-Targets auf tatsächlich existierende Targets bereinigt. |
| `bench.yml` | Benchmark-Suite und Dry-Run-Validierung | `workflow_dispatch`, `schedule` (wöchentlich) | Nein (`continue-on-error: true`) | Manuell und zeitgesteuert, ohne blockierende Regression-Gates. |
| `bench-regression.yml` | Criterion-Benchmark-Regressionsprüfung für Kern-Crates | `workflow_dispatch`, `schedule` (wöchentlich) | Nein (`continue-on-error: true`) | Manuell und zeitgesteuert; `xtask`-Gate entfernt. |
| `mutation-testing.yml` | Mutation-Testing via `cargo-mutants` | `schedule` (wöchentlich Mo 03:00 UTC), `workflow_dispatch` | Nein (`continue-on-error: true`) | Nicht blockierend; `xtask`-Gate und Historien-Commit entfernt. |
| `contextra-py-ci.yml` | Eigenständiger CI-Stack für Python-Bindings (`contextra-py`) | `push` (main, master), `pull_request` | Ja | Prüft Check, Test, Clippy und Deny für Python-Crate; `xtask`-Panic-Check entfernt. |
| `build-wheels.yml` | Multi-Plattform-Build der Python-Wheels via Maturin | `workflow_dispatch`, `push` (tags `v*.*.*`) | Ja | Erzeugt Wheels für Linux, macOS und Windows. |
| `publish-pypi.yml` | Veröffentlichung der Wheels auf PyPI | `workflow_dispatch` | Ja | Manuell autorisierter Release-Workflow. |
| `release-mcp-binary.yml` | Erstellung und Release der `contextra-mcp-server`-Binaries | `push` (tags `v*.*.*`), `workflow_dispatch` | Ja | Kompiliert Multi-Plattform-Binärpakete und publiziert Release. |
| `tsan.yml` | ThreadSanitizer-Prüfung zur Erkennung von Race Conditions in Store, Vektor und Graph | `schedule` (täglich 03:00 UTC), `workflow_dispatch`, `pull_request` | Ja | Gezielte TSan-Testläufe auf Concurrency-Prüfungen. |
| `simd-cross-validation.yml` | Validierung der SIMD-Distanzkerne gegen Skalar-Referenzen | `pull_request`, `workflow_dispatch` | Ja | Testet `contextra-simd` mit und ohne CPU-Features (AVX2/AVX-512). |

---

## 2. Deaktivierte Workflows (`.github/_disabled/`)

Folgende 21 Workflows wurden per `git mv` nach `.github/_disabled/` verschoben und sind damit in GitHub Actions inaktiv:

| Workflow | Ursprünglicher Zweck | Grund für Deaktivierung |
|---|---|---|
| `audit-integrity.yml` | Audit-Integritätsprüfung | Veraltete Governance/Ratchets; referenziert nicht existierendes `xtask`. |
| `baukasten-gates.yml` | Baukasten-Architektur-Gates | Veraltetes Metasystem; Governance-Überbau abgebaut. |
| `chaos.yml` | Chaos-Fault-Injection-Läufe | Entkoppelt; Concurrency- und Durability-Prüfungen in dedizierten Tests konsolidiert. |
| `commit-integrity-guard.yml` | Commit-Integritätsprüfung | Veraltetes Gate; blockiert saubere Entwicklungs- und Rebase-Prozesse. |
| `context-gates.yml` | Context-Integrity-Gates | Veralteter Governance-Überbau. |
| `coverage.yml` | Separates Coverage-Gate mit Schwellenwert | Redundant zu `rust-ci.yml` (wo `llvm-cov` bereits integriert ist); referenziert nicht existierendes `xtask`. |
| `deep-gates.yml` | Tiefenprüfungs-Metagate | Veralteter Workflow-Call-Überbau; referenziert nicht existierendes `xtask`. |
| `flake-report.yml` | Flaky-Test-Reporting | Veraltetes Tooling; referenziert nicht existierendes `xtask`. |
| `jules-dispatch.yml` | Dispatcher für Bot-Interaktionen | Veraltetes Dispatch-Tooling; referenziert nicht existierendes `xtask`. |
| `merge-gate.yml` | Workspace-weites Merge-Gate | Veralteter Gate-Überbau; referenziert nicht existierende Crates und `xtask`. |
| `nucleation-recall-history.yml` | Tracking historischer Recall-Werte | Veraltetes Tracking; referenziert nicht existierendes `xtask`. |
| `post-merge-verification.yml` | Nachgelagerte Merge-Verifikation | Veraltetes Tooling; referenziert nicht existierendes `xtask`. |
| `proof-correctness.yml` | Nachweis- und Löschkorrektheits-Gates | Veraltete Nachweis-/Löschlogik; referenziert nicht existierendes `xtask`. |
| `prune-branches.yml` | Automatisches Löschen alter Branches | Administratives Skript; kein CI-Bestandteil. |
| `refresh-jules-context.yml` | Kontext-Aktualisierung für Bot | Veraltetes Agent-Tooling; fehlerhafte YAML-Syntax und `xtask`-Abhängigkeit. |
| `scheduled-audit.yml` | Periodische Workspace-Audits | Veraltete Governance; fehlerhafte YAML-Syntax und `xtask`-Abhängigkeit. |
| `shell-commit-gate.yml` | Shell-Skript-Commit-Prüfung | Veraltetes Gate; referenziert nicht existierendes `xtask`. |
| `unwrap-baseline-check.yml` | Automatisches Ratchet-Schärfen für Unwraps | Veraltetes Ratchet-Skript; referenziert nicht existierendes `xtask`. |
| `update-prompter-data.yml` | Prompter-Datenaktualisierung | Veraltetes Bot-Tooling; referenziert nicht existierendes `xtask`. |
| `verdict.yml` | Governance-Verdict-Gate | Veralteter Governance-Überbau; referenziert nicht existierendes `xtask`. |
| `veto-deadline-gate.yml` | Veto-Fristen-Prüfung | Veralteter Governance-Überbau; referenziert nicht existierendes `xtask`. |

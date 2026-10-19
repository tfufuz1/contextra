# ADR-N31: PR-Gate-Konsolidierung und dreistufiges Testsystem

* **Status:** Accepted
* **Datum:** 2026-10-19
* **Autoren:** Principal Architect (Contextra)
* **Betroffene Komponenten:** Governance, GitHub Actions (`.github/workflows/`), Harness Gate-Orchestrator (`xtask/`)
* **Spezifikationsreferenzen:** Spezifikation v15 Teil G.3 (Gate-Katalog), `governance/verdict-required.toml`, `docs/ci/BRANCH_PROTECTION_CHECKLIST.md`

---

## 1. Ist-Zustand & Befund (mit Pfad- und Zeilennachweisen)

Eine detaillierte Inventur der CI/CD-Pipelines und Governance-Konfigurationen zeigt erhebliche Zersplitterungen und Redundanzen:

1. **Workflow-Wildwuchs:**
   - In `.github/workflows/` befinden sich 32 Workflow-Dateien (`.yml`).
   - Davon werden 21 Workflows automatisch bei PRs und/oder Pushes auf den `main`-Branch ausgelöst (`bench.yml`, `commit-integrity-guard.yml`, `context-gates.yml`, `contextra-py-ci.yml`, `coverage.yml`, `deny.yml`, `loom-gate.yml`, `merge-gate.yml`, `mutation-testing.yml`, `proof-correctness.yml`, `rust-ci.yml`, `simd-cross-validation.yml`, `tsan.yml`, `unwrap-gate.yml`, `verdict.yml` u. a.).

2. **Funktionale Überlappungen in Einzel-Workflows:**
   - `context-gates.yml`, `merge-gate.yml`, `rust-ci.yml` und `verdict.yml` führen identische oder überschneidende Schritte aus (z. B. doppelte Kompilierung, doppelte Clippy-Prüfungen und doppelte Workspace-Tests).

3. **Inkonsistente Gate-Kataloge an vier Stellen:**
   - `AGENTS.md:32–40`: Listet 7 Kern-Gates (`scope-guard`, `protected-paths`, `symbol-exists`, `ratchet`, `gate-weakening`, `test-integrity`, `session-report`).
   - `governance/gates.toml:1–200`: Definiert 28 Einzel-Gates.
   - `governance/verdict-required.toml:1–80`: Listet 20 required Gates für den PR-Verdict.
   - `.jules/harness-phases.toml`: Enthält phasenspezifische Gate-Zuordnungen.

4. **Komplexität der Branch Protection Rules:**
   - Gemäß `docs/ci/BRANCH_PROTECTION_CHECKLIST.md:1–115` sind bis zu 16 Status Checks als GitHub Required Checks namentlich registriert. Jeder Namenswechsel einer Workflow-Matrix erfordert manuelle Admin-Eingriffe im GitHub UI.

---

## 2. Entscheidung & Soll-Zustand

1. **Höchstens 6 Required Checks mit Aggregator-Job `gate`:**
   - Die GitHub Branch Protection wird auf maximal 6 Status Checks konsolidiert.
   - Der zentral steuernde Status Check ist der Aggregator-Job `gate` aus `verdict.yml`. Alle atomaren Prüfungen melden ihr Ergebnis an `gate` zurück. Schlägt eine Teilprüfung fehl, schlägt `gate` fehl.

2. **Dreistufiges Testsystem (Tiered Testing):**
   - **Stage 1 (PR-blockierend, Time-Budget < 10 Minuten):**
     Format, Clippy, Quick-Tests, `check-ring-layering`, `protected-paths`, `scope-guard`, `claims-in-pr`, `verdict`. Läuft verpflichtend bei jedem PR.
   - **Stage 2 (Nightly / Scheduled):**
     Intensive/zeitaufwendige Prüfungen: `tsan`, `coverage`, `loom-gate`, Extended Fuzzing, Mutation-Testing (`cargo mutants`), `bench-regression`.
   - **Stage 3 (Release):**
     PyPI-Wheels Build, Cross-Platform Binaries (macOS/Windows/Linux ARM64/x86_64), finale Release-Attestierung.

3. **Keine Gate-Abschwächung (Explizite Abgrenzung zu `gate-weakening`):**
   - **Begründung:** Die Reduzierung der GitHub Required Status Checks auf den Aggregator-Job `gate` stellt **keine** Abschwächung von Qualitätsstandards im Sinne der Invariante `gate-weakening` dar.
   - Sämtliche 28 funktionale Gates aus `governance/gates.toml` bleiben zu 100 % erhalten. Sie werden nicht gelöscht, sondern innerhalb des Aggregator-Jobs `gate` oder in Stage 2/3 ausgeführt.

4. **Sticky Verdict-Kommentar am PR:**
   - Nach jedem CI-Lauf postet oder aktualisiert der `verdict`-Workflow einen einzigen ("sticky") Kommentar im PR. Dieser enthält die Tabelle aller geprüften Gates inklusive Dauer, Status (PASS/FAIL) und Artefakt-Links.

---

## 3. Anhang: Mapping Alt → Neu (Workflows & Gates)

| Alter Workflow / Gate | Alte Location | Neue Stufe | Neuer Host-Workflow / Aggregator |
| :--- | :--- | :--- | :--- |
| `protected-paths` | `merge-gate.yml` | **Stage 1 (PR)** | `verdict.yml` (via `gate`) |
| `gate-weakening` | `merge-gate.yml` | **Stage 1 (PR)** | `verdict.yml` (via `gate`) |
| `ratchet` | `merge-gate.yml` | **Stage 1 (PR)** | `verdict.yml` (via `gate`) |
| `scope-guard` | `context-gates.yml` | **Stage 1 (PR)** | `verdict.yml` (via `gate`) |
| `claims-in-pr` | `commit-integrity-guard.yml` | **Stage 1 (PR)** | `verdict.yml` (via `gate`) |
| `symbol-exists` | `context-gates.yml` | **Stage 1 (PR)** | `verdict.yml` (via `gate`) |
| `test-integrity` | `rust-ci.yml` | **Stage 1 (PR)** | `verdict.yml` (via `gate`) |
| `diff-budget` | `context-gates.yml` | **Stage 1 (PR)** | `verdict.yml` (via `gate`) |
| `registry-check` | `context-gates.yml` | **Stage 1 (PR)** | `verdict.yml` (via `gate`) |
| `session-report` | `verdict.yml` | **Stage 1 (PR)** | `verdict.yml` (via `gate`) |
| `cargo deny` | `deny.yml` | **Stage 1 (PR)** | `rust-ci.yml` |
| `tsan` | `tsan.yml` | **Stage 2 (Nightly)** | `scheduled-audit.yml` |
| `coverage` | `coverage.yml` | **Stage 2 (Nightly)** | `scheduled-audit.yml` |
| `loom-gate` | `loom-gate.yml` | **Stage 2 (Nightly)** | `scheduled-audit.yml` |
| `mutation-testing` | `mutation-testing.yml` | **Stage 2 (Nightly)** | `scheduled-audit.yml` |
| `simd-cross-validation` | `simd-cross-validation.yml` | **Stage 2 (Nightly)** | `scheduled-audit.yml` |
| `bench-regression` | `bench-regression.yml` | **Stage 2 (Nightly)** | `bench.yml` |
| `publish-pypi` | `publish-pypi.yml` | **Stage 3 (Release)** | `release.yml` |
| `release-mcp-binary` | `release-mcp-binary.yml` | **Stage 3 (Release)** | `release.yml` |

---

## 4. Konsequenzen

* **Positiv:**
  - Dramatisch verkürzte Feedback-Zeiten bei PRs (PR-CI unter 10 Minuten).
  - Keine fehleranfällige GitHub Admin-Pflege von 16 Einzel-Strings in den Required Status Checks.
  - Ein lokalisierter, übersichtlicher PR-Status durch den sticky Verdict-Kommentar.
* **Negativ:**
  - Sehr tiefe Fehler (z. B. seltene Data Races via TSan) werden erst im Nightly-Lauf statt im PR erkannt.

---

## 5. Migrationsschritte

1. Umstrukturierung von `verdict.yml` zur Bereitstellung des zentralen `gate`-Aggregators.
2. Konsolidierung der Workflow-Dateien in `.github/workflows/` auf die drei Stufen.
3. Anpassung der Branch Protection Rules auf GitHub auf die maximal 6 konsolidierten Status Checks.
4. Harmonisierung der Gate-Kataloge in `governance/verdict-required.toml` und `AGENTS.md`.

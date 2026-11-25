# ADR-116: xtask ohne Jules-Agent und ohne `.jules/`-Abhängigkeit

* **Datum**: 2026-11-25
* **Status**: Accepted (Auftraggeber-Auftrag 2026-11-25). Umsetzung im Repository.
* **Betroffene Komponenten**: `xtask` (Befehle, Harness, Tests, Governance-Konfiguration)
* **Bezug**: ADR-115 (vereinfachte Löschung). Dieser ADR ist die Gate-Entscheidung nach `AGENTS.md` §10.

## 1. Kontext

Jules war der frühere Coding-Agent des Projekts. Er ist abgesetzt. Die Koordination über `.jules/` (Claims, Session, Task-Karten, Prompter-Daten, Harness-Phasen, Audit-Intake) hängt damit an einem nicht mehr genutzten Werkzeug. `xtask` soll ohne `.jules/` laufen.

## 2. Entscheidung

1. Module, die ausschließlich den Jules-Workflow bedienen, werden entfernt: `harness/jules.rs`, `harness/jules_dispatch.rs`, `harness/task_card.rs`, `harness/plan_lint.rs`, `harness/claim_guard.rs`, `harness/scope_guard.rs`, `harness/gates_catalog_sync.rs`, `harness/env_attest.rs`, `jules_preflight.rs`, `jules_submit_gate.rs`, `check_jules_context_freshness.rs`, `claim.rs`, `context_pack.rs`, `session_init.rs`, `gen_prompter_data.rs`, `init_audit_fix.rs`, `session_history.rs`, `agent_lifecycle/branch_overlap.rs`.
2. Befehle, die nur diese Module nutzen, werden ebenfalls entfernt: `jules-preflight`, `jules-submit-gate`, `pre-push`, `claim`, `context-pack`, `session-init`, `gen-prompter-data`, `init-audit-fix`, `check-branch-overlap`, `check-jules-context-freshness`.
3. Gates, die nur `.jules`-Konfiguration prüfen oder nur im Jules-Workflow vorkamen, werden entfernt. Das betrifft `scope-guard` (inkl. Eintrag in `gate-guard`), `env-attest` und `gates-catalog-sync`. Die Einträge in `governance/gates.toml` und `governance/verdict-required.toml` entfallen.
4. Allgemeine Werkzeuge bleiben und werden umgebogen. Scratch-Ausgaben (`loop-guard`, `flake-report`, `session-report`) landen in `target/xtask-local/` (gitignoriert). Die Task-Karten-Auflösung im `diff-budget` liegt in `task-cards/`.
5. Die Tier-Daten für den Mutations-Gate liegen unverändert in `governance/crate-tiers.toml`. Die Schwellen bleiben gleich.
6. Die Doc-Truth-Prüfung verliert die `.jules`-Dokumentliste und die Crate-Tabellenprüfung für `JULES_CONTEXT.md`.
7. `cli/jules.rs` wird zu `cli/tools.rs`, weil dort allgemeine Werkzeuge liegen.

## 3. Begründung

- Die Module sind an einen abgesetzten Agenten gebunden. Ihre Pflege kostet mehr, als sie nutzen.
- Die entfernten Gates kontrollieren Abläufe der Jules-Koordination (Scope über Task-Karten, Claims, Tool-Versionen aus `.jules`). Sie sind nicht Teil der Produktqualität.
- Die verbleibenden Gates (`protected-paths`, `gate-weakening`, `diff-budget`, `doc-truth`) decken die Schutzfunktionen weiter ab.

## 4. Konsequenzen

- Der Scope-Schutz über Task-Karten entfällt. Für Änderungen gilt künftig die Prüfung über `protected-paths` und `diff-budget`.
- Die Tool-Pinning-Prüfung aus `env-attest` entfällt. `env-validate` bleibt erhalten.
- Die Harness-Phasen-Synchronisation entfällt. `governance/gates.toml` ist die einzige Quelle für Gates.
- Das Verzeichnis `.jules/` selbst bleibt im Repository, wird von `xtask` aber nicht mehr gelesen. Die Löschung ist eine separate Entscheidung.

## 5. Bekannte vorbestehende Fehler (nicht durch diesen ADR verursacht)

- `harness_unwired_ports_test` kompiliert nicht (`crate::`-Pfade im `#[path]`-Include).
- `check-result-dropped-io` meldet zwei Verstöße in `contextra-crypto` und `contextra-wire`.
- `gate-arch` und der Doc-Truth-Test auf dem echten Repo schlagen bereits im Ausgangszustand fehl.

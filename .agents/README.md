# .agents – Kontext für Agenten

Dieser Ordner enthält Kontext für KI-Agenten (Jules, Claude, Antigravity, Gemini). **Er ist nicht normativ.** Normativ ist ausschließlich [`docs/SPEC.md`](../docs/SPEC.md), Version . Nach D-98 gehört der Ordner zum Repository und wird mit der SPEC gepflegt; T-DOC-1 prüft die Konsistenz.

## Ladeordnung für Agenten

1. [`AGENTS.md`](../AGENTS.md) im Wurzelverzeichnis (gemeinsame Regeln, Ringe, Tabelle der Fachprofile).
2. `docs/SPEC.md`: Kapitel 0 und 2, dann nur die Abschnitte der Aufgabe (Gliederung: `docs/specs/00-INDEX.md`).
3. `crates/<crate>/AGENTS.md`: Fachprofil, normative Anker, Verbote, Abnahme und Prüfbefehle des betroffenen Crates.
4. `experts/querschnitt.md`, wenn die Arbeit mehrere Crates berührt (Lösch-Kette, Crash, Concurrency, Build, Compliance, Performance).
5. `rules/`: knappe Regeln je Themengebiet.
6. `jules/` nur für Jules-Sitzungen.

## Inhalt

| Bereich | Dateien | Zweck |
|---|---|---|
| Fachprofile | `experts/README.md`, `experts/querschnitt.md` | gemeinsamer Kopf, sechs Querschnittsprofile; die Crate-Profile stehen in `crates/*/AGENTS.md` |
| Regeln | `rules/async-io.md`, `concurrency.md`, `crash_recovery.md`, `deletion_pipeline.md`, `dependencies.md`, `simd_safety.md`, `test_quality.md`, `testing.md`, `wal_crypto.md`, `wording.md`, `zero_panic.md` | je Thema, mit SPEC-Verweisen |
| Claude | `claude/CLAUDE.md` | Betriebsanleitung für Claude und Antigravity |
| Skills | `skills/*/SKILL.md` (gespiegelt in `.gemini/skills/`) | Löschpipeline, Workspace-Diagnose, Invarianten, Produktstufen, Sicherheits-Audit |
| Jules | `jules/PREAMBLE.md`, `SESSION_BOOTSTRAP.md`, `JULES_CONTEXT.md`, `tasks/BACKLOG.md`, Routinen | Ablauf, Stopp-Katalog, Arbeitsliste |
| Governance | `governance/*` | maschinenlesbare Gate- und Regeldateien (geschützte Pfade, siehe `governance/README.md`) |

## Pflegeregeln

- Jede Datei nennt den SPEC-Abschnitt, auf den sie sich stützt, und enthält keine eigenen Anforderungen.
- Ändert sich ein Fakt, wird zuerst die SPEC geändert, im selben Änderungspaket danach alle abgeleiteten Dokumente (D-98 (4)).
- Statusangaben („baut nicht“, „fehlt“) stehen nicht in statischen Dateien. Sie werden mit Befehlen ermittelt (`AGENTS.md`, Abschnitt 3) und stehen in `jules/tasks/BACKLOG.md`.
- `.gemini/skills/` ist eine Spiegelung von `.agents/skills/`. Änderungen erfolgen in `.agents/skills/` und werden dorthin kopiert.
- Sitzungsverläufe und Prüfprotokolle gehören nicht in den Quellbaum (D-98 (3), T-ARCH-8).

## Bekannte offene Punkte (außerhalb dieser Dateien)

Geschützte oder nicht-Markdown-Dateien, die gesondert zu prüfen sind: `jules/setup/*.sh` (Toolchain-Pin `1.89.0` gegenüber `rust-toolchain.toml`), `mcp_config.json` (Verweis auf `contextra-mcp`, existiert nicht), `hooks/session_start.sh`, `governance/*.toml` (u. a. veraltete Messbegriffe, Verweise auf `contextra-wire`, `contextra-mcp`), `.github/workflows/scheduled-audit.yml` (Ablage `.jules/`).

# .agents – Kontext für Agenten

Dieser Ordner enthält Kontext für KI-Agenten (Jules, Claude, Antigravity). **Er ist nicht normativ.** Normativ ist ausschließlich [`docs/SPEC.md`](../docs/SPEC.md), Version 4.2.0.

Nach SPEC D-98 ist dieser Ordner Teil des Repositorys und wird mit der SPEC gepflegt. Alle Markdown-Dateien sind auf die SPEC 4.3.0 abgeglichen.

## Ladeordnung für Agenten

1. `AGENTS.md` im Wurzelverzeichnis.
2. `docs/SPEC.md`, die Abschnitte zur jeweiligen Aufgabe (Kapitel 0 und Abschnitt 15.4 als Einstieg).
3. `crates/<crate>/AGENTS.md` für das betroffene Crate.
4. `.agents/rules/` für das Themengebiet.
5. `.agents/jules/` nur für Jules-Sitzungen.

## Dateien

| Bereich | Dateien | Zustand |
|---|---|---|
| Übersicht und Ladeordnung | `README.md`, `claude/CLAUDE.md`, `governance/README.md` | überarbeitet |
| Jules: Grundregeln und Ablauf | `jules/PREAMBLE.md`, `jules/JULES_CONTEXT.md`, `jules/SESSION_BOOTSTRAP.md` | überarbeitet |
| Jules: Prozesse | `jules/AUDIT_INTAKE_PROTOCOL.md`, `jules/COMMON_LLM_ERRORS.md`, `jules/GITHUB_WORKFLOW.md`, `jules/HARNESS_QUICKSTART.md`, `jules/ISSUE_AUTOMATION_POLICY.md`, `jules/SCHEDULED_AUDIT.md`, `jules/tasks/README.md` | überarbeitet |
| Jules: Sitzungsprotokolle | `jules/JULES_LOG.md`, `jules/JULES_LOG_2.md` | auf Hinweis reduziert; Inhalt bleibt in der Git-Historie |
| Jules: Routinen | `jules/routines/README.md` und 8 Routinen | überarbeitet |
| Regeln | `rules/*.md` (8 Dateien) | überarbeitet |
| Skills | `skills/*/SKILL.md` und `skills/*/references/*.md` (8 Dateien) | überarbeitet |

**Nicht überarbeitet** (keine Markdown-Dateien): Shell-Skripte (`hooks/session_start.sh`, `jules/setup/*.sh`, `skills/*/scripts/*.sh`), Konfiguration (`hooks.json`, `settings.json`, `skills.json`, `mcp_config.json`, `jules/setup/required-tools.toml`, `jules/tasks/_TEMPLATE.toml`, `governance/*.toml`). Diese Dateien müssen gesondert geprüft werden, insbesondere auf Pfade (`.jules/` existiert im Repository nicht, die Ablage ist `.agents/jules/`) und auf Verweise auf nicht vorhandene Workflows.

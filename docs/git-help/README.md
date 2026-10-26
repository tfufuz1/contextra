# Contextra — GitHub Wizard Suite

<!-- doc-stamp: 2026-10-26 | Zweck: Gebrauchsanleitung git-help-Skripte -->

Diese Skripte führen dich Schritt für Schritt durch die GitHub-Einrichtung und den täglichen Workflow für das Contextra-Projekt. Sie sind als interaktive zsh-Wizards geschrieben — du wirst bei jedem wichtigen Schritt gefragt, bevor etwas passiert.

## Voraussetzungen

- **zsh** (Standard auf macOS, via `brew install zsh` oder `apt install zsh` auf Linux)
- **gh CLI** ([cli.github.com](https://cli.github.com/)) — wird von Skript 01 geprüft
- **Schreibrechte** auf dem Repository `tfufuz1/contextra`
- **Git** konfiguriert mit SSH-Key oder HTTPS-Credentials

## Skripte im Überblick

| Skript | Wann nutzen |
|--------|-------------|
| [`00_wizard_menu.zsh`](./00_wizard_menu.zsh) | Immer — Einstiegspunkt mit Menü |
| [`01_setup_github.zsh`](./01_setup_github.zsh) | **Einmalig** bei Ersteinrichtung |
| [`02_create_labels.zsh`](./02_create_labels.zsh) | **Einmalig** nach dem ersten Setup |
| [`03_create_milestones.zsh`](./03_create_milestones.zsh) | **Einmalig** nach Labels |
| [`04_create_issues.zsh`](./04_create_issues.zsh) | **Einmalig** — WP-Issues anlegen |
| [`05_branch_protection.zsh`](./05_branch_protection.zsh) | **Einmalig** — main schützen |
| [`06_archive_tag.zsh`](./06_archive_tag.zsh) | **Bei WP 2.1** — Archiv-Tag setzen |
| [`07_daily_workflow.zsh`](./07_daily_workflow.zsh) | **Täglich** — Issues/CI/PRs |

## Empfohlene Reihenfolge (Ersteinrichtung)

```bash
# 1. Alle Skripte ausführbar machen (einmalig)
chmod +x docs/git-help/*.zsh

# 2. Menü starten
zsh docs/git-help/00_wizard_menu.zsh

# Im Menü dann der Reihe nach:
#   1 → GitHub-Ersteinrichtung (gh auth login)
#   2 → Labels erstellen
#   3 → Milestones erstellen
#   4 → WP-Issues erstellen (wähle "alle")
#   5 → Branch-Protection einrichten
```

## Einzelne Skripte direkt aufrufen

```bash
# Täglichen Status anzeigen
zsh docs/git-help/07_daily_workflow.zsh

# Archiv-Tag setzen (WP 2.1)
zsh docs/git-help/06_archive_tag.zsh

# Nur Labels neu erstellen/aktualisieren
zsh docs/git-help/02_create_labels.zsh
```

## Skript-Details

### `00_wizard_menu.zsh` — Hauptmenü
Interaktiver Einstiegspunkt. Prüft automatisch ob `gh` installiert und authentifiziert ist.
Ruft alle anderen Skripte auf.

### `01_setup_github.zsh` — Ersteinrichtung
- Prüft ob `gh` CLI installiert ist (zeigt Installationsanleitung wenn nicht)
- Führt `gh auth login` durch (Browser-Flow — kein Passwort nötig)
- Setzt `gh repo set-default tfufuz1/contextra`
- Prüft und setzt den Git-Remote `origin`

### `02_create_labels.zsh` — Labels
Erstellt alle Labels aus `.agents/jules/GITHUB_WORKFLOW.md` **idempotent** (kann mehrfach ausgeführt werden, aktualisiert vorhandene):

| Label | Farbe | Bedeutung |
|-------|-------|-----------|
| `jules` | blau | Task für Jules-Agent |
| `priority-p0` | rot | Blocker/kritisch |
| `bug` | rot | Defekt |
| `enhancement` | cyan | Feature/Verbesserung |
| `task` | gelb | WP-Arbeitsauftrag |
| `claimed` | grau | Von Agent reserviert |
| `question` | lila | Klärung erforderlich |
| `wontfix` | weiß | Wird nicht bearbeitet |
| `invalid` | hellgrau | Ungültig/Duplikat |
| `protected-path` | rosa | Berührt geschützten Pfad |

### `03_create_milestones.zsh` — Milestones/Sprints
Erstellt Milestones für Phase 0–3 aus `docs/PLAN.md`. Fragt für jeden Milestone nach einem optionalen Fälligkeitsdatum. Idempotent (überspringt vorhandene).

| Milestone | WPs |
|-----------|-----|
| Phase 0 – Baseline | WP 0.1–0.4 |
| Phase 1 – Stubs | WP 1.1–1.3 |
| Phase 2 – Archivierung | WP 2.1–2.6 |
| Phase 3 – Fundament | WP 3.1–3.5 |

### `04_create_issues.zsh` — WP-Issues
Erstellt alle Arbeitspaket-Issues mit dem exakten Format aus `GITHUB_WORKFLOW.md`:
- Automatisch mit Labels `jules,task` und passendem Milestone
- Idempotent (überspringt Issues die schon existieren)
- Wählbar: nur Phase 0, 1, 2 oder alle

### `05_branch_protection.zsh` — Branch-Protection
Richtet GitHub Branch-Protection für `main` ein:
- Erzwingt Pull Requests (kein direktes Pushen auf main)
- Pflicht-CI-Status-Check (Name konfigurierbar, Standard: `CI`)
- Verhindert Force-Pushes

> [!IMPORTANT]
> Du brauchst **Admin-Rechte** auf dem Repository für Branch-Protection.
> Ohne Admin-Rechte: Manuell unter `github.com/tfufuz1/contextra/settings/branches`.

### `06_archive_tag.zsh` — Archiv-Tag (WP 2.1)
Setzt den Git-Tag `archiv/v0` auf den aktuellen HEAD — **Pflicht-Akzeptanzkriterium für WP 2.1**.
Dieser Tag markiert den Stand vor der Archivierung der Archiv-Crates unveränderlich.
Idempotent (prüft ob Tag schon existiert, pusht ggf. nur).

### `07_daily_workflow.zsh` — Täglicher Helfer
Zeigt beim Start:
1. Offene Jules-Issues (mit Milestone-Zuordnung)
2. Letzte 5 CI-Läufe (✅/❌/🔄)
3. Offene Pull Requests

Auf Wunsch: Issue-Details anzeigen und Branch für das Issue erstellen.

## Fehlerbehandlung

Alle Skripte:
- Zeigen **jeden** `gh`-Befehl vor der Ausführung an
- Brechen bei Fehlern sauber ab (`set -euo pipefail`)
- Geben verständliche Fehlermeldungen und Hilfe-Hinweise aus
- Sind **idempotent** — sicher mehrfach ausführbar

## Fehlerbehebung

**`gh: command not found`**
→ Skript 01 ausführen, folge der Installationsanleitung.

**`Error: Not Found` bei API-Aufrufen**
→ Prüfe ob das Repo `tfufuz1/contextra` auf GitHub existiert und du Zugriff hast.

**`gh auth status` schlägt fehl**
→ `gh auth login --web` manuell ausführen.

**Branch-Protection schlägt fehl**
→ Du brauchst Admin-Rechte. Manuell unter: `github.com/tfufuz1/contextra/settings/branches`.

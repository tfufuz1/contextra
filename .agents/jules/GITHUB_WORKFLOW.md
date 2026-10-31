# GitHub als agiler Kontext — Konventionen & Protokoll

> **Status 30.10.2026 (Reduktion):** Diese Regel gilt weiter, soweit sie nicht auf entfernte Crates oder entfernte Routinen verweist. Solche Stellen sind veraltet. Maßgeblich: `AGENTS.md` und `docs/SPEC.md` (Fassung 4.0).

> **Prinzip:** GitHub-Issues = Task-Karten. GitHub-Milestones = Sprints.
> GitHub-PRs = Reviews und Merge-Geschichte. GitHub-Actions = CI-Signale.
> Statische Repo-Dateien enthalten NUR unveränderliche Regeln, niemals Projektstatus.

---

## Issues als Task-Karten

### Label-Schema

| Label | Bedeutung | Jules-Verhalten |
|-------|-----------|-----------------|
| `jules` | Dieser Task ist für Jules bestimmt | Sofort aufnehmen |
| `priority-p0` | Blocker / kritisch | STOP alles andere, zuerst fixen |
| `bug` | Defekt im Code | Reproduzieren → Fix → Test → PR |
| `enhancement` | Feature / Verbesserung | VETOES.md prüfen → ADR nötig? → implementieren |
| `task` | Allgemeiner Arbeitsauftrag | Abarbeiten nach Beschreibung |
| `claimed` | Von einem Agenten reserviert | Nicht doppelt bearbeiten |
| `claim:<crate>` | Crate-spezifischer Claim | Nur diese Crate bearbeiten |
| `question` | Klärung erforderlich | Antwort kommentieren, nicht raten |
| `wontfix` / `invalid` | Nicht bearbeiten | Überspringen |

### Issue-Anatomie (Task-Karte)

```
Titel: <verb>(<scope>): <was>
Body:
  ## Ziel
  <was soll nach dem Task anders sein>

  ## Akzeptanzkriterien
  - [ ] <messbares Kriterium>

  ## Scope
  crates/<CRATE>/

  ## Veto-Check
  keine / F-NN geprüft: nicht betroffen

  ## Out-of-scope
  <was Jules NICHT anfassen soll>
```

---

## GitHub-Befehle (Jules-Arbeitsablauf)

### Session-Start

```bash
# Welche Tasks stehen an?
gh issue list --state open --label "jules" --json number,title,body,labels,milestone

# Task-Volltext lesen (IMMER vor Codebeginn)
gh issue view <N>

# Task reservieren (verhindert Doppelarbeit)
gh issue edit <N> --add-label "claimed"
gh issue comment <N> --body "🤖 Claiming: starte Arbeit — Plan: <1-2 Sätze>"
```

### Während der Arbeit

```bash
# Fortschritt melden (bei längerem Task nach jedem Teilschritt)
gh issue comment <N> --body "🔄 Fortschritt: <was getan> — nächster Schritt: <was kommt>"

# Frage stellen (statt Annahme treffen → reduziert Rework)
gh issue comment <N> --body "❓ <Frage>: <Kontext/warum unklar>"

# Neuen Defekt gefunden (nicht im aktuellen Scope)
gh issue create \
  --title "bug(<crate>): <kurze Beschreibung>" \
  --label "bug" \
  --body "## Gefunden bei\nPR #<N> / Task #<M>\n\n## Symptom\n<was passiert>\n\n## Datei\n<Pfad>:<Zeile>"
```

### CI-Fehler auf main (Pflicht: sofort handeln)

```bash
# Fehlgeschlagene Jobs identifizieren
gh run list --limit 5 --json name,conclusion,databaseId
gh run view <RUN-ID> --log-failed | head -60

# Wenn Ursache klar → direkt fixen, kein neues Issue nötig
# Wenn Ursache unklar → Issue öffnen
gh issue create \
  --title "bug: CI rot auf main — <Workflow-Name>" \
  --label "bug,priority-p0" \
  --body "Run: <URL>\nJob: <Name>\nFehler: <Fehlermeldung>"
```

### Session-Ende

```bash
# 1. Rebase auf main (PFLICHT — hochfrequentierte Entwicklung)
git fetch origin
git rebase origin/main
# Bei Konflikten: git status → auflösen → git add <datei> → git rebase --continue
# Falls unlösbar: git rebase --abort → gh issue comment <N> --body "⚠️ Rebase-Konflikt"

# 2. Gates nach Rebase erneut prüfen (Rebase kann neue Fehler einbringen)
just check && just test

# 3. PR erstellen und Issue schließen
gh pr create \
  --title "<verb>(<scope>): <was>" \
  --body "Closes #<N>\n\n## Was\n<Beschreibung>\n\n## Getestet\n\`cargo nextest run -p <crate>\`"

gh issue close <N> --comment "✅ Erledigt in PR #<PR-N>"
```

### Blocker / Eskalation

```bash
# Sofort sichtbar machen, nicht im Commit-Kommentar vergraben
gh issue create \
  --title "🚫 Blocker: <Titel>" \
  --label "bug,priority-p0" \
  --body "## Kontext\nTask #<N>\n\n## Problem\n<Fehlermeldung>\n\n## Was bereits versucht\n1. ...\n2. ..."
```

---

## Milestones als Sprints

```bash
# Aktueller Sprint
gh api repos/tfufuz1/contextra/milestones | jq -r '.[] | "\(.title): \(.open_issues) offen, fällig: \(.due_on // "∞")"'

# Welche Issues gehören zum Sprint?
gh issue list --milestone "<Milestone-Titel>" --json number,title,labels,state
```

---

## Was NICHT über GitHub-Issues läuft

| Thema | Stattdessen |
|-------|-------------|
| Routineaudits (kein Fehlersignal) | PR-basierter Handoff via `.agents/jules/routines/` |
| Architekturentscheidungen | ADR in `docs/decisions/` |
| Unveränderliche Regeln | `AGENTS.md`, `VETOES.md`, `capabilities.toml` |
| Automatisierte Issue-Erstellung durch Workflows | DEAKTIVIERT (Issue-Automation-Policy) |

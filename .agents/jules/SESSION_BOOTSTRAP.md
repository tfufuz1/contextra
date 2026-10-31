# Contextra — Jules Session Bootstrap

> **Status 30.10.2026 (Reduktion):** Diese Regel gilt weiter, soweit sie nicht auf entfernte Crates oder entfernte Routinen verweist. Solche Stellen sind veraltet. Maßgeblich: `AGENTS.md` und `docs/SPEC.md` (Fassung 4.0).
> Maschinenausführbare Checkliste. Jede Session beginnt mit GitHub, nicht mit statischen Dateien.
>
> **Prinzip:** GitHub = agiler Kontext (Tasks, Reviews, CI). Statische Dateien = unveränderliche Regeln.

---

## Schritt 0 — GitHub-Kontext laden (ERSTE Aktion, immer)

`refresh-context.sh` hat `SESSION_START.md` bereits erzeugt. Es reicht, sie zu lesen:

```bash
cat .agents/jules/context/SESSION_START.md
```

Die Datei enthält bereits: Milestones · Task-Issues · Backlog · offene PRs · CI-Status mit fehlgeschlagenen Jobs · letzte Commits · CRITICAL-Tags · Claims · WORKING_STATE.

**Bei Bedarf direkt nachschlagen (gh CLI ist immer bevorzugt):**

```bash
# Welche Issues sind für mich (Jules)?
gh issue list --state open --label "jules" --json number,title,body,milestone

# Was steckt in einem Task-Issue?
gh issue view <N>

# Welcher CI-Run ist rot, und warum?
gh run list --limit 5
gh run view <RUN-ID> --log-failed

# Was sind die neuesten PRs?
gh pr list --state all --limit 10

# Was ist der aktuelle Sprint?
gh api repos/tfufuz1/contextra/milestones | jq '.[] | "\(.title): \(.open_issues) open"'
```

> **Stop-Bedingung:** Sind CRITICAL/BLOCKER-Tags oder rote CI-Runs auf `main` in SESSION_START.md gelistet
> → erst beheben, bevor anderer Code geschrieben wird.

---

## Schritt 1 — Unveränderliche Regeln lesen (nur die relevanten)

Rangfolge: `docs/SPEC.md` > `docs/VETOES.md` > `capabilities.toml` > `AGENTS.md`

```bash
# Immer: Veto-Register prüfen (vor jeder Feature-Arbeit, F-NN-Bezeichnung)
grep -A5 "feature_id: F-" docs/VETOES.md

# Crate-spezifische Regeln (Pfade in SESSION_START.md → Abschnitt "Crate-AGENTS.md")
cat crates/<CRATE>/AGENTS.md
```

Aufgabenspezifisch:

| Aufgabe | Zu lesende Regeldate |
|---------|----------------------|
| Store/WAL | `crates/contextra-store/AGENTS.md` · `rules/wal_crypto.md` |
| SIMD/Vector | `crates/contextra-vector/AGENTS.md` · `rules/simd_safety.md` |
| Neue Dependency | `rules/dependencies.md` |
| Neue API | `docs/CONSTITUTION.md` · `docs/TYPE_REGISTRY.md` |
| ADR schreiben | `docs/decisions/README.md` (letzte 5 lesen) |
| Tests | `rules/testing.md` · `rules/test_quality.md` |
| unsafe | `rules/simd_safety.md` — NUR in approved crates |

---

## Schritt 2 — Toolchain verifizieren

```bash
cargo check --workspace
```

---

## Schritt 3 — Pre-Write-Check (vor JEDER Code-Änderung)

```bash
# Symbol vor Nutzung verifizieren (niemals aus dem Gedächtnis annehmen)
grep -n "pub fn <METHODE>" crates/<CRATE>/src/<DATEI>.rs

# Typ-Duplikat verhindern
grep "<TYPNAME>" docs/TYPE_REGISTRY.md
```

---

## Schritt 4 — GitHub-Kommunikation (Pflicht bei Task-Arbeit)

```bash
# Task-Start protokollieren
gh issue comment <N> --body "🤖 Starte Arbeit: <kurze Beschreibung des Plans>"

# Blocker sofort melden (nicht still ignorieren)
gh issue create --title "🚫 Blocker: <Titel>" --label "bug,priority-p0"

# Frage an Maintainer (statt Annahme treffen)
gh issue comment <N> --body "❓ <Frage>"
```

---

## Schritt 5 — Session-Ende: Gates

```bash
cargo fmt --all
cargo check --workspace
cargo test --workspace
```

---

## Schritt 6 — Rebase auf origin/main (PFLICHT, BLOCKIEREND)

> **Warum:** Contextra wird hochfrequentiert entwickelt. Ein PR ohne vorherigen Rebase
> erzeugt garantiert Merge-Konflikte auf main. Kein Submit ohne sauberen Rebase.

```bash
# 1. Neuesten Stand holen
git fetch origin

# 2. Rebase auf main
git rebase origin/main
```

**Bei Konflikten während des Rebase:**

```bash
# Konflikt-Dateien anzeigen
git status

# Für jede Konflikt-Datei: manuell auflösen, dann:
git add <datei>
git rebase --continue

# Abbruch (zurück zum Ausgangszustand) falls zu komplex:
git rebase --abort
# → dann: gh issue comment <N> --body "⚠️ Rebase-Konflikt in <Dateien> — Maintainer bitte merge/rebase klären"
```

**Nach erfolgreichem Rebase — Gates ERNEUT laufen lassen:**

```bash
# Gates müssen auf dem rebased Stand grün sein
cargo check --workspace && cargo test --workspace
```

> **Regel:** Schlägt ein Gate nach dem Rebase fehl, liegt ein echter Konflikt mit
> Änderungen auf main vor. Fix durchführen, dann Gate erneut prüfen — KEIN skip.

---

## Schritt 7 — Pre-Submit Prüfung (BLOCKIEREND)

```bash
just check && just test
# Kein submit() vor erfolgreichem Durchlauf.
```

```bash
# Task schließen und verlinken
gh issue close <N> --comment "✅ Erledigt in PR #<PR-N>"
```

> **Commit-Message-Regel:** Jede im PR-Body genannte Datei MUSS in
> `git diff --name-only origin/main...HEAD` erscheinen.

---

## Notfall-Eskalation (Prompt-Thrashing)

Gleicher Compiler-Fehler nach 2 Iterationen → **STOPP**:

```bash
# Blocker-Issue öffnen mit minimalem Reproduktions-Beispiel
gh issue create \
  --title "🚫 Thrash: <Fehlermeldung kurz>" \
  --label "bug,priority-p0" \
  --body "## Fehler\n\`\`\`\n<Fehlermeldung>\n\`\`\`\n\n## Diff\n\`\`\`\n$(git diff)\n\`\`\`"
```

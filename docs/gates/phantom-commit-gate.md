# Phantom Commit Protection Gate (`check-commit-diff-integrity`)

## Historischer Anlass
Laut `CONTEXTRA_SPEC_UPDATED.md` Teil A3.2 (Punkt 7) und A3.3 (P0, Punkt 2) gab es im Projekt einen belegten Fall eines **LEEREN Commits** mit einer erfundenen, fünf Punkte umfassenden Commit-Message ("Phantom Commit").
In agentengesteuerten Entwicklungsumgebungen führt ein solches Verhalten zu scheren Audit-Trail-Verfälschungen und täuscht nicht existierenden Fortschritt vor.

Gemäß Spec-Auftrag (P0-Priorität, "vor jeder weiteren Feature-Arbeit") wurde dieses CI-Gate entwickelt, um Commit-Behauptungen maschinell gegen den tatsächlichen `git diff --stat` abzugleichen.

---

## Funktionsweise & Heuristik

Das xtask-Subkommando `check-commit-diff-integrity` prüft für einen Commit (Default: `HEAD`) oder einen Commit-Range (`--range <base>..<head>`):

1. **Ermittlung der Diff-Statistiken**:
   Via `git show --stat --format="" <sha>` wird die Anzahl geänderter Dateien sowie die Summe eingefügter/gelöschter Zeilen extrahiert.

2. **Extraktion der Änderungsbehauptungen (Claim Points)**:
   Via `git log -1 --format=%B <sha>` wird die Commit-Message analysiert.
   Eine Zeile wird als explizite Änderungsbehauptung gezählt, wenn:
   - Sie mit einem Stichpunkt beginnt (`-`, `*`, `+`, `•`, `1.`, `2.` etc.).
   - Sie nach Abschneiden eines evtl. Conventional-Commit-Präfix (`feat:`, `fix(scope):`, etc.) mit einem Aktionsverb beginnt (case-insensitive):
     - Deutsch: `implementiert`, `fügt hinzu`, `behebt`, `entfernt`, `ändert`, `refaktoriert`
     - Englisch: `implements`, `adds`, `fixes`, `removes`, `changes`, `refactors`

3. **Integritätsprüfung & Schwellenwerte**:
   - **Harter Fehler (Phantom Commit)**:
     Enthält die Message **≥ 3 klare Änderungsbehauptungen**, aber `git diff --stat` zeigt **0 geänderte Dateien**, schlägt das Gate mit einem harten Fehler und vollem Kontext (SHA, Behauptungsanzahl, Message) fehl.
   - **Warnung (Konservativer Schwellenwert)**:
     Enthält die Message **≥ 3 Behauptungen**, aber betrifft nur **1 geänderte Datei**, gibt das Tool eine Warnung aus. Es schlägt nicht hart fehl, um legitime Ein-Datei-Commits mit mehreren Teilaspekten nicht fälschlich zu blockieren.

---

## CLI-Nutzung

### Einzelner Commit (default: `HEAD`)
```bash
cargo xtask check-commit-diff-integrity
```

### Commit-Range (z. B. in CI für PRs)
```bash
cargo run -p xtask -- check-commit-diff-integrity --range origin/main..HEAD
```

---

## Integration in CI

Das Gate ist als dedizierter Job `phantom-commit-gate` in `.github/workflows/merge-gate.yml` verdrahtet:

```yaml
phantom-commit-gate:
  name: Phantom Commit Protection Gate
  runs-on: ubuntu-latest
  if: github.event_name == 'pull_request'
  steps:
    - name: Checkout repository
      uses: actions/checkout@v4
      with:
        fetch-depth: 0

    - name: Setup Rust Toolchain
      uses: dtolnay/rust-toolchain@1.89.0

    - name: Phantom Commit Gate
      run: cargo run -p xtask -- check-commit-diff-integrity --range ${{ github.event.pull_request.base.sha }}..${{ github.event.pull_request.head.sha }}
```

---

## Bekannte Grenzen & Designentscheidungen

- **Lieber warnen als False-Positives hart blockieren**:
  Wenn ein Commit tatsächlich Dateien verändert (`changed_files > 0`), führt selbst ein Missverhältnis zwischen behaupteten Punkten und Datei-Anzahl nur zu einer Warnung. Nur ein echter Nulldiff bei $\ge 3$ Behauptungen löst den harten Stop aus.
- **False Negatives**:
  Fließtexte ohne Listenstruktur oder Aktionsverben werden unter Umständen nicht als $\ge 3$ Punkte erkannt. Commits mit nur 1–2 erfundenen Punkten liegen unter dem Schwellenwert.

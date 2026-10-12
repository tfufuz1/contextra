# Contextra — Protokoll für Parallele Sessions (PARALLEL_SESSIONS.md)

An Contextra arbeiten parallel mehrere isolierte Sessions gleichzeitig, jede in einer eigenen, nach Abschluss gelöschten VM. Die einzige gemeinsame Wahrheitsquelle zwischen ihnen ist der **Remote-Git-Zustand** (`origin`), nicht der lokale Dateizustand.

## 1. Vertragesdateien der Parallelsitzungen
Folgende Dateien werden von den parallelen Skripten/Abläufen erzeugt bzw. referenziert (sie entstehen dynamisch):
- `.jules/context/CONTEXT_PACK.md` <!-- doc-ref-ignore --> (enthält Repo-Stand, Baseline, Parallel-Branches, Karte)
- `.jules/context/SESSION_START.md` <!-- doc-ref-ignore --> (Session-Einstieg)
- `REPO_STATE.md` <!-- doc-ref-ignore --> (Remote Git-Zustand)
- `BASELINE.md` <!-- doc-ref-ignore --> (Baseline-Messungen)
- `context-card` <!-- doc-ref-ignore --> (Task-Kontext-Referenz)

## 2. Anchor- & Konflikt-Protokoll

Jede Aufgabe ist datei-scoped auf die im `scope`-Feld der Task-Karte angegebenen Dateien beschränkt. Vor der ersten Änderung MUSS geprüft werden, ob eine andere Session denselben Bereich bereits bearbeitet.

### Prüfbefehle vor Schritt 1:
```bash
# Branch-Überschneidungen und Kollisionen prüfen
cargo run -p xtask -- check-branch-overlap

# Remote-Git-Zustand und Log prüfen:
git fetch origin 2>&1 | tail -3
git log origin/main --oneline -5 -- <FILE_HINT>
git branch -r | grep -i "<CRATE>" || echo "Keine erkennbaren Remote-Branches"

# IN-PROGRESS-ANCHOR eines anderen Agenten prüfen:
grep -n "ANCHOR\[.*\] STATUS:IN-PROGRESS" <FILE_HINT> 2>/dev/null
```

### Konfliktbehandlung:
- **Falls ein IN-PROGRESS-ANCHOR eines anderen Agenten in genau dieser Datei existiert:**
  Aufgabe NICHT beginnen. Im Session-Log dokumentieren („Konflikt mit ANCHOR AGENT:N erkannt, Aufgabe übersprungen“) und mit der nächsten freien Aufgabe fortfahren. Ein doppelt bearbeiteter Scope erzeugt Merge-Konflikte.
- **Falls ein Remote-Branch oder offener PR dieselbe Crate (aber eine andere Datei) betrifft:**
  Fortfahren, aber im PR-Body MUSS auf den parallelen Branch/PR verwiesen werden (`Siehe auch #<NR>` bzw. Branch-Name), damit ein menschlicher Reviewer den Zusammenhang sieht.

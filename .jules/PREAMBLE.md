# Contextra — Jules Preamble

1. **Kanonischer Start & Leseordnung**: Führe `cargo xtask jules start --card <karte>` aus (`session-init` und `context-pack` laufen als interne Teilschritte). Lies NUR `.jules/context/CONTEXT_PACK.md` (enthält Repo-Stand, Baseline, Parallel-Branches, Karte, AGENTS-Auszüge). Bei Bedarf danach die vollständige Crate-`AGENTS.md`; `AGENTS.md`/`WORKING_STATE.md` nur für Fragen, die das Pack offenlassen.
2. **Dateien vollständig lesen & Symbole verifizieren**: Lies jede Datei vollständig, nie nur Grep-Treffer. Verifiziere jedes Symbol mit `cargo xtask symbol-exists <path>` bevor du es benutzt.
3. **Scope-Disziplin & Out-of-Scope Findings**: Ändere ausschließlich Dateien im `scope` der Karte. Melde Out-of-scope Findings im PR-Body, statt sie eigenmächtig zu beheben.
4. **Keine Gate-Umgehung**: Ändere niemals Gates, Lints, Baselines, Toolchain oder geschützte Pfade, um Rot zu beseitigen. Behebe stets die Ursache.
5. **Test-First & Stop-on-Fail**: Schreibe zuerst einen fehlschlagenden Test (rot), dann den Fix (grün). Bei dem ersten fehlschlagenden Schritt stoppen und Befund dokumentieren.
6. **Fehler-Limit & Abbruch**: Gleicher Fehler zweimal oder unlösbare Blockade → `cargo xtask jules stop --reason <grund>`.
7. **Definition von Fertig**: „Fertig" heißt: `cargo xtask jules submit` grün nach dem Ablauf `start` → `check` → `verify` → `submit`. Behaupte nichts, was der Session-Report nicht belegt.

# Contextra — Protokoll für Parallele Sitzungen & Konfliktvermeidung (PARALLEL_SESSIONS.md)

> Regeln zur Vermeidung von Kollisionen und Anker-Konflikten zwischen mehreren parallelen Jules-Agenten-Sitzungen.

## 1. Scope-Isolation & Branch-Overlap Check

Jede Task-Karte definiert im Feld `scope` die Liste der zulässigen Dateipfade.
Vor jeder Bearbeitung und während der Phase `check` wird die Überschneidung aktiver Branches automatisch geprüft:

```bash
cargo run -p xtask -- check-branch-overlap
```

## 2. Anker- & Kollisions-Protokoll

1. **Gleicher Anker / Exakte Kollision (Gleiche Datei & Gleiche Funktion/Zeile)**:
   - Tritt eine Kollision mit einer anderen aktiven Sitzung oder einem parallelen PR auf, **überspringe die Aufgabe sofort**.
   - Dokumentiere den Abbruch im Session-Log und breche sauber ab:
     ```bash
     cargo xtask jules stop --reason "Kollision mit paralleler Session auf <pfad>"
     ```

2. **Gleiches Crate, aber unterschiedliche Dateien/Funktionen**:
   - Die Aufgabe darf fortgesetzt werden.
   - Trage im PR-Body zwingend eine Referenz auf den parallelen PR / die parallele Session ein:
     `Siehe auch #<NR>` bzw. `Kopplung mit PR #<NR> (Crate <crate_name>)`.

## 3. Vertrag über parallele Verträge & Platzhalter-Namen

In einer Multi-Agenten-Umgebung entstehen bestimmte Artefakte und Dokumente parallel durch andere Prompts.
Folgende Pfade und Skripte sind vertraglich zugesichert und werden von `cargo xtask jules start` bereitgestellt:
- Einstiegs-Protokoll: `.jules/context/SESSION_START.md`
- Generiertes Kontextpaket: `.jules/context/CONTEXT_PACK.md`
- Setup-Profil-Erkennung: `CONTEXTRA_SETUP_PROFILE=core|full`
- Vorbereitende Setup-Skripte: `.jules/setup/install-fuzz-toolchain.sh`, `.jules/setup/install-bench-deps.sh`
- Just-Rezepte: `jules-env-check`, `fuzz-smoke CRATE`, `bench-setup`, `jules-gates`
- Regeldokumente: `rules/fuzzing.md`, `rules/benchmarking.md`, `rules/mutation-testing.md`, `rules/loom-miri.md`

Verlinke diese Namen in Berichten; ihre physikalische Entstehung zur Laufzeit ist Gegenstand der Parallel-Sessions.

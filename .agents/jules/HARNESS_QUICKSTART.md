# Harness Quickstart

> **Status 30.10.2026 (Reduktion):** Diese Regel gilt weiter, soweit sie nicht auf entfernte Crates oder entfernte Routinen verweist. Solche Stellen sind veraltet. Maßgeblich: `AGENTS.md` und `docs/SPEC.md` (Fassung 4.0).

Dieses Dokument beschreibt die Benutzung der schlanken 5-Phasen-Fassade für Jules-Sessions.

## Entwickler-Workflows

1. **`just check`**
   Prüft Syntax, Formatierung (`cargo fmt --check`), Lints (`cargo clippy`) und Workspace-Kompilierung.
2. **`just test`**
   Führt die Test-Suite aus.
3. **`just gates`**
   Führt Architektur- und Layering-Tests aus (`tests/layering.rs`).
4. **`just pre-push`**
   Führt den vollständigen Pre-Push-Check aus (`check test gates`).

## Beispielablauf

```bash
# 1. Formatierung und Lints prüfen
just check

# 2. Tests ausführen
just test

# 3. Layering & Architektur prüfen
just gates
```

## Exit-Codes

- `0` (pass / not_applicable): Phase erfolgreich durchlaufen.
- `1` (fail / Verstoß): Determinister Gate-Verstoß oder Testfehler.
- `2` (error / fail-closed): Unvollständiges Setup, fehlendes Pflichtkommando oder Systemfehler.

## Eskalationsleiter

1. **Phase 1: Lokales Stoppen (Prompt-Thrashing)**
   Derselbe Compiler- oder Testfehler nach 2 Iterationen → `jules stop --reason thrash`.
2. **Phase 2: Plan-Lint-Ablehnung (Revisionszähler >= 3)**
   3. Ablehnung im Plan-Linting → Status `error` mit Aufforderung: Task teilen.
3. **Phase 3: Menschliche Intervention**
   Mensch prüft Log und passt Task-Karte / Scope an.

Für Prompt-Details siehe [PREAMBLE.md](PREAMBLE.md).

# Routine: Chaos- & Fuzz-Langlauf

> **Status 30.10.2026 (Reduktion):** Diese Regel gilt weiter, soweit sie nicht auf entfernte Crates oder entfernte Routinen verweist. Solche Stellen sind veraltet. Maßgeblich: `AGENTS.md` und `docs/SPEC.md` (Fassung 4.0).

- **Takt**: Nächtlich (02:00 UTC)
- **Ziel**: Automatisierter Ausführungslauf der Chaos-Matrix und Fuzz-Smoke-Tests.
- **Eingaben**:
  - `.github/workflows/chaos.yml`
  - `just fuzz-all`
- **Harte Grenzen**: Max. 15 Dateien / 1 Crate pro PR. Scope-Globs: `crates/*/**`.
- **Stopp-Bedingungen**: Chaos-Crash durch Hardware/OS-Simulationseinschränkung.
- **Ergebnisformat**: Issue bei Fuzz/Chaos-Fund mit Repro-Artifacts.
- **Eskalation**: Siehe Architekturdokument §9.3.

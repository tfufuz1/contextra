# Routine: Drift-Wächter

> **Status 30.10.2026 (Reduktion):** Diese Regel gilt weiter, soweit sie nicht auf entfernte Crates oder entfernte Routinen verweist. Solche Stellen sind veraltet. Maßgeblich: `AGENTS.md` und `docs/SPEC.md` (Fassung 4.0).

- **Takt**: Täglich
- **Ziel**: Früherkennung von Doku-, Typ- und Registrierungs-Drifts im Repository.
- **Eingaben**:
  - `git diff --check`
- **Harte Grenzen**: Max. 15 Dateien / 1 Crate pro PR. Scope-Globs: `docs/**`, `capabilities.toml`, `AGENTS.md`.
- **Stopp-Bedingungen**: Bei manuell geänderten generierten Dateien ohne Quellcode-Anpassung.
- **Ergebnisformat**: PR mit aktualisierten Doku-Artefakten oder GitHub Issue bei unklaren Quellcode-Drifts.
- **Eskalation**: Siehe Architekturdokument §9.3.

# Routine: Unsafe-Prüfung

> **Status 30.10.2026 (Reduktion):** Diese Regel gilt weiter, soweit sie nicht auf entfernte Crates oder entfernte Routinen verweist. Solche Stellen sind veraltet. Maßgeblich: `AGENTS.md` und `docs/SPEC.md` (Fassung 4.0).

- **Takt**: Wöchentlich
- **Ziel**: Verifikation aller Unsafe-Inseln auf Gültigkeit der SAFETY-Kommentare und Miri-Sauberkeit.
- **Eingaben**:
  - `cargo miri test`
- **Harte Grenzen**: Max. 15 Dateien / 1 Crate pro PR. Scope-Globs: `crates/*/**`.
- **Stopp-Bedingungen**: Miri-Undefined-Behavior im Fremdcode ohne direkten Fix.
- **Ergebnisformat**: Audit-Report im PR-Text mit Miri-Verifikationsprotokoll.
- **Eskalation**: Siehe Architekturdokument §9.3.

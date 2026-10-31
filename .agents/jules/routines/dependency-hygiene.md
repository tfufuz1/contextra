# Routine: Dependency-Hygiene

> **Status 30.10.2026 (Reduktion):** Diese Regel gilt weiter, soweit sie nicht auf entfernte Crates oder entfernte Routinen verweist. Solche Stellen sind veraltet. Maßgeblich: `AGENTS.md` und `docs/SPEC.md` (Fassung 4.0).

- **Takt**: Wöchentlich
- **Ziel**: Prüfung von Lizenzen, Sicherheitsberichten und Supply-Chain-Audits neuer/aktualisierter Abhängigkeiten.
- **Eingaben**:
  - `cargo deny check licenses`
  - `cargo vet`
- **Harte Grenzen**: Max. 15 Dateien / 1 Crate pro PR. Scope-Globs: `Cargo.toml`, `Cargo.lock`, `supply-chain/**`, `deny.toml`.
- **Stopp-Bedingungen**: Unzulässige Lizenz ohne geschäftlich freigegebene Ausnahme.
- **Ergebnisformat**: PR mit aktualisierten Vet-Audits oder Dependency-Fixes.
- **Eskalation**: Siehe Architekturdokument §9.3.

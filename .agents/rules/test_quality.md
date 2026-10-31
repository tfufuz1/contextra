# Governance Rule: Test Quality & Zero-Panic Assurance (`rules/test_quality.md`)

> **Status 30.10.2026 (Reduktion):** Diese Regel gilt weiter, soweit sie nicht auf entfernte Crates oder entfernte Routinen verweist. Solche Stellen sind veraltet. Maßgeblich: `AGENTS.md` und `docs/SPEC.md` (Fassung 4.0).

## Gilt für
- `crates/*/tests/`
- Workspace-weite Test-Suiten

## Pflichtregeln
1. **Red-to-Green Workflow**: Vor der Implementierung eines Fixes/Features muss der fehlschlagende Test (Red Phase) nachgewiesen und dokumentiert sein (Quelle: `AGENTS.md`).
2. **Keine Assertion-losen Tests**: Testfunktionen dürfen nicht nur ohne Panics durchlaufen, sondern müssen explizite Invarianten-Assertions (`assert!`, `assert_eq!`) enthalten.
3. **Kein Auto-Accept von Snapshots**: Snapshot-Updates (`insta::assert_snapshot!`) dürfen keinesfalls ungeprüft committet werden; Differenzen müssen manuell verifiziert werden.
4. **Zero-Panic Invariante in Produktion**: Produktionscode darf keine `unwrap()`, `expect()` oder `panic!` enthalten, außer in explizit geführten Baseline-Dateien (`.github/unwrap_baseline.txt`) (Quelle: `.github/unwrap_baseline.txt`, `clippy::unwrap_used`).

## Häufige Fehler
- Nutzung von `.unwrap()` in Produktionspfaden statt richtiger `ContextraError`-Propagation.
- Leere Test-Rümpfe, die als "erfolgreich" gewertet werden.

## Verweise
- `.github/unwrap_baseline.txt`
- `AGENTS.md`

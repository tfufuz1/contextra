# acta-base

acta-base: Newtypes, Fehlerklassen, Ports, Festkomma.

> Teil des Acta-Workspace. Verbindliche Vorgaben: Wurzel-`CLAUDE.md` und `docs/SPEC.md`.

## Einordnung
- **Level:** 0 (`rings.toml`)
- **Größenbudget:** 2500 LOC Produktionscode (G-HUB); Überschreitung nur mit ADR
- **Aufgabe laut SPEC 3.3:** Newtypes (Ids, Ordinale), `DataClass`, `ErrorClass`, `ClassifiedError`, Ports `Clock`, `CryptoRng` (fallibel), kanonische Kodierung (geprüfte Big-Endian-Felder), Festkomma-Arithmetik `fx` (Q32.32, `fx_ln`, `isqrt`), Grenzwerte
- **Erlaubte interne Abhängigkeiten (SPEC 3.3):** –
- **Ist-Abhängigkeiten (Cargo.toml):** keine
- **Externe Abhängigkeiten:** thiserror
- **Dev-Abhängigkeiten (zählen nicht für G1):** proptest
- **Referenz im Quelltext:** Level L0 · Budget 2500 LOC · Spezifikation docs/SPEC.md Kap. 2.2, 2.4, 3.5, 3.7, 4.6, 5.1.

## Status
- **Stand:** implementiert (Teil: Ids, Klassen, Fehler, Ports, Festkomma)
- **Produktionscode (ohne Tests/Leerzeilen/Kommentare):** 214 LOC
- **Archiv-Quelle:** `archive/acta-base/` (siehe `archive/acta-base/MANIFEST.md`)

## Regeln für diesen Crate
- Abhängigkeiten nur nach unten: `level(A) > level(B)`; keine Querkanten (G1).
- Keine Alias-Re-Exports, kein `include!`, kein `#[deprecated]` (HR-6).
- Ein Fehlertyp je Crate, `thiserror`, `#[non_exhaustive]`, kein `Other(String)` (HR-4).
- Workspace-Lints gelten (`[lints] workspace = true`): kein `unwrap`, `expect`, `panic`, Indexierung mit `[]` auf Produktionspfaden.
- `unsafe` ist hier verboten (G-UNSAFE, nur `acta-sys` darf es).
- Keine Zeit- oder Zufallsquellen außerhalb der injizierten Ports `Clock` und `CryptoRng` (G3).
- Verbotene Formulierungen aus SPEC 1.6 (Liste A) nicht verwenden (G-WORD).
- Vor Abschluss: `cargo xtask gates`, `cargo test -p acta-base`, `cargo clippy -p acta-base --all-targets -- -D warnings`, `cargo fmt --all -- --check`.

## Vorgehen bei Überführung aus dem Archiv
1. Archivdateien lesen, nicht blind kopieren. Namen, Importe und Verweise auf frühere Spezifikationsstände sind auf Acta umzuschreiben (siehe `archive/INDEX.md`).
2. Jede Funktion gegen den Lebenszyklus in SPEC Kap. 19 und die Zusagen in 1.5 und 16.3 prüfen. Funktionen ohne Eintrag dort werden nicht implementiert (SPEC 0.4 Regel 3).
3. Tests mit Positivkontrolle mitführen (SPEC 16.4).
4. Nach vollständiger, grün geprüfter Überführung wird der Archiv-Ordner dieses Crates entfernt.

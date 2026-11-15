# acta-pstore

acta-pstore: P-Speicher: Dokumentkopf, Versionen, Betroffenenverzeichnis, Handle-Register, Vorgaenge.

> Teil des Acta-Workspace. Verbindliche Vorgaben: Wurzel-`CLAUDE.md` und `docs/SPEC.md`.

## Einordnung
- **Level:** 3 (`rings.toml`)
- **Größenbudget:** 5000 LOC Produktionscode (G-HUB); Überschreitung nur mit ADR
- **Aufgabe laut SPEC 3.3:** P-Speicher: Dokumentkopf, Versionen, Betroffenenverzeichnis, Handle-Register, Vorgänge, Fallindex (DP), Auflösung von Schlüsseln und DEK
- **Erlaubte interne Abhängigkeiten (SPEC 3.3):** base, crypto, fs, gate, ledger, vault, seg
- **Ist-Abhängigkeiten (Cargo.toml):** acta-base, acta-crypto, acta-fs, acta-gate, acta-ledger, acta-seg, acta-vault
- **Externe Abhängigkeiten:** roaring, thiserror
- **Dev-Abhängigkeiten (zählen nicht für G1):** acta-testkit, proptest
- **Referenz im Quelltext:** Level L3 · Budget 5000 LOC · Spezifikation docs/SPEC.md Kap. 5.4

## Status
- **Stand:** Gerüst (nur Fehlertyp)
- **Produktionscode (ohne Tests/Leerzeilen/Kommentare):** 17 LOC
- **Archiv-Quelle:** keine (Gerüst oder bereits überführt)

## Regeln für diesen Crate
- Abhängigkeiten nur nach unten: `level(A) > level(B)`; keine Querkanten (G1).
- Keine Alias-Re-Exports, kein `include!`, kein `#[deprecated]` (HR-6).
- Ein Fehlertyp je Crate, `thiserror`, `#[non_exhaustive]`, kein `Other(String)` (HR-4).
- Workspace-Lints gelten (`[lints] workspace = true`): kein `unwrap`, `expect`, `panic`, Indexierung mit `[]` auf Produktionspfaden.
- `unsafe` ist hier verboten (G-UNSAFE, nur `acta-sys` darf es).
- Keine Zeit- oder Zufallsquellen außerhalb der injizierten Ports `Clock` und `CryptoRng` (G3).
- Verbotene Formulierungen aus SPEC 1.6 (Liste A) nicht verwenden (G-WORD).
- Vor Abschluss: `cargo xtask gates`, `cargo test -p acta-pstore`, `cargo clippy -p acta-pstore --all-targets -- -D warnings`, `cargo fmt --all -- --check`.

## Vorgehen bei Überführung aus dem Archiv
1. Archivdateien lesen, nicht blind kopieren. Namen, Importe und Verweise auf frühere Spezifikationsstände sind auf Acta umzuschreiben (siehe `archive/INDEX.md`).
2. Jede Funktion gegen den Lebenszyklus in SPEC Kap. 19 und die Zusagen in 1.5 und 16.3 prüfen. Funktionen ohne Eintrag dort werden nicht implementiert (SPEC 0.4 Regel 3).
3. Tests mit Positivkontrolle mitführen (SPEC 16.4).
4. Nach vollständiger, grün geprüfter Überführung wird der Archiv-Ordner dieses Crates entfernt.

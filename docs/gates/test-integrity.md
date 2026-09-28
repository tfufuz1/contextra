# Test Integrity Gate (`test-integrity`)

## Zweck
Prüft mit `syn` modifizierte und gelöschte Rust-Testdateien auf Testintegrität und Manipulationen.

## Funde
### Harte Funde (`fail`, Exit 1)
- Testfunktionen (`#[test]`, `#[tokio::test]`) ohne Assertion oder Result-Rückgabe.
- `assert!(true)` Tautologien.
- Gelöschte Testdateien oder gelöschte `#[test]`-Funktionen ohne gleichnamigen Ersatz.
- Neu hinzugefügtes `#[ignore]`.
- Modifizierte `*.snap` Snapshot-Dateien ohne Änderung an Nicht-Test-Quellcode.
- Verwendung von `INSTA_UPDATE`, `cargo insta accept` oder `--bless` in Diff-Skripten.

### Weiche Funde (`warn`, mit `--strict` `fail`)
- Geänderte Erwartungswerte in `assert_eq!` ohne Änderung an Nicht-Test-Quellcode im selben Diff.

# Harness Infrastructure Foundation

## Konvention
Das xtask Infrastructure Harness erlaubt das Registrieren neuer xtask-Kommandos über isolierte Dateien in `xtask/src/harness/<stem>.rs`.

- **Dateiname & Stem**: `<stem>.rs`, Dateiname muss `[a-z][a-z0-9_]*` entsprechen.
- **Kommandoname**: Stem mit `-` statt `_` (z. B. `registry_check.rs` -> `registry-check`).

## Modulvertrag
1. **Erste Zeile**: `//! <ein Satz Zusammenfassung>`.
2. **Einstieg**: `pub fn run_<stem>(args: &[String]) -> i32`.
3. **Exit-Codes**:
   - `0`: pass / not_applicable
   - `1`: Verstoß (fail)
   - `2`: Fehler / nicht entscheidbar (fail-closed)
4. **Standard-Flags**: `--root`, `--base`, `--head`, `--json`.
5. **Symbolnamen**: Alle Top-Level-Items müssen modul-eindeutige Präfixe tragen.

## Builtin-Kommandos
- `cargo xtask harness-list [--json]`
- `cargo xtask harness-help <cmd>`

## Build-Sicherheit & Fehlerbilder
`xtask/build.rs` prüft vor der Generierung von `OUT_DIR/harness_generated.rs`:
- Ungültiger Stem-Name -> Abbruch mit `BUILD ERROR`.
- Fehlende oder leere `//!`-Zeile -> Abbruch mit `BUILD ERROR`.
- Fehlende `pub fn run_<stem>` -> Abbruch mit `BUILD ERROR`.
- Kollision mit bestehendem Arm in `main.rs` -> Abbruch mit `BUILD ERROR`.

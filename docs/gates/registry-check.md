# Registry Check Gate (`registry-check`)

## Zweck
Das Gate `cargo xtask registry-check` stellt sicher, dass alle im Repository verwendeten xtask-Kommandos vollständig und konsistent in `xtask/registry.toml` deklariert sind.

## Abgedeckte Quellen
Das Gate prüft folgende Aufrufstellen:
1. Match-Arme in `xtask/src/main.rs`.
2. `cargo xtask`-Aufrufe in `justfile`.
3. Workflows unter `.github/workflows/`.
4. Harness-Module unter `xtask/src/harness/*.rs` (gelten automatisch als registriert).

## Fehlerbilder & Fixes
- `MISSING_REGISTRY_ENTRY`: Ein Aufruf existiert in `main.rs`, `justfile` oder Workflows, ist jedoch nicht in `xtask/registry.toml` eingetragen.
  - **FIX**: Füge den fehlenden Eintrag mit Pflichtfeldern (`name`, `owner_tier`, `summary`, `blocking`, `phase`) in `xtask/registry.toml` ein.
- `ORPHANED_REGISTRY_ENTRY`: Ein Legacy-Eintrag existiert in `xtask/registry.toml`, wird jedoch an keiner Stelle aufgerufen.
  - **FIX**: Entferne den verwaisten Eintrag aus `xtask/registry.toml`.
- `HARNESS_LEGACY_COLLISION`: Ein Harness-Modul nutzt den gleichen Namen wie ein bestehender Arm in `main.rs`.
  - **FIX**: Benenne das Harness-Modul um oder migriere/entferne den Arm in `main.rs`.

# Baseline: contextra-agent

- **Datum:** 2026-11-10
- **Crate:** `contextra-agent`
- **Umfang:** 4.374 LOC in 22 Dateien (laut `tools/decouple_inspector.py`)
- **Modul-Inventar:** 7 Traits, 34 Structs, 7 Enums
- **Golden-Korpus:** `agent_errors.json` verifiziert via `sha256sum -c MANIFEST.sha256` -> OK
- **Kompilier-Status:**
  - `cargo check -p contextra-types -p contextra-ports`: GRÜN
  - `cargo tree -p contextra-agent -e normal --no-default-features`: GRÜN
  - Entkopplungsziel: Kern-Closure auf `allowed_internal = ["contextra-types", "contextra-ports"]` reduziert, `contextra-graph` in dev-dependencies isoliert, alle Speicher-/Engine-/Router-Bindungen hinter Feature `storage-bindings` und `workspace` modularisiert.

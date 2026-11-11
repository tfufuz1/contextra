# FINDINGS: contextra-ingest

Stand: 2026-11-11 · Auftrag INGEST-B

## 1. Randnotizen & Nicht-blockierende Befunde

1. **`.gitignore` Sperre für `**/fuzz/corpus/`**:
   - In der Root-`.gitignore` (Zeile 44) ist `**/fuzz/corpus/` global ignoriert.
   - Die Seed-Dateien unter `fuzz/corpus/` sind für lokale Fuzzing-Runs hinterlegt. Im Git-Index wird die Integrität durch den Integrationstest `tests/fuzz_regression_tests.rs` und den referenzierten Korpus in `tests/corpus/` gesichert.
2. **`export_crate.py` Manifest-Ersetzung bei optionalen Workspace-Deps**:
   - `tools/export_crate.py` entfernt Zeilen mit `contextra-*` Abhängigkeiten, belässt aber Features wie `workspace = ["dep:contextra-types"]`. Konsumenten von `export_crate.py` sollten dies beachten, wenn Crates ohne `types-compat` exportiert werden.
3. **Format-Erweiterungen**:
   - Die Formate `Csv` und `Tsv` benötigen das Feature `fmt-sheets` für die Registrierung des TabularExtractors. Im Minimal-Build (`--no-default-features`) reagiert die Pipeline mit einem sauberen `UnsupportedFormat`-Fehler (Fail-Closed).

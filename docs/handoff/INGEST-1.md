# Handoff `INGEST-1` · Ingest-Sicherheitskern & Autarkie

## 1. Status-Übersicht
- **Crate:** `contextra-ingest` (Klasse A Baustein)
- **Status:** Vollständig autark, 0 interne Abhängigkeiten zu `contextra-*`
- **Sicherheitskern:**
  - `INV-INGEST-BOUNDED-1`: `detect` und `detect_reader` lesen strikt maximal `MAX_DETECT_BYTES` (64 KiB) ein.
  - `INV-INGEST-NO-ARCHIVE-BOMB-1`: Zip-Bomb-, Zip-Slip-, Pfadlängen-, Dekompressions- und XML-Billion-Laughs/XXE-Schutz gehärtet und verifiziert.
- **Lizenzen:** `LICENSE-MIT` und `LICENSE-APACHE` hinzugefügt (B12).
- **Zero-Panic & Unsafe:** `#![forbid(unsafe_code)]` strikt eingehalten, `zero_panic_analyzer` meldet 0 Prod-Verstöße.

## 2. Invarianten & Tests
- `INV-INGEST-BOUNDED-1`: `tests/inv_ingest_bounded_tests.rs` (Stream-Limits, Proptests)
- `INV-INGEST-NO-ARCHIVE-BOMB-1`: `tests/inv_ingest_archive_bomb_tests.rs` (Zip-Slip, Zip-Bomb, DTD-Angriffe)
- Fuzzing: `fuzz_safe_archive.rs` und `fuzz_safe_xml.rs` voll ausprogrammiert.

## 3. Externe Feststellungen & Workspace CI Blocker in Read-Only Scopes
- **Stand-alone Export:** `contextra-ingest` exportiert und baut absolut standalone (Gate G3) ohne jegliche Fehler.
- **Workspace-Manifest Fehler außerhalb OWNS-Scope:**
  1. `crates/contextra-infer-ollama/Cargo.toml:22:1`: `duplicate key 'features' in document root`
  2. `crates/contextra-vector/Cargo.toml`: `feature 'ports-compat' includes 'dep:contextra-ports', but 'contextra-ports' is not an optional dependency`
  3. `xtask/Cargo.toml`: Mehrere CI-Workflows erwarten `xtask/Cargo.toml`, welches nicht im Workspace existiert.
  4. `.github/unwrap_baseline.txt`: Workspace-weiter Baseline-Wert liegt bei 1197 während im aktuellen Tree 1310 vorkommen (im Crate `contextra-ingest` existieren 0 unwrap/expect im Produktionscode).
- Es mussten keine Adapter nach `adapter_src/` übergeben werden, da `contextra-ingest` bereits vollkommen autark entkoppelt war.

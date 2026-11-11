# Baseline-Erfassung: STORE-MOD-H

Datum: 2026-11-11
Commit: HEAD (origin/main)
Arbeitsbereich: `compat/`, verwaiste Duplikate in `scanner/` und `engine/`, `examples/`

## 1. Cargo Manifest & Workspace Blocker Analyse

Befehl: `cargo check -p contextra-store --locked`
Ergebnis:
```text
error: failed to load manifest for workspace member `/app/crates/contextra-store`
referenced by workspace at `/app/Cargo.toml`

Caused by:
  failed to parse manifest at `/app/crates/contextra-store/Cargo.toml`

Caused by:
  feature `workspace` includes `dep:contextra-types`, but `contextra-types` is not an optional dependency
  A non-optional dependency of the same name is defined; consider adding `optional = true` to its definition.
```

Befund:
In `crates/contextra-store/Cargo.toml` ist `workspace = ["dep:contextra-types", "dep:contextra-ports"]` deklariert, während unter `[dependencies]` die Einträge `contextra-types = { workspace = true }` und `contextra-ports = { workspace = true }` ohne `optional = true` stehen.
Gemäß den strikten BESITZ-Regeln gehört `Cargo.toml` Agent G und ist für Agent H strikt read-only.
Der Blocker und der notwendige Patch werden in `REPORT.md` unter Übergaben dokumentiert.

## 2. G-L2 Status vor Umbau (Kern frei von Brückentypen)

Befehl:
```bash
grep -rEn 'contextra_(types|ports|testkit)' crates/contextra-store/src --include=*.rs | grep -v '/compat/'
```

Ergebnis vor Umbau:
- `crates/contextra-store/src/scanner/sstable.rs` (Zeilen 17, 18, 83, 188)
- `crates/contextra-store/src/scanner/runtime.rs` (Zeile 5)
- `crates/contextra-store/src/scanner/memtable.rs` (Zeilen 22, 23)
- `crates/contextra-store/src/scanner/wal.rs` (Zeilen 17, 18, 96, 169)
- `crates/contextra-store/src/scanner/store_scan_view.rs` (Zeile 29)
- `crates/contextra-store/src/scanner/tests.rs` (Zeilen 15, 16, 17, 18, 39, 40, 43, 44, 96, 149, 150, 210, 307, 361)
- `crates/contextra-store/src/engine/memory_only.rs` (Zeilen 11, 12)

Alle 7 Dateien sind unreferenzierte verwaiste Duplikate der kanonischen Dateien unter `crates/contextra-store/src/compat/`. Nach ihrer Archivierung in WP-H.2 ist G-L2 im Eigentum von Agent H vollständig frei von Brückentypen.

## 3. Unerreichbarkeitsbeweis der verwaisten Duplikate (WP-H.1)

Prüfung auf Modul-Deklarationen:
`grep -rEn '(^|\s)mod (memtable|runtime|sstable|store_scan_view|tests|wal|purge|memory_only)\b' crates/contextra-store/src/scanner crates/contextra-store/src/engine crates/contextra-store/src/engine.rs`
Ergebnis: Keine `mod`-Deklarationen für diese Dateien vorhanden.

Inhaltliche Unterschiede zu den kanonischen Kopien unter `src/compat/`:
- `runtime.rs`, `sstable.rs`, `tests.rs`, `wal.rs`: Byte-identisch zu `src/compat/scanner/*.rs`.
- `purge.rs`: Veraltete Version (in `src/compat/scanner/purge.rs` liegt die aktuelle Version).
- `memtable.rs`: Feldbezeichnung `_runtime` vs `#[allow(dead_code)] runtime` in `compat`.
- `store_scan_view.rs`: Import `contextra_ports::Vfs` vs `crate::io_seam::Vfs` in `compat`.
- `memory_only.rs`: Doc-String ("designed to" vs "guaranteed to").

Fazit: Alle Dateien unter `src/scanner/{memtable,runtime,sstable,store_scan_view,tests,wal,purge}.rs` und `src/engine/memory_only.rs` sind nachweislich verwaist und nicht im Modulgraphen eingebunden.

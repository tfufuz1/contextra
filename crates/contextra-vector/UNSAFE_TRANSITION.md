# UNSAFE Transition tracking for `contextra-vector`

> **Status 30.10.2026 (Reduktion):** Dieses Crate gehört zum Kern v1. Ring und Abhängigkeiten stimmen mit `docs/SPEC.md` §4 überein. Verweise auf entfernte Crates oder Funktionen sind veraltet. Enthält Code für DiskANN (entfernt) und RaBitQ (Nicht-Ziel, offen: D-R4). Maßgeblich ist der Quelltext. Bereinigung: `docs/PLAN.md` Phase R0 bis R3.

| Datei:Zeile | Grund | Ziel-Crate für Migration | Tracking-ID | Status |
| --- | --- | --- | --- | --- |
| `src/persistence.rs:451` | `memmap2::Mmap::map(&file)` Zero-Copy Read-Only Memory-Mapping | `contextra-sys` (`contextra_sys::mmap_readonly`) | `TRANS-VEC-001` | MIGRATED |
| `src/diskann.rs:1589` | `memmap2::Mmap::map(&file)` Zero-Copy Read-Only Memory-Mapping | `contextra-sys` (`contextra_sys::mmap_readonly`) | `TRANS-VEC-002` | MIGRATED |

## Summary
All historical `unsafe` blocks in `crates/contextra-vector` have been migrated to safe facades in `contextra-sys` (specifically `contextra_sys::mmap_readonly`).
`crates/contextra-vector` now strictly enforces `#![forbid(unsafe_code)]` at the crate root (`src/lib.rs`) without any conflicting `#![allow(unsafe_code)]` directives.

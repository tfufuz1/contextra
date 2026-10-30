# UNSAFE Transition tracking for `contextra-store`

> **Status 30.10.2026 (Reduktion):** Dieses Crate gehört zum Kern v1. Ring und Abhängigkeiten stimmen mit `docs/SPEC.md` §4 überein. Verweise auf entfernte Crates oder Funktionen sind veraltet. Verweise auf `contextra-core` (Trait-Herkunft) und auf die entfernte Scanner-Logik (SPEC §16, B2). Maßgeblich ist der Quelltext. Bereinigung: `docs/PLAN.md` Phase R0 bis R3.

| Datei:Zeile | Grund | Ziel-Crate für Migration | Tracking-ID | Status |
| --- | --- | --- | --- | --- |
| `src/wal/replay.rs:118` | `memmap2::Mmap::map(&file)` Zero-Copy Read-Only Memory-Mapping in WAL replay | `contextra-sys` (`contextra_sys::mmap_readonly`) | `TRANS-STO-001` | MIGRATED |
| `src/wal/replay.rs:159` | `memmap2::Mmap::map(&file)` Zero-Copy Read-Only Memory-Mapping in WAL scan | `contextra-sys` (`contextra_sys::mmap_readonly`) | `TRANS-STO-002` | MIGRATED |
| `src/wal/io.rs:838` | Platform restrictive file ACL permission enforcement | `contextra-sys` (`contextra_sys::set_restrictive_file_acl`) | `TRANS-STO-003` | MIGRATED |

## Summary
Audit verification of `crates/contextra-store` confirmed that all low-level memory mapping and OS ACL operations in `crates/contextra-store` have been successfully migrated to safe facades in `contextra-sys` (`contextra_sys::mmap_readonly` and `contextra_sys::set_restrictive_file_acl`).
`crates/contextra-store` strictly enforces `#![forbid(unsafe_code)]` at the crate root (`src/lib.rs`) with zero `unsafe` blocks in production code.

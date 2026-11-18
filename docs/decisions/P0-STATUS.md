# Store-Refactoring: Status

| Phase | Stand |
|---|---|
| P0 Baseline | **blockiert**: keine Rust-Toolchain 1.89 in der Umgebung, Egress gesperrt. Nicht verifiziert. |
| P0b Purge-Hotfix | **statisch geliefert, nicht kompiliert**: `compat/scanner/purge.rs`, `lsm/engine.rs` |
| P1 Cargo-Hygiene | nicht begonnen (E2 offen: REFACTORING.md vs. Auftrag) |
| P2-P7 | nicht begonnen (ohne Compiler und Crash-Tests nicht verantwortbar) |

## P0b Aenderungen (B6, P-a bis P-c)
- Keine verschluckten Fehler mehr (`unwrap_or_default`, `let _ =`): Fehler brechen Purge ab, vor dem RAM-Tausch, Zustand bleibt unveraendert.
- Neuschreibung nutzt `storage.key_manager()` statt `None` (T-DEL-1).
- Nullung in 1-MiB-Chunks + `sync_all` + unlink; Dir-Fsync nach SST- und WAL-Bereinigung.
- Unveraendert (P6): kein Manifest-Edit, `block_on`, `builder.add(.., 1)`, Vollscan, WAL-Rewrite-Struktur.

## Pflicht vor Merge
`cargo test -p contextra-store`, mit `--features test-hooks,fault-injection,legacy-wal-key`, `cargo clippy -p contextra-store -- -D warnings`.
Zu pruefen: Typ der Fehlerkonvertierung in `purge.rs` (`ContextraError::Io(e)` fuer `std::io::Error`, wie im Altcode), `vfs.sync_dir`-Rueckgabetyp (`StoreError`, mit `{e}` formatiert).

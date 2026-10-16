# Audit-Bericht: `std::fs` → `tokio::fs` Migrationsprüfung in `contextra-store`

**Datum:** 2026-09-25
**Invariante (`crates/contextra-store/src/lib.rs:15`):**
> "tokio::fs für Metadaten/Lifecycle, std::fs::File ausschließlich innerhalb spawn_blocking für Block-Level Random-Access."

---

## Übersicht der Fundstellen & Klassifikationen

| Datei | Zeile | Code-Auszug | Klassifikation | Begründung & Kontext |
|---|---|---|---|---|
| `crates/contextra-store/src/wal/mod.rs` | 27 | `std::fs::read(path)` | (a) Konform | Befindet sich in `#[cfg(loom)] pub(crate) mod fs` innerhalb der Loom-Simulation (Single-Threaded deterministic harness). Keinerlei Aufruf im echten Production Async-Hotpath (`#[cfg(not(loom))]` re-exportiert `tokio::fs::*`). |
| `crates/contextra-store/src/wal/mod.rs` | 33 | `std::fs::write(path, contents)` | (a) Konform | Befindet sich ebenfalls in `#[cfg(loom)] pub(crate) mod fs` für die Loom-Testsimulation. |
| `crates/contextra-store/src/wal/mod.rs` | 57 | `perm: std::fs::Permissions` | (a) Konform | Typ-Signatur der Hilfsfunktion `set_permissions` innerhalb des Loom-Mocks. |
| `crates/contextra-store/src/wal/mod.rs` | 73 | `pub fn permissions(&self) -> std::fs::Permissions` | (a) Konform | Rückgabetyp-Signatur der Methode `LoomMetadata::permissions` im Loom-Mock. |
| `crates/contextra-store/src/wal/mod.rs` | 77 | `std::fs::Permissions::from_mode(0o644)` | (a) Konform | In `LoomMetadata::permissions` (Unix-Zweig) zur Erstellung von Mock-Berechtigungen im Loom-Testmodus. |
| `crates/contextra-store/src/wal/mod.rs` | 81 | `std::fs::metadata(".").map(...).unwrap()` | (a) Konform | Non-Unix Fallback in `LoomMetadata::permissions` im Loom-Testmodus. |
| `crates/contextra-store/src/lsm/recovery.rs` | 533 | `std::fs::File::open(&parent)` | (a) Konform | Wird in `rollback_to_tx_locked` explizit innerhalb eines `tokio::task::spawn_blocking(move || { ... })` Blocks ausgeführt. Dient zum Directory FSync des Parent-Ordners nach Schreiben eines Intent-Files bei Crash-Recovery / Rollback (kein Async Hot-Path). |

---

## Fazit & Ergebnis
Sämtliche überprüften `std::fs`-Stellen in `crates/contextra-store/src/wal/mod.rs` und `crates/contextra-store/src/lsm/recovery.rs` entsprechen vollumfänglich der definierten Speicher-Engine-Invariante. Es existiert keine synchrone `std::fs`-Blockierung im async Hot-Path. Alle Fundstellen wurden mit entsprechenden `// INVARIANT-KONFORM: ...` Inline-Kommentaren im Code dokumentiert.

# contextra-durable-fs

Crash-sichere Dateisystem-Operationen, synchrone Disk-Flushes und synchrones VFS.

## Zweck
`contextra-durable-fs` ist ein vollständig autarker Baustein für crash-konsistente Dateisystem-Operationen (`vfs`), deterministisches Append-only Logging mit Prüfsummen (`journal`) und unumkehrbares Handle-Poisoning bei `fsync`-Fehlern (`DurableHandle`).

## Reifegrad
- **Status:** Vorbereitung zur R3-Heraufstufung (Klasse A)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]` – 100 % sicheres Rust ohne `unsafe`.
- **Zero-Panic-Doktrin:** Typisierte Fehlercodes via `DurableFsError`.
- **Durability-Invariante:** Strikter Fail-Closed-Zustandsautomat (`AtomicHealthState`) und unumkehrbares Handle-Poisoning (`INV-FSYNC-1`).

## Interne Abhängigkeiten
`contextra-durable-fs` ist ein vollständig eigenständiger Baustein (Ring 0, Ebene 0):
- **0 interne Workspace-Abhängigkeiten** (keine Abhängigkeit zu `contextra-store`, `contextra-types`, `contextra-ports` oder anderen Crates).
- Externe Abhängigkeiten: `thiserror`, optional: `crc32fast`, `serde`, `tracing`.

## Features
- `default = ["std", "crc"]`
- `std`: Standardbibliotheksunterstützung.
- `crc`: CRC32-Prüfsummenvalidierung für das Append-only Journal.
- `serde`: Serde-Serialisierung.
- `tracing`: Tracing-Integration.
- `inspect`: Diagnose- und Inspektionswerkzeuge.

## Minimalbeispiel
```rust
use contextra_durable_fs::vfs::{StdVfs, Vfs};
use tempfile::tempdir;

let dir = tempdir().unwrap_or_else(|e| panic!("tempdir: {e}"));
let test_file = dir.path().join("durable_test.txt");

let vfs = StdVfs;
vfs.write_atomic(&test_file, b"crash-consistent-data").unwrap_or_else(|e| panic!("write: {e}"));

let read_back = vfs.read(&test_file).unwrap_or_else(|e| panic!("read: {e}"));
assert_eq!(read_back, b"crash-consistent-data");
```

## Entnahme-Rezept (Einzeln entnehmbar)
`contextra-durable-fs` ist vollständig entkoppelt und kann ohne Vorbedingungen einzeln in jedes beliebige Rust-Projekt entnommen werden:
1. Kopiere den Ordner `crates/contextra-durable-fs` in dein Projektverzeichnis.
2. Füge ihn in deiner `Cargo.toml` hinzu:
```toml
[dependencies]
contextra-durable-fs = { path = "../contextra-durable-fs" }
```
3. Es sind keine weiteren internen Crates oder Laufzeitkomponenten erforderlich.

## Bekannte Grenzen
- Atomare Schreiboperationen (`write_atomic`) nutzen temporäre Dateien und `rename`; Quell- und Zielpfad müssen auf demselben physischen Dateisystem liegen.
- Synchrones fsync blockiert den aufrufenden Thread bis zum Abschluss durch das Betriebssystem.

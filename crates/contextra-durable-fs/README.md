# contextra-durable-fs

Crash-sichere Dateisystem-Operationen, synchrone Disk-Flushes und synchrones VFS.

Normativitätsklasse: **Klasse A (Autarker Baustein)** (Ring 0; B1–B12, `capabilities.fragment.toml`).

## Rolle & Zweck
`contextra-durable-fs` ist ein vollständig autarker Baustein ohne interne Workspace-Abhängigkeiten. Es kapselt plattformspezifische I/O-Aufrufe, garantiert crash-konsistente Schreiboperationen (`INV-VFS-ATOMIC-1`), deterministisches Append-only Logging mit Prüfsummen und striktes Fail-Closed-Verhalten bei `fsync`-Fehlern (`INV-FSYNC-1`).

## Baukasten-Einordnung & Status (M1–M11)
- **Ring:** Ring 0 (Fundament)
- **Status:** 🟢 Klasse A Baustein (Modularitäts-Score 100/100)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`, zero-panic doctrine (`deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)`).
- **Abhängigkeiten (Produktion):** 0 interne Workspace-Abhängigkeiten. Reines safe Rust auf Basis von `std`.
- **Features:** `default = ["std", "crc"]`, optional: `serde`, `tracing`, `inspect`.

## Kernkomponenten & Garantien
- **`StdVfs` / `RealFs` (`vfs.rs`):** Synchroner `Vfs`-Port für Dateisystemoperationen mit striktem `INV-VFS-ATOMIC-1`.
- **`AppendOnlyJournal` (`journal.rs`):** Append-only Log mit CRC32-Prüfsummen, monotoner Sequenznummer und deterministischer Recovery (Erkennung von Torn-Writes und Bit-Rot).
- **`FaultInjectingVfs` (`fault_injector.rs`):** VFS-Dekorator für Chaos-Engineering, Disk-Full-Injektion und Crash-Punkt-Simulation.
- **`DurableHandle` (`lib.rs`):** Dateihandle-Wrapper mit unumkehrbarem fsync-Poisoning (`INV-FSYNC-1`).
- **`AtomicHealthState` (`health.rs`):** Lock-freier Fail-Closed-Zustandsautomat (`Healthy` → `Degraded` → `ReadOnly` → `Poisoned`).

## Minimalbeispiel

```rust
use contextra_durable_fs::vfs::{StdVfs, Vfs};
use contextra_durable_fs::{atomic_replace, DirLock, LockMode};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = Path::new("/tmp/my_storage");
    std::fs::create_dir_all(dir)?;

    // 1. Verzeichnissperre erwerben
    let _lock = DirLock::acquire(dir, LockMode::Normal)?;

    // 2. Atomares Ersetzen mit fsync und Directory-Sync
    let target = dir.join("data.bin");
    atomic_replace(&target, b"durable-content")?;

    // 3. Zugriff über den synchronen Vfs-Port
    let vfs = StdVfs;
    let data = vfs.read(&target)?;
    assert_eq!(data, b"durable-content");

    Ok(())
}
```

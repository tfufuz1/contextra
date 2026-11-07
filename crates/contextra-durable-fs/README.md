# contextra-durable-fs

Crash-sichere Dateisystem-Operationen, synchrone Disk-Flushes und POSIX-VFS-Implementierung.

Normativitätsklasse: **Kern `[K]`** (Ring 0, `beta`; §4.1, `capabilities.toml`).

## Rolle & Zweck
`contextra-durable-fs` stellt die kanonische Produktionsimplementierung des virtuellen Dateisystem-Ports (`contextra_ports::Vfs` und `contextra_ports::VfsFile` via `RealFs` und `RealFsFile`) bereit. Es kapselt plattformspezifische I/O-Aufrufe, garantiert crash-konsistente Schreiboperationen und Fail-Closed-Verhalten bei `fsync`-Fehlern (`INV-FSYNC-1`).

## Baukasten-Einordnung & Status (M1–M11)
- **Ring:** Ring 0 (Fundament)
- **Status:** 🟢 Autarker Baustein
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`, zero-panic doctrine (`deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)`).
- **Abhängigkeiten (Produktion):**
  - `contextra-ports`: Abstrakte Port-Definitionen (`Vfs`, `VfsFile`, `VfsOpenOptions`, `VerifiedVia`)
  - `contextra-sys`: Low-Level Plattform-Durability-Primitive (`durable_flush_file`, `durable_flush_dir`)
  - `contextra-types`: Kanonische Fehlertypen (`ContextraError`, `Result`)
- **Features:** Minimal, keine schweren Abhängigkeiten.

## Kernkomponenten & Garantien
- **`RealFs` (`vfs.rs`):** Implementierung des einheitlichen `contextra_ports::Vfs`-Ports für POSIX/Windows-Dateisysteme. Unterstützt positionsbasiertes I/O (`read_at`, `write_at`), atomare Ersetzungen (`write_atomic`), Metadaten-Synchronisation (`sync_dir`) und Verifikation (`read_verified`).
- **`DurableHandle` (`lib.rs`):** Kapselt Dateihandles und vergiftet sich dauerhaft (`Poisoned`) bei jedem Hardware- oder `fsync`-Fehler (`INV-FSYNC-1`). Kein stilles Wiederholen auf demselben Deskriptor.
- **`DirLock` (`lib.rs`):** Exklusive Verzeichnissperre mit Prüfung der Dateisystemklasse (`FsClass`). Weist unsichere Netzwerk- (NFS/SMB), FUSE- und Cloud-Sync-Ordner (OneDrive, Dropbox, iCloud) ab.
- **`AtomicHealthState` (`health.rs`):** Lock-freier Fail-Closed-Zustandsautomat (`Healthy` → `Degraded` → `ReadOnly` → `Poisoned`) nach Spezifikation §7.8/§18.4.
- **`PendingDeleteJournal` (`journal.rs`):** Crash-sicheres Journal zur idempotenten Aufzeichnung und Abarbeitung ausstehender Dateientfernungen.
- **`FaultInjector` (`fault_injector.rs`):** Zero-Cost Monomorphisierungs-Trait für deterministische Chaos- und Fehlerinjektion in Persistenzpfaden.

## Einbettungsbeispiel

```rust
use contextra_durable_fs::{RealFs, Vfs, VfsOpenOptions, atomic_replace, DirLock, LockMode};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = Path::new("/tmp/my_storage");
    std::fs::create_dir_all(dir)?;

    // 1. Verzeichnissperre erwerben
    let _lock = DirLock::acquire(dir, LockMode::Normal)?;

    // 2. Atomares Ersetzen mit fsync und Directory-Sync
    let target = dir.join("data.bin");
    atomic_replace(&target, b"durable-content")?;

    // 3. Zugriff über den standardmäßigen Vfs-Port
    let vfs = RealFs;
    let data = vfs.read(&target)?;
    assert_eq!(data, b"durable-content");

    Ok(())
}
```

Ein vollständiges, autark lauffähiges Beispiel findet sich unter `examples/durable_ops.rs`:
```bash
cargo run --example durable_ops -p contextra-durable-fs
```

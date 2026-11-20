# contextra-durable-fs — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E0 (Autarkes Fundament / Ring 0)  
**Kern-Closure:** Leer (0 interne Workspace-Abhängigkeiten)  
**Status:** Normativ (Soll-Zustand für Entwickler)  
**Version:** 2.0  
**Geltungsbereich:** `crates/contextra-durable-fs`  
**Referenzdokumente:** `docs/SPEC.md` §2, §5.3, §13 (INV-1, INV-6); `ANALYSIS-E0-CONTRADICTIONS.md` (D-04); `ALTLASTEN-AUDIT.md`  

---

## 1. Zweck & Architekturgarantien

`contextra-durable-fs` ist das fundamentale I/O- und Durabilitäts-Fundament für Write-Ahead-Logs (WAL), Manifeste, SSTable-Persistierung und das DSGVO-Erasure-Journal. Es garantiert atomare Dateitransaktionen, physisches Schreddern (Scrubbing) und Schutz vor stiller Datenkorruption durch irreversibles Handle-Poisoning.

### Kerninvariante 1: Handle-Poisoning (`INV-FSYNC-1`)
Schlägt ein `sync_all()`- oder `sync_data()`-Aufruf auf Betriebssystemebene fehl (z. B. `EIO`, `ENOSPC`), wechselt der Dateideskriptor irreversibel in den Zustand `Poisoned`. Alle nachfolgenden Schreib- und Synchronisationsaufrufe auf diesem Deskriptor werden sofort mit `Err(DurableFsError::HandlePoisoned)` abgewiesen.

### Kerninvariante 2: Atomare Dateitransaktion (`INV-VFS-ATOMIC-1`)
Dateimutationen (z. B. Manifest-Updates, SSTable-Writes) erfolgen strikt in 4 Schritten:
1. Temporäre Datei im **selben Elternverzeichnis** wie das Ziel anlegen (`.target.tmp.<pid>.<uuid>`).
2. Daten schreiben und Datei synchronisieren (`sync_file`).
3. Atomares Ersetzen des Ziels via `std::fs::rename`.
4. Durables Persistieren des Verzeichniseintrags via `sync_dir_path` (`contextra-sys`).

### Architekturgarantien (R1–R6, INV-3, INV-4)
1. **0 interne Workspace-Abhängigkeiten:** Vollständig autark.
2. **`#![forbid(unsafe_code)]`:** 100 % speichersicher.
3. **Zero-Panic-Policy:** Keine unkontrollierten Panics.
4. **Leere Default-Features (R3):** `default = []`.

---

## 2. Kanonische Komponenten & Schnittstellen

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            contextra-durable-fs                             │
│                                                                             │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │           Vfs            │             │          platform          │   │
│   │  (StdVfs, RealFs,        │             │ (sync_file, sync_dir_path, │   │
│   │   write_atomic, TmpGuard)│             │  FlushKind: Data/Metadata) │   │
│   └─────────────┬────────────┘             └─────────────┬──────────────┘   │
│                 │                                        │                  │
│                 ▼                                        ▼                  │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │     health & Poison      │             │           scrub            │   │
│   │ (AtomicHealthState,      │             │(scrub_file_with_rate_limit,│   │
│   │  HealthEvent, INV-FSYNC-1)             │ ScrubPolicy, ScrubReceipt) │   │
│   └──────────────────────────┘             └────────────────────────────┘   │
│                                                                             │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │         journal          │             │       fault_injector       │   │
│   │(AppendOnlyJournal,       │             │(FaultInjector, NoFault,    │   │
│   │ PendingDeleteJournal)    │             │ Crash-Punkt-Simulation)    │   │
│   └──────────────────────────┘             └────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### 2.1 Virtuelles Dateisystem (`Vfs` & `write_atomic`)

```rust
use std::path::{Path, PathBuf};

/// Abstraktion für Dateisystemoperationen mit Durabilitätsgarantien.
pub trait Vfs: Send + Sync {
    /// Führt ein atomares, crash-sicheres Schreiben gemäß INV-VFS-ATOMIC-1 aus.
    fn write_atomic(&self, path: &Path, content: &[u8]) -> Result<(), DurableFsError>;

    /// Erstellt Verzeichnisse rekursiv und synchronisiert das Elternverzeichnis.
    fn create_dir_all(&self, path: &Path) -> Result<(), DurableFsError>;

    /// Entfernt eine Datei sicher und verzeichnet den Intent bei Bedarf im PendingDeleteJournal.
    fn remove_file(&self, path: &Path) -> Result<(), DurableFsError>;

    /// Plant eine Datei zur verzögerten Entfernung, bis kein Snapshot-Pin sie mehr referenziert.
    fn pending_delete(&self, path: &Path, min_seq: u64) -> Result<PendingDelete, DurableFsError>;

    /// Überschreibt Dateiinhalt nur, wenn Exklusivität und Dateisystem dies sicher erlauben.
    /// Liefert ScrubSkipped statt Erfolg, wenn die Voraussetzung nicht erfüllt ist.
    fn scrub_if_safe(&self, path: &Path) -> Result<ScrubOutcome, DurableFsError>;

    /// Prüft die Existenz eines Pfades.
    fn exists(&self, path: &Path) -> bool;

    /// Benennt eine Datei atomar um und synchronisiert das Elternverzeichnis.
    fn rename_durably(&self, from: &Path, to: &Path) -> Result<(), DurableFsError>;
}

/// Eintrag des PendingDelete-Journals. Physische Entfernung wartet auf `min_seq`.
pub struct PendingDelete {
    pub path: PathBuf,
    pub min_seq: u64,
}

/// Ergebnis einer sicheren Scrub-Prüfung. `Skipped` ist kein Löschbeleg.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrubOutcome {
    Scrubbed { bytes: u64 },
    Skipped,
}

/// Standard-Implementierung auf echtem Betriebssystem-Dateisystem.
pub struct StdVfs;

impl Vfs for StdVfs {
    fn write_atomic(&self, path: &Path, content: &[u8]) -> Result<(), DurableFsError> {
        let parent = path.parent().ok_or_else(|| DurableFsError::InvalidPath(path.to_path_buf()))?;
        let tmp_file_name = format!(
            ".{}.tmp.{}.{}",
            path.file_name().and_then(|s| s.to_str()).unwrap_or("file"),
            std::process::id(),
            fastrand::u64(..)
        );
        let tmp_path = parent.join(tmp_file_name);

        // 1. Temporäre Datei schreiben
        {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&tmp_path)
                .map_err(DurableFsError::Io)?;

            file.write_all(content).map_err(DurableFsError::Io)?;
            file.sync_all().map_err(DurableFsError::Io)?;
        }

        // 2. Atomares Rename
        std::fs::rename(&tmp_path, path).map_err(|e| {
            let _ = std::fs::remove_file(&tmp_path);
            DurableFsError::Io(e)
        })?;

        // 3. Verzeichnis durabel flushen
        self.sync_directory(parent)?;

        Ok(())
    }

    fn rename_durably(&self, from: &Path, to: &Path) -> Result<(), DurableFsError> {
        std::fs::rename(from, to).map_err(DurableFsError::Io)?;
        if let Some(parent) = to.parent() {
            self.sync_directory(parent)?;
        }
        Ok(())
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), DurableFsError> {
        std::fs::create_dir_all(path).map_err(DurableFsError::Io)?;
        if let Some(parent) = path.parent() {
            let _ = self.sync_directory(parent);
        }
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<(), DurableFsError> {
        std::fs::remove_file(path).map_err(DurableFsError::Io)?;
        if let Some(parent) = path.parent() {
            self.sync_directory(parent)?;
        }
        Ok(())
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
}

impl StdVfs {
    fn sync_directory(&self, dir: &Path) -> Result<(), DurableFsError> {
        let f = std::fs::File::open(dir).map_err(DurableFsError::Io)?;
        f.sync_all().map_err(DurableFsError::Io)?;
        Ok(())
    }
}
```

---

### 2.2 Handle-Poisoning & Zustandsüberwachung (`health.rs`)

```rust
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthState {
    Healthy,
    Poisoned,
}

/// Atomarer Wächter zur Durchsetzung von INV-FSYNC-1.
pub struct AtomicHealthState {
    poisoned: AtomicBool,
}

impl AtomicHealthState {
    pub const fn new() -> Self {
        Self {
            poisoned: AtomicBool::new(false),
        }
    }

    #[inline]
    pub fn is_poisoned(&self) -> bool {
        self.poisoned.load(Ordering::SeqCst)
    }

    /// Markiert den Zustand irreversibel als vergiftet.
    pub fn poison(&self) {
        self.poisoned.store(true, Ordering::SeqCst);
    }

    /// Validiert den Zustand vor I/O-Aufrufen.
    pub fn check(&self) -> Result<(), DurableFsError> {
        if self.is_poisoned() {
            Err(DurableFsError::HandlePoisoned)
        } else {
            Ok(())
        }
    }
}
```

---

### 2.3 Physisches Schreddern (`scrub.rs`)

Überschreibt Dateien vor dem Löschen mit Nullen:

```rust
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub enum ScrubPolicy {
    Zeros,
    PseudoRandom(u64),
}

#[derive(Debug, Clone)]
pub struct ScrubReceipt {
    pub bytes_scrubbed: u64,
}

/// Überschreibt eine physische Datei sequentiell mit Nullen vor dem Unlink.
pub fn scrub_file(path: &Path, policy: ScrubPolicy) -> Result<ScrubReceipt, DurableFsError> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .map_err(DurableFsError::Io)?;

    let len = file.metadata().map_err(DurableFsError::Io)?.len();
    file.seek(SeekFrom::Start(0)).map_err(DurableFsError::Io)?;

    let chunk_size = 64 * 1024;
    let zeros = vec![0u8; chunk_size];
    let mut remaining = len;

    while remaining > 0 {
        let to_write = std::cmp::min(remaining, chunk_size as u64) as usize;
        file.write_all(&zeros[..to_write]).map_err(DurableFsError::Io)?;
        remaining -= to_write as u64;
    }

    file.sync_all().map_err(DurableFsError::Io)?;
    std::fs::remove_file(path).map_err(DurableFsError::Io)?;

    Ok(ScrubReceipt {
        bytes_scrubbed: len,
    })
}
```

---

### 2.4 Durabilität der neun Saga-Schritte (SPEC §5.3)

Der durabele Journal-Intent aus Schritt 1 wird vor Aktivierung der `LiveMask` synchronisiert. Zustandsübergänge werden jeweils vor dem zugehörigen destruktiven Schritt protokolliert. Ebene-A-Purge und Messung (Schritte 3–4) erfolgen vor Schlüsselvernichtung (Schritt 5); es gibt keinen alten `KeyRevoked`-Anker. `KeyTable`-Replace, Epoch Seal, Hygiene und Bericht verwenden Datei- und Verzeichnis-`fsync` gemäß ihrer jeweiligen atomaren Transaktion. Der Bericht und das Ledger werden erst nach Schritt 9 geschrieben.

Die `Vfs`-Schnittstelle stellt `PendingDelete` für verzögerte Dateientfernung sowie `scrub_if_safe` bereit. `scrub_if_safe` überschreibt nur exklusiv und sicher beschreibbare Dateien. Ist Scrubbing nicht sicher möglich, wird der Rest als solcher gemeldet; die Operation darf keinen erfolgreichen Scrub vortäuschen.

---

## 3. Fehlerbehandlung (`DurableFsError`)

```rust
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum DurableFsError {
    #[error("File descriptor is poisoned due to previous sync/flush failure (INV-FSYNC-1)")]
    HandlePoisoned,

    #[error("Atomic rename transaction failed: {0}")]
    RenameFailed(#[source] std::io::Error),

    #[error("Invalid path: {0:?}")]
    InvalidPath(std::path::PathBuf),

    #[error("Filesystem type unsupported for durable operations: {0}")]
    UnsupportedFilesystem(String),

    #[error("Underlying I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

---

## 4. `Cargo.toml` Spezifikation

```toml
[package]
name = "contextra-durable-fs"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Crash-safe filesystem operations, atomic transactions, and durability guarantees for Contextra"

[lints]
workspace = true

[dependencies]
thiserror = { workspace = true }
tracing = { workspace = true, optional = true }
crc32fast = { workspace = true, optional = true }
fastrand = "2.0"

[dev-dependencies]
tempfile = { workspace = true }

[features]
default = []
std = []
crc = ["dep:crc32fast"]
tracing = ["dep:tracing"]
```

---

## 5. Implementierungs-Arbeitspakete für Entwickler

1. **`Cargo.toml` Bereinigung:**
   - Default-Features leeren (`default = []`), `crc` und `std` als opt-in Features führen (R3).
   - `fastrand` für temporäre Dateiendungen einbinden.
2. **Handle-Poisoning Konsistenzprüfung:**
   - Sicherstellen, dass jeder I/O-Fehler in `platform.rs` sofort `health.poison()` triggert.
3. **Altlasten-Bereinigung:**
   - Keine Bezüge zu entfernten Crates.
4. **Verifikation:**
   - `cargo test -p contextra-durable-fs`

# contextra-checkpoint — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E1 (Domänenbaustein / Ring 1)  
**Kern-Closure:** `ids`, `env`, `durable-fs` (E0-Fundamente, max. 5)  
**Status:** Normativ (Soll-Zustand für Entwickler)  
**Version:** 2.0  
**Geltungsbereich:** `crates/contextra-checkpoint`  
**Referenzdokumente:** `docs/SPEC.md` §2, §4, §5; `ANALYSIS-E0-CONTRADICTIONS.md`; `ALTLASTEN-AUDIT.md` (AP-21)  

---

## 1. Zweck & Architekturgarantien

`contextra-checkpoint` stellt leichtgewichtige Point-in-Time Checkpoints und Verzweigungen (Branching/Forking) für Contextra bereit:
- **Zero-Copy Checkpoints:** Momentaufnahmen des Datenbestands via Dateisystem-Hardlinks auf unveränderliche SSTables (`.sst`).
- **Atomare Manifest-Kopie:** Konsistente Sicherung des LSM-Manifests zum Checkpoint-Zeitpunkt.
- **Krypto-Shredding Sicherheit (SPEC §4):** Checkpoints teilen physische Datenblöcke mit dem Primärspeicher. Die Zerstörung des Schlüssels im Tresor macht Daten in allen Checkpoints unverzüglich $O(1)$ unlesbar.
- **Waisen- & Fragment-Bereinigung:** Erkennung und Bereinigung verwaister temporärer Checkpoint-Verzeichnisse nach Systemabstürzen.

### Beseitigung von Ebenen- und Dev-Dependency-Verstößen (R1, R2)
1. **Keine E2-Abhängigkeiten:** Die optionalen Abhängigkeiten zu `contextra-types` und `contextra-ports` sowie die Features `workspace`, `types-compat` und `ports-compat` werden ersatzlos gestrichen.
2. **Keine E3-Dev-Abhängigkeiten (R2):** Die Dev-Abhängigkeit auf `contextra-testkit` (Ebene 3) in `Cargo.toml` ist ein Verstoß gegen R2 (*„Dev-Abhängigkeiten auf höhere Ebenen sind verboten"*) und wird entfernt.
3. **Altlasten-Bereinigung (`AP-21`):** In `src/lib.rs` Zeile 34 wird die Referenz auf das entfernte `contextra-agent` gelöscht.

### Architekturgarantien (R1–R6, INV-1, INV-3, INV-4)
1. **`#![forbid(unsafe_code)]`:** 100 % speichersicher.
2. **Zero-Panic-Policy:** Alle Operationen liefern `CheckpointError`.
3. **Leere Default-Features (R3):** `default = []`.
4. **Keine Klartext-Kopien:** Checkpoint-Dateien bleiben stets exakt so verschlüsselt wie die Original-SSTables.

---

## 2. Architektur & Komponenten

```
┌─────────────────────────────────────────────────────────────┐
│                    contextra-checkpoint                     │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                  CheckpointManager                  │   │
│   │                                                     │   │
│   │   Primary Storage Dir            Checkpoint Dir     │   │
│   │   ┌─────────────────┐            ┌────────────────┐ │   │
│   │   │ Manifest        │──Copy─────►│ Manifest       │ │   │
│   │   │ sst-001.sst ────┼──Hardlink──┼► sst-001.sst   │ │   │
│   │   │ sst-002.sst ────┼──Hardlink──┼► sst-002.sst   │ │   │
│   │   └─────────────────┘            └────────────────┘ │   │
│   │                                                     │   │
│   │   ┌────────────────────┐    ┌───────────────────┐   │   │
│   │   │   HardlinkCloner   │    │   OrphanCleaner   │   │   │
│   │   │  (Atomic Linker)   │    │(Crash Recovery)   │   │   │
│   │   └────────────────────┘    └───────────────────┘   │   │
│   └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Kanonische Schnittstellen & Typen

### 3.1 `CheckpointManager` & Optionen

```rust
use std::path::{Path, PathBuf};
use contextra_ids::Seq;

#[derive(Debug, Clone)]
pub struct CheckpointOptions {
    /// Erlaubt Fallback auf physische Dateikopie, falls Hardlinks nicht unterstützt werden (z. B. Cross-Device).
    pub allow_copy_fallback: bool,
    /// Erzwingt einen synchronen Fsync auf das Checkpoint-Manifest.
    pub fsync_manifest: bool,
}

impl Default for CheckpointOptions {
    fn default() -> Self {
        Self {
            allow_copy_fallback: false,
            fsync_manifest: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CheckpointInfo {
    pub checkpoint_dir: PathBuf,
    pub seq: Seq,
    pub file_count: usize,
    pub bytes_linked: u64,
}

pub struct CheckpointManager;

impl CheckpointManager {
    /// Erstellt einen atomaren Checkpoint aller SSTables und des Manifests.
    pub fn create_checkpoint(
        src_dir: &Path,
        dst_dir: &Path,
        current_seq: Seq,
        opts: &CheckpointOptions,
    ) -> Result<CheckpointInfo, CheckpointError> {
        if dst_dir.exists() {
            return Err(CheckpointError::DestinationAlreadyExists(dst_dir.to_path_buf()));
        }

        std::fs::create_dir_all(dst_dir).map_err(CheckpointError::Io)?;

        let mut file_count = 0;
        let mut bytes_linked = 0;

        for entry in std::fs::read_dir(src_dir).map_err(CheckpointError::Io)? {
            let entry = entry.map_err(CheckpointError::Io)?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    let file_name = entry.file_name();
                    let target_path = dst_dir.join(&file_name);

                    if ext == "sst" {
                        // Hardlink für unveränderliche SSTable
                        match std::fs::hard_link(&path, &target_path) {
                            Ok(()) => {
                                file_count += 1;
                                bytes_linked += entry.metadata().map(|m| m.len()).unwrap_or(0);
                            }
                            Err(e) if opts.allow_copy_fallback => {
                                std::fs::copy(&path, &target_path).map_err(CheckpointError::Io)?;
                                file_count += 1;
                                bytes_linked += entry.metadata().map(|m| m.len()).unwrap_or(0);
                            }
                            Err(e) => return Err(CheckpointError::HardlinkFailed { src: path, err: e }),
                        }
                    } else if ext == "manifest" {
                        // Manifest wird kopiert und gesynct
                        std::fs::copy(&path, &target_path).map_err(CheckpointError::Io)?;
                        if opts.fsync_manifest {
                            let f = std::fs::File::open(&target_path).map_err(CheckpointError::Io)?;
                            f.sync_all().map_err(CheckpointError::Io)?;
                        }
                        file_count += 1;
                    }
                }
            }
        }

        Ok(CheckpointInfo {
            checkpoint_dir: dst_dir.to_path_buf(),
            seq: current_seq,
            file_count,
            bytes_linked,
        })
    }
}
```

---

## 4. Fehlerbehandlung (`CheckpointError`)

```rust
use std::path::PathBuf;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum CheckpointError {
    #[error("Destination checkpoint directory already exists: {0:?}")]
    DestinationAlreadyExists(PathBuf),

    #[error("Failed to create hardlink from {src:?}: {err}")]
    HardlinkFailed {
        src: PathBuf,
        #[source]
        err: std::io::Error,
    },

    #[error("Underlying I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Durable filesystem error: {0}")]
    DurableFs(String),
}
```

---

## 5. `Cargo.toml` Spezifikation

```toml
[package]
name = "contextra-checkpoint"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Backup, zero-copy checkpointing, and branch management for Contextra storage"

[lints]
workspace = true

[dependencies]
contextra-ids = { workspace = true }
contextra-env = { workspace = true }
contextra-durable-fs = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true, optional = true }

[dev-dependencies]
tempfile = { workspace = true }
proptest = "1.4"

[features]
default = []
std = ["contextra-ids/std", "contextra-env/std"]
tracing = ["dep:tracing"]
```

---

## 6. Implementierungs-Arbeitspakete für Entwickler

1. **`Cargo.toml` Bereinigung:**
   - Entfernen von `contextra-types` und `contextra-ports` aus `[dependencies]`.
   - Entfernen von `contextra-testkit` aus `[dev-dependencies]` (R2-Verstoß).
   - Features `workspace`, `types-compat`, `ports-compat` entfernen.
   - `default = []` setzen (R3).
2. **Altlasten-Bereinigung (`AP-21`):**
   - Referenz auf `contextra-agent` in `src/lib.rs` Zeile 34 bereinigen.
3. **Verifikation:**
   - `cargo test -p contextra-checkpoint`

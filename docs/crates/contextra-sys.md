# contextra-sys — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E0 (Autarkes Fundament / Ring 0 — Unsafe-Insel)  
**Kern-Closure:** Leer (0 interne Workspace-Abhängigkeiten)  
**Status:** Normativ (Soll-Zustand für Entwickler)  
**Version:** 2.0  
**Geltungsbereich:** `crates/contextra-sys`  
**Referenzdokumente:** `docs/SPEC.md` §2.3 (R5), §4; `ANALYSIS-E0-CONTRADICTIONS.md`; `ALTLASTEN-AUDIT.md` (AP-15, AP-21)  

---

## 1. Zweck & Architekturgarantien

`contextra-sys` ist eine der exakt zwei autorisierten **Unsafe-Inseln** im Contextra-Workspace (neben `contextra-simd`).
Es kapselt alle hardware- und betriebssystemnahen Systemaufrufe (POSIX und Win32) hinter typsicheren, sicheren Rust-Schnittstellen:
- **Speichersperrung (`mlock` / `munlock` / `VirtualLock`):** Verhindert das Auslagern sensibler Schlüssel in die Swap-Partition (Art. 32 DSGVO, SPEC §4).
- **Speicherabgebildete Dateien (`mmap`):** Speichersichere Kapselung von Memory-Mapped I/O für LSM-SSTables und Vektor-Indizes via `memmap2`.
- **I/O-Beratung & Deskriptor-Flags (`posix_fadvise` / `O_DIRECT`):** Optimierung sequentieller Kompaktions-Reads und Vermeidung von Page-Cache-Pollution.
- **Verzeichnis-Synchronisation (`fsync_dir`):** Persistierung von Directory-Inodes für atomare Renames.

### Unsafe-Insel-Garantien (R5, INV-3, INV-11)
1. **Isolations-Garantie:** Sämtlicher FFI- und Low-Level-OS-Code ist hier konzentriert. Außerhalb von `sys` und `simd` gilt im gesamten Workspace ausnahmslos `#![forbid(unsafe_code)]`.
2. **Dokumentationspflicht für Unsafe:** Jeder einzelne `unsafe`-Block MUSS zwingend mit einem fundierten `// SAFETY:`-Kommentar begründet werden.
3. **Sichere Außengrenze:** Alle nach außen exportierten Structs, Enums und Funktionen sind zu 100 % sicher aufrufbar (Safe Rust). Rohe Zeiger und ungültige Dateideskriptoren verlassen das Crate nicht.
4. **Fehlerresilienz:** Betriebssystemfehler (`errno`, `GetLastError`) werden ausnahmslos in das typsichere `SysError`-Enum übersetzt; keine Panics.
5. **Leere Default-Features (R3):** `default = []`.

---

## 2. Kanonische Komponenten & Schnittstellen

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                contextra-sys                                │
│                                                                             │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │       LockedMemory       │             │         MmapHandle         │   │
│   │ mlock / VirtualLock RAII │             │ Safe Mmap / Read-only Slice│   │
│   │ Zeroize on Drop          │             │ madvise Sequential/Random  │   │
│   └─────────────┬────────────┘             └─────────────┬──────────────┘   │
│                 │                                        │                  │
│                 ▼                                        ▼                  │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │       posix / os         │             │          vault             │   │
│   │ sync_dir, advise_io,     │             │ SecureMemoryPool,          │   │
│   │ direct_io flags          │             │ LockedKeySlot              │   │
│   └──────────────────────────┘             └────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### 2.1 RAM-Sperrung (`LockedMemory<T>`)

Verhindert Swap-Leaks kryptographischer Schlüssel:

```rust
use std::mem::size_of;

/// RAII-Wrapper zur Bindung von Speicher im physischen RAM (mlock).
///
/// Beim Erzeugen wird `mlock` aufgerufen; beim Droppen wird der Speicher
/// zuerst mit Nullen überschrieben (`Zeroize`) und anschließend per `munlock` freigegeben.
pub struct LockedMemory<T> {
    data: T,
}

impl<T> LockedMemory<T> {
    /// Versucht, den Speicherbereich der übergebenen Daten im RAM zu sperren.
    pub fn new(data: T) -> Result<Self, SysError> {
        let ptr = &data as *const T as *const libc::c_void;
        let size = size_of::<T>();

        #[cfg(unix)]
        {
            // SAFETY: ptr verweist auf eine gültige, initialisierte Instanz von T
            // auf dem Stack/Heap des aktuellen Threads mit exakter Größe size_of::<T>().
            let res = unsafe { libc::mlock(ptr, size) };
            if res != 0 {
                return Err(SysError::MemoryLockFailed(std::io::Error::last_os_error()));
            }
        }

        #[cfg(windows)]
        {
            // SAFETY: ptr und size entsprechen einem gültigen virtuellen Speicherbereich des Prozesses.
            let res = unsafe {
                windows_sys::Win32::System::Memory::VirtualLock(
                    ptr as *mut _,
                    size,
                )
            };
            if res == 0 {
                return Err(SysError::MemoryLockFailed(std::io::Error::last_os_error()));
            }
        }

        Ok(Self { data })
    }

    #[inline]
    pub fn get(&self) -> &T {
        &self.data
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T> Drop for LockedMemory<T> {
    fn drop(&mut self) {
        let ptr = &self.data as *const T as *const libc::c_void;
        let size = size_of::<T>();

        #[cfg(unix)]
        {
            // SAFETY: Der Speicherbereich wurde im Konstruktor erfolgreich per mlock gesperrt
            // und ist bis zum Ende des Destruktors gültig.
            unsafe {
                libc::munlock(ptr, size);
            }
        }

        #[cfg(windows)]
        {
            // SAFETY: Entspricht dem im Konstruktor gesperrten Adressbereich.
            unsafe {
                windows_sys::Win32::System::Memory::VirtualUnlock(
                    ptr as *mut _,
                    size,
                );
            }
        }
    }
}
```

---

### 2.2 Speichersicheres Memory-Mapping (`MmapHandle`)

Kapselung von `memmap2::Mmap` mit I/O-Hints:

```rust
use std::fs::File;
use std::path::Path;

/// Speichersichere, unveränderliche Speicherabbildung einer Datei.
pub struct MmapHandle {
    mmap: memmap2::Mmap,
}

impl MmapHandle {
    /// Öffnet eine Datei schreibgeschützt und erstellt das Speicherabbild.
    pub fn open_read_only(file: &File) -> Result<Self, SysError> {
        // SAFETY: Das Mapping ist schreibgeschützt. Der Aufrufer muss sicherstellen,
        // dass die Datei während der Lebensdauer nicht von extern abgeschnitten (truncated) wird.
        let mmap = unsafe {
            memmap2::MmapOptions::new()
                .map(file)
                .map_err(|e| SysError::MmapFailed(e))?
        };
        Ok(Self { mmap })
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.mmap[..]
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.mmap.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.mmap.is_empty()
    }

    /// Weist das Betriebssystem an, das Mapping für sequentielle Lesezugriffe vorzuladen.
    pub fn advise_sequential(&self) -> Result<(), SysError> {
        #[cfg(unix)]
        {
            // SAFETY: mmap.as_ptr() und mmap.len() beschreiben einen gültigen gemappten Speicherbereich.
            let res = unsafe {
                libc::madvise(
                    self.mmap.as_ptr() as *mut libc::c_void,
                    self.mmap.len(),
                    libc::MADV_SEQUENTIAL,
                )
            };
            if res != 0 {
                return Err(SysError::MadviseFailed(std::io::Error::last_os_error()));
            }
        }
        Ok(())
    }

    /// Deaktiviert Read-Ahead für zufällige Zugriffsmuster (Sparse Point Lookups).
    pub fn advise_random(&self) -> Result<(), SysError> {
        #[cfg(unix)]
        {
            // SAFETY: Gültiger Pointer und Länge aus memmap2.
            let res = unsafe {
                libc::madvise(
                    self.mmap.as_ptr() as *mut libc::c_void,
                    self.mmap.len(),
                    libc::MADV_RANDOM,
                )
            };
            if res != 0 {
                return Err(SysError::MadviseFailed(std::io::Error::last_os_error()));
            }
        }
        Ok(())
    }
}
```

---

### 2.3 POSIX & OS Dateisystem-Primitive (`posix.rs`)

```rust
use std::os::unix::io::AsRawFd;

/// Synchronisiert einen Verzeichniseintrag durabel auf Speichermedien.
///
/// Unerlässlich nach atomarem `rename`, um Dateisystem-Inodes auf ext4/xfs abzusichern.
pub fn sync_directory(dir: &std::fs::File) -> Result<(), SysError> {
    #[cfg(unix)]
    {
        let fd = dir.as_raw_fd();
        // SAFETY: dir ist eine geöffnete, gültige Datei/Verzeichnisreferenz.
        let res = unsafe { libc::fsync(fd) };
        if res != 0 {
            return Err(SysError::SyncFailed(std::io::Error::last_os_error()));
        }
    }
    #[cfg(windows)]
    {
        dir.sync_all().map_err(SysError::SyncFailed)?;
    }
    Ok(())
}
```

---

## 3. Fehlerbehandlung (`SysError`)

```rust
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum SysError {
    #[error("Memory lock operation failed (mlock/VirtualLock): {0}")]
    MemoryLockFailed(#[source] std::io::Error),

    #[error("Memory mapping failed: {0}")]
    MmapFailed(#[source] std::io::Error),

    #[error("Memory advice failed (madvise): {0}")]
    MadviseFailed(#[source] std::io::Error),

    #[error("File descriptor synchronization failed (fsync): {0}")]
    SyncFailed(#[source] std::io::Error),

    #[error("Platform capability unsupported on current OS: {0}")]
    UnsupportedPlatform(&'static str),
}
```

---

## 4. `Cargo.toml` Spezifikation

```toml
[package]
name = "contextra-sys"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Low-level unsafe system abstractions and FFI island for Contextra (Ring 0)"

[lints]
workspace = true

[target.'cfg(unix)'.dependencies]
libc = "0.2"

[target.'cfg(windows)'.dependencies]
windows-sys = { version = "0.61", features = [
    "Win32_Foundation",
    "Win32_Security",
    "Win32_Security_Authorization",
    "Win32_Storage_FileSystem",
    "Win32_System_Memory",
    "Win32_System_Threading",
] }

[dependencies]
memmap2 = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }

[dev-dependencies]
tempfile = "3"

[features]
default = []
```

---

## 5. Implementierungs-Arbeitspakete für Entwickler

1. **`Cargo.toml`:**
   - Default-Features leeren (`default = []`).
   - `publish = false` sicherstellen.
2. **Altlasten-Bereinigung (`AP-15`, `AP-21`):**
   - In `tests/unsafe_islands.rs` sicherstellen, dass nur `contextra-sys` und `contextra-simd` als Unsafe-Islands registriert sind (`contextra-wire` entfernt).
   - In `src/vault.rs` und `src/lib.rs` alle veralteten Kommentare zu `contextra-vault` bereinigen.
3. **Audit aller `unsafe`-Blöcke:**
   - Sicherstellen, dass ausnahmslos jeder `unsafe`-Block einen `// SAFETY:`-Kommentar enthält.
4. **Verifikation:**
   - `cargo test -p contextra-sys`

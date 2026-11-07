# contextra-sys

Systemnahe Bindings, Low-Level I/O, Durability-Flush, Memory-Locking und Memory-Mapping.

Normativitätsklasse: **Kern `[K]`** (Ring 0 Unsafe-Insel, `stable`; §6.1, §6.7, §6.9, `capabilities.toml`).

## Rolle in einem Satz
`contextra-sys` bietet systemnahes Memory-Mapping (`mmap`, `SealedMap`), Durability-Flushes (`fsync`, `fdatasync`, `F_FULLFSYNC`, `FlushFileBuffers`), Prozess-Härtung (`rlimit`) und plattformspezifische I/O-Primitiven über eine sichere, typisierte Fassade.

## Ring, Klasse & Reifegrad
- **Ring:** Ring 0 (Unsafe-Insel, `#![deny(unsafe_code)]` ausgenommen)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **Abhängigkeiten:** Keine internen Workspace-Abhängigkeiten. Externe Crates: `libc` (POSIX), `windows-sys` (Windows), `memmap2`, `tracing`.

## Zielmodule & Baukasten-Architektur
- `mem`: Kapselt `mlock`, `munlock`, `MemLockPolicy`, `LockedRegions` und volatile Zeroization (`zeroize_slice`).
- `durability`: Plattformspezifische Synchronisation (`durable_flush_file`, `durable_flush_dir`, `write_all_durable`).
- `fsmeta`: Plattformspezifische Datei-ACLs (`set_restrictive_file_acl`, `verify_file_acl_owner_only`).
- `map`: Sichere Speicherabbildung via `mmap_readonly` sowie typgestützte `SealedFile` / `SealedMap` (gemäß INV-IO-1 / INV-ENCRYPT-REST-2).
- `posix`: Dateideskriptorlimits (`ensure_fd_limit`, `set_fd_limit`) und atomare Umleitung (`reopen_and_dup2`).

## Einbettungsbeispiel
```rust,no_run
use contextra_sys::{durable_flush_file, SealedFile, write_all_durable};
use std::fs::{File, OpenOptions};

fn main() -> std::io::Result<()> {
    let path = std::path::Path::new("sealed_data.bin");
    {
        let mut file = OpenOptions::new().create(true).write(true).open(path)?;
        write_all_durable(&mut file, b"sensible Nutzlast")?;
        durable_flush_file(&file, true)?;
    }
    let file = File::open(path)?;
    let sealed = SealedFile::new_verified(file, 17)?;
    let map = sealed.map_readonly()?;
    assert_eq!(&map[..], b"sensible Nutzlast");
    Ok(())
}
```

Ein vollständiges, autark lauffähiges Beispiel findet sich unter `examples/sys_primitives.rs`.

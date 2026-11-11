# contextra-sys

Low-Level-Systemabstraktionen, Durability-Flush, Memory-Locking und Memory-Mapping hinter einer sicheren Rust-Fassade (Ring 0 / Foundation).

## Zweck & Abgrenzung
`contextra-sys` ist die zentrale Unsafe-Insel (Ring 0, Ebene E0) für hardware- und betriebssystemnahe Operationen:
- **Durability-Synchronisation:** Plattformgerechtes fsync (`fdatasync`, `F_FULLFSYNC`, `FlushFileBuffers`, Verzeichnis-Sync) und unumkehrbares Handle-Poisoning via `DurableFile` (INV-SYS-FSYNC-1).
- **Speicherabbildung:** Sicheres, typisiertes Nur-Lese-Mmap (`mmap_readonly`, `SealedFile`, `SealedMap`). Schreibende Mmaps sind im Rust-Adressraum ausgeschlossen (INV-IO-1 / INV-ENCRYPT-REST-2).
- **RAM-Sperrung:** Schutz vor Paging/Swapping via `mlock` / `VirtualLock` mit konfigurierbaren Richtlinien (`MemLockPolicy`) und `madvise(MADV_DONTDUMP)`.
- **Tresor-Speicher (Vault):** RAII-verwaltete Regionen (`VaultBuffer`) und flüchtiges Nullsetzen (`write_volatile` + Memory Barrier).
- **POSIX-Limits & ACLs:** Restriktive Dateizugriffsrechte (`0600` und UID-Verifikation unter Unix, DACL unter Windows) sowie Dateideskriptor-Limits (`ensure_fd_limit`).
- **Vertrags-Abstraktion:** [`MemoryLocker`]-Trait und I/O-freie Referenzimplementierung [`NoopMemoryLocker`] in `contract.rs` für Test- und Isolationszwecke.

## Reifegrad & Stabilität
- **Ebene:** 0 (E0 Foundation, Kern-Closure leer)
- **Tier:** `stable` (Ring 0 / Foundation)
- **Unsafe-Insel:** Isolierte Unsafe-Stellen (22 Blöcke) mit lückenloser `// SAFETY:`-Dokumentation; vollständig sichere öffentliche Rust-Fassade (`#![deny(unsafe_op_in_unsafe_fn)]`, `#![deny(missing_docs)]`).
- **Fehlermodell:** Strukturierte Fehlercodes über `Error` / `SysError` (`thiserror`, `#[non_exhaustive]`) mit maschinenlesbarem OS-Errno.

## Plattform- & Fehlermodell

| Komponente | Linux (`cfg(target_os = "linux")`) | macOS (`cfg(target_os = "macos")`) | Windows (`cfg(windows)`) | Fehlerverhalten |
|---|---|---|---|---|
| **Durability** | `fdatasync` / `fsync` | `fcntl(fd, F_FULLFSYNC, 0)` | `FlushFileBuffers` | Fail-Closed; Handle-Poisoning bei E/A-Fehler |
| **Memory Locking** | `libc::mlock` + `MADV_DONTDUMP` | `libc::mlock` + `MADV_ZERO_WIRED_PAGES` | `VirtualLock` | `MemLockPolicy::FailClosed` vs. `Tolerant` |
| **Memory Mapping** | `memmap2::Mmap::map` | `memmap2::Mmap::map` | `memmap2::Mmap::map` | Nur-Lese; `InvalidData` bei Größenabweichung |
| **Dateiberechtigungen** | `chmod 0600` + EUID-Prüfung | `chmod 0600` + EUID-Prüfung | DACL Owner-Only (SID) | Verweigerung bei Fremdzugriff |

## Tatsächliche Abhängigkeiten
- **Interne Workspace-Abhängigkeiten:** **0** (Kern-Closure ist leer: Ebene E0).
- **Externe Abhängigkeiten (`cargo tree -p contextra-sys -e normal --no-default-features`):**
  - `memmap2 = "0.9"`
  - `thiserror = "2.0"`
  - `tracing = "0.1"`
- **Plattformspezifische Abhängigkeiten:**
  - Unix (`cfg(unix)`): `libc = "0.2"`
  - Windows (`cfg(windows)`): `windows-sys = "0.61"`
- **Dev-Abhängigkeiten:**
  - `criterion = "0.5"`
  - `tempfile = "3"`

## Features
- Keine optionalen Features (`default = []`). Das Crate ist standardmäßig autark und minimal.

## Quickstart (Kompilierbares Beispiel)
```rust
use contextra_sys::mmap_readonly;
use std::fs::File;
use std::io::Write;
use tempfile::NamedTempFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut temp = NamedTempFile::new()?;
    temp.write_all(b"contextra-sys standalone payload")?;
    let file = File::open(temp.path())?;
    let mmap = mmap_readonly(&file)?;
    assert_eq!(&mmap[..], b"contextra-sys standalone payload");
    Ok(())
}
```

## Standalone-Beispiel ausführen
```bash
cargo run -p contextra-sys --example standalone --no-default-features
```

## Entnahme-Rezept
1. Kopiere `crates/contextra-sys` in dein Projekt oder binde es als autarke Pfad-/Git-Abhängigkeit ein.
2. Trage es in deiner `Cargo.toml` ein:
```toml
[dependencies]
contextra-sys = { path = "../contextra-sys" }
```
Da `contextra-sys` keine internen Abhängigkeiten besitzt, baut und testet es sofort autark in jedem Workspace.

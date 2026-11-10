# contextra-sys

Low-level unsafe Systemabstraktionen, Durability-Flush, Memory-Locking und Memory-Mapping hinter einer sicheren Rust-Fassade.

## Zweck
`contextra-sys` stellt autarke, betriebssystemnahe Primitive für Contextra bereit:
- **Durability-Synchronisation:** Plattformgerechtes fsync (`fdatasync`, `F_FULLFSYNC`, `FlushFileBuffers`, Verzeichnis-Sync) und unumkehrbares Handle-Poisoning via `DurableFile`.
- **Speicherabbildung:** Sicheres, typisiertes Nur-Lese-Mmap (`mmap_readonly`, `SealedFile`, `SealedMap`).
- **RAM-Sperrung:** Schutz vor Paging/Swapping via `mlock` / `VirtualLock` mit Richtlinien (`MemLockPolicy`) und `madvise(MADV_DONTDUMP)`.
- **Tresor-Speicher (Vault):** RAII-verwaltete Regionen (`VaultBuffer`) und flüchtiges Nullsetzen (`write_volatile` + Memory Barrier).
- **POSIX-Limits & ACLs:** Restriktive Dateizugriffsrechte (`0600` und UID-Verifikation unter Unix, DACL unter Windows) sowie Dateideskriptor-Limits (`ensure_fd_limit`).

## Reifegrad
- **Status:** Stabil / Produktionsreif (P1–P6 erfüllt)
- **Unsafe-Insel:** Isolierte Unsafe-Stellen (22 Blöcke) mit lückenloser `// SAFETY:`-Dokumentation; vollständig sichere öffentliche Rust-Fassade.
- **Fehlermodell:** Strukturierte Fehlercodes über `Error` (`thiserror`, `#[non_exhaustive]`).

## Interne Abhängigkeiten
`contextra-sys` ist ein autarker Baustein und hat **0 interne Workspace-Abhängigkeiten**.

## Features
- `default = []` (reines Standard-Build mit Ziel-OS Abhängigkeiten `libc` unter Unix bzw. `windows-sys` unter Windows).

## Minimalbeispiel
```rust
use contextra_sys::acl_win32::set_restrictive_file_acl;
use tempfile::NamedTempFile;

let file = NamedTempFile::new().unwrap();
let res = set_restrictive_file_acl(file.path());
assert!(res.is_ok());
```

## Entnahme-Rezept
1. Kopiere `crates/contextra-sys` in dein Projekt.
2. Binde es in deiner `Cargo.toml` ein:
```toml
[dependencies]
contextra-sys = { path = "../contextra-sys" }
```

## Bekannte Grenzen
- `VaultBuffer` und `mlock` setzen ausreichende OS-Privilegien voraus (z. B. `RLIMIT_MEMLOCK` unter Linux).
- Mmap (`SealedMap`) erfordert unveränderliche Dateien während der Abbildung.

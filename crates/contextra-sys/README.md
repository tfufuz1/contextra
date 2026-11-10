# contextra-sys

Low-level unsafe Systemabstraktionen, Durability-Flush, Memory-Locking und Memory-Mapping hinter einer sicheren Rust-Fassade (Ring 0 / Foundation).

## Zweck
`contextra-sys` stellt autarke, betriebssystemnahe Primitive für Contextra bereit:
- **Durability-Synchronisation:** Plattformgerechtes fsync (`fdatasync`, `F_FULLFSYNC`, `FlushFileBuffers`, Verzeichnis-Sync) und unumkehrbares Handle-Poisoning via `DurableFile`.
- **Speicherabbildung:** Sicheres, typisiertes Nur-Lese-Mmap (`mmap_readonly`, `SealedFile`, `SealedMap`).
- **RAM-Sperrung:** Schutz vor Paging/Swapping via `mlock` / `VirtualLock` mit Richtlinien (`MemLockPolicy`) und `madvise(MADV_DONTDUMP)`.
- **Tresor-Speicher (Vault):** RAII-verwaltete Regionen (`VaultBuffer`) und flüchtiges Nullsetzen (`write_volatile` + Memory Barrier).
- **POSIX-Limits & ACLs:** Restriktive Dateizugriffsrechte (`0600` und UID-Verifikation unter Unix, DACL unter Windows) sowie Dateideskriptor-Limits (`ensure_fd_limit`).

## Reifegrad
- **Tier:** `beta` (Ring 0 / Ebene 0 Foundation)
- **Unsafe-Insel:** Isolierte Unsafe-Stellen (22 Blöcke) mit lückenloser `// SAFETY:`-Dokumentation; vollständig sichere öffentliche Rust-Fassade (`#![deny(unsafe_op_in_unsafe_fn)]`).
- **Fehlermodell:** Strukturierte Fehlercodes über `Error` (`thiserror`, `#[non_exhaustive]`).

## Tatsächliche Abhängigkeiten
- **Interne Workspace-Abhängigkeiten:** **0** (Kern-Closure ist leer: Ebene 0).
  - *Hinweis:* `contextra-durable-fs` ist unabhängig von `contextra-sys`.
- **Externe Abhängigkeiten:**
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
- Keine optionalen Features (`default = []`).

## Minimalbeispiel
```rust
use contextra_sys::acl_win32::set_restrictive_file_acl;
use tempfile::NamedTempFile;

let file = NamedTempFile::new().unwrap();
let res = set_restrictive_file_acl(file.path());
assert!(res.is_ok());
```

## Entnahme-Rezept
1. Kopiere `crates/contextra-sys` in dein Projekt oder binde es als autarke Pfad-/Git-Abhängigkeit ein.
2. Binde es in deiner `Cargo.toml` ein:
```toml
[dependencies]
contextra-sys = { path = "../contextra-sys" }
```

## Bekannte Grenzen
- `VaultBuffer` und `mlock` setzen ausreichende OS-Privilegien voraus (z. B. `RLIMIT_MEMLOCK` unter Linux).
- Mmap (`SealedMap`) erfordert unveränderliche Dateien während der Abbildung.

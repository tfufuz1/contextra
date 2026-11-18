# AGENTS.md — contextra-sys — OS-Primitive & Unsafe-Insel

**Status:** Normativ  
**Crate-Spezifikation:** [`docs/crates/contextra-sys.md`](file:///home/freddy/Projekte/contextra/docs/crates/contextra-sys.md)  
**Gesamtspezifikation:** [`docs/SPEC.md`](file:///home/freddy/Projekte/contextra/docs/SPEC.md) — Bindung: docs/SPEC.md §2 (Architektur), §2.3 (R5 Unsafe-Inseln), §4 (Schlüsselhierarchie), §13 (INV-3, INV-11)  
**Workspace-Richtlinie:** [`AGENTS.md`](file:///home/freddy/Projekte/contextra/AGENTS.md) (Wurzel)  
**Ebene:** E0 (Autarkes Fundament / Ring 0 — Genehmigte Unsafe-Insel)  
**Kern-Closure:** Leer (0 interne Workspace-Abhängigkeiten)  
**Unsafe-Status:** Erlaubt (Genehmigte Unsafe-Insel nach SPEC §2.3 R5 & INV-3)  

---

## 1. Zweck & Systemrolle

Betriebssystemnahe Speichersteuerung (`mlock`, `munlock`, `madvise`) und I/O-Flags (`O_DIRECT`, `O_DSYNC`).

---

## 2. Kern-Richtlinien für Agenten

- **Unsafe-Disziplin (INV-3, INV-11):**
  - Jeder `unsafe`-Block MUSS zwingend durch einen vorangestellten `// SAFETY:`-Kommentar begründet werden.
  - Sämtliche `unsafe`-Aufrufe von `libc` oder FFI sind strikt hinter sicheren, typsicheren Rust-Schnittstellen gekapselt.
  - Rückgabewerte des Betriebssystems müssen immer auf `errno` geprüft und in `SysError` übersetzt werden.
- **RAM-Sperrung (`LockedMemory`):** Kapselt `mlock`/`munlock`, damit kryptographische Schlüssel im Arbeitsspeicher niemals auf unverschlüsselte Swap-Partitionen ausgelagert werden.
- **I/O-Beratung:** Plattformspezifische I/O-Hinweise (`posix_fadvise`, `madvise`) für sequentielle Kompaktionen oder zufällige Lookups.

---

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-sys` · **Ring:** 0 · **Version:** 0.1.0
- **Beschreibung:** Low-level unsafe system abstractions and FFI island for Contextra (Ring 0)
- **src:** 9 Dateien, 1564 LOC · **unsafe:** 22 Vorkommen, Policy `keiner`
- **Interne Abhängigkeiten (normal, 0):** –
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 5):** `libc`, `memmap2`, `thiserror`, `tracing`, `windows-sys`
- **Reverse-Abhängigkeiten (3):** `contextra-store`, `contextra-vault`, `contextra-vector`
- **Kopplung an Hub-Crates:** ausgehend –; eingehend –

### Öffentliche API (grob, aus src/)
- struct (9): `DurableFile`, `LockedRegions`, `NoopMemoryLocker`, `OsMemoryLocker`, `SealedFile`, `SealedMap`, `SealedReadOnly`, `Unsealed`, `VaultBuffer`
- enum (2): `Error`, `MemLockPolicy`
- trait (1): `MemoryLocker`
- fn (42): `allocate`, `allocate_with_policy`, `apply_dont_dump`, `as_file`, `as_mut_slice`, `as_slice`, `durable_flush_dir`, `durable_flush_file`, `ensure_fd_limit`, `from_slice`, `get_ref`, `get_subslice`, `is_empty`, `is_locked`, `is_poisoned` … (+27)
- type (2): `Result`, `SysError`
- mod (11): `acl_win32`, `contract`, `durability`, `error`, `fsmeta`, `map`, `mem`, `mlock`, `mmap`, `posix`, `vault`
- pub use: `acl_win32::{set_restrictive_file_acl, verify_file_acl_owner_only}`, `contract::{MemoryLocker, NoopMemoryLocker, OsMemoryLocker}`, `crate::acl_win32::{set_restrictive_file_acl, verify_file_acl_owner_only}`, `crate::contract::{MemoryLocker, NoopMemoryLocker, OsMemoryLocker}`, `crate::mlock::{ apply_dont_dump, mem_lock, mem_lock_typed, mem_lock_with_policy, mem_unlock, MemLockPolicy, }`, `crate::mmap::{mmap_readonly, SealedFile, SealedMap, SealedReadOnly, Unsealed}`, `crate::vault::{LockedRegions, VaultBuffer}`, `durability::{durable_flush_dir, durable_flush_file, write_all_durable, DurableFile}`, `error::{Error, Result, SysError}`, `mlock::{ apply_dont_dump, mem_lock, mem_lock_typed, mem_lock_with_policy, mem_unlock, MemLockPolicy, }` … (+3)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->

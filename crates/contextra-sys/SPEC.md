# SPEC: `contextra-sys` (Baustein-Spezifikation v4.1)

> **Klasse:** A (Vollständig autarker Baustein nach SPEC §5.1, B1–B12)
> **Ring:** Ring 0 (Unsafe-Insel)
> **Status:** Stabil
> **Geltung:** Verbindliche Verhaltens- und Schnittstellenspezifikation für Systemprimitive in Contextra.

---

## 1. Zweck und Rolle
`contextra-sys` stellt systemnahe Betriebssystem-Primitive hinter einer **100 % sicheren Rust-Fassade** bereit:
1. **Durability-Synchronisation:** Plattformangepasste Datei- und Verzeichnissynchronisation (`fdatasync`, `fsync`, macOS `F_FULLFSYNC`, Windows `FlushFileBuffers`) sowie typsicheres Handle-Poisoning via [`DurableFile`].
2. **Speicherabbildung (`mmap` / `SealedMap`):** Schreibgeschützte Speicherabbildungen unveränderlicher Dateien mit strenger Längenvalidierung und Signalanomalie-Schutz.
3. **RAM-Sperrung (`mlock`):** Schutz sensibler Speicherseiten vor Paging/Swapping auf persistenten Speicher inklusive `madvise(MADV_DONTDUMP)`.
4. **Tresor-Speicherverwaltung (`vault`):** RAII-verwaltete gesperrte Regionen ([`LockedRegions`]), geschützte Allokationen ([`VaultBuffer`]) und volatile Speicherlöschung (`write_volatile` + Compiler-Fence `SeqCst`).
5. **Prozess-Ressourcenlimits (`posix`):** Abfrage und Erhöhung von Dateideskriptorlimits (`rlimit`) und Deskriptorumleitung (`dup2`).
6. **Zugriffsbeschränkung (`acl_win32`):** POSIX `0600`-Modus und UID-Besitzprüfung unter Unix; DACL-Erstellung ohne Vererbung unter Win32.
7. **Typisiertes Fehlermodell (`error`):** Strukturierte Fehlerklassifikation via [`Error`] (`thiserror`, `#[non_exhaustive]`) mit maschinenlesbarem OS-Errno (`SyncFailed(i32)`, `MemoryLockFailed`).

---

## 2. Invarianten

| Invariante | Beschreibung | Durchsetzung |
|---|---|---|
| **INV-SYS-FSYNC-1** | Durability-Fehler dürfen niemals verschluckt oder still wiederholt werden (nur `EINTR` wird wiederholt). Tritt ein Flush-Fehler auf, wird das Handle permanent vergiftet und jeder nachfolgende Flush- oder Schreibversuch schlägt fehl. | `durable_flush_file`, `durable_flush_dir`, `write_all_durable`, `DurableFile` |
| **INV-SYS-MEMLOCK-1** | Über `VaultBuffer` allozierte Puffer werden mit `mlock` gesperrt, per `MADV_DONTDUMP` vom Core-Dump ausgenommen, beim Drop mit einem nicht wegoptimierbaren Zeroize überschrieben (`write_volatile` + Fence) und erst danach `munlock`/freigegeben. Scheitert `mlock`, schlägt die Allokation fail-closed fehl. | `mlock.rs`, `vault.rs`, `VaultBuffer` |
| **INV-IO-1** | `mmap` darf nur auf verifizierten, unveränderlichen Dateien ausgeführt werden. Mutationen während bestehender Mappings sind unzulässig. | `SealedFile::new_verified`, `SealedMap` |
| **INV-ENCRYPT-REST-2** | Zero-Copy-`mmap` erfordert volumenlokale Speicherung und verifizierte Integrität. | `mmap_readonly`, `SealedFile` |
| **INV-HEAP-ZERO-1** | Sensible Puffer werden vor der Freigabe mit flüchtigen Schreibzugriffen (`write_volatile`) und Compiler-Fence genullt, um Compiler-Elimination (Dead-Store Elimination) auszuschließen. | `LockedRegions::zeroize_slice`, `VaultBuffer::drop` |
| **INV-PLATFORM-1** | Fehlende Plattformunterstützung führt niemals zu stillschweigendem Erfolg. Plattformspezifische Operationen deklarieren deterministisches Verhalten oder typisierte Fehler (`MemLockPolicy::FailClosed`, `Error::Unsupported`). | `mlock.rs`, `acl_win32.rs`, `durability.rs` |
| **INV-SAFE-FACADE** | Sämtliche öffentlichen Funktionen und Typen bieten eine sichere API ohne Vorbedingung für `unsafe`-Blöcke. Jedes interne `unsafe` ist mit `// SAFETY:` annotiert. | `#![deny(unsafe_op_in_unsafe_fn)]`, Clippy-Gates |

---

## 3. Fehlermodell und Typen
- Fehler werden primär als [`Error`] (`thiserror`, `#[non_exhaustive]`) bzw. [`Result<T>`] mit verlustfreier Konvertierung nach `std::io::Error` zurückgegeben.
- Keine Typen aus fremden `contextra-*`-Crates in öffentlichen Schnittstellen (B3).
- Panic-Freiheit (B9): Keine Panics, Unwraps oder Expects in Produktionspfaden. Mutex-Poisoning existiert im Crate nicht (keine internen Locks).

---

## 4. Nebenläufigkeit & Prozessgarantien
- Keine globalen statischen Zustände außer atomaren OS-Syscalls und Test-Fault-Injection.
- Alle Datei- und Flush-Operationen sind synchron und blockierend. In asynchronen Tokio-Umgebungen müssen sie in `tokio::task::spawn_blocking` ausgeführt werden.
- `ensure_fd_limit` und `set_fd_limit` manipulieren den prozessweiten Zustand und dürfen nur im System-Start/Composition-Root aufgerufen werden.

---

## 5. Plattform-Matrix

| Primitiv | Linux | macOS | Windows | Fallback |
|---|---|---|---|---|
| `durable_flush_file` / `DurableFile` | `fdatasync` / `fsync` | `fcntl(F_FULLFSYNC)` / `fsync` | `FlushFileBuffers` | `Error::Unsupported` |
| `durable_flush_dir` | `open(O_DIRECTORY)` + `fsync` | `open(O_RDONLY)` + `fsync` | Best Effort (`Ok(())`) | `ErrorKind::Unsupported` |
| `mem_lock` / `VaultBuffer` | `libc::mlock` + `MADV_DONTDUMP` | `libc::mlock` + `MADV_ZERO_WIRED_PAGES` | `VirtualLock` | `MemLockPolicy::FailClosed` |
| `mmap_readonly` | `memmap2::Mmap` | `memmap2::Mmap` | `memmap2::Mmap` | `memmap2::Mmap` |
| `ACL / Permissions` | `chmod 0600` + `getuid()` | `chmod 0600` + `getuid()` | DACL via `SetNamedSecurityInfoW` | Best Effort / No-Op |

# SPEC: `contextra-sys` (Baustein-Spezifikation v4.0)

> **Klasse:** A (Vollständig autarker Baustein nach SPEC §5.1, B1–B12)
> **Ring:** Ring 0 (Unsafe-Insel)
> **Status:** Stabil
> **Geltung:** Verbindliche Verhaltens- und Schnittstellenspezifikation für Systemprimitive in Contextra.

---

## 1. Zweck und Rolle
`contextra-sys` stellt systemnahe Betriebssystem-Primitive hinter einer **100 % sicheren Rust-Fassade** bereit:
1. **Durability-Synchronisation:** Plattformangepasste Datei- und Verzeichnissynchronisation (`fdatasync`, `fsync`, macOS `F_FULLFSYNC`, Windows `FlushFileBuffers`).
2. **Speicherabbildung (`mmap` / `SealedMap`):** Schreibgeschützte Speicherabbildungen unveränderlicher Dateien mit strenger Längenvalidierung und Signalanomalie-Schutz.
3. **RAM-Sperrung (`mlock`):** Schutz sensibler Speicherseiten vor Paging/Swapping auf persistenten Speicher.
4. **Tresor-Speicherverwaltung (`vault`):** RAII-verwaltete gesperrte Regionen und volatile Speicherlöschung (`write_volatile`).
5. **Prozess-Ressourcenlimits (`posix`):** Abfrage und Erhöhung von Dateideskriptorlimits (`rlimit`) und Deskriptorumleitung (`dup2`).
6. **Zugriffsbeschränkung (`acl_win32`):** POSIX `0600`-Modus und UID-Besitzprüfung unter Unix; DACL-Erstellung ohne Vererbung unter Win32.

---

## 2. Invarianten

| Invariante | Beschreibung | Durchsetzung |
|---|---|---|
| **INV-FSYNC-1** | Durability-Fehler dürfen niemals verschluckt werden. Jeder I/O- oder Flush-Fehler wird als typisierter `std::io::Error` zurückgegeben. Aufrufer müssen Handles nach Fehlschlag vergiften. | `durable_flush_file`, `durable_flush_dir`, `write_all_durable` |
| **INV-IO-1** | `mmap` darf nur auf verifizierten, unveränderlichen Dateien ausgeführt werden. Mutationen während bestehender Mappings sind unzulässig. | `SealedFile::new_verified`, `SealedMap` |
| **INV-ENCRYPT-REST-2** | Zero-Copy-`mmap` erfordert volumenlokale Speicherung und verifizierte Integrität. | `mmap_readonly`, `SealedFile` |
| **INV-HEAP-ZERO-1** | Sensible Puffer werden vor der Freigabe mit flüchtigen Schreibzugriffen (`write_volatile`) genullt, um Compiler-Elimination (Dead-Store Elimination) auszuschließen. | `LockedRegions::zeroize_slice` |
| **INV-PLATFORM-1** | Fehlende Plattformunterstützung führt niemals zu stillschweigendem Erfolg. Plattformspezifische Operationen deklarieren deterministisches Verhalten oder typisierte Fehler (`MemLockPolicy::FailClosed`). | `mlock.rs`, `acl_win32.rs` |
| **INV-SAFE-FACADE** | Sämtliche öffentlichen Funktionen und Typen bieten eine sichere API ohne Vorbedingung für `unsafe`-Blöcke. Jedes interne `unsafe` ist mit `// SAFETY:` annotiert. | `#![deny(unsafe_op_in_unsafe_fn)]`, Clippy-Gates |

---

## 3. Fehlermodell und Typen
- Fehler werden ausschließlich als Standard-`std::io::Result<T>` bzw. `std::io::Error` oder deterministische Booleans zurückgegeben.
- Keine Typen aus fremden `contextra-*`-Crates in öffentlichen Schnittstellen (B3).
- Panic-Freiheit (B9): Keine Panics, Unwraps oder Expects in Produktionspfaden. Mutex-Poisoning existiert im Crate nicht (keine internen Locks).

---

## 4. Nebenläufigkeit & Prozessgarantien
- Keine globalen statischen Zustände außer atomaren OS-Syscalls.
- Alle Datei- und Flush-Operationen sind synchron und blockierend. In asynchronen Tokio-Umgebungen müssen sie in `tokio::task::spawn_blocking` ausgeführt werden.
- `ensure_fd_limit` und `set_fd_limit` manipulieren den prozessweiten Zustand und dürfen nur im System-Start/Composition-Root aufgerufen werden.

---

## 5. Plattform-Matrix

| Primitiv | Linux | macOS | Windows | Fallback |
|---|---|---|---|---|
| `durable_flush_file` | `fdatasync` / `fsync` | `fcntl(F_FULLFSYNC)` / `fsync` | `FlushFileBuffers` | `sync_all` |
| `durable_flush_dir` | `open(O_DIRECTORY)` + `fsync` | `open(O_RDONLY)` + `fsync` | `FlushFileBuffers` (Best Effort) | No-Op / `ErrorKind::Unsupported` |
| `mem_lock` | `libc::mlock` | `libc::mlock` | `VirtualLock` | `MemLockPolicy` |
| `mmap_readonly` | `memmap2::Mmap` | `memmap2::Mmap` | `memmap2::Mmap` | `memmap2::Mmap` |
| `ACL / Permissions` | `chmod 0600` + `getuid()` | `chmod 0600` + `getuid()` | DACL via `SetNamedSecurityInfoW` | Best Effort / No-Op |

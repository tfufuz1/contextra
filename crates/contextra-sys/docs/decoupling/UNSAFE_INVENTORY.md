# Unsafe-Inventar `contextra-sys` (E0 / Ring 0)

Stand: HEAD
Gemessen mit: `python3 tools/unsafe_boundary_auditor.py --crate contextra-sys`

## Übersicht
- Unsafe-Insel: **JA** (laut Architekturplan E0a / Ring 0 legitimiert)
- Unsafe-Stellen gesamt: **22**
- Stellen ohne `// SAFETY:`: **0**
- Missing Safety Ratio: **0.0%**

---

## Detaillierte Stellen

### 1. `src/posix.rs`
1. `posix.rs:24`: `libc::setrlimit(libc::RLIMIT_NOFILE, &new_limit)`
   - **SAFETY**: `libc::setrlimit` wird mit Zeiger auf ein valides, initialisiertes `libc::rlimit`-Struct aufgerufen.
2. `posix.rs:60`: `libc::getrlimit(libc::RLIMIT_NOFILE, &mut limit)`
   - **SAFETY**: `libc::getrlimit` wird mit Zeiger auf ein valides, initialisiertes `libc::rlimit`-Struct aufgerufen.
3. `posix.rs:65`: `libc::setrlimit(libc::RLIMIT_NOFILE, &limit)`
   - **SAFETY**: `libc::setrlimit` wird mit Zeiger auf valides `libc::rlimit` aufgerufen.
4. `posix.rs:107`: `libc::open(path_c.as_ptr(), flags, 0o600)`
   - **SAFETY**: `path_c` ist ein valider, nullterminierter C-String.
5. `posix.rs:114`: `libc::dup2(new_fd, target_fd)`
   - **SAFETY**: `new_fd` und `target_fd` sind valide bzw. definierte Filedeskriptoren.
6. `posix.rs:115`: `libc::close(new_fd)`
   - **SAFETY**: Schließen des temporären Filedeskriptors.

### 2. `src/mlock.rs`
7. `mlock.rs:88`: `libc::madvise(aligned_ptr, aligned_len, libc::MADV_DONTDUMP)`
   - **SAFETY**: `aligned_ptr` und `aligned_len` sind seitenbündig und validieren den Speicherbereich.
8. `mlock.rs:145`: `libc::mlock(aligned_ptr, aligned_len)`
   - **SAFETY**: `aligned_ptr` und `aligned_len` decken den Speicherbereich ab und sind auf Seitengröße gerundet.
9. `mlock.rs:172`: `libc::munlock(aligned_ptr, aligned_len)`
   - **SAFETY**: `aligned_ptr` und `aligned_len` sind seitenbündig und heben vorherige Sperren auf.

### 3. `src/vault.rs`
10. `vault.rs:37`: `std::ptr::write_bytes(slice.as_mut_ptr(), 0, slice.len())`
    - **SAFETY**: `slice` ist ein gültiger, veränderlicher Byte-Slice.
11. `vault.rs:43`: `std::ptr::read_volatile(&*slice.as_ptr().add(offset))`
    - **SAFETY**: Offset ist durch Prüfung `< slice.len()` strikt innerhalb der Grenzen.
12. `vault.rs:108`: `std::ptr::write_bytes(ptr as *mut u8, 0, len)`
    - **SAFETY**: `ptr` und `len` stammen aus einem alloziierten Puffer.
13. `vault.rs:128`: `std::ptr::write_bytes(ptr, 0, len)`
    - **SAFETY**: `ptr` ist ein gültiger Rohzeiger mit Länge `len`.

### 4. `src/mmap.rs`
14. `mmap.rs:42`: `memmap2::MmapOptions::new().map(file)`
    - **SAFETY**: Aufruf von `memmap2::MmapOptions::map`. Die Datei ist im Nur-Lese-Modus geöffnet und bleibt während der Lebensdauer unverändert.
15. `mmap.rs:98`: `memmap2::MmapOptions::new().map(&file)`
    - **SAFETY**: Nur-Lese-Mapping auf die versiegelte Datei.

### 5. `src/durability.rs`
16. `durability.rs:46`: `libc::fcntl(fd, libc::F_FULLFSYNC)` / `libc::fsync(fd)`
    - **SAFETY**: `fd` ist ein gültiger Filedeskriptor aus `file.as_raw_fd()`.
17. `durability.rs:80`: `libc::open(path_c.as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY)`
    - **SAFETY**: `path_c` ist ein gültiger C-String.
18. `durability.rs:87`: `libc::fsync(fd)`
    - **SAFETY**: `fd` ist ein geöffneter Verzeichnisdeskrpitor.
19. `durability.rs:88`: `libc::close(fd)`
    - **SAFETY**: Freigabe des geöffneten Verzeichnisdeskriptors.

### 6. `src/acl_win32.rs`
20. `acl_win32.rs:85`: `libc::stat(path_c.as_ptr(), &mut stat_buf)`
    - **SAFETY**: `path_c` ist ein nullterminierter C-String, `stat_buf` ist initialisiert.
21. `acl_win32.rs:88`: `libc::geteuid()`
    - **SAFETY**: `libc::geteuid` hat keine Vorbedingungen und schlägt nie fehl.
22. `acl_win32.rs:113`: `libc::chmod(path_c.as_ptr(), 0o600)`
    - **SAFETY**: `path_c` ist ein nullterminierter C-String.

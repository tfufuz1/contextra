# Sicherheitsaudit: contextra-sys (Ring 0 Unsafe-Insel)

- **Datum:** 2026-10-12
- **Crate:** `crates/contextra-sys`
- **Scope:** `src/mmap.rs`, `src/mlock.rs`, `src/acl_win32.rs`, `src/posix.rs`, `src/durability.rs`, `src/vault.rs`, `src/lib.rs`
- **Session:** 917bac5c
- **Auditor:** Principal Senior Rust Architect (Jules)

---

## 1. Unsafe-Block-Inventar & SAFETY-Kommentar-Status

Alle `unsafe`-Blöcke in `contextra-sys` sind gemäß `INV-SYS-UNSAFE-SAFETY-DOC` mit strukturierten `// SAFETY:`-Kommentaren versehen. Die Crate-Ebene erzwingt `#![allow(unsafe_code)]` in Kombination mit `#![deny(unsafe_op_in_unsafe_fn)]`.

| Modul | Unsafe-Blöcke | SAFETY-Status | Beschreibung / Gekapselte OS-Primitive |
|---|---|---|---|
| `src/mmap.rs` | 1 | ✅ Vollständig | `memmap2::Mmap::map(file)` für Zero-Copy Read-Only Mapping. |
| `src/mlock.rs` | 4 (`unix` / `windows`) | ✅ Vollständig | `libc::mlock`, `libc::munlock`, `VirtualLock`, `VirtualUnlock`. |
| `src/posix.rs` | 1 (`linux`) | ✅ Vollständig | `libc::open`, `libc::dup2`, `libc::close`. |
| `src/acl_win32.rs` | 18 (`windows`) | ✅ Vollständig | `OpenProcessToken`, `GetTokenInformation`, `SetNamedSecurityInfoW`, `GetNamedSecurityInfoW`, `GetSecurityDescriptorControl`, `AddAccessAllowedAce`, `EqualSid`, `CloseHandle`, `LocalFree`. |
| `src/durability.rs` | 10 | ✅ Vollständig | Platform-spezifische Direct Syscalls (`fdatasync`, `fsync`, `fcntl(F_FULLFSYNC)`, `FlushFileBuffers`, `MoveFileExW`). |

---

## 2. mmap-Lifetime-Analyse (`mmap.rs`)

- **Lifetime-Kopplung:** `mmap_readonly(file: &File) -> io::Result<memmap2::Mmap>` nimmt `&File`. Die zugrundeliegende Bibliothek `memmap2` dupliziert das OS-File-Handle intern beim Erstellen des Memory-Mappings.
- **Use-After-Close-Schutz:** Sollte der Aufrufer den `&File`-Deskriptor schließen oder droppen, bleibt das im Kernel gemappte Virtual-Memory-Segment über die `memmap2::Mmap`-RAII-Struktur gültig.
- **RAII & Safety-Guarantees:**
  - `Mmap` implementiert `Deref<Target = [u8]>`, womit die Slice-Lebensdauer an das `Mmap`-Objekt gebunden ist.
  - Leere Dateien (0 Bytes) werden vor dem Mapping abgewiesen (`io::ErrorKind::InvalidInput`), um Kernel-Undefined-Behavior oder ungültige Mappings zu verhindern.
  - Nach dem Mapping wird `mmap.len() as u64 == file_len` verifiziert.

`EVIDENCE-P2-MMAP-LIFETIME`: RAII-Bindung via `memmap2::Mmap` und Vorab-Prüfung auf 0 Bytes verifiziert.

---

## 3. Error-Propagation-Nachweis

- **Zero-Panic-Garantie (`INV-SYS-ZERO-PANIC`):** Keines der Module in `contextra-sys` verwendet `unwrap()`, `expect()` oder unkontrollierte `panic!`-Aufrufe.
- **I/O-Fehler:** `mmap_readonly`, `reopen_and_dup2`, `set_restrictive_file_acl`, `verify_file_acl_owner_only`, `flush_file`, `flush_dir`, `fs_class` und `rename_durable` geben Fehler ausnahmslos als `std::io::Result<T>` zurück.
- **`mlock` / `munlock` Verhalten:**
  - `mem_lock(ptr: *const u8, len: usize) -> bool` ist als Best-Effort-Speichersperre spezifiziert (AGENTS.md §7). Bei fehlenden OS-Rechten (z. B. `RLIMIT_MEMLOCK` auf CI-Knoten) schlägt der Aufruf fehl, protokolliert eine Warnung via `tracing::warn!` und gibt wahrheitsgemäß `false` zurück, anstatt das Programm abzubrechen.
  - Auf nicht unterstützten Plattformen gibt `mem_lock` bei `len > 0` gemäß `INV-VAULT-TRAP-1` wahrheitsgemäß `false` zurück.

`EVIDENCE-P3-MLOCK-PROPAGATION`: Fehlerbehandlung und Best-Effort-Rückgabe ohne Panics verifiziert.

---

## 4. Prüfpunkte & Nachweise (P1 - P6)

### P1 SAFE-FASSADE
Ein `grep -rn "^pub fn\|^pub unsafe fn" crates/contextra-sys/src/` ergab 0 Vorkommen von `pub unsafe fn`. Alle öffentlichen Funktionen in der API bieten eine 100% sichere Rust-Fassade.
`EVIDENCE-P1-SAFE-FACADE`: Safe API Facade verifiziert.

### P2 MMAP-SICHERHEIT
Geprüft in `mmap.rs`: Vorab-Prüfung der Dateigröße, Post-Verification der Mapping-Länge, RAII-Garantie durch `memmap2::Mmap`.
`EVIDENCE-P2-MMAP-LIFETIME`: Verified.

### P3 MLOCK-ERROR-PROPAGATION
Geprüft in `mlock.rs` und `vault.rs`: Keine stillschweigenden Verschluckungen; Aufrufstellen erhalten Bools oder `std::io::Result` und verwalten Sperren via RAII (`LockedRegions`).
`EVIDENCE-P3-MLOCK-PROPAGATION`: Verified.

### P4 POSIX-FD-SICHERHEIT
Geprüft in `posix.rs`: `reopen_and_dup2` prüft `libc::open`, `libc::dup2` und `libc::close` auf Rückgabewerte `< 0` und gibt Fehler als `std::io::Error::last_os_error()` zurück. Ungültige File-Deskriptoren führen zu geordneten POSIX-Fehlercodes (`EBADF`) anstelle von Undefined Behavior.
`EVIDENCE-P4-POSIX-FD`: Verified.

### P5 WINDOWS-ACL KORREKTHEIT
Geprüft in `acl_win32.rs`: Für Nicht-Windows-Plattformen existiert eine korrekte Compile-Guard-Abstraktion (`#[cfg(not(windows))]`), die No-Op-Stubs bereitstellt, die ausnahmslos `Ok(())` zurückgeben und sauber kompilieren.
`EVIDENCE-P5-WIN32-ACL`: Verified.

### P6 KEINE FACHCRATE-IMPORTS
Ausführung von `cargo metadata` ergab folgende Abhängigkeiten für `contextra-sys`: `bytes`, `flatbuffers`, `serde`, `serde_json`, `memmap2`, `libc`, `windows-sys`, `tracing`. Es wird keine einzige `contextra-*` Fachcrate importiert (Ring 0 Konformität).
`EVIDENCE-P6-NO-DOMAIN-DEPS`: Verified.

---

## VERDICT

- **Status:** PASSED / CONFORMANT
- **Unsafe-Island Status:** Verified (Ring 0, Unsafe Island)
- **Zero-Panic Guarantee:** Verified (INV-SYS-ZERO-PANIC)
- **Public API Safety:** 100% Safe Rust Facade
- **Dependencies:** Ring 0 compliant (0 domain crate dependencies)

# Unsafe-Inventar: contextra-sys

Erstellt gemäß WP-Y2 (AUFTRAG sys).
Werkzeuge: `tools/unsafe_boundary_auditor.py`, `tools/unsafe_boundary_tracer.py`, `/home/jules/self_created_tools/unsafe_auditor.py`.

## Übersicht
- **Crate:** `contextra-sys` (Ring 0 Unsafe-Insel, `#![allow(unsafe_code)]`, `#![deny(unsafe_op_in_unsafe_fn)]`)
- **Gesamtzahl unsafe-Blöcke:** 22
- **unsafe fn:** 0 (vollständig sichere öffentliche Schnittstelle)
- **unsafe impl:** 0
- **Fehlende `// SAFETY:`-Kommentare:** 0 (100% lückenlos dokumentiert)

## Detailinventar der Unsafe-Stellen

| Nr. | Datei:Zeile | Konstrukt / Aufruf | SAFETY-Dokumentation | Begründung / Invariante |
|---|---|---|---|---|
| 1 | `crates/contextra-sys/src/acl_win32.rs:98` | `let proc_uid = unsafe { libc::geteuid() };` | Vorhanden | // SAFETY: `libc::geteuid` ist ein nebenwirkungsfreier POSIX-Systemaufruf, der stets die effektive UID des aufrufenden Prozesses liefert. Es werden keine Zeiger dereferenziert. |
| 2 | `crates/contextra-sys/src/acl_win32.rs:122` | `unsafe {` | Vorhanden | // SAFETY: `self.0` ist ein gültiges, nicht-nulles Win32-HANDLE, das von `OpenProcessToken` stammt und exklusiv von dieser Guard gehalten wird. |
| 3 | `crates/contextra-sys/src/acl_win32.rs:149` | `unsafe {` | Vorhanden | // SAFETY: `GetCurrentProcess` liefert ein Pseudohandle. `token_handle` ist ein gültiger Zeiger auf ein HANDLE. `GetTokenInformation` wird zunächst zur Größenermittlung mit Null-Puffer und danach mit einem ausreichend großen Puffer aufgerufen. |
| 4 | `crates/contextra-sys/src/acl_win32.rs:211` | `unsafe {` | Vorhanden | // SAFETY: `token_user` zeigt auf den zuvor erfolgreich gelesenen Puffer. Der `owner_sid`-Zeiger wird auf Null geprüft. `acl_buf` ist ausreichend groß dimensioniert für ACL + ACCESS_ALLOWED_ACE + sid_len. `path_wide` ist nullterminiert. |
| 5 | `crates/contextra-sys/src/acl_win32.rs:302` | `unsafe {` | Vorhanden | // SAFETY: `path_wide` ist ein gültiger nullterminierter UTF-16-Puffer. Die Ausgabepointer `p_dacl` und `p_sec_desc` sind gültig und werden vom Win32-Subsystem befüllt. |
| 6 | `crates/contextra-sys/src/acl_win32.rs:328` | `unsafe {` | Vorhanden | // SAFETY: `self.0` ist ein gültiger Zeiger auf einen Windows-Sicherheitsdeskriptor, der von `GetNamedSecurityInfoW` stammt. |
| 7 | `crates/contextra-sys/src/acl_win32.rs:347` | `unsafe {` | Vorhanden | // SAFETY: `p_sec_desc` und `p_dacl` wurden von `GetNamedSecurityInfoW` erfolgreich initialisiert und auf Nicht-Null geprüft. `GetAce` liest den ersten Eintrag (Index 0). `EqualSid` vergleicht zwei gültige SID-Zeiger. |
| 8 | `crates/contextra-sys/src/durability.rs:78` | `let res = unsafe {` | Vorhanden | // SAFETY: `fd` ist ein gültiger, offener Dateideskriptor aus `std::fs::File::as_raw_fd()`. |
| 9 | `crates/contextra-sys/src/durability.rs:102` | `let res = unsafe { libc::fcntl(fd, libc::F_FULLFSYNC, 0) };` | Vorhanden | // SAFETY: `fd` ist ein gültiger Dateideskriptor aus `std::fs::File`. F_FULLFSYNC leert physische APFS-Caches. |
| 10 | `crates/contextra-sys/src/durability.rs:120` | `let res = unsafe { libc::fsync(fd) };` | Vorhanden | // SAFETY: `fd` ist ein gültiger offener Dateideskriptor aus `std::fs::File`. |
| 11 | `crates/contextra-sys/src/durability.rs:144` | `let res = unsafe { windows_sys::Win32::Storage::FileSystem::FlushFileBuffers(handle) };` | Vorhanden | // SAFETY: `handle` ist ein gültiges OS-Datei-Handle aus `std::fs::File::as_raw_handle()`. |
| 12 | `crates/contextra-sys/src/mlock.rs:81` | `let ret = unsafe { libc::madvise(ptr as *mut libc::c_void, len, advice) };` | Vorhanden | // SAFETY: `ptr` ist nicht null und verweist auf `len` Bytes gültig allokierten Speicher. |
| 13 | `crates/contextra-sys/src/mlock.rs:130` | `let ret = unsafe { libc::mlock(ptr as *const libc::c_void, len) };` | Vorhanden | // SAFETY: `ptr` ist nicht null und verweist auf `len` Bytes allokierten Speicher. |
| 14 | `crates/contextra-sys/src/mlock.rs:159` | `let ret = unsafe {` | Vorhanden | // SAFETY: `ptr` ist nicht null und verweist auf `len` Bytes gültig allokierten Speicher. |
| 15 | `crates/contextra-sys/src/mlock.rs:220` | `unsafe {` | Vorhanden | // SAFETY: `addr` und `len` referenzieren eine zuvor gesperrte Region. |
| 16 | `crates/contextra-sys/src/mlock.rs:228` | `unsafe {` | Vorhanden | // SAFETY: `addr` und `len` referenzieren eine zuvor an `VirtualLock` übergebene Region. |
| 17 | `crates/contextra-sys/src/mmap.rs:32` | `unsafe { memmap2::Mmap::map(file) }` | Vorhanden | // SAFETY: `file` ist ein gültiger, geöffneter Read-Only-Dateideskriptor. Die Read-Only-Abbildung verhindert Data Races im Rust-Adressraum. Seitenfehler (Page Faults) werden transparent vom Betriebssystemkernel behandelt. |
| 18 | `crates/contextra-sys/src/posix.rs:39` | `unsafe {` | Vorhanden | // SAFETY: `path_c` ist ein gültiger nullterminierter C-String. `target_fd` ist validiert >= 0. `libc::open`, `libc::dup2` und `libc::close` sind Standard-POSIX-Syscalls, deren Rückgabewerte unmittelbar auf Fehler geprüft werden. |
| 19 | `crates/contextra-sys/src/posix.rs:71` | `unsafe {` | Vorhanden | // SAFETY: `getrlimit` und `setrlimit` lesen und schreiben die `rlimit`-Struktur des Prozesses über gültige Zeiger auf allokierten/uninitialisierten Speicher. |
| 20 | `crates/contextra-sys/src/posix.rs:107` | `let res = unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &rlim) };` | Vorhanden | // SAFETY: `rlim` ist eine vollständig initialisierte C-Struktur auf dem Stack. `setrlimit` liest diese sicher ein. |
| 21 | `crates/contextra-sys/src/posix.rs:165` | `let mut duped_file = unsafe {` | Vorhanden | // SAFETY: `target_fd` wurde via `dup2` ersetzt und ist ein gültiger offener Deskriptor. `libc::dup` erzeugt ein neues dupliziertes Handle für die sichere Übergabe an Rusts `File::from_raw_fd`. |
| 22 | `crates/contextra-sys/src/vault.rs:85` | `unsafe {` | Vorhanden | // SAFETY: `byte` ist eine gültige, ausgerichtete mutable Referenz auf ein `u8` im übergebenen Slice. Volatile Schreibzugriffe verhindern das Wegoptimieren. |

## Befunde und Bewertung (FINDINGS)
- Alle 22 `unsafe`-Blöcke verfügen über fundierte, am Quelltext überprüfbare `// SAFETY:`-Kommentare.
- Keine `unsafe`-Konstrukte lecken nach außen; alle Vorbedingungen werden vor Eintritt in `unsafe` validiert.
- Es gibt keine fehlenden Sicherheitsnachweise; ein Eintrag in `FINDINGS.md` für undokumentiertes `unsafe` ist daher nicht erforderlich.

# Unsafe-Register: contextra-sys

Das Unsafe-Register erfasst alle `unsafe`-Blöcke in `crates/contextra-sys/src` lückenlos.
Jeder Eintrag wird über automatisierte Tests (`crates/contextra-sys/tests/sy_audit_unsafe.rs`) gegen den Quelltext abgeglichen. Ein Drift führt zum Testfehler.

Gesamtanzahl `unsafe`-Blöcke: **18** (Basislimit: <= 57).

## Tabelle der Unsafe-Blöcke

| Datei:Zeile | Funktion / Kontext | Vorbedingung | Begründung / Sicherheit |
|---|---|---|---|
| `crates/contextra-sys/src/durability.rs:80` | `flush_file_sync` (Linux) | `fd >= 0`, `file` gültig | Gültiger Dateideskriptor aus `file.as_raw_fd()`, Deskriptorlebensdauer durch `&File` gesichert; Aufruf von `fdatasync`/`fsync`. |
| `crates/contextra-sys/src/durability.rs:103` | `flush_file_sync` (macOS) | `fd >= 0`, `file` gültig | Deskriptor aus `file.as_raw_fd()`, macOS `fcntl(F_FULLFSYNC, 0)` leert Plattenhardwarecache. |
| `crates/contextra-sys/src/durability.rs:120` | `flush_file_sync` (POSIX) | `fd >= 0`, `file` gültig | Deskriptor aus `file.as_raw_fd()`, POSIX `fsync`. |
| `crates/contextra-sys/src/fs_probe.rs:136` | `fs_class_spec` (Linux) | `c_path` nullterminiert, `stat` Stack-Speicher | `libc::statfs` schreibt in vorbereiteten `stat`-Puffer. |
| `crates/contextra-sys/src/fs_probe.rs:142` | `fs_class_spec` (Linux) | `statfs` Rückgabewert war 0 | Kernel hat `stat` bei Erfolg vollständig initialisiert; `assume_init()` ist wohldefiniert. |
| `crates/contextra-sys/src/fs_probe.rs:182` | `fs_class_spec` (macOS) | `c_path` nullterminiert, `stat` Stack-Speicher | `libc::statfs` schreibt in vorbereiteten `stat`-Puffer auf macOS. |
| `crates/contextra-sys/src/fs_probe.rs:188` | `fs_class_spec` (macOS) | `statfs` Rückgabewert war 0 | Kernel hat `stat` vollständig initialisiert; `assume_init()` ist sicher. |
| `crates/contextra-sys/src/fs_probe.rs:198` | `fs_class_spec` (macOS) | `type_name_bytes` im C-String | Cast von `[c_char]` zu `[u8]` für UTF-8-Prüfung. |
| `crates/contextra-sys/src/hardening.rs:71` | `apply_process_hardening` | `rlim` auf Stack | `libc::setrlimit(RLIMIT_CORE)` mit Werten 0/0 deaktiviert Coredumps. |
| `crates/contextra-sys/src/hardening.rs:78` | `apply_process_hardening` (Linux) | Gültiger Syscall | `libc::prctl(PR_SET_DUMPABLE, 0)` deaktiviert Speicherabzug durch Kernel. |
| `crates/contextra-sys/src/hardening.rs:89` | `apply_process_hardening` | `dummy` Stack-Puffer | Testaufruf `libc::mlock` auf Puffer zur Funktionsprüfung. |
| `crates/contextra-sys/src/hardening.rs:93` | `apply_process_hardening` | `dummy` zuvor gesperrt | Rückgängigmachen des Test-Locks mit `libc::munlock`. |
| `crates/contextra-sys/src/hardening.rs:141` | `mark_dont_dump` (Linux) | `aligned_addr` & `aligned_len` decken Slice ab | `libc::madvise(MADV_DONTDUMP)` schließt Seiten von Coredumps aus; keine Speichermodifikation. |
| `crates/contextra-sys/src/hardening.rs:172` | `secure_zero_volatile` | `byte` gültige &mut Referenz | Fluchtiges Schreiben `write_volatile` verhindert Dead-Store-Eliminierung. |
| `crates/contextra-sys/src/hardening.rs:190` | `lock_pages` | `slice` gültig initialisiert | `libc::mlock` sperrt die gültigen Seiten des Slice im RAM gegen Auslagerung. |
| `crates/contextra-sys/src/hardening.rs:221` | `ensure_fd_limit` | `rlim` Stack-Speicher | `getrlimit`/`setrlimit` für `RLIMIT_NOFILE`; `assume_init()` nach Return 0. |
| `crates/contextra-sys/src/hardening.rs:256` | `set_fd_limit` | `rlim` Stack-Speicher | `libc::setrlimit(RLIMIT_NOFILE, &rlim)` setzt Limit. |
| `crates/contextra-sys/src/perm.rs:55` | `verify_owner_only` | Keine Parameter | `libc::geteuid()` ist seiteneffektfreier POSIX-Syscall. |

# AGENTS.md — `memfuse-sys` (Ring 0)

## Modul-Übersicht & Architektur
`memfuse-sys` ist die isolierte **Unsafe-Insel** (Ring 0) des MemFuse Cognitive OS.
Es kapselt alle OS- und Speicher-Grenzflächenaufrufe (`mmap`, `mlock`, Win32-ACL)
und stellt eine sichere Rust-API bereit.

| Datei | Zweck & Schutzbereich |
| --- | --- |
| `lib.rs` | Ring-0 Export-Schnittstelle, `#![allow(unsafe_code)]`, `#![deny(unsafe_op_in_unsafe_fn)]` |
| `mmap.rs` | Zero-Copy Read-Only Memory-Mapping (`mmap_readonly`) |
| `mlock.rs` | Physische Speichersperre gegen OS-Swap (`mem_lock`, `mem_unlock`) |
| `acl_win32.rs` | Windows Win32 ACL-Sicherheitsrestriktionen (`set_restrictive_file_acl`, `verify_file_acl_owner_only`) |
| `posix.rs` | POSIX System-Call Abstraktionen für File Descriptor Re-Open/Dup2 |

## Sicherheits-Invarianten & Richtlinien
1. **Keine externen Fach-Crates:** `memfuse-sys` liegt in Ring 0 und darf keine anderen MemFuse-Fachcrates importieren.
2. **Unsafe-Kapselung:** Unsafe-Code darf AUSSCHLIESSLICH in `memfuse-sys` vorkommen. Alle öffentlichen Funktionen bieten eine 100% sichere Rust-Fassade.
3. **Zero-Panic-Doctrine:** Keine unkontrollierten Panics in OS-Interaktionen. Fehler werden sauber als `std::io::Result` zurückgegeben.

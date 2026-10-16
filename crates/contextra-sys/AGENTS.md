# AGENTS.md — contextra-sys
> Ring 0 · stable · Quelle: capabilities.toml · Spec: K.27, L.13

1. Zweck
Systemnahe OS-Bindings, mmap, mlock/VirtualLock und Platten-/ACL-Grenzflächen in Ring 0. Der Crate bildet eine Unsafe-Insel, die OS-Systemaufrufe und FFI-Operationen kapselt und als 100% sichere Rust-API für höhere Ringe bereitstellt.

2. Modul-Karte
| Datei/Verzeichnis | Verantwortung |
|---|---|
| `src/lib.rs` | Ring-0 Export-Schnittstelle und Re-Exports aller Systemmodul-Funktionen |
| `src/acl_win32.rs` | Windows Win32 ACL-Sicherheitsfunktionen (`set_restrictive_file_acl`, `verify_file_acl_owner_only`) |
| `src/mlock.rs` | RAM-Speichersperre gegen OS-Swap (`mem_lock`, `mem_unlock`) für Unix, Windows und Fallback |
| `src/mmap.rs` | Zero-Copy Read-Only Memory-Mapping (`mmap_readonly`, `MmapChunk`) |
| `src/posix.rs` | POSIX System-Call Abstraktionen für File Descriptor Re-Open/Dup2 (`reopen_file_read_only`) |
| `src/vault.rs` | `VolatileVault` — Speichersichere Speicherung sensibler Secrets im gesperrten RAM |

3. Invarianten
- `INV-SYS-UNSAFE-SAFETY-DOC`: Jede `unsafe`-Stelle und jede `unsafe fn` MUSS durch einen strukturierten `// SAFETY:`-Kommentar begründet sein.
- `INV-VAULT-TRAP-1`: `mem_lock` gibt auf nicht-Unix/Windows-Systemen bei `len > 0` und Nicht-Null-Zeigern wahrheitsgemäß `false` zurück, um anzuzeigen, dass keine Speichersperrung verfügbar ist.
- `INV-SYS-ZERO-PANIC`: System- und I/O-Grenzflächen geben Fehler als `std::io::Result` zurück und lösen NIEMALS unkontrollierte Panics aus.

4. Verboten / Anti-Patterns
- Direkte `unsafe`-Aufrufe von OS-Systemcalls außerhalb von `contextra-sys` (Unsafe-Insel-Doktrin).
- Aufruf von FFI-Funktionen ohne vorherige Null-Zeiger- und Längen-Validierung.

5. Nebenläufigkeit, Async- und Lock-Regeln
- Reines synchrones I/O und OS-Primitive ohne tokio-/async-Abhängigkeit (Ring 0, Rule P26).
- `MmapChunk` und OS-Handles implementieren `Send` + `Sync`, sofern der OS-Kontext dies sicherstellt.

6. Verifikation
- `cargo test -p contextra-sys`

7. Bekannte Lücken / SOLL
- `mem_lock` erfolgt als "Best-Effort" und schlägt ohne erhöhte Rechte (`RLIMIT_MEMLOCK`) fehl, was mit einer `tracing::warn!`-Meldung protokolliert wird.

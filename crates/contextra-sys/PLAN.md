# Implementierungsplan: contextra-sys

**Ebene:** L0 · **Ziel-Zustand:** einzige Unsafe-Insel für OS-Aufrufe, Dateisystem-Sync, Speichersperren. Nicht-Unsafe-Konsumenten sehen nur sichere Funktionen.
**Basis:** SPEC R14 §2.1, §2.6, §2.7, §13.3 (`memmap2` ohne Feature), DEC-39; R15 §2.3.

## 1. Ist-Befund (gemessen)

- 8 Dateien, 2 204 Zeilen. Module: `acl_win32`, `durability`, `error`, `mlock`, `mmap`, `posix`, `vault`, `mem`, `fsmeta`, `map`.
- `forbid(unsafe_code)` fehlt korrekt; 22 `unsafe`-Vorkommen in den Modulen.
- `Cargo.toml`: `memmap2` **ohne** Feature, `windows-sys` ohne optionales Gate, `criterion` mit `async_tokio` (Dev-Abhängigkeit, verletzt Verbot für Testcode nicht, aber unnötig).
- **Abweichung von der SPEC:** `memmap2` ist Standardabhängigkeit. SPEC verlangt Feature `mmap`, Standard aus (R14 §13.3).
- `vault` ist ein Modul, das nach R14 nicht in §2.1 vorkommt. Prüfen, ob es zum Pseudonym-Tresor gehört (§11.9) und dorthin gehört oder entfernt wird.

## 2. Entfernen

| Ziel | Aktion |
|---|---|
| `memmap2` als Standard | `optional = true`, Feature `mmap` (Standard aus) |
| `mmap` aus dem Default-Export in `lib.rs:56,67` | nur hinter `#[cfg(feature = "mmap")]` |
| `vault` | verschieben nach `contextra-crypto` oder löschen, nach Befund (siehe Schritt 1 unten) |
| `criterion` mit `async_tokio` | auf `criterion` ohne async-Feature umstellen; SPEC verbietet Async im L0 nicht ausdrücklich, aber kein Nutzen |

**Schritt 1 (vor Löschung):** `grep -rn 'vault' crates/` und Lesen von `vault.rs` (bzw. Modul). Ergebnis im Plan festhalten.

## 3. Ändern

- **`lib.rs`:** Crate-Doku aktualisieren. Sie beschreibt heute `memmap2`, `windows-sys` und `mmap` als Standard; Aussage korrigieren.
- **`#![deny(unsafe_op_in_unsafe_fn)]`:** bereits über Workspace-Lint. Zusätzlich prüfen, dass jeder `unsafe`-Block einen `// SAFETY:`-Kommentar hat (`undocumented_unsafe_blocks = deny`, Workspace).
- **`durability.rs`:** `F_FULLFSYNC` (macOS), `fdatasync` (Linux), `FlushFileBuffers` (Windows). Fehler-Rückgabe `Result` mit Errno-Erhalt; kein stilles `Ok`.
- **`posix.rs`:** `fsync` auf Verzeichnisse nach `rename` (AN-04) als eigene Funktion `sync_dir`.
- **`mlock.rs`:** `mlock`/`munlock` mit Rückgabe, ob gesperrt. Ergebnis fließt nur in `HostProbe` (Diagnose), nie in Korrektheit (§2.2).

## 4. Implementieren

1. **`sync_data`, `sync_all`, `sync_dir`** für Linux, macOS, Windows. Je ein Konformanztest in `durable-fs`, nicht hier.
2. **`read_at`/`write_at`** (pread/pwrite, `FileExt`), Windows `seek_read`/`seek_write`. Teil des Pakets P0-B.
3. **`preallocate`:** `posix_fallocate` (Linux), `F_PREALLOCATE` (macOS), Fallback `set_len` mit Hinweis. Rückgabe erkennt `ENOSPC` als eigene Variante (F-2).
4. **`free_space`:** `statvfs` (Unix), `GetDiskFreeSpaceEx` (Windows).
5. **`fsmeta`:** Dateisystemtyp und Gerätekennung (`st_dev`) für `HostProbe` und für `LedgerSameDevice` (§8.9).
6. **`mmap` (Feature):** `mmap_readonly`, `SealedFile`, `SealedMap`. Nur lesend, nur hinter Feature (NZ-9).
7. **`memmap2`-Abhängigkeit** nur in diesem Feature.
8. **Selbsttest der Unsafe-Insel:** Test `unsafe_islands` (DEC-52) prüft, dass genau `sys` und `simd` `unsafe` enthalten.

## 5. Tests

- Pro Plattform eine Test-Datei: `sync_data` auf Datei, dann `sync_dir` nach `rename`; Fehlerinjektion über Dateideskriptor-Dummy.
- Gegentest: Ohne `sync_dir` darf der Test in der Sim-Umgebung scheitern (Nachweis, dass der Test etwas prüft).
- Miri auf reine Logik (`cargo miri test -p contextra-sys`, ohne FFI).

## 6. Abhängigkeiten nach Umbau

Erlaubt: `thiserror`, `tracing`, `libc` (nur hier), `windows-sys` (nur hier, Windows-Target). Verboten im Standard: `memmap2`, `criterion` mit Async.

## 7. Gate und Reihenfolge

1. `memmap2` hinter Feature, Konsumenten (`vector`, `contextra-sys` selbst) anpassen. (P0-B)
2. Sync-Funktionen und `preallocate`/`free_space` implementieren, Tests je Plattform.
3. `vault` klären.
4. Gate **G-0**: Konformanz `Fs` (über `durable-fs`) grün für Linux. Windows bleibt bis zur Freigabe ausgeschlossen (NZ-8).

**Risiko:** Ohne Windows-Konformanz ist `FlushFileBuffers` ungetestet. SPEC §17.3 sperrt Windows-Produktion; der Code bleibt bis dahin hinter `cfg`.

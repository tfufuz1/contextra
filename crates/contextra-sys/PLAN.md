# contextra-sys — Implementierungsspezifikation

**Ebene:** E0 (Fundament)
**Status:** Normativ, abgeleitet aus `docs/SPEC.md` v4.1
**Version:** 4.1
**Geltungsbereich:** `crates/contextra-sys`
**Normative Quellen:** SPEC 3.1 (Zeile `contextra-sys`), 3.2 (R5, R6), 3.3, 3.5, 5.5, 5.6, 5.5 REQ-KEY-6, 8.7, 11.2 (T-OPS-09), 15, ADR-41, ADR-53, ADR-19, G-02, G-03, G-05, G-13, REM-13, REM-14.

---

## 1. Zweck und Ebene

`contextra-sys` ist die **einzige** Stelle des Workspace für Systemaufrufe, die `unsafe` benötigen, außer `contextra-simd` (R5, ADR-53). Das Crate kapselt die Aufrufe in sichere Funktionen mit Fehlertyp. Es enthält keine Logik über Daten, Schlüssel oder Dateiformate.

SPEC 3.1 nennt drei Aufgaben: **mlock** (Speichersperre), **Dumpable** (Core-Dumps aus), **Direct-IO**. Dazu kommen zwei Primitive, die SPEC an anderer Stelle verlangt: der Prozess-Lock auf `data_dir` (SPEC 3.5, 8.7) und die Mapping-Funktion für schreibgeschützte Dateien (SPEC 5.8, Store-Replay).

### 1.1 Ist-Befund (gemessen am HEAD `f8b44b2`)

| Messgröße | Wert | Befund |
|---|---|---|
| Quelldateien | 14 `.rs` inkl. `benches/`, `examples/`, `tests/`, 2.987 Zeilen | Benches und Beispiele gehören nach SPEC 13.1 nicht in dieses Crate |
| `unsafe`-Blöcke in `src/` | 22 (vault 1, acl_win32 7, posix 4, mmap 1, mlock 5, durability 4) | Die Verteilung ist die Grundlage von U-SYS-1 |
| Abhängigkeiten | `memmap2`, `thiserror`, `tracing`, `libc` (Unix-Zielabhängigkeit) | `tracing` entfällt (E-SYS-09). `libc` bleibt |
| Features | keine | ok |
| Fehlende SPEC-Primitive | `PR_SET_DUMPABLE = 0` (ADR-41), `O_DIRECT` (SPEC 3.1), Datei-Lock für `LOCK` (SPEC 3.5, 8.7) | Es gibt kein einziges Vorkommen von `prctl`, `O_DIRECT` oder `flock` |
| Fehlerinjektion im Produktcode | `fault_injection::set_fail_mlock` (globaler Atomar-Zustand), `set_fail_fsync` | Verletzt REM-13. Injektion gehört in `contextra-testkit` über Traits |
| Richtlinie für `mlock` | `MemLockPolicy` mit Warnung statt Verweigerung | ADR-41 verlangt Verweigerung des Starts. Die Richtlinie entfällt |
| Verbraucher außerhalb des Crates | `mmap_readonly`/`Sealed*` in 8 Dateien (7 DiskANN in `vector`, 1 `store/wal/replay.rs`); `DurableFile`, `write_all_durable`, `durable_flush_*`, `LockedRegions`, `VaultBuffer`, `MemoryLocker`, `mem_lock`, `apply_dont_dump`, `posix`-Funktionen, `acl_win32` jeweils 0 | Die meisten Module sind ohne Verbraucher. Die Schlüssel-Sperre (REQ-KEY-6) hat heute keine Implementierung |
| Tests | 48 bestanden (`error_paths`, `integration_tests`) | Grün |
| Namensgebung | `vault` (Modul, `VaultBuffer`, `LockedRegions`), Inline-Module `mem`, `fsmeta`, `map` als „Baukasten-Zielmodule“ | Widerspricht ADR-19 und REM-14. Der Begriff `vault` ist nach SPEC 1.3 ein Altbegriff (REM-01, REM-19) |

Ergebnis: Das Crate enthält viel nicht Benötigtes und fehlt genau die Primitive, auf die der Schlüsselpfad angewiesen ist.

---

## 2. Vertrag (öffentliche API)

Alle Funktionen liefern `Result<_, SysError>`. Es gibt keinen Fallback, der einen Fehler stillschweigend verschluckt. Die Fehlermeldung trägt nie Speicherinhalte, Pfade mit Personenbezug oder Schlüsselbytes.

### 2.1 Speicher (`mlock.rs`, `locked.rs`)

```rust
/// Sperrt Seiten im Arbeitsspeicher (mlock). Fehler verweigern den Start (ADR-41). Kein Fallback.
pub fn lock_pages(ptr: *const u8, len: usize) -> Result<(), SysError>;

/// Hebt die Sperre auf. Fehler werden nicht propagiert, weil der Aufrufer nach Drop nicht mehr reagieren kann.
pub fn unlock_pages(ptr: *const u8, len: usize);

/// Schließt eine Region aus Core-Dumps aus (madvise MADV_DONTDUMP).
pub fn exclude_from_core_dumps(ptr: *const u8, len: usize) -> Result<(), SysError>;

/// Setzt PR_SET_DUMPABLE auf 0 für den Prozess (ADR-41). Linux.
pub fn disable_process_dumps() -> Result<(), SysError>;

/// Abstraktion für Tests (T-OPS-09). Produktionsimplementierung ist `OsMemoryLocker`.
pub trait MemoryLocker: Send + Sync + 'static {
    fn lock(&self, ptr: *const u8, len: usize) -> Result<(), SysError>;
    fn unlock(&self, ptr: *const u8, len: usize);
}
#[derive(Debug, Clone, Copy, Default)]
pub struct OsMemoryLocker;

/// Fester, gesperrter Puffer für Schlüsselmaterial (REQ-KEY-6, ADR-41).
/// Mlock bei Erzeugung, MADV_DONTDUMP, Nullung und Freigabe bei Drop.
/// Kein Clone, kein Debug des Inhalts, kein Serialize.
pub struct LockedBuffer { /* private */ }
impl LockedBuffer {
    pub fn new(len: usize, locker: &dyn MemoryLocker) -> Result<Self, SysError>;
    pub fn as_slice(&self) -> &[u8];
    pub fn as_mut_slice(&mut self) -> &mut [u8];
    pub fn len(&self) -> usize;
}
impl Drop for LockedBuffer { /* zeroize, unlock, dealloc */ }
```

Begründung für die Injektion über `&dyn MemoryLocker`: Der Test T-OPS-09 („mlock nicht möglich: `Refused`“) braucht einen Fehlerpfad ohne globalen Zustand. Ein globaler Schalter im Produktcode ist verboten (REM-13).

### 2.2 Dateien (`io.rs`, `lock.rs`, `sync.rs`)

```rust
/// Öffnet eine Datei mit O_DIRECT (Linux). Ausrichtung der Puffer ist Aufgabe des Aufrufers.
pub fn open_direct(path: &Path, write: bool, create: bool) -> Result<File, SysError>;

/// Datensynchronisation. fdatasync (Linux), F_FULLFSYNC (macOS). Die Plattform-Entscheidung liegt hier.
pub fn sync_data_full(file: &File) -> Result<(), SysError>;

/// Exklusive Prozesssperre auf eine Datei (flock LOCK_EX | LOCK_NB). Gibt bei Drop frei.
pub struct ProcessLock { /* File, privat */ }
impl ProcessLock {
    pub fn acquire(path: &Path) -> Result<Self, SysError>;   // `LockHeld`, wenn belegt
}
```

`sync_dir` (`fsync` auf ein Verzeichnis) ist eine Standardbibliotheksoperation (`File::open(dir)?.sync_all()`) und gehört zu `contextra-durable-fs`, nicht hierher.

### 2.3 Speicherabbildung (`map.rs`)

```rust
/// Schreibgeschützte Abbildung einer Datei (memmap2). Deref zu `[u8]`. Null-Länge-Dateien liefern eine leere Abbildung.
pub struct ReadOnlyMap { /* memmap2::Mmap oder leer */ }
impl Deref for ReadOnlyMap { type Target = [u8]; /* ... */ }

pub fn map_readonly(file: &File) -> Result<ReadOnlyMap, SysError>;
```

### 2.4 Fehler (`error.rs`)

```rust
#[non_exhaustive]
pub enum SysError {
    MemoryLockFailed { errno: i32 },
    MemoryUnlockFailed { errno: i32 },
    CoreDumpPolicyFailed { errno: i32 },
    DirectIoUnsupported,
    LockHeld,                          // ProcessLock: Datei bereits gesperrt
    Sync { errno: i32 },
    Map { errno: i32 },
    Io(std::io::ErrorKind),
}
```

Die Variante `Io` trägt nur den `ErrorKind`, nie den Pfad.

---

## 3. Abhängigkeiten

| Crate | Status | Begründung |
|---|---|---|
| `libc` | behalten (nur `cfg(unix)`) | Systemaufrufe `mlock`, `madvise`, `prctl`, `flock`, `fcntl` |
| `memmap2` | behalten | Abbildung in `map.rs`. Die `unsafe`-Stelle `Mmap::map` bleibt in `sys` |
| `thiserror` | behalten | `SysError` |
| `zeroize` | **hinzufügen** | `LockedBuffer` (Nullung bei Drop) |
| `tracing` | **entfernen** (E-SYS-09) | Kein Logging in `sys`. Fehler gehen als Wert nach oben |

Interne Abhängigkeiten: keine (SPEC 3.3, `sys` = `[]`).

---

## 4. Module

```
src/
  lib.rs           # #![deny(unsafe_op_in_unsafe_fn)], Re-Exports, Modul-Dokumentation. `forbid` nicht möglich, siehe 4.1
  error.rs         # SysError
  mlock.rs         # lock_pages, unlock_pages, exclude_from_core_dumps, disable_process_dumps
  locker.rs        # MemoryLocker, OsMemoryLocker
  locked.rs        # LockedBuffer
  io.rs            # open_direct, sync_data_full
  lock.rs          # ProcessLock
  map.rs           # ReadOnlyMap, map_readonly
tests/
  memory.rs        # T-SYS-01, T-SYS-02, T-SYS-03
  process_lock.rs  # T-SYS-04
  io.rs            # T-SYS-05, T-SYS-07
  map.rs           # T-SYS-06
  unsafe_islands.rs # T-SYS-08 (Allowlist für Test-Targets)
```

### 4.1 `unsafe`-Regel

R5 verlangt `#![forbid(unsafe_code)]` außer in `sys` und `simd`. In `sys` ist `unsafe` erlaubt, aber nur in den Modulen `mlock.rs`, `io.rs`, `lock.rs`, `map.rs`. Jeder `unsafe`-Block trägt einen `// SAFETY:`-Kommentar mit der Bedingung, die der Aufruf einhält. `#![deny(unsafe_op_in_unsafe_fn)]` ist gesetzt. `forbid` ist nicht möglich, weil das Crate die Aufrufe selbst macht.

---

## 5. Entfernungen

| ID | Entfernen | Ort | Grund | Ersatz | Verbraucher |
|---|---|---|---|---|---|
| E-SYS-01 | `acl_win32.rs` (7 `unsafe`, `set_restrictive_file_acl`, `verify_file_acl_owner_only`) | `src/acl_win32.rs`, `fsmeta` | Kein Verbraucher. Windows ist nicht Zielplattform (SPEC 1.2 nennt Linux-Hardware). Dateirechte setzt der Betrieb | keiner. Dateirechte werden über `0600`-Prüfung in `contextra` (`KeyFile`, SPEC REQ-KEY-7) geprüft | 0 |
| E-SYS-02 | `posix.rs` (`set_fd_limit`, `ensure_fd_limit`, `reopen_and_dup2`, 4 `unsafe`) | `src/posix.rs` | Kein Verbraucher. Kein SPEC-Bedarf | keiner | 0 |
| E-SYS-03 | `durability.rs` vollständig (`DurableFile`, `write_all_durable`, `durable_flush_file`, `durable_flush_dir`, `set_fail_fsync`) | `src/durability.rs` | Duplikat zu `contextra-durable-fs` (SPEC 3.3, `durable-fs` besitzt Durabilität). Globale Fehlerinjektion verletzt REM-13 | `sync_data_full` in `io.rs`, `sync_dir` in `durable-fs` | 0 |
| E-SYS-04 | `vault.rs` (`LockedRegions`, `VaultBuffer`) | `src/vault.rs` | Name `vault` ist ein Altbegriff (REM-01, REM-19). Die Funktionalität ersetzt `LockedBuffer` mit Drop-Nullung und `&dyn MemoryLocker` | `locked.rs` | 0 |
| E-SYS-05 | `mlock.rs`-Fehlerinjektion (`fault_injection`, `set_fail_mlock`, `should_fail_mlock`) | `src/mlock.rs` | REM-13. Injektion über `MemoryLocker`-Trait | `&dyn MemoryLocker` | 0 |
| E-SYS-06 | `MemLockPolicy` mit Fallback (`mem_lock_typed`, `mem_lock_with_policy`, `mem_lock` mit `bool`) | `src/mlock.rs` | ADR-41: Start verweigert, kein Fallback. Die Richtlinie widerspricht der Entscheidung | `lock_pages -> Result` | 0 |
| E-SYS-07 | `contract.rs` (`NoopMemoryLocker`) | `src/contract.rs` | ADR-19 (Selbstdeklaration), REM-14. Der Noop-Locker liegt in `contextra-testkit` | `testkit::NoopLocker` | 0 |
| E-SYS-08 | Inline-Module `mem`, `fsmeta`, `map` in `lib.rs` mit LINK- und SECTION-Kommentaren | `src/lib.rs` | „Baukasten-Zielmodule“ ohne Funktion. REM-14 | flache Re-Exports | 0 |
| E-SYS-09 | Abhängigkeit `tracing` | `Cargo.toml` | Kein Logging in `sys` (T-LOG-01: Protokolle ohne Nutzdaten, hier gibt es keine Protokolle) | keine | 0 |
| E-SYS-10 | `SealedFile`, `SealedMap`, `SealedReadOnly`, `Unsealed`, `new_verified`, `verified_len`, `get_subslice` | `src/mmap.rs` | Altes Siegel-Modell (INV-IO-1 aus Altdokumentation). Der Seal-Begriff der SPEC ist ein anderer (Kapitel 4) | `map_readonly` | 8 (davon 7 in DiskANN, entfernt durch REM-09) |
| E-SYS-11 | `examples/` (2 Dateien), `benches/sys_bench.rs` | Crate-Verzeichnis | Benchmarks liegen in `contextra-bench` (SPEC 3.1). Beispiele gehören nach SPEC 13.1 nicht in den Produktbaum | keine | 0 |
| E-SYS-12 | ANCHOR-, LINK-, SECTION-, NOTE-Kommentare (`ANCHOR[id=...,epic=...]`) in allen Dateien | alle `src/` | ADR-19, REM-14 | Dokumentationskommentare | |
| E-SYS-13 | `tests/integration_tests.rs`, `tests/error_paths.rs` in der bestehenden Form | `tests/` | Testen entfernte Funktionen | `tests/memory.rs` u. a. | |

---

## 6. Umbau (je Schritt ein Commit, Workspace grün)

| Schritt | Inhalt | Verbraucher-Anpassung | Prüfung |
|---|---|---|---|
| U-SYS-1 | Verbraucherlose Module entfernen: `acl_win32.rs`, `posix.rs`, `durability.rs`, `vault.rs`, `contract.rs`, Inline-Module in `lib.rs` (E-SYS-01 bis E-SYS-04, E-SYS-07, E-SYS-08) | keine (Verbraucher 0) | `cargo check --workspace` |
| U-SYS-2 | Fehlerinjektion und Policy aus `mlock.rs` entfernen (E-SYS-05, E-SYS-06). Neue Funktionen `lock_pages`, `unlock_pages`, `exclude_from_core_dumps` mit `Result`. `MemoryLocker` und `OsMemoryLocker` in `locker.rs` | keine | `cargo check -p contextra-sys` |
| U-SYS-3 | `mmap.rs` auf `map_readonly`/`ReadOnlyMap` reduzieren (E-SYS-10). Verbraucher `store/src/wal/replay.rs` auf `map_readonly` umstellen. DiskANN-Verbraucher entfallen mit REM-09 in `vector` (siehe `vector`-PLAN) | 1 Datei in `store` | `cargo check -p contextra-store` |
| U-SYS-4 | Neue Primitive: `disable_process_dumps` (A-SYS-01), `open_direct` (A-SYS-02), `ProcessLock` (A-SYS-03), `LockedBuffer` (A-SYS-04), `sync_data_full` (A-SYS-05) | keine, Verbraucher kommen mit den PLANs von `crypto`, `store`, `durable-fs` | Tests aus Abschnitt 7 |
| U-SYS-5 | `tracing` entfernen (E-SYS-09). `zeroize` und `libc` (cfg unix) als Abhängigkeiten. `Cargo.toml` aufräumen | keine | `cargo tree -p contextra-sys` zeigt `libc`, `memmap2`, `thiserror`, `zeroize` |
| U-SYS-6 | `examples/` und `benches/` löschen (E-SYS-11) | keine | `cargo build -p contextra-sys --all-targets` |
| U-SYS-7 | Tests neu (`tests/memory.rs`, `process_lock.rs`, `io.rs`, `map.rs`, `unsafe_islands.rs`). Alte Testdateien löschen (E-SYS-13) | keine | `cargo test -p contextra-sys` |

**Abhängigkeit zu anderen PLANs.** `durable-fs` verliert `durability`-Funktionen aus `sys` und übernimmt `sync_dir`. `crypto` braucht `LockedBuffer` (U-SYS-4). Die Reihenfolge ist: U-SYS-1 bis U-SYS-3 zuerst, dann `durable-fs`, dann U-SYS-4 zusammen mit `crypto`.

---

## 7. Tests

| ID | Prüfung | Ort |
|---|---|---|
| T-SYS-01 | Nach `disable_process_dumps()` liefert `prctl(PR_GET_DUMPABLE)` den Wert 0 (Test-Target, nach ADR-53 erlaubt) | `tests/memory.rs` |
| T-SYS-02 | `lock_pages` auf eine 4-KiB-Region gelingt. Bei `RLIMIT_MEMLOCK = 0` in einem Kindprozess liefert es `MemoryLockFailed { errno: ENOMEM|EPERM }` ohne Fallback | `tests/memory.rs` (Kindprozess über `std::process::Command` mit Umgebungsvariable) |
| T-SYS-03 | `LockedBuffer::new` mit einem `MemoryLocker`-Double, das `Err` liefert, gibt `Err` zurück und hinterlässt keinen Puffer. Mit `OsMemoryLocker` ist `len()` korrekt | `tests/memory.rs` |
| T-SYS-04 | `ProcessLock::acquire` auf dieselbe Datei aus einem Kindprozess liefert `LockHeld`. Nach Drop ist die Sperre frei | `tests/process_lock.rs` |
| T-SYS-05 | `open_direct` auf einem ext4-Pfad schreibt einen 4-KiB-ausgerichteten Block. Auf einem Dateisystem ohne O_DIRECT liefert es `DirectIoUnsupported`. Test wird auf CI-Runner mit ext4 ausgeführt, sonst als `#[ignore]` markiert | `tests/io.rs` |
| T-SYS-06 | `map_readonly` auf eine leere Datei liefert Länge 0. Auf einer 1-MiB-Datei liefert es denselben Inhalt wie `std::fs::read` | `tests/map.rs` |
| T-SYS-07 | `sync_data_full` auf einer regulären Datei liefert `Ok` | `tests/io.rs` |
| T-SYS-08 | Ein Test liest den Quellbaum und prüft, dass `unsafe` nur in `mlock.rs`, `io.rs`, `lock.rs`, `map.rs` steht (Allowlist aus Abschnitt 4.1) | `tests/unsafe_islands.rs` |
| T-SYS-09 | `Debug`/`Display` von `SysError` enthalten keine Pfade, keine Byte-Sequenzen aus Puffern (Property-Test mit Pfad-Canary) | `tests/memory.rs` |
| T-SYS-10 | `LockedBuffer` nullt beim Drop den Speicher. Test über `zeroize_slice` einzeln und über einen Drop-Zähler im Test-Feature | `tests/memory.rs` |

**Bekannte Lücke:** T-SYS-02 und T-SYS-04 brauchen Kindprozesse. Der Test wird als Integrationstest mit eigenem Binary (`[[test]]` mit `harness = false`) ausgeführt, damit `RLIMIT_MEMLOCK` nicht den Testlauf stört.

---

## 8. Abnahme

| Gate | Prüfung für `contextra-sys` |
|---|---|
| G-01 | `cargo check`, `cargo test`, `cargo clippy -- -D warnings` auf Linux, Toolchain 1.89.0 |
| G-02 | `unsafe` nur in `mlock.rs`, `io.rs`, `lock.rs`, `map.rs` (T-SYS-08). `#![deny(unsafe_op_in_unsafe_fn)]` gesetzt |
| G-03 | `sys` hat keine Workspace-Abhängigkeit |
| G-05 | `cargo tree -p contextra-sys` enthält keine Netzwerk-Crates |
| G-06 | `default = []` (keine Features) |
| G-13 | keine `unwrap`/`expect`/`panic!` in `src/` außerhalb `#[cfg(test)]` |
| ADR-41 | `disable_process_dumps` und `lock_pages` sind vorhanden und ohne Fallback (T-SYS-01, T-SYS-02) |
| SPEC 3.5 | `ProcessLock` existiert und ist mit dem Verzeichnis `data_dir/LOCK` nutzbar (T-SYS-04) |

**Reihenfolge im Repository:** Nach dem `env`-PLAN als dritter Commit. Die Umsetzung U-SYS-1 bis U-SYS-7 folgt. `durable-fs` setzt auf U-SYS-1 bis U-SYS-3 auf.

# `contextra-sys` – Implementierungsspezifikation

|Feld|Wert|
|---|---|
|Zweck-Satz (D-97)|Einzige `unsafe`-Stelle für Betriebssystemaufrufe: `fsync`, `mlock`, Prozesshärtung.|
|Ring|1|
|Erlaubte Workspace-Abhängigkeiten|`types` (SPEC 5.3); Verbraucher: `durable-fs`, `store`, `engine` (nur Prozesshärtung, D-88)|
|Stufe / Gates|S0; G-B, G-C, G-F|
|Arbeitspakete|WP-0.3 (Aufrufer von `mmap_readonly`), WP-1.2 (Windows, `mmap`), WP-5.2 (Prozesshärtung)|
|SPEC-Anker|8.3, D-29, D-63, D-84, D-94, D-55, D-56, AR-8, F-18, F-22|

## 1. Ist-Zustand (HEAD `1454bbec0`) [V]

|Punkt|Befund|
|---|---|
|Umfang|2.035 Zeilen: `durability.rs`, `mlock.rs`, `mmap.rs` (169 Z.), `posix.rs`, `vault.rs`, `acl_win32.rs` (536 Z.), `lib.rs`|
|Öffentliche Funktionen|`durable_flush_file`, `durable_flush_dir`, `write_all_durable`, `flush_file`, `flush_dir`, `fs_class`, `rename_durable`; `mem_lock_with_policy`/`mem_unlock`/`mem_lock` (alle `unsafe fn`), `mem_lock_slice`, `mem_lock_slice_tolerant`; `mmap_readonly` (`unsafe fn`); `reopen_and_dup2` (`unsafe fn`); `ensure_fd_limit`, `set_fd_limit`; ein `MemLockGuard`-artiger Typ mit `lock_slice`, `zeroize_slice`, `unlock_all`; `set_restrictive_file_acl`, `verify_file_acl_owner_only` (Windows- und POSIX-Varianten)|
|Härtung|Die Funktionen mit offener Vorbedingung sind `pub unsafe fn` (Commit `9f1e77633`). Folge: Aufrufer in `store` (`wal/replay.rs`) und `vector` (`persistence/mmap.rs`) brechen den Build, weil sie `forbid(unsafe_code)` tragen (F-32).|
|Windows|`acl_win32.rs` und 24 `cfg(windows)`-Stellen (F-22)|
|Abhängigkeiten|`memmap2`, `tracing`|
|Tests|30/30 Lib-Tests bestanden **[M]** (3.0.0)|

## 2. Verantwortung und Abgrenzung

`sys` ist die **einzige** Stelle für Betriebssystemaufrufe mit `unsafe` (neben SIMD in `simd`). Sie bietet kleine, einzeln begründete Primitive:

- `fsync`/`fdatasync`/`F_FULLFSYNC`, `flush_dir`, `rename_durable` (Basis von D-55, D-56),
- Prozesshärtung nach D-63: `RLIMIT_CORE = 0`, `PR_SET_DUMPABLE = 0` (Linux), `madvise(MADV_DONTDUMP)`, `mlock` best effort,
- Dateirechte (POSIX, 0700/0600),
- Dateisystemklassifikation für die Startprobe (`statfs`-Typ).

**Nicht** in `sys`: Speicherabbildung (`mmap`, D-94), Windows-Code (D-84), jegliche Geschäftslogik, Fehlerwiederholung. Jede Funktion, deren Sicherheit von einer Vorbedingung des Aufrufers abhängt, ist `unsafe fn`; sonst ist sie eine sichere Funktion mit **geprüften** Vorbedingungen (D-29).

## 3. Zielstruktur (Module) [E]

|Modul|Inhalt|
|---|---|
|`durability`|`flush_file(&File, FlushKind)`, `flush_dir(&Path)`, `rename_durable(from, to)`, `write_all_durable`; macOS `F_FULLFSYNC`|
|`fs_probe`|`fs_class(path) -> FsClass` (Typ, `rotational`, COW-Kennzeichen) als Eingabe der Startprobe (D-56)|
|`hardening`|`apply(policy: Hardening) -> HardeningReport` (`RLIMIT_CORE`, `PR_SET_DUMPABLE`, `fd`-Limit); `dont_dump(slice)`, `lock(slice, policy)`|
|`perm`|`set_mode(path, 0o700/0o600)`, `verify_owner_only(path)` (POSIX)|
|`lib.rs`|Crate-Kopf (Q-01); Re-Exporte; `#![deny(unsafe_op_in_unsafe_fn)]`|

Entfallen: `mmap.rs`, `acl_win32.rs`, jedes `cfg(windows)`; `vault.rs` und `reopen_and_dup2` sind zu prüfen (kein Kernkonsument ⇒ entfernen, P8).

## 4. Schnittstellen [E]

```rust
pub enum FlushKind { Data, All }
pub fn flush_file(f: &File, k: FlushKind) -> Result<(), SyncFailure>;   // SyncFailure: nie in Ok umdeutbar (D-55)
pub fn flush_dir(d: &Path) -> Result<(), SyncFailure>;
pub fn rename_durable(from: &Path, to: &Path) -> Result<(), SyncFailure>; // rename, dann flush_dir(parent)

pub enum FsKind { Ext4, Xfs, Apfs, Btrfs, Zfs, Unsupported(UnsupportedKind) }
pub struct FsClass { pub kind: FsKind, pub rotational: Option<bool>, pub cow: bool, pub journal_data: bool }
pub fn fs_class(p: &Path) -> Result<FsClass, SysError>;

pub enum HardeningPolicy { Required, BestEffort, Off }       // D-63; Standard BestEffort
pub struct HardeningReport { pub core_limit_zero: bool, pub dumpable_off: bool, pub dontdump_ok: bool, pub mlock_ok: bool }
pub fn apply_process_hardening(p: HardeningPolicy) -> Result<HardeningReport, SysError>; // Required: Fehler bei Teilerfolg
/// Sicher, weil `slice` für die Dauer des Aufrufs lebt; die Vorbedingung des Kernels wird geprüft.
pub fn mark_dont_dump(slice: &[u8]) -> bool;
pub fn lock_pages(slice: &[u8], p: HardeningPolicy) -> Result<bool, SysError>;
```

Rohzeiger-basierte Primitive (`mem_lock`, `mem_unlock`, Adressen als `usize`) bleiben `unsafe fn` und werden **nur** von den sicheren Slice-Wrappern in `sys` selbst benutzt; sie werden nicht mehr öffentlich exportiert.

## 5. Implementierungsvorgaben

|ID|Vorgabe|Quelle|Test|
|---|---|---|---|
|SY-01|Jeder `unsafe`-Block trägt einen `// SAFETY:`-Kommentar mit konkreter Vorbedingung; `undocumented_unsafe_blocks = deny`.|Workspace-Lints, D-29|T-UNSAFE-1|
|SY-02|Funktionen mit offener Vorbedingung sind `unsafe fn` oder entfallen. Kein Crate mit `forbid(unsafe_code)` ruft eine `unsafe fn` aus `sys` oder `simd` auf; die Lösung ist, die Funktion nicht anzubieten.|D-29, D-94|T-UNSAFE-1|
|SY-03|Kein `mmap`, kein `memmap2` in `sys`; Lesezugriffe laufen über `Vfs`-Positionslesen.|D-94, D-84|T-UNSAFE-1, T-LINT-2|
|SY-04|Kein Windows-Code, kein `cfg(windows)`; nicht unterstützte Plattformen liefern beim Bauen oder in `fs_class` einen eindeutigen Fehler (`UnsupportedFilesystem` im Verbraucher).|D-56, D-84|T-PLAT-1|
|SY-05|Ein Fehler aus `fsync`/`fdatasync`/`F_FULLFSYNC`/`flush_dir` wird **unverändert** nach oben gemeldet; `sys` wiederholt nie und verschluckt nichts. (Das Nicht-Wiederholen auf demselben Handle erzwingt `durable-fs`.)|D-55, I-10|T-FSYNC-1|
|SY-06|macOS: `F_FULLFSYNC` statt `fsync`; Linux: `fdatasync` für `sync_data`, `fsync` für `sync_all`; `flush_dir` öffnet das Verzeichnis und synchronisiert es.|D-56, 14.7|T-PLAT-1|
|SY-07|Prozesshärtung nach D-63: `RLIMIT_CORE = 0`, `PR_SET_DUMPABLE = 0` (nur Linux), `MADV_DONTDUMP`, `mlock` best effort. `Required` verweigert bei einer gescheiterten Maßnahme; das Ergebnis (`HardeningReport`) fließt in `health()` und Bericht (`ProcessMemory`).|D-63, D-64|T-DUMP-1, T-SEC-2|
|SY-08|`RLIMIT_CORE` und `PR_SET_DUMPABLE` sind prozessglobal; die Rustdoc nennt diese Nebenwirkung (Debugger, `perf`, Crash-Reporter).|D-63|Doc-Test|
|SY-09|Dateimodi: Verzeichnisse 0700, Dateien 0600; `verify_owner_only` prüft Eigentümer und Modus.|5.6, 14.1|T-OPS-2|
|SY-10|Keine Panics: keine Indexierung, kein `unwrap`; alle Fehler als `SysError`/`SyncFailure` mit festem Text, ohne Pfad im `Display`.|P6, HR-4|T-LOG-1|
|SY-11|Keine Netzwerk- oder Zeitzugriffe; die Fassung bleibt `types` + `tracing` + `libc`-nahe Abhängigkeit (Ziel: ohne `memmap2`).|AR-7, P8|`cargo tree`, T-ARCH-4|

## 6. Rückbau und Migration

|Position|Maßnahme|WP|Nachweis|
|---|---|---|---|
|`mmap.rs`, `memmap2`-Abhängigkeit|entfernen. Aufrufer: `store/src/wal/replay.rs:158,210`, `vector/src/persistence/mmap.rs:24` stellen auf `Vfs::read_at` um.|WP-0.3, WP-1.4|`check --locked` grün (G-A); Grep `mmap` = 0|
|`acl_win32.rs`, `cfg(windows)` (24 Stellen)|entfernen; POSIX-Varianten behalten|WP-1.2|Grep = 0|
|`vault.rs`, `reopen_and_dup2`, `ensure_fd_limit`/`set_fd_limit`|Kernkonsument prüfen; ohne Beleg entfernen (P8). `ensure_fd_limit` kann in `hardening` aufgehen.|WP-1.2|Grep|
|`mem_lock*` mit Rohadressen (`usize`)|nicht mehr öffentlich; Slice-Wrapper bleiben|WP-5.2|T-UNSAFE-1|
|`flush_*`-Signaturen mit `std::io::Result`|auf `SyncFailure` umstellen (nicht in `Ok` umdeutbar)|WP-1.4|T-FSYNC-1|

## 7. Arbeitspakete

1. WP-0.3 (G-A): `mmap_readonly`-Aufrufer ablösen, `Cargo.lock` erneuern.
2. WP-1.2: Windows-Code und `mmap.rs` entfernen.
3. WP-5.2 (rot): T-DUMP-1 (Coredump-Prüfung mit Kindprozess), T-SEC-2; grün: `hardening`-Modul.
4. WP-1.4: `SyncFailure` einführen, `durable-fs` anbinden.

## 8. Tests und Benchmarks

|ID|Prüft|
|---|---|
|T-UNSAFE-1|`unsafe` nur in `sys` und `simd`; Safe-Funktionen ohne offene Vorbedingung; `// SAFETY:` vorhanden|
|T-FSYNC-1|Fehlerpfad: ein injizierter Fehler wird unverändert gemeldet (mit `FaultVfs` in `durable-fs`)|
|T-DUMP-1, T-SEC-2|Prozesshärtung: Kindprozess mit `Required`/`BestEffort`/`Off`; Kernel-Limits und `dumpable` geprüft; Schlüsselseiten mit `MADV_DONTDUMP`|
|T-PLAT-1|`fs_class` klassifiziert ext4/XFS/APFS/btrfs/ZFS; unbekannte Typen ⇒ `Unsupported`|
|Sanitizer/Miri|`asan` auf `sys`; `miri` für reine Hilfsfunktionen (SPEC 15.1)|

## 9. Abnahme

Definition of Done (Index Abschnitt 8) und: Grep `windows|mmap|acl_win32` = 0; `cargo geiger`-artige Zählung der `unsafe`-Blöcke dokumentiert und gegenüber Basis (57 Treffer am HEAD der SPEC) nicht gewachsen; jede verbleibende `unsafe fn` hat einen Rustdoc-Abschnitt `# Safety`.

## 10. Offene Punkte

- L-4 (Atomarität von 80-Byte-Slots und Probengröße 64 B) berührt die Startprobe: `fs_class` liefert nur die Klasse, die Atomaritätsprobe selbst läuft in `durable-fs` über `Vfs`.
- Ob `F_FULLFSYNC` allein reicht oder zusätzlich `fcntl(F_BARRIERFSYNC)` zu bewerten ist, bleibt eine Tier-2-Frage (macOS) und gehört in den Evidenzlauf von G-F.

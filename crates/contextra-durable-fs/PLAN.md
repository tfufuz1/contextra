# contextra-durable-fs — Implementierungsspezifikation

**Ebene:** E0 (Fundament)
**Status:** Normativ, abgeleitet aus `docs/SPEC.md` v4.1
**Version:** 4.1
**Geltungsbereich:** `crates/contextra-durable-fs`
**Normative Quellen:** SPEC 3.1 (Zeile `contextra-durable-fs`), 3.2 (R3, R6), 3.3, 3.5, 5.8, 7.9, 8.1, 8.7, 12.6, 15, ADR-04, ADR-19, ADR-41, ADR-42, ADR-44, G-01, G-03, G-05, G-06, G-12, G-13, REM-07, REM-13, REM-14, REM-10, REM-09.

---

## 1. Zweck und Ebene

`contextra-durable-fs` ist die Schicht zwischen den Speicher-Crates und dem Dateisystem. Sie stellt atomares Ersetzen, Verzeichnis-Synchronisation, dauerhaftes Umbenennen und Entfernen, die Prozesssperre des Datenverzeichnisses und die Klassifikation des Dateisystems bereit.

Das Crate kennt keine Dateiformate, keine Schlüssel, keine Zustände des Systems (`ReadOnly`, `Quarantine`, `Rebuilding` gehören `db`, SPEC 8.1). Die Fehlerinjektion liegt in `contextra-testkit`, nicht hier (REM-13).

### 1.1 Ist-Befund (gemessen am HEAD `f8b44b2`)

| Messgröße | Wert | Befund |
|---|---|---|
| Quelldateien `src/` | 9 Dateien, 1.910 Zeilen | `lib.rs` 455, `journal.rs` 436, `scrub.rs` 318, `fault_injector.rs` 300, `health.rs` 139, `vfs.rs` 141, `error.rs` 73, `platform.rs` 48 |
| Tests | 14 Dateien, 43 bestanden | Viele testen entfernte Module (Journal, Scrub, Health, Fault-Injektion) |
| Features | `default = [std, crc]`, `crc`, `serde`, `tracing`, `inspect`, `std` | `crc` entfällt (SPEC 12.6). `serde`, `tracing`, `inspect` sind nicht SPEC-Bestand |
| Abhängigkeiten | `thiserror`, `serde`, `tracing`, `crc32fast` | `crc32fast`, `serde`, `tracing` entfallen (E-DFS-08). Neu: `contextra-sys` |
| `thread::sleep` | in `scrub.rs` (Drosselung) | Verstößt gegen REM-07 und REQ-TIM-2 |
| Fehlerinjektion im Produktcode | `FaultInjector`, `NoFault`, `inject_fsync_failure`, `inject_rename_failure`, `inject_dir_fsync_failure` | REM-13. Gehört nach `contextra-testkit` |
| Zustand im Produktcode | `PoisonedHandle`, `DurableHandle`, `AtomicHealthState`, `HealthState`, `HealthEvent` | Zustandsautomat der Datenwahrheit liegt nach SPEC 8.1 in `db`. REM-13 |
| Verbraucher außerhalb | `Vfs`/`StdVfs`/`RealFs` in 42 Dateien; `FlushKind` in 3; `FaultInjector` in 4; Health in 4; Journal-Typen in 10 (alle in `contextra-erasure/src/chain/`); Scrub in 5 (`vector/src/purge.rs`, `store/src/lsm/erasure.rs`) | Journal-Verbraucher sind alter Saga-Code (REM-10). Scrub-Verbraucher sind DiskANN und altes Erasure (REM-09, REM-10) |
| `unsafe` | keiner | gut. Wird durch `sys` ersetzt (`fdatasync`, `flock`) |
| `fs_class` / `is_supported_local` | vorhanden | nützlich für die Prüfung „kein Netzwerkdateisystem“ (SPEC 9) |
| Atomares Ersetzen | `atomic_replace`, `sync_dir`, `rename_durable` vorhanden | Richtige Grundlage, aber die Reihenfolge ist in Teilen nur über den Fault-Injector geprüft |

Ergebnis: Die Kernfunktionen (atomar ersetzen, Verzeichnis synchronisieren, umbenennen) sind vorhanden und richtig. Der Rest des Crates ist Altlast: Zustandsautomat, Fehlerinjektion, Journal und Scrub.

---

## 2. Vertrag (öffentliche API)

```rust
/// Dateisystemzugriff. Einzige Implementierung im Produktbau: `StdVfs`. Testkit implementiert `FaultVfs`.
pub trait Vfs: Send + Sync + 'static {
    /// Liest eine Datei vollständig.
    fn read(&self, path: &Path) -> Result<Vec<u8>>;
    /// Schreibt `data` atomar nach `path`: tmp-Datei, `sync_data`, `rename`, `sync_dir` des Elternverzeichnisses.
    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()>;
    /// Benennt um und synchronisiert das Zielverzeichnis. Ersetzt ein vorhandenes Ziel.
    fn rename_durable(&self, from: &Path, to: &Path) -> Result<()>;
    /// Entfernt eine Datei und synchronisiert das Elternverzeichnis.
    fn remove_durable(&self, path: &Path) -> Result<()>;
    /// Synchronisiert ein Verzeichnis.
    fn sync_dir(&self, dir: &Path) -> Result<()>;
    /// Synchronisiert eine geöffnete Datei (fdatasync; F_FULLFSYNC auf macOS).
    fn sync_file(&self, file: &File) -> Result<()>;
    /// Legt ein Verzeichnis an, einschließlich Eltern, und synchronisiert das übergeordnete Verzeichnis.
    fn create_dir_durable(&self, dir: &Path) -> Result<()>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StdVfs;
impl Vfs for StdVfs {}

/// Exklusive Prozesssperre auf ein Verzeichnis (`data_dir/LOCK`, SPEC 3.5). Freigabe bei Drop.
pub struct DirLock { /* sys::ProcessLock */ }
impl DirLock {
    pub fn acquire(dir: &Path) -> Result<Self>;   // Fehler: DurableFsError::Locked
}

/// Klassifikation eines Dateisystems für die Startprüfung (SPEC 9, C0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsClass { Local, Network, Unknown }
pub fn classify_fs(path: &Path) -> Result<FsClass>;

/// Fehler. Keine Pfade in der Anzeige (T-SYS-09 gilt sinngemäß).
#[non_exhaustive]
pub enum DurableFsError {
    Io(std::io::ErrorKind),
    Locked,
    Sync { errno: i32 },
    Rename,
    Unsupported(&'static str),
}
pub type Result<T> = core::result::Result<T, DurableFsError>;
```

**Begründung der Reduktion.** Die Methodenmenge ist auf das Notwendige begrenzt, die in SPEC 7.4, 5.6, 5.7 und 5.8 vorkommen. `read`, `write_atomic`, `rename_durable`, `remove_durable`, `sync_dir` und `create_dir_durable` decken die Anforderungen von Anker, KeyTable, Gate-Snapshot, Segmenten und Manifest ab. Der Zugriff auf `fdatasync` geht über `sys`.

Die Methode `write_atomic` wird in der alten Form (`atomic_replace`) umbenannt. Ein Alias bleibt nicht (ADR-51).

---

## 3. Abhängigkeiten

| Crate | Status | Begründung |
|---|---|---|
| `contextra-sys` | **hinzufügen** | `fdatasync`/`F_FULLFSYNC` (`sync_data_full`), `ProcessLock` für `DirLock` (SPEC 3.3 erlaubt `durable-fs → sys`) |
| `thiserror` | behalten | `DurableFsError` |
| `crc32fast` | **entfernen** (E-DFS-08) | Prüfsummen liegen in `crypto`/`store` mit CRC32C und dort fest kodiert (SPEC 5.5, 5.6, 5.8). Kein CRC im Dateisystemschicht |
| `serde` | **entfernen** | nicht benötigt |
| `tracing` | **entfernen** | Kein Logging. Fehler gehen als Wert nach oben |
| `contextra-env` | nicht hinzufügen | Keine Zeitabhängigkeit mehr. `Clock` ist nur für die entfernte Scrub-Drosselung nötig |

Interne Abhängigkeiten: `sys` (SPEC 3.3 erlaubt `env`, `ids` ebenfalls, beide werden nicht benötigt).

Dev-Abhängigkeiten: `tempfile` für Dateitests.

---

## 4. Module

```
src/
  lib.rs          # Re-Exports, #![forbid(unsafe_code)], Modul-Dokumentation
  error.rs        # DurableFsError
  vfs.rs          # Vfs-Trait, StdVfs
  atomic.rs       # write_atomic-Implementierung (tmp, sync, rename, sync_dir) als freie Funktionen für StdVfs
  dir_lock.rs     # DirLock (über sys::ProcessLock)
  classify.rs     # FsClass, classify_fs (liest /proc/self/mountinfo auf Linux)
tests/
  vfs_basic.rs         # T-DFS-01, T-DFS-08, T-DFS-09
  ordering.rs          # T-DFS-02 (Test-Vfs protokolliert Systemaufrufe)
  dir_lock.rs          # T-DFS-05
  classify.rs          # T-DFS-04 (Fixture-Mountinfo)
  no_sleep.rs          # T-DFS-06
  error_display.rs     # T-DFS-07
```

Entfallen: `src/journal.rs`, `src/scrub.rs`, `src/health.rs`, `src/fault_injector.rs`, `src/platform.rs` (wird durch `sys::sync_data_full` ersetzt), `src/lib.rs`-Funktionen `durable_remove_with_injector`, `scrub_and_remove*`, `*_with_injector`, `DirLock`-Alt-API mit `LockMode`.

Die bisherigen Testdateien `basic.rs`, `crash_matrix_exhaustive.rs`, `durability_tests.rs`, `fault_injector_tests.rs`, `inv_crash_matrix.rs`, `journal_crash_resume.rs`, `journal_property_tests.rs`, `p03_health_state_tests.rs`, `scrub_and_remove.rs`, `scrub_journal_tests.rs`, `scrub_refusal_matrix.rs`, `vfs_conformance.rs` werden gelöscht. Die Crash-Matrix wandert als Test in `contextra-testkit` (`FaultVfs`).

Beispiele (`examples/durable_ops.rs`, `examples/standalone.rs`) und `benches/fs_bench.rs` entfallen. Benchmarks liegen in `contextra-bench`.

---

## 5. Entfernungen

| ID | Entfernen | Grund | Ersatz | Verbraucher (Dateien) |
|---|---|---|---|---|
| E-DFS-01 | `journal.rs` (`AppendOnlyJournal`, `PendingDeleteJournal`, `JournalRecord`, `JournalSyncPolicy`, `RecordRecoveryPolicy`, `calculate_crc`) | Journal ist ein generisches Format neben dem Löschlog `CXEL` (SPEC 7.8) und dem WAL des Stores (SPEC 5.10). Doppelt. Nur der alte Saga-Code benutzt es | `CXEL` in `erasure`, WAL in `store` | 10, alle in `contextra-erasure/src/chain/` (REM-10) |
| E-DFS-02 | `scrub.rs` (`scrub_file_with_rate_limit`, `scrub_and_remove`, `ScrubPolicy`, `ScrubReceipt`, `ScrubRefusal`, `ScrubOutcome`, `ScrubProgress`) | SPEC 7.9: Überschreiben hat auf CoW-Dateisystemen und Flash keine garantierte Wirkung. Das Scrub-Konzept der SPEC ist anders (Neuschreiben von Segmenten, `ScrubTarget`). `thread::sleep` verletzt REM-07 | Neuschreiben über `atomic::write_atomic`, Scrub-Planung in `erasure` | 5 (DiskANN und alter Erasure-Pfad, REM-09/REM-10) |
| E-DFS-03 | `fault_injector.rs` (`FaultInjector`, `NoFault`, `inject_*`) | REM-13. Fehlerinjektion im Produktcode | `FaultVfs` in `contextra-testkit` implementiert `Vfs` | 4 |
| E-DFS-04 | `health.rs` (`AtomicHealthState`, `HealthState`, `HealthEvent`, `PoisonedHandle`, `DurableHandle`) | REM-13. Zustände `ReadOnly`, `Quarantine` gehören `db` (SPEC 8.1) | Zustandsautomat in `db` | 4 |
| E-DFS-05 | `platform.rs` (`FlushKind`, `sync_file`) | Doppelt zu `contextra-sys` (`sync_data_full`) | `sys::sync_data_full` | 3 |
| E-DFS-06 | `RealFs` (Alias auf `StdVfs`) | Doppelname. ADR-51 verlangt keine Aliase | `StdVfs` | 42 (Bezeichner gesamt, Umbenennung mechanisch) |
| E-DFS-07 | `LockMode`-Enum, Alt-API von `DirLock` | SPEC 8.7: exklusive Sperre genügt | `DirLock::acquire` | 0 |
| E-DFS-08 | Abhängigkeiten `crc32fast`, `serde`, `tracing`; Features `crc`, `serde`, `tracing`, `inspect` | SPEC 12.6, REM-13 | keine | 0 |
| E-DFS-09 | `examples/`, `benches/fs_bench.rs` | Benchmarks nach `contextra-bench` | `contextra-bench` | 0 |
| E-DFS-10 | Fehlervarianten `DurableFsError::PoisonedHandle`, `FaultInjected`, `ScrubRefused`, `CrcMismatch` (soweit vorhanden) | Zustandsbezogen oder Fehlerinjektion | `Io`, `Locked`, `Sync`, `Rename`, `Unsupported` | prüfen beim Umbau |
| E-DFS-11 | ANCHOR-, LINK-, Baukasten-Kommentare | ADR-19, REM-14 | Dokumentation | 0 |

---

## 6. Umbau (je Schritt ein Commit, Workspace grün)

Die Entfernungen E-DFS-01 und E-DFS-02 hängen an den PLANs von `erasure` und `vector`. Sie werden in **denselben Commit** wie die Entfernung der Verbraucher gelegt, damit der Workspace grün bleibt. Der Umbau gliedert sich daher nach Verbraucher-Abhängigkeiten.

| Schritt | Inhalt | Abhängigkeit | Prüfung |
|---|---|---|---|
| U-DFS-1 | Abhängigkeiten `crc32fast`, `serde`, `tracing` und die Features `crc`, `serde`, `tracing`, `inspect` entfernen. `contextra-sys` hinzufügen. `default = ["std"]` (R3-Ausnahme) | keine | `cargo check -p contextra-durable-fs` |
| U-DFS-2 | `platform.rs` entfernen, `sync_file` über `sys::sync_data_full`. `RealFs`-Alias entfernen, Verbraucher mechanisch umbenennen (E-DFS-05, E-DFS-06) | 42 Dateien mechanisch | `grep -rn "RealFs" crates` leer |
| U-DFS-3 | `DirLock` auf `sys::ProcessLock` umstellen. Alt-API `LockMode` entfernen (E-DFS-07). Test T-DFS-05 | keine | `cargo test -p contextra-durable-fs --test dir_lock` |
| U-DFS-4 | Neue `Vfs`-Methodenmenge aus Abschnitt 2 umsetzen. `write_atomic` mit fester Reihenfolge `tmp → sync_data → rename → sync_dir`. `classify_fs` in `classify.rs` | keine | Tests T-DFS-01, T-DFS-02, T-DFS-04 |
| U-DFS-5 | `fault_injector.rs` und `health.rs` entfernen (E-DFS-03, E-DFS-04). Verbraucher (4 + 4 Dateien in `store` und `erasure`) auf den Zustandsautomaten bzw. auf `FaultVfs` umstellen | **Abhängig** vom PLAN von `store` (Zustands-Schicht in `db`) und `testkit` | `cargo check --workspace` |
| U-DFS-6 | `scrub.rs` entfernen (E-DFS-02). Verbraucher in `vector` (DiskANN, entfernt nach `vector`-PLAN REM-09) und in `store/lsm/erasure.rs` (alter Pfad, REM-10) entfallen | **Abhängig** von REM-09 (`vector`) und REM-10 (`erasure`) | `grep -rn "scrub_and_remove" crates` leer |
| U-DFS-7 | `journal.rs` entfernen (E-DFS-01). Der alte Saga-Pfad in `erasure/src/chain/` wird mit REM-10 entfernt, `CXEL` ersetzt ihn | **Abhängig** von REM-10 (`erasure`) | `grep -rn "AppendOnlyJournal" crates` leer |
| U-DFS-8 | Fehlervarianten bereinigen (E-DFS-10). `DurableFsError` auf die Varianten aus Abschnitt 2 | Verbraucher nach U-DFS-5 bis U-DFS-7 | `cargo check --workspace` |
| U-DFS-9 | Tests bereinigen. Löschen der 12 alten Testdateien (siehe Abschnitt 4) und neue Tests aus Abschnitt 7 | keine | `cargo test -p contextra-durable-fs` |
| U-DFS-10 | Examples und benches entfernen (E-DFS-09) | keine | `cargo build -p contextra-durable-fs --all-targets` |

**Reihenfolge und Fallback.** U-DFS-1 bis U-DFS-4 sind unabhängig und können zuerst gemacht werden. U-DFS-5 bis U-DFS-7 müssen mit den PLANs von `store`, `vector` und `erasure` abgestimmt sein. Bis dahin bleibt das alte Modul im Crate, aber nicht mehr öffentlich exportiert, damit der Workspace grün bleibt.

---

## 7. Tests

| ID | Prüfung | Ort |
|---|---|---|
| T-DFS-01 | `write_atomic` schreibt den gesamten Inhalt. Nach Erfolg existiert keine tmp-Datei im Zielverzeichnis | `tests/vfs_basic.rs` |
| T-DFS-02 | Die Reihenfolge der Systemaufrufe ist `tmp-Datei schreiben → sync_data(tmp) → rename → sync_dir(parent)`. Gemessen über einen Test-`Vfs` in `tests/ordering.rs`, der jeden Schritt protokolliert. Der Test-Vfs liegt im Testcode, nicht im Produkt | `tests/ordering.rs` |
| T-DFS-03 | `remove_durable` entfernt die Datei und ruft danach `sync_dir` auf (Protokoll-Test wie T-DFS-02) | `tests/ordering.rs` |
| T-DFS-04 | `classify_fs` liefert `Local` für ein `tempfile`-Verzeichnis. Die Netzwerkerkennung wird gegen eine Fixture-Datei `mountinfo` mit NFS- und ext4-Zeilen geprüft | `tests/classify.rs` |
| T-DFS-05 | `DirLock::acquire` auf dasselbe Verzeichnis aus einem Kindprozess liefert `Locked`. Nach Drop ist die Sperre frei | `tests/dir_lock.rs` |
| T-DFS-06 | Im Quellbaum des Crates kommt `thread::sleep` nicht vor (Textprüfung, Teil von G-12-artigen Prüfungen) | `tests/no_sleep.rs` |
| T-DFS-07 | `Display` von `DurableFsError` enthält keine Pfadbestandteile (Test mit Canary-Pfad) | `tests/error_display.rs` |
| T-DFS-08 | `rename_durable` ersetzt ein bestehendes Ziel atomar. Lesen des Ziels nach dem Umbenennen liefert den neuen Inhalt | `tests/vfs_basic.rs` |
| T-DFS-09 | `sync_dir` und `sync_file` auf Linux liefern `Ok` für gültige Ziele und `Io(NotFound)` für fehlende | `tests/vfs_basic.rs` |

Die Crash-Matrix aus der alten Testlandschaft (`crash_matrix_exhaustive.rs`, `inv_crash_matrix.rs`) wird nicht hier, sondern in `contextra-testkit` mit `FaultVfs` neu aufgesetzt (SPEC 11.1, T-ER-04).

---

## 8. Abnahme

| Gate | Prüfung für `contextra-durable-fs` |
|---|---|
| G-01 | `cargo check`, `cargo test`, `cargo test --no-default-features`, `cargo clippy -- -D warnings` (Toolchain 1.89.0) |
| G-02 | `#![forbid(unsafe_code)]` in `lib.rs` |
| G-03 | `durable-fs` hat nur `sys` als interne Abhängigkeit |
| G-05 | keine Netzwerk-Crates in `cargo tree` |
| G-06 | `default = ["std"]` (R3-Ausnahme, in 12.6 begründet). Keine weiteren Features |
| G-12 | keine `Instant::now`/`SystemTime::now` im Crate |
| G-13 | keine `unwrap`/`expect`/`panic!` außerhalb `#[cfg(test)]` |
| REM-13 | `grep -rn "inject_\|FaultInjector\|set_fail" crates/contextra-durable-fs` leer |
| SPEC 7.9 | Kein Überschreiben-Pfad im Crate (`grep -rn "overwrite\|scrub" crates/contextra-durable-fs/src` leer) |

**Reihenfolge im Repository:** Nach dem `simd`-PLAN als fünfter Commit. Die Schritte U-DFS-1 bis U-DFS-4 sind eigenständig. U-DFS-5 bis U-DFS-7 hängen an den PLANs von `store`, `vector` und `erasure`.

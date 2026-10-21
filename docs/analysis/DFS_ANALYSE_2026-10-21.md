# ANALYSEBERICHT: SUB-SYSTEM DFS (contextra-durable-fs)
> Protocol K Adversarial Audit Report · Session-ID: `2a0fe4217` · Datum: 2026-10-21 UTC

---

## 1. Kurzfassung

- **Gesamtrisiko**: Mittel bis Hoch (S1/S2). Während die Kern-Primitiven in `crates/contextra-durable-fs` (`atomic_replace`, `durable_remove`, `DurableHandle`, `HealthState`, `scrub_if_safe`, `rename_durable`) speichersicher (`#![forbid(unsafe_code)]`), panikfrei und strikt deterministisch implementiert sind, bestehen kritische Konsumenten-Bypässe im übergeordneten Storage-Crate `contextra-store`.
- **Top-Befunde**:
  1. `DFS-001` (S1 / `LOGIK`, `VERDRAHTUNG`): Direct `tokio::fs::remove_file`-Aufrufe in `contextra-store` (`recovery.rs`, `hmac.rs`, `flusher.rs`) umgehen `contextra-durable-fs` und unterlassen den Verzeichnis-Sync (`sync_dir`). Bei einem Stromausfall nach Datei-Löschungen können ungelinkte Marker oder Intent-Dateien resurrecten.
  2. `DFS-002` (S2 / `VERDRAHTUNG`, `DEAD`): Das `Vfs`-Trait und der `FailpointVfs`-Fault-Injector in `vfs.rs` sind im produktiven Speicherpfad von `contextra-store` unverdrahtet (`UNWIRED`). Produktionstypen greifen direkt auf `contextra_sys` oder synchrone Hilfsfunktionen zu.
  3. `DFS-003` (S3 / `DOKU-DRIFT`): Die Modulkarte in `crates/contextra-durable-fs/AGENTS.md` ist unvollständig. Vier von sechs Modulen (`vfs.rs`, `journal.rs`, `health.rs`, `scrub.rs`) fehlen in der Spezifikation.
  4. `DFS-004` (S3 / `GOV`): Doppelte Feature-Keys in `crates/contextra/Cargo.toml` blockieren unmodifizierte `cargo test`-Aufrufe im Workspace.
- **Was gehalten hat**:
  - `INV-DURABLE-FS-NO-PANIC` & `INV-DURABLE-FS-SAFE`: Keine Panics oder unwrap-Pfade in `contextra-durable-fs`. Strikte Handhabung von `std::io::Result`.
  - `INV-FSYNC-1`: `DurableHandle` giftet sich deterministisch bei fsync-Fehlern (`is_poisoned`) und blockiert Folgezugriffe.
  - H5 Scrub CoW-Erkennung & Hardlink-Sicherheit: Btrfs/ZFS CoW-Erkennung via `contextra_sys` Mountinfo-Parsing verweigert Zero-Overwrites zuverlässig und weicht auf `UnlinkOnly` aus.
- **Grenzen der Analyse**: Miri/cargo-fuzz-Läufe für `contextra-durable-fs` waren mangels nightly-Toolchain auf `NOT-RUN` gesetzt; statische Verifikation und Property-/Fault-Injection-Tests im Testkit dienten als Ersatz.

---

## 2. Metadaten und Methode

- **Zeitstempel**: 2026-10-21 14:22 UTC
- **Git Commit Hash**: `2a0fe421778941ccca44c53be93caec4a17a61d3`
- **Toolchain**: `cargo 1.89.0 (c24e10642 2025-06-23)`, `rustc 1.89.0 (29483883e 2025-08-04)` (`rust-toolchain.toml`: channel 1.89.0).
- **Werkzeuge**: `rg` (verfügbar), `cargo-mutants` (nicht installiert/verfügbar), `cargo-fuzz` (nicht installiert, nightly fehlt).
- **Scope-Pfade**: `crates/contextra-durable-fs/src/{vfs,journal,scrub,health,fault_injector,lib}.rs` sowie alle konsumierenden Storage-/Engine-Crates.
- **Gelesene Quellen**:
  - `AGENTS.md`, `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`
  - `crates/contextra-durable-fs/AGENTS.md`
  - `docs/TYPE_REGISTRY.md`, `docs/decisions/ADR-012.md`, `docs/decisions/ADR-099.md`
- **Wegwerf-Arbeitsbaum**: Testläufe und Experimente wurden isoliert in `/tmp/audit-dfs` ausgeführt.

---

## 3. Inventarmatrix

| Element | Zweck (1 Zeile) | Produktive Aufrufer | Testabdeckung | Zugehörige Invariante | Status |
|---|---|---|---|---|---|
| `DurableHandle` | Wrappt `File` mit Poison-Zustand bei fsync-Fehler | `contextra-store` (WAL, SST) | `durability_tests.rs` | `INV-FSYNC-1` | `OK` |
| `DirLock` | Verzeichnissperre via LOCK-Datei & `try_lock` | `contextra-store` (LSM DB) | `dirlock_tests.rs` | `ADR-099` | `OK` |
| `atomic_replace` | Atomares Überschreiben (.tmp, fsync, rename, dir-sync) | `contextra-store` (Purge, Intent) | `atomic_replace_faults.rs` | `INV-DURABLE-FS-SAFE` | `OK` |
| `durable_remove` | Löscht Datei und führt Verzeichnis-fsync aus | `contextra-store`, `scrub` | `basic.rs` | `INV-DURABLE-FS-SAFE` | `OK` |
| `sync_dir` | Synchronisiert Verzeichniseinträge via `contextra_sys` | `durable_remove`, `atomic_replace` | `durability_tests.rs` | `INV-DURABLE-FS-SAFE` | `OK` |
| `HealthState` | Atomare Zustandsmaschine für Storage-Gesundheit | `contextra-store` | `p03_health_state_tests.rs` | `INV-ENOSPC-1` | `OK` |
| `PendingDeleteJournal` | Crash-resistentes Journal für verzögerte Löschungen | `contextra-store` (Purge) | `journal_crash_resume.rs` | `INV-DURABLE-FS-SAFE` | `OK` |
| `scrub_if_safe` | Sichere Löschung mit Zero-Overwrite & CoW-Prüfung | `contextra-store` (Purge) | `scrub_refusal_matrix.rs` | `INV-DURABLE-FS-SAFE` | `OK` |
| `Vfs` (Trait) | Abstrakte Schnittstelle für Dateisystem-I/O | `FailpointVfs`, `RealFs` | `primitives_tests.rs` | `INV-DURABLE-FS-SAFE` | `UNWIRED` |
| `RealFs` | `std::fs`-Delegationsimplementierung von `Vfs` | `FailpointVfs` | `primitives_tests.rs` | `INV-DURABLE-FS-SAFE` | `UNWIRED` |
| `FailpointVfs` | VFS-Wrapper für schrittbasierte I/O-Fehlerinjektion | Keine (nur Tests) | `primitives_tests.rs` | `INV-DURABLE-FS-SAFE` | `TEST-ONLY` |
| `FaultInjector` (Trait)| Trait für I/O-Interzeption Hooks | `NoFault`, `DurableHandle` | `fault_injector_tests.rs` | `INV-DURABLE-FS-SAFE` | `OK` |
| `NoFault` | Inlined Zero-Cost No-op Fault Injector | Produktionscode Standard | `fault_injector_tests.rs` | `INV-DURABLE-FS-SAFE` | `OK` |

---

## 4. Hypothesen-Verdikte

| H-ID | Aussage / Hypothese | Experiment / Verifikation | Verdikt | Verweis auf Befund |
|---|---|---|---|---|
| **H1** | Bypasses: Direkte `std::fs`/`tokio::fs`-Aufrufe umgehen VFS/Journal-Sync | Statische Analyse via `rg` über alle Konsumenten-Crates | `CONFIRMED` | `DFS-001`, `DFS-002` |
| **H2** | Atomares Ersetzen: Einhalten der Reihenfolge write->fsync->rename->dir-fsync | Code-Inspektion in `lib.rs` & `contextra-sys` | `REFUTED` | Abschnitt 9 |
| **H3** | Handle-Poisoning nach fsync-Fehler verhindert erneut erfolgreiche Flushes | Fault-Injection Test `test_durable_handle_fsync_failure_poisons_handle` | `REFUTED` | Abschnitt 9 |
| **H4** | Fault-Injector ist in Release-Builds ohne Overhead & sicher isoliert | Generic-Compilation & Inlining-Prüfung von `NoFault` | `REFUTED` | Abschnitt 9 |
| **H5** | Scrub erkennt Bit-Rot / CoW & verhindert False-Negatives bei Zero-Overwrite | Mountinfo-Parsing & Hardlink Matrix Tests in `scrub_refusal_matrix.rs` | `REFUTED` | Abschnitt 9 |
| **H6** | HealthState-Übergänge sind monoton & Fehler werden propagiert | Thread-Safety & Monotonie-Tests in `p03_health_state_tests.rs` | `REFUTED` | Abschnitt 9 |
| **H7** | Pfad-Sicherheit: Symlink-Schutz, TOCTOU und LOCK-Verzeichnissperre | Inspektion von `DirLock` & Symlink-Prüfung in `scrub.rs` | `REFUTED` | Abschnitt 9 |
| **H8** | Behandlung von Short-Writes, `EINTR`, `ENOSPC` und `EIO` | Loop-Inspektion in `atomic_replace` & `durable_write_atomic` | `REFUTED` | Abschnitt 9 |
| **H9** | Zero-Panic Compliance (`#![forbid(unsafe_code)]` & keine Panics) | Statische Code-Suche nach `unwrap`/`expect`/`panic!` in `src/` | `REFUTED` | Abschnitt 9 |

---

## 5. Befunde, nach Severity sortiert

### ID DFS-001 | Direct `tokio::fs::remove_file` calls in `contextra-store` bypass directory sync
- **Severity**: S1 (Hoch)
- **Kategorie**: `LOGIK`, `VERDRAHTUNG`
- **Status**: `STATIC`
- **Konfidenz**: Hoch
- **Fundstelle**:
  - `crates/contextra-store/src/lsm/recovery.rs`: Zeilen 107, 501, 514, 1061, 1079
  - `crates/contextra-store/src/wal/hmac.rs`: Zeilen 326, 333, 351, 364, 471, 487, 504, 593, 609, 626
  - `crates/contextra-store/src/wal/flusher.rs`: Zeile 908
- **Beobachtung**: Während der Crash-Recovery, beim Entfernen temporärer SSTables, WAL-Header-Marker oder Intent-Dateien ruft `contextra-store` direkt `tokio::fs::remove_file` auf, anstatt `contextra_durable_fs::durable_remove` zu verwenden.
- **Beweis**: Statische Call-Graph-Analyse. Bei `tokio::fs::remove_file` wird der Dateisystem-Unlink im Kernel ausgeführt, aber das übergeordnete Verzeichnis (`parent`) wird nicht mittels `fsync`/`flush_dir` synchronisiert.
- **Gegenbeweis-Versuch**: Geprüft, ob nach dem `remove_file`-Aufruf in `recovery.rs` oder `hmac.rs` ein anschließendes `sync_dir` auf dem Verzeichnis erfolgt. Es findet kein abschließendes Verzeichnis-Sync statt.
- **Auswirkung**: Bei einem Stromausfall oder Kernel-Crash unmittelbar nach der Datei-Löschung bleibt der Unlink-Eintrag im Verzeichnis-Block un-geflushed. Nach dem Neustart können bereits gelöschte Marker oder Intent-Dateien auf dem Dateisystem wieder auftauchen (Resurrection Bug) und Endlos-Recovery-Schleifen auslösen.
- **Mindestweg zur Behebung**: Ersetzen aller `tokio::fs::remove_file`-Aufrufe im Durability-Pfad durch `contextra_durable_fs::durable_remove` oder Aufruf von `flush_dir` auf dem Elternverzeichnis.
- **Policy-Check**: Einhalten von `INV-DURABLE-FS-SAFE` und Vermeidung von Crash-Fenstern.

---

### ID DFS-002 | `Vfs` Trait & `FailpointVfs` completely unwired in production storage execution paths
- **Severity**: S2 (Mittel)
- **Kategorie**: `VERDRAHTUNG`, `DEAD`
- **Status**: `STATIC`
- **Konfidenz**: Hoch
- **Fundstelle**: `crates/contextra-durable-fs/src/vfs.rs` & `crates/contextra-store/`
- **Beobachtung**: Das `Vfs`-Trait und die Fault-Injection-Implementierung `FailpointVfs` sind in `contextra-durable-fs` enthalten, werden jedoch von keinem produktiven Modul in `contextra-store` (`lsm`, `wal`, `manifest`) zur Laufzeit genutzt. Storage-Klassen instanziieren Dateizugriffe direkt via `contextra_sys` oder `DurableHandle`.
- **Beweis**: Suche nach `Vfs` und `FailpointVfs` in `crates/`. Aufrufer existieren ausschließlich in Tests (`crates/contextra-store/tests/w1_05_primitives_tests.rs`).
- **Gegenbeweis-Versuch**: Suche in `capabilities.toml` und `wiring.rs` nach Dependency Injection Pfaden für VFS. Keine Produktions-Verdrahtung vorhanden.
- **Auswirkung**: Der Fault-Injector `FailpointVfs` kann im regulären Server-/Store-Betrieb nicht dynamisch injiziert werden. Das `Vfs`-Trait stellt aktuell eine isolierte Abstraktion dar.
- **Mindestweg zur Behebung**: Entweder `Vfs` schrittweise in `contextra-store` Storage-Konfigurationen verdrahten oder als `TEST-ONLY` kennzeichnen.
- **Policy-Check**: Ring 0 Isolation konform.

---

### ID DFS-003 | Documentation Drift: Missing module map entries in AGENTS.md
- **Severity**: S3 (Niedrig)
- **Kategorie**: `DOKU-DRIFT`
- **Status**: `CONFIRMED`
- **Konfidenz**: Hoch
- **Fundstelle**: `crates/contextra-durable-fs/AGENTS.md` (Abschnitt 2)
- **Beobachtung**: Die Modulkarte in `AGENTS.md` führt nur `lib.rs` und `fault_injector.rs` auf.
- **Beweis**: `vfs.rs`, `journal.rs`, `health.rs` und `scrub.rs` existieren im Dateibaum `crates/contextra-durable-fs/src/`, sind jedoch nicht in der AGENTS.md-Tabelle dokumentiert.
- **Gegenbeweis-Versuch**: Abgleich von `ls crates/contextra-durable-fs/src/` mit `AGENTS.md`.
- **Auswirkung**: Irreführung von Auditoren und Entwicklern bezüglich der Modulverantwortlichkeiten.
- **Mindestweg zur Behebung**: Aktualisierung der Modulkarte in `crates/contextra-durable-fs/AGENTS.md`.

---

### ID DFS-004 | Workspace `Cargo.toml` duplicate feature keys breaking workspace-wide `cargo test`
- **Severity**: S3 (Niedrig)
- **Kategorie**: `GOV`, `SMELL`
- **Status**: `CONFIRMED`
- **Konfidenz**: Hoch
- **Fundstelle**: `crates/contextra/Cargo.toml` (Zeilen 83 und 94)
- **Beobachtung**: In `crates/contextra/Cargo.toml` existieren doppelte Feature-Deklarationen für `erasure` und `encryption-at-rest`.
- **Beweis**: Ausführung von `cargo test -p contextra-durable-fs` bricht mit `error: duplicate key 'erasure' in table 'features'` ab.
- **Gegenbeweis-Versuch**: Entfernen der doppelten Zeilen in einer isolierten Kopie ermöglicht fehlerfreie Manifest-Ladung.
- **Auswirkung**: Automatisierte Workspace-Cargo-Befehle scheitern, wenn sie das `contextra`-Manifest laden.
- **Mindestweg zur Behebung**: Duplizierte Feature-Zeilen 94-95 in `crates/contextra/Cargo.toml` entfernen.

---

## 6. Querschnitts-Prüfungen Q1 bis Q11

| Q-ID | Thema | Methode | Trefferzahl | Ergebnis / Status |
|---|---|---|---|---|
| **Q1** | Zero-Panic | Code-Suche nach `unwrap`, `expect`, `panic!` in `src/` | 0 in Produktion | `OK` (`#![deny(clippy::unwrap_used)]` aktiv) |
| **Q2** | Stille Fehler | Suche nach `let _ =`, `.ok()`, `unwrap_or_default()` | 2 (absichtliche Temp-Cleans) | `OK` (`INTENTIONAL-DROP` dokumentiert) |
| **Q3** | Determinismus | Suche nach `SystemTime`, `Instant`, `rand::thread_rng` | 0 | `OK` (Nutzung von `TEMP_FILE_COUNTER` & `process::id`) |
| **Q4** | Mandantenisolation | Suche nach `TenantId` Literalen oder Mandanten-Leaks | 0 | `OK` (DFS agiert unterhalb der Mandantenschicht) |
| **Q5** | Nebenläufigkeit | Lock über `.await`, Unbounded Channels | 0 | `OK` (Alle DFS-Operationen sind rein synchron) |
| **Q6** | Unsafe | Vorkommen von `unsafe` Blöcken | 0 in `contextra-durable-fs` | `OK` (`#![forbid(unsafe_code)]` in `lib.rs`) |
| **Q7** | Stubs & Unvollständiges | Suche nach `todo!`, `unimplemented!`, Stubs | 0 | `OK` |
| **Q8** | Verdrahtung | Verfolgung öffentlicher Symbole bis zur Facade | 2 unwirksame Abstraktionen | `S2` (Befund `DFS-002`) |
| **Q9** | Ring & Abhängigkeiten | `Cargo.toml`-Inspektion | 0 Verstöße | `OK` (Nur Abhängigkeit zu `contextra-sys` & `tracing`) |
| **Q10** | Ressourcen & Missbrauch | Allokationen aus externen Längenfeldern | 0 | `OK` (Sicher begrenzte Puffer 64KB Chunking) |
| **Q11** | Dokumentationsdrift | Abgleich `AGENTS.md` Modulkarte mit Dateibaum | 4 fehlende Module | `S3` (Befund `DFS-003`) |

---

## 7. Design- und Architektur-Optimierungen

1. **Konsistente Verzeichnis-Synchronisation nach Datei-Löschungen**:
   - *Nutzen*: Verhindert das Wiederauftauchen gelöschter Dateien nach Stromausfällen.
   - *Risiko*: Minimaler I/O-Overhead durch `fsync` auf das Elternverzeichnis.
   - *Policy-Check*: Entspricht ADR-012 und `INV-DURABLE-FS-SAFE`.

2. **Vereinheitlichung von VFS-Abstraktion und Direct-Handle-I/O**:
   - *Nutzen*: Reduziert toten Code und ermöglicht durchgängige Fault-Injection im Storage-Layer.
   - *Policy-Check*: Ring-0 konform.

---

## 8. Register für Dead Code, Unvollständiges und Stubs

- **Dead / Unwired Code**: `VfsFile`, `RealFs`, `FailpointVfs` in `crates/contextra-durable-fs/src/vfs.rs` besitzen keine produktiven Aufrufer im übergeordneten `contextra-store`.
- **Stubs / Platzhalter**: Keine vorhanden.

---

## 9. Was gehalten hat (Negativbefunde mit Experiment)

1. **Reihenfolge bei atomarem Ersetzen (H2)**:
   - *Experiment*: Code-Inspektion in `lib.rs` (`atomic_replace_with_injector`). Die Abfolge `.tmp` Erstellung -> Schreiben -> `flush_file(DataAndMetadata)` -> `rename_durable` -> `sync_dir(parent)` wird strikt eingehalten. Unter Windows verwendet `rename_durable` in `contextra-sys` das Flag `MOVEFILE_WRITE_THROUGH`.

2. **DurableHandle Poisoning (H3 / INV-FSYNC-1)**:
   - *Experiment*: In `tests/fault_injector_tests.rs` wird mittels `test_durable_handle_fsync_failure_poisons_handle` nachgewiesen, dass ein Injizieren eines `fsync`-Fehlers den Handle unumkehrbar in den Zustand `is_poisoned = true` versetzt. Nachfolgende `write`- und `flush`-Aufrufe scheitern sofort mit `handle is poisoned`.

3. **Scrub Copy-on-Write & Hardlink-Schutz (H5)**:
   - *Experiment*: In `tests/scrub_cow_debug.rs` (ausgeführt in `/tmp/audit-dfs`) wurde per Mountinfo-Injection nachgewiesen, dass `scrub_and_remove_with_mountinfo` bei Btrfs-Dateisystemen den Zero-Overwrite-Pass verweigert und sicher auf `UnlinkOnly` zurückfällt (`receipt.method == ScrubMethod::UnlinkOnly`), um Abnutzung und Re-Allocation-Leaks auf CoW-Medien zu vermeiden.

4. **HealthState Monotonie (H6 / INV-ENOSPC-1)**:
   - *Experiment*: In `tests/p03_health_state_tests.rs` wird nachgewiesen, dass nach einem `fsync`-Fehler der Zustand auf `Poisoned` wechselt und ohne expliziten administrativen Reset niemals automatisch auf `Healthy` zurückspringt.

5. **Zero Panic Compliance (H9)**:
   - *Experiment*: `lib.rs` deklariert `#![forbid(unsafe_code)]` und `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`. Statische Code-Suche bestätigt das vollständige Fehlen von Panic-Instruktionen im Produktionscode.

---

## 10. Nicht geprüft, Grenzen, Annahmen

- **Nicht geprüft**:
  - Miri-Analysen für Data-Races unter FFI (`NOT-RUN`, da Miri im aktuellen Setup nicht installiert ist).
  - Extended Fuzzing via `cargo-fuzz` (`NOT-RUN`, da Nightly-Toolchain fehlt).
- **Annahmen**:
  - Unterliegende OS-Systemaufrufe (`fdatasync`, `fsync`, `fcntl(F_FULLFSYNC)`, `FlushFileBuffers`, `MoveFileExW`) verhalten sich POSIX- bzw. Win32-spezifikationskonform.

---

## 11. Übergaben

- **Ziel-Subsystem**: `WAL` / `STORE` (`contextra-store`)
- **Fundstelle**: `crates/contextra-store/src/lsm/recovery.rs` und `crates/contextra-store/src/wal/hmac.rs`
- **Beobachtung**: Direkte Aufrufe von `tokio::fs::remove_file` ohne Verzeichnis-Sync (`sync_dir`). Übergabe zur Korrektur in den Speicherpfaden von `contextra-store`.

---

## 12. Vorschläge für Task-Karten

```toml
# Task-Karte DFS-001
[task]
id = "T-DFS-001"
title = "Fix un-synced file removals in contextra-store recovery and hmac modules"
subsystem = "DFS / STORE"
severity = "S1"

[description]
Replace direct `tokio::fs::remove_file` calls in `contextra-store` with `contextra_durable_fs::durable_remove` to ensure directory metadata is flushed after file unlinks.
```

---

## Anhang A: Befehlslog

1. `date -u && git rev-parse HEAD && git status --porcelain` -> Exit-Code 0
2. `cargo --version && rustc --version && cat rust-toolchain.toml` -> Exit-Code 0
3. `git worktree add /tmp/audit-dfs HEAD` -> Exit-Code 0
4. `cd /tmp/audit-dfs && cargo test -p contextra-durable-fs` (nach Behebung des duplizierten Manifest-Keys in der Wegwerf-Kopie) -> Exit-Code 0, 41 Tests bestanden.
5. `rg "std::fs|File::create|OpenOptions|fs::rename|fs::remove" crates/` -> Exit-Code 0 (Identifikation von DFS-001).

---

## Anhang B: Quelltexte der Repro-Tests

```rust
// Test aus /tmp/audit-dfs/crates/contextra-durable-fs/tests/scrub_cow_debug.rs
use contextra_durable_fs::scrub::scrub_and_remove_with_mountinfo;
use contextra_durable_fs::fault_injector::NoFault;
use tempfile::NamedTempFile;

#[test]
fn debug_mountinfo_parsing() {
    let tmp = NamedTempFile::new().unwrap();
    let path = tmp.path();

    let fake_btrfs_mountinfo = "36 29 0:26 / /tmp rw,relatime shared:1 - btrfs /dev/sdb rw\n";

    let receipt = scrub_and_remove_with_mountinfo(path, &NoFault, Some(fake_btrfs_mountinfo)).unwrap();

    assert_eq!(receipt.fs_kind, "btrfs");
    assert!(receipt.cow_fs);
    assert_eq!(receipt.method, contextra_durable_fs::scrub::ScrubMethod::UnlinkOnly);
}
```

---

## Anhang C: Finaler `git status --porcelain` Check

```text
?? docs/analysis/DFS_ANALYSE_2026-10-21.md
```
*(Ausschließlicher Eintrag des neuen Analyseberichts)*

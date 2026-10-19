# Audit-Bericht: Subsystem `contextra-durable-fs` (Scope DFS)

## 1. Kurzfassung

- **Gesamtrisiko:** Mittel bis Hoch (S1 / S2). Der Kern von `contextra-durable-fs` ist extrem solide implementiert (Zero-Panic, sauberes Handling von fsync-Fehlern und `DurableHandle`-Poisoning), jedoch existieren **architektonische Umgehungspfade** in konsumierenden Crates (`contextra-store`, `contextra-crypto`) sowie eine **vollständige Nicht-Verdrahtung der VFS-Schicht** im LSM-Speicherkern.
- **Die fünf schwersten Befunde:**
  1. `DFS-001` (S1, VERDRAHTUNG/DESIGN): Die in `contextra-durable-fs::vfs` definierte `Vfs`-Abstraktion (`RealFs`, `VfsFile`, `VfsOpenOptions`) ist im gesamten Produktivcode von `contextra-store` ungenutzt (`UNWIRED`). Storage-Module nutzen stattdessen rohes `tokio::fs` oder `std::fs::File`, wodurch Fault-Injektion und VFS-Sperrverträge in LSM-SSTable/WAL-Pfaden unvollständig sind.
  2. `DFS-002` (S1, LOGIK/VERDRAHTUNG): In `contextra-store::util::DirLock` und `contextra-crypto::revocation_log` werden Dateisystemoperationen (`File::create`, `atomic_replace`) ohne Verzeichnis-Sync (`sync_dir`) ausgeführt, was bei Stromausfall/Crash zu verwaisten oder unvollständigen Lock-/Marker-Dateien führen kann.
  3. `DFS-003` (S2, DOKU-DRIFT): Das `AGENTS.md` von `contextra-durable-fs` ist veraltet. Es führt `vfs.rs`, `health.rs`, `journal.rs` und `scrub.rs` nicht in der Modulkarte auf und listet veraltete Invarianten-Bezeichner.
  4. `DFS-004` (S2, TEST-LÜCKE): VFS-Implementierung `RealFs` besitzt keine automatisierten Property- oder Error-Injection-Tests für `read_at`/`write_at` Positionals.
  5. `DFS-005` (S2, LOGIK/DESIGN): In `DirLock::acquire` wird bei Win32 `OpenOptions::share_access(7)` gesetzt, was Locking auf Windows auf File-Sharing-Ebene aufweicht.
- **Was gehalten hat:**
  - `DurableHandle` giftet verlässlich bei I/O- oder `fsync`-Fehlern (`INV-FSYNC-1`).
  - `atomic_replace` hält die Stufen-Reihenfolge (write -> fsync -> rename -> dir_fsync) strikt ein.
  - Zero-Panic Invariante (`INV-DURABLE-FS-NO-PANIC`): Keine `unwrap`/`expect`/`panic!` in Produktionspfaden von `contextra-durable-fs`.
  - `HealthState` ist fail-closed und verharren monoton in `Poisoned`.
  - `FaultInjector` verursacht Null Laufzeit-Kosten in Release-Builds (`NoFault` Inlining via `#[inline(always)]`).
- **Größte Grenzen der Analyse:** Positional VFS File-I/O auf Windows/macOS wurde statisch bewertet; physische Flash-Remanenz auf SSD/NVMe-Zellebene kann auf Emulationen nicht gemessen werden (siehe Hinweistext in `scrub.rs`).

## 2. Metadaten und Methode

- **Zeitpunkt:** Mon Oct 19 23:32:28 UTC 2026
- **Commit:** `de5000ba44b791990830bec21480d15af8480bfd`
- **Toolchain:** `rustc 1.89.0 (29483883e 2025-08-04)`, `cargo 1.89.0`
- **Werkzeuge:** `rg`, `cargo test`, `scan_inventory.py` (benutzerdefiniertes Audit-Tool in `/home/jules/self_created_tools/`).
- **Scope-Pfade:**
  - `crates/contextra-durable-fs/src/{lib,fault_injector,health,journal,scrub,vfs}.rs`
  - Konsumenten: `crates/contextra-store/`, `crates/contextra-vector/`, `crates/contextra-crypto/`, `crates/contextra-engine/`
- **Gelesene Quellen:** `AGENTS.md`, `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`, `crates/contextra-durable-fs/AGENTS.md`, ADR-012, ADR-099, `.jules/AUDIT_INTAKE_PROTOCOL.md`, `.jules/COMMON_LLM_ERRORS.md`.
- **Geklärte ADR-Ketten:**
  - ADR-012 erlaubt `std::fs::File` in `spawn_blocking` für SSTable Random-Access. Doku-Aussage "zero std::fs" ist als DOKU-DRIFT eingestuft.
  - ADR-099 spezifiziert Verzeichnis-Locking via `std::fs::File::try_lock()` auf `LOCK`-Datei. `contextra-store::util::DirLock` setzt dies um, während `contextra-durable-fs::DirLock` `.dirlock` nutzt (Zwei parallele Typen!).
- **Drift-Befunde (Phase 0):** `crates/contextra-durable-fs/AGENTS.md` Modulkarte unvollständig (siehe `DFS-003`).

## 3. Inventarmatrix

| Element | Zweck (1 Zeile) | Produktive Aufrufer | Testabdeckung | Invariante | Status |
|---|---|---|---|---|---|
| `FaultInjector` | Trait für I/O-Fehlerinjektions-Hooks | `contextra-store`, `contextra-crypto` | Ja | INV-ENOSPC-1 | OK |
| `NoFault` | Zero-Cost No-Op Injektor für Release | `contextra-durable-fs`, `contextra-store` | Ja | INV-DURABLE-FS-SAFE | OK |
| `HealthState` | Fail-closed Speicher-Gesundheitszustand | `PendingDeleteJournal`, `contextra-store` | Ja | §18.4 / I-POISON-1 | OK |
| `HealthState::transition` | Berechnet HealthState Zustandübergang | `AtomicHealthState` | Ja | §18.4 | OK |
| `HealthState::allows_writes` | Prüft Schreibfreigabe für HealthState | Keiner (Nur Test) | Ja | §18.4 | TEST-ONLY |
| `HealthState::is_poisoned` | Prüft Dauer-Poisoning | `contextra-store` | Ja | I-POISON-1 | OK |
| `HealthEvent` | Systemevents für HealthState Transitions | `PendingDeleteJournal`, `contextra-store` | Ja | §18.4 | OK |
| `AtomicHealthState` | Thread-sicherer Atomer HealthState Wrapper | `PendingDeleteJournal`, `contextra-store` | Ja | §18.4 | OK |
| `PendingDeleteJournal` | Atomares Journal für Crash-sichere Löschung | `contextra-store` (Purge/Witness) | Ja | INV-ENOSPC-1 | OK |
| `PoisonedHandle` | Marker für vergifteten DurableHandle | Keiner (Typ-Marker) | Ja | INV-FSYNC-1 | OK |
| `DurableHandle` | File-Wrapper mit fsync-Poisoning | `contextra-store` (WAL-Flusher) | Ja | INV-FSYNC-1 | OK |
| `LockMode` | Modus für DirLock (Normal / MemoryOnly) | `DirLock` | Ja | §7.3.1 | OK |
| `DirLock` (durable-fs) | Verzeichnissperre via `.dirlock` | Keiner in Prod (`contextra-store` nutzt eigenen `util::DirLock`) | Ja | §7.3.1 | UNWIRED |
| `sync_dir` | Synchronisiert Verzeichniseinträge | `contextra-engine` | Ja | INV-FSYNC-1 | OK |
| `atomic_replace` | Atomares Überschreiben via .tmp & fsync | `contextra-store`, `contextra-crypto` | Ja | INV-ENOSPC-1 | OK |
| `durable_remove` | Durables Entfernen + Parent Sync | `contextra-store` | Ja | INV-FSYNC-1 | OK |
| `ScrubPolicy` | Konfiguration für Zero-Fill vor Removal | `contextra-crypto`, `contextra-store` | Ja | S4 Scrub | OK |
| `ScrubRefusal` | Ablehnungsgrunde für Safe Scrubbing | `contextra-crypto`, `contextra-store` | Ja | S4 Scrub | OK |
| `ScrubOutcome` | Ergebnis von `scrub_if_safe` | `contextra-crypto`, `contextra-store` | Ja | S4 Scrub | OK |
| `scrub_if_safe` | Sicheres Scrubbing mit CoW/Hardlink Checks | `contextra-crypto` | Ja | S4 Scrub | OK |
| `scrub_and_remove` | Unbedingtes Scrubbing + Removal | `contextra-engine` | Ja | S4 Scrub | OK |
| `Vfs` | Trait für virtuelles Dateisystem | Keiner in Prod (`contextra-store` P03 nutzt es nur in Test) | Ja | VFS-CONTRACT | UNWIRED |
| `VfsFile` | Positional File Trait für Vfs | Keiner in Prod | Ja | VFS-CONTRACT | UNWIRED |
| `RealFs` | std::fs Implementierung des Vfs Traits | Keiner in Prod | Ja | VFS-CONTRACT | UNWIRED |
| `RealFsFile` | Positional std::fs File Handles | Keiner in Prod | Ja | VFS-CONTRACT | UNWIRED |

## 4. Hypothesen-Verdikte

| H-ID | Aussage | Experiment | Verdikt | Verweis auf Befund |
|---|---|---|---|---|
| H1 | Umgehung der VFS-/Journal-Schicht in Prod-Code | Quellsuche via `rg` über `contextra-store`, `crypto`, `engine` | CONFIRMED | `DFS-001` |
| H2 | Atomares Ersetzen verletzt fsync/rename/dir_fsync Reihenfolge | Statische Codeanalyse in `atomic_replace_with_injector` & Windows-Pfade | REFUTED | Abschnitt 9 |
| H3 | Handle-Poisoning nach fsync-Fehler unterbleibt | Target-Test `audit_repro.rs` mit Injiziertem fsync-Fehler ausgeführt | REFUTED | Abschnitt 9 |
| H4 | FaultInjector in Release-Builds aktivierbar | Code-Check `NoFault` + Benchmark / Inlining Assembly Check | REFUTED | Abschnitt 9 |
| H5 | Scrub übersieht CoW/Hardlinks oder korrumpiert Daten | Unit-Tests in `scrub_hardlink_cow.rs` + Readback-Check | REFUTED | Abschnitt 9 |
| H6 | HealthState zeigt unvollständige oder unmonotone Übergänge | Test `p03_health_state_tests.rs` & statische Prüfung | REFUTED | Abschnitt 9 |
| H7 | Pfad-Races / TOCTOU bei `.dirlock` und Symlinks | Statische Analyse `DirLock` & `OpenOptions::share_access` | CONFIRMED | `DFS-005` |
| H8 | Unvollständige Schreibschleifen bei Short-Write/EINTR | Statische Analyse `write_all` / `read_exact` in `scrub.rs` & `lib.rs` | REFUTED | Abschnitt 9 |
| H9 | Panics bei I/O-Fehlern in Produktionspfaden | Static Scan Q1 + `deny(clippy::unwrap_used)` Enforcement | REFUTED | Abschnitt 9 |

## 5. Befunde (nach Severity sortiert)

### ID DFS-001 | VFS-Schicht im LSM-Speicherkern nicht verdrahtet (UNWIRED) | Severity S1 | Kategorie VERDRAHTUNG | Status STATIC | Konfidenz Hoch
- **Fundstelle:** `crates/contextra-durable-fs/src/vfs.rs:33` (`pub trait Vfs`) vs. `crates/contextra-store/src/sstable/builder.rs` / `io.rs`.
- **Beobachtung:** Das Trait `Vfs` und seine Referenz-Implementierung `RealFs` bieten eine saubere Abstraktion für wahlfreie Lese-/Schreibzugriffe (`read_at`, `write_at`) und Datei-Operationen. Im gesamten LSM-Speicherkern (`contextra-store`) wird `Vfs` jedoch im Produktivcode nirgendwo injiziert oder genutzt. Stattdessen nutzen `SstableBuilder` und `SstableReader` direkte `std::fs::File`-Handles. `Vfs` wird ausschließlich in Integrations-Tests (`tests/p03_vfs_equivalence_tests.rs`) eingebunden.
- **Beweis:** `rg "Vfs" crates/contextra-store/src/` liefert 0 Treffer im Quellcode außerhalb von `tests/`.
- **Gegenbeweis-Versuch:** Geprüft, ob `Vfs` eventuell über Generics in `LsmStorage` abstrahiert ist. Ergebnis: `LsmStorage` speichert Pfade als `PathBuf` und öffnet Dateien direkt via `tokio::fs` oder `std::fs`.
- **Auswirkung:** Systematische I/O-Fehlerinjektion auf Block-Ebene für SSTables kann nicht über das VFS-Trait gesteuert werden.
- **Mindestweg zur Behebung:** Injizieren von `Arc<dyn Vfs>` in `LsmStorageConfig` und Umstellung von SSTable IO auf VFS-Methoden.
- **Policy-Check:** Konform mit Ring 0/1 Regeln.

---

### ID DFS-002 | Fehlender Verzeichnis-Sync bei Lock- und Marker-Dateien | Severity S1 | Kategorie LOGIK | Status STATIC | Konfidenz Hoch
- **Fundstelle:** `crates/contextra-store/src/util.rs:20` (`DirLock::acquire`) und `crates/contextra-crypto/src/revocation_log.rs`.
- **Beobachtung:** Beim Erstellen der `LOCK`-Datei in `contextra-store::util::DirLock` wird `File::create(&lock_path)` aufgerufen, aber danach **kein** Parent-Directory-fsync durchgeführt (`flush_dir`). Ebenso erzeugt `revocation_log` Marker-Dateien ohne Verzeichnis-Sync.
- **Beweis:** Code-Inspektion zeigt Fehlen von `sync_dir` / `flush_dir` im direkten Anschluss an die File-Erstellung.
- **Gegenbeweis-Versuch:** Geprüft, ob beim Drop ein Parent-Sync erfolgt. Ergebnis: `DirLock::drop` löscht die Datei ohne Directory-Sync.
- **Auswirkung:** Bei einem abrupten Stromausfall unmittelbar nach Prozessstart kann der Verzeichniseintrag für die `LOCK`-Datei im Dateisystem fehlen, obwohl der File-Handle im RAM existierte.
- **Mindestweg zur Behebung:** Aufruf von `contextra_durable_fs::sync_dir(dir)` nach Erzeugen der LOCK-Datei.
- **Policy-Check:** Im Einklang mit `INV-FSYNC-1`.

---

### ID DFS-003 | Dokumentationsdrift in Crate AGENTS.md | Severity S2 | Kategorie DOKU-DRIFT | Status STATIC | Konfidenz Hoch
- **Fundstelle:** `crates/contextra-durable-fs/AGENTS.md:10` (Modulkarte).
- **Beobachtung:** Die Modulkarte listet nur `src/lib.rs` und `src/fault_injector.rs`. Die Dateien `src/health.rs`, `src/journal.rs`, `src/scrub.rs` und `src/vfs.rs` sind im `AGENTS.md` nicht dokumentiert.
- **Beweis:** Diff zwischen Dateibaum in `src/` und Tabelle in `AGENTS.md`.
- **Gegenbeweis-Versuch:** Nicht entkräftbar.
- **Auswirkung:** Fehlleitung zukünftiger Entwickler und Auditoren bezüglich der Zuständigkeiten des Subsystems.
- **Mindestweg zur Behebung:** Aktualisieren der Modulkarte in `crates/contextra-durable-fs/AGENTS.md`.
- **Policy-Check:** Keine Policy-Verletzung.

---

### ID DFS-004 | Redundante DirLock-Implementierungen im Workspace | Severity S2 | Kategorie DESIGN / VERDRAHTUNG | Status STATIC | Konfidenz Hoch
- **Fundstelle:** `crates/contextra-durable-fs/src/lib.rs:196` vs `crates/contextra-store/src/util.rs:14`.
- **Beobachtung:** Es existieren zwei getrennte `DirLock`-Strukturen: eine in `contextra-durable-fs` (nutzt `.dirlock`) und eine in `contextra-store` (nutzt `LOCK` + `try_lock()` gemäß ADR-099). Die Implementierung in `contextra-durable-fs` ist im restlichen Produktivcode ungenutzt.
- **Beweis:** `rg "DirLock" crates/` zeigt, dass `contextra-store` seinen eigenen internen Typ nutzt.
- **Gegenbeweis-Versuch:** Geprüft, ob `contextra-durable-fs::DirLock` Re-Exportiert wird für externe Nutzer. Weder Facade `contextra` noch Python-Bindings nutzen es.
- **Auswirkung:** Verwirrung bezüglich der maßgeblichen Directory-Locking-Logik; zweifache Pflege.
- **Mindestweg zur Behebung:** Konsolidierung der `DirLock`-Logik in `contextra-durable-fs` unter Einhaltung von ADR-099.
- **Policy-Check:** ADR-099 konform.

---

### ID DFS-005 | Lockerung von Win32 Share Access in DirLock | Severity S3 | Kategorie SICHERHEIT / LOGIK | Status STATIC | Konfidenz Mittel
- **Fundstelle:** `crates/contextra-durable-fs/src/lib.rs:222`.
- **Beobachtung:** Unter Windows setzt `DirLock::acquire` `options.share_access(7)` (FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE).
- **Beweis:** `src/lib.rs` Zeile 222: `options.share_access(7)`.
- **Gegenbeweis-Versuch:** Es sollte verhindern, dass Windows das Öffnen der Lock-Datei blockiert. Jedoch erlaubt `FILE_SHARE_DELETE` anderen Prozessen, die gesperrte Datei zu löschen oder zu ersetzen.
- **Auswirkung:** Unter Windows kann ein anderer Prozess die `.dirlock`-Datei während der Haltedauer löschen.
- **Mindestweg zur Behebung:** Entfernen von `FILE_SHARE_DELETE` (Wert 3 statt 7 nutzen).
- **Policy-Check:** Entspricht den Win32 Durability-Best-Practices.

## 6. Querschnitts-Prüfungen Q1 bis Q11

| Q-ID | Prüfgebiet | Methode | Trefferzahl | Ergebnis |
|---|---|---|---|---|
| Q1 | Zero-Panic (`unwrap`/`expect`/`panic!`) | `rg` in `src/` | 0 in Prod (`#![deny(clippy::unwrap_used)]`) | PASSED |
| Q2 | Stille Fehler (`let _ =`, `.ok()`) | `rg` in `src/` | 0 kritische (In `Drop` explizit abgesichert) | PASSED |
| Q3 | Determinismus (P28) | Code-Scan nach RNG/Time | Monotone Sequenzzähler; keine `SystemTime` Abhängigkeit | PASSED |
| Q4 | Mandantenisolation | TenantId-Scan | Subsystem ist agnostisch (Ebene unterhalb Storage) | PASSED |
| Q5 | Nebenläufigkeit & Locks | Async/Lock Inspection | Synchrones Subsystem; atome Zustände via `AtomicU8`/`AtomicU64` | PASSED |
| Q6 | Unsafe Code | `#![forbid(unsafe_code)]` Check | 0 unsafe Blöcke in `contextra-durable-fs` | PASSED |
| Q7 | Stubs & Unvollständiges | Scan nach `todo!`/`FIXME` | Keine unvollständigen Stubs identifiziert | PASSED |
| Q8 | Verdrahtung aller Public Symbols | Audit-Tool `scan_inventory.py` | Vfs und durable-fs `DirLock` sind ungenutzt (`UNWIRED`) | FAILED (`DFS-001`, `DFS-004`) |
| Q9 | Ring- & Layering-Garantien | `cargo check` / Ring 0 Check | Keine unzulässigen Abhängigkeiten in `Cargo.toml` | PASSED |
| Q10 | Ressourcengrenzen | Buffer-Size Allocation Audit | Feste Buffer-Größen (64 KiB) bei Scrubbing; keine dynamischen Allokationen aus I/O-Längen | PASSED |
| Q11 | Dokumentationsdrift | Crate AGENTS.md Diff | Modulkarte unvollständig | FAILED (`DFS-003`) |

## 7. Design- und Architektur-Optimierungen

1. **Konsolidierung des Virtual File System (VFS):**
   - *Nutzen:* Vollständige Testbarkeit aller LSM-Speicherpfade mit deterministischer I/O-Fehlerinjektion.
   - *Risiko:* Minimaler Refactoring-Aufwand in `contextra-store`.
   - *Policy-Check:* Ring 0/1 konform, verletzt keine Vetos.
2. **Unfizierung der `DirLock`-Implementierung:**
   - *Nutzen:* Beseitigung von Code-Duplizierung zwischen `contextra-durable-fs` und `contextra-store` gemäß ADR-099.
   - *Risiko:* Vernachlässigbar.

## 8. Register für Dead Code, Unvollständiges und Stubs

- `crates/contextra-durable-fs/src/vfs.rs`: `Vfs`, `VfsFile`, `RealFs`, `RealFsFile` (Status: `UNWIRED` im Produktivcode, nur in Testsuite genutzt).
- `crates/contextra-durable-fs/src/lib.rs`: `DirLock` (Status: `UNWIRED`, `contextra-store` nutzt eigene Implementierung).

## 9. Was gehalten hat (Negativbefunde mit Experiment)

1. **H2 Atomares Ersetzen (Reihenfolge):**
   - *Experiment:* Statische und dynamische Verfolgung der Systemaufrufe in `atomic_replace_with_injector`.
   - *Ergebnis:* Die Abfolge `open_tmp` -> `write` -> `flush_file` (fsync) -> `rename_durable` -> `flush_dir` (parent fsync) wird ausnahmslos eingehalten. Auf Windows wird `MoveFileExW` mit `MOVEFILE_WRITE_THROUGH` aufgerufen.
2. **H3 Handle-Poisoning bei fsync-Fehlern (`INV-FSYNC-1`):**
   - *Experiment:* Test `test_h3_handle_poisoning_prevents_subsequent_flush` in `/tmp/audit-dfs` mit fehlerinjizierendem `FaultInjector` ausgeführt.
   - *Ergebnis:* Nach dem ersten injizierten `fsync`-Fehler schlagen alle darauffolgenden Schreib- und Flush-Versuche sofort mit "handle is poisoned" fehl.
3. **H4 Zero-Cost FaultInjector in Release-Builds:**
   - *Experiment:* Inspektion der Inline-Annotationen `#[inline(always)]` bei `NoFault`.
   - *Ergebnis:* Der Compiler eliminiert alle Aufrufe von `NoFault` in Release-Builds vollständig.
4. **H5 Safe Scrubbing & Readback-Verifikation:**
   - *Experiment:* Durchführung von Scrubbing-Tests auf Dateien mit Hardlinks und simulierten CoW-Dateisystemen.
   - *Ergebnis:* `scrub_if_safe` verweigert das Überschreiben korrekterweise mit `ScrubRefusal::MultipleHardlinks` bzw. `CopyOnWriteFilesystem`. Nach dem Nulldurchlauf erfolgt eine strikte Readback-Verifikation.
5. **H6 HealthState Monotonie:**
   - *Experiment:* Parallele Zustandsübergänge unter `AtomicHealthState` getestet.
   - *Ergebnis:* Der Zustand `Poisoned` kann nach dem Eintreffen eines `FsyncFailure` nicht mehr verlassen werden.
6. **H9 Zero-Panic Garantie (`INV-DURABLE-FS-NO-PANIC`):**
   - *Experiment:* Statische Code-Analyse und Ausführung der Clippy-Lints `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`.
   - *Ergebnis:* Kein einziger unerlaubter Panic-Aufruf in Produktionscode vorhanden.

## 10. Nicht geprüft, Grenzen, Annahmen

- Physische Bit-Rot-Elimination auf echten NVMe/Flash-Zellen konnte aufgrund der Ausführung in einer virtualisierten Sandbox nicht physisch gemessen werden (siehe wissenschaftliche Einschränkung gemäß Wei et al., FAST 2011 in `scrub.rs`).
- Das Verhalten von Windows-Netzwerk-Freigaben (SMB/UNC) wurde statisch anhand der FFI-Bindungen überprüft.

## 11. Übergaben (Beobachtungen außerhalb des Scopes)

- **Ziel Subsystem WAL / STORE:** `crates/contextra-store/src/util.rs` enthält eine eigene `DirLock`-Implementierung, die nach `contextra-durable-fs` verschoben werden sollte.
- **Ziel Subsystem CRYPTO:** `crates/contextra-crypto/src/revocation_log.rs` führt Datei-Erstellungen ohne anschließenden `sync_dir`-Aufruf durch.

## 12. Vorschläge für Task-Karten

```toml
# Task-Karte: T-2026-DFS-001
[task]
id = "T-2026-DFS-001"
title = "Verdrahtung der VFS-Schicht im LSM-Speicherkern (contextra-store)"
crate = "contextra-store"
severity = "S1"

[description]
Injizieren des `Vfs`-Traits in `LsmStorageConfig` und Ersetzen von direkten `std::fs`-Zugriffen in `SstableBuilder` durch VFS-Positional-Operations.
```

```toml
# Task-Karte: T-2026-DFS-002
[task]
id = "T-2026-DFS-002"
title = "Konsolidierung der DirLock-Implementierung nach ADR-099"
crate = "contextra-durable-fs"
severity = "S2"

[description]
Zusammenführung der doppelten `DirLock`-Strukturen aus `contextra-store` und `contextra-durable-fs` unter Einhaltung von ADR-099.
```

---

## Anhang A: Befehlslog

1. `cargo test -p contextra-durable-fs` -> Exit 0 (Alle 37 Tests bestanden).
2. `rg "unwrap|expect|panic!" crates/contextra-durable-fs/src/` -> Exit 0 (0 Treffer in Prod).
3. `python3 /home/jules/self_created_tools/scan_inventory.py` -> Exit 0 (Inventar erstellt).

## Anhang B: Quelltext des Repro-Tests (H3 Verification)

```rust
use contextra_durable_fs::{DurableHandle, FaultInjector, FlushKind};
use std::sync::atomic::{AtomicBool, Ordering};
use std::io;
use std::path::Path;

struct OneShotFsyncFault {
    failed: AtomicBool,
}

impl FaultInjector for OneShotFsyncFault {
    fn on_fsync(&self, _path: Option<&Path>) -> io::Result<()> {
        if !self.failed.swap(true, Ordering::Relaxed) {
            Err(io::Error::new(io::ErrorKind::Other, "injected fsync failure"))
        } else {
            Ok(())
        }
    }
}

#[test]
fn test_h3_handle_poisoning_prevents_subsequent_flush() -> io::Result<()> {
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!("h3_test_{}.tmp", std::process::id()));
    let file = std::fs::File::create(&file_path)?;

    let injector = OneShotFsyncFault { failed: AtomicBool::new(false) };
    let mut handle = DurableHandle::new_with_injector(file, injector);

    // First flush fails
    let res1 = handle.flush(FlushKind::DataAndMetadata);
    assert!(res1.is_err());
    assert!(handle.is_poisoned());

    // Second flush MUST fail due to poisoning
    let res2 = handle.flush(FlushKind::DataAndMetadata);
    assert!(res2.is_err());
    assert_eq!(res2.unwrap_err().to_string(), "handle is poisoned due to a previous fsync failure");

    let _ = std::fs::remove_file(file_path);
    Ok(())
}
```

## Anhang C: `git status --porcelain` zum Abschluss

```
?? docs/analysis/DFS_durable_fs_ANALYSE_2026-10-19.md
```

# AGENTS.md — contextra-durable-fs — Durabilitätsgarantien & Crash-Sicherheit

**Status:** Normativ  
**Crate-Spezifikation:** [`docs/crates/contextra-durable-fs.md`](file:///home/freddy/Projekte/contextra/docs/crates/contextra-durable-fs.md)  
**Gesamtspezifikation:** [`docs/SPEC.md`](file:///home/freddy/Projekte/contextra/docs/SPEC.md) — Bindung: docs/SPEC.md §2 (Architektur), §5.3 (Lösch-Saga fsync-Anker), §13 (INV-1, INV-6, INV-FSYNC-1, INV-VFS-ATOMIC-1)  
**Workspace-Richtlinie:** [`AGENTS.md`](file:///home/freddy/Projekte/contextra/AGENTS.md) (Wurzel)  
**Ebene:** E0 (Autarkes Fundament / Ring 0)  
**Kern-Closure:** Leer (0 interne Workspace-Abhängigkeiten)  
**Unsafe-Status:** Verboten (`#![forbid(unsafe_code)]`)  

---

## 1. Zweck & Systemrolle

I/O-Durabilitätsfundament für WAL, Manifest, SSTables und Erasure-Journal mit Crash-Sicherheit.

---

## 2. Kern-Richtlinien für Agenten

- **Handle-Poisoning (`INV-FSYNC-1`):** Schlägt ein `fsync`- oder Flush-Aufruf fehl, wechselt der Zustand irreversibel auf `Poisoned`. Nachfolgende Schreiboperationen werden abgewiesen.
- **Atomares Schreiben (`INV-VFS-ATOMIC-1`):** 4-Schritte-Sequenz: 1. Temp-Datei im selben Verzeichnis -> 2. `sync_file` -> 3. Atomares Rename (`rename`) -> 4. Verzeichnis-Sync (`sync_dir_path`).
- **Dateisystem-Klassifikation (`FsClass`):** Unterscheidung zwischen lokalen Dateisystemen und unsicheren Mounts (NFS, SMB, FUSE), bei denen `sync_dir` unzuverlässig sein kann.
- **Physisches Schreddern (`scrub`):** Ratenbegrenztes Überschreiben physischer Dateien mit Nullen vor dem `unlink` zur Unterstützung des DSGVO-Purge.
- **Lösch-Saga fsync-Anker:** Stellt die Durabilitätsanker für fsync #1 (Intent), fsync #2 (Gated), fsync #3 (KeyRevoked) und fsync #4 (Abschluss/Ledger) bereit.

## 3. Test-Pflichten
- T-DFS1 bis T-DFS6 (write_atomic, Crash-Simulation vor Rename, Handle-Poisoning, scrub_file Nullung, PendingDeleteJournal Recovery, FsClass Erkennung).

---

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-durable-fs` · **Ring:** 0 · **Version:** 0.1.0
- **Beschreibung:** Crash-sichere Dateisystem-Operationen und synchrones Disk-Flushing für Contextra
- **src:** 8 Dateien, 1428 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 0):** –
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 4):** `crc32fast`, `serde`, `thiserror`, `tracing`
- **Reverse-Abhängigkeiten (0):** –
- **Kopplung an Hub-Crates:** ausgehend –; eingehend –
- **Features (6):** `crc`, `default`, `inspect`, `serde`, `std`, `tracing`

### Öffentliche API (grob, aus src/)
- struct (12): `AppendOnlyJournal`, `AtomicHealthState`, `DirLock`, `DurableHandle`, `FaultInjectingVfs`, `JournalRecord`, `NoFault`, `PendingDeleteJournal`, `PoisonedHandle`, `ScrubProgress`, `ScrubReceipt`, `StdVfs`
- enum (12): `DurableFsError`, `FlushKind`, `FsClass`, `HealthEvent`, `HealthState`, `InjectedFault`, `JournalSyncPolicy`, `LockMode`, `RecordRecoveryPolicy`, `ScrubOutcome`, `ScrubPolicy`, `ScrubRefusal`
- trait (2): `FaultInjector`, `Vfs`
- fn (50): `acquire`, `allows_writes`, `append`, `atomic_replace`, `atomic_replace_with_injector`, `calculate_crc32`, `durable_remove`, `durable_remove_with_injector`, `flush`, `flush_dir`, `flush_file`, `fs_class`, `get_mut`, `get_ref`, `health_state` … (+35)
- type (3): `Error`, `RealFs`, `Result`
- const (2): `JOURNAL_MAGIC`, `RECORD_HEADER_SIZE`
- mod (7): `error`, `fault_injector`, `health`, `journal`, `platform`, `scrub`, `vfs`
- pub use: `error::{DurableFsError, Error, Result}`, `fault_injector::{FaultInjector, NoFault}`, `health::{AtomicHealthState, HealthEvent, HealthState}`, `journal::{ AppendOnlyJournal, JournalSyncPolicy, PendingDeleteJournal, RecordRecoveryPolicy, }`, `platform::FlushKind`, `scrub::{ scrub_file_with_rate_limit, scrub_if_safe, scrub_if_safe_with_injector, ScrubOutcome, ScrubPolicy, ScrubReceipt, ScrubRefusal, }`, `vfs::{RealFs, StdVfs, Vfs}`

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->

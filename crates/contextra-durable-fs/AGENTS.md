# AGENTS.md — `contextra-durable-fs`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

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

# Architektur & Schnittstellenvertrag contextra-store

---

## 1. Crate-Gesamtkarte & Schichtenarchitektur

```
+-------------------------------------------------------------+
|                      contextra-store                        |
+-------------------------------------------------------------+
| 1. Basis-Module & I/O-Nähte                                 |
|    - error.rs         : Lokaler StoreError / StoreResult     |
|    - traits.rs        : StoreFs, StoreFile (autark)          |
|    - io_seam/         : Vfs, RealFs, StoreIo (modular)       |
|    - fsync.rs         : Durable sync primitives (INV-FSYNC-1)|
|    - key_source.rs    : KeyMaterialSource                    |
|    - util.rs          : RecordSealer, RecordAuthenticator    |
|    - budget.rs        : ResourceBudget, ResourceTracker      |
|    - stats.rs         : StorageStats                         |
|    - tenant_codec.rs  : TenantKeyCodec                       |
+-------------------------------------------------------------+
| 2. Speicherkern (LSM-Subsysteme)                            |
|    - wal/             : Append-Only Log, HMAC-Kette, Flusher |
|    - memtable.rs      : Nebenläufige BTreeMap im RAM         |
|    - sstable/         : Unveränderliche Segmente (Format V4) |
|    - manifest/        : VersionSet & Commit-Journal (MFMN)   |
|    - compaction/      : Leveled / Adaptive Compaction Planner|
|    - system_pressure  : Speicherdruck-Erkennung              |
|    - lsm/             : Haupt-Orchestrierung LsmStorage      |
+-------------------------------------------------------------+
| 3. Brücken & Integration (Feature `workspace`)              |
|    - compat/engine/   : StorageEngine Port-Adapter           |
|    - compat/scanner/  : LayerScanner & Purger Integration    |
|    - compat/error.rs  : From<StoreError> for ContextraError  |
+-------------------------------------------------------------+
```

## 2. Interne Abhängigkeitsrichtung

- Basis-Module (`error`, `traits`, `io_seam`, `fsync`, `key_source`, `util`, `budget`, `stats`) sind unabhängig voneinander und vom Speicherkern.
- Der Speicherkern (`wal`, `memtable`, `sstable`, `manifest`, `compaction`, `lsm`) nutzt die Basis-Module.
- Die Kompatibilitätsschicht `compat/` adaptiert den Speicherkern auf externe Ports und Typen.
- Zirkuläre Abhängigkeiten innerhalb des Crates sind verboten.

## 3. Sperrenhierarchie & Deadlock-Vermeidung

Zur Vermeidung von Deadlocks gilt ausnahmslos folgende Akquisitionsreihenfolge:

1. `commit_mutex` (Transaktionsordnung und Group-Commit)
2. `wal.truncate_lock` (Abschneiden des WAL-Logs)
3. `memtable_shard_lock` (Aktualisierung von RAM-Partitionen)
4. `sstable_manifest_lock` (Manifest-Commit für Dateigenerationen)

*Regel:* Niemals darf ein `parking_lot`-Lock über einen `.await`-Punkt gehalten werden.

## 4. Invarianten
- **INV-FSYNC-1:** Alle synchronen I/O-Operationen propagieren Fehler strikt fail-closed.
- **INV-STORE-1 bis INV-STORE-5:** Siehe `AGENTS.md`.
- **Zero Panic:** Keine `unwrap()`, `expect()` oder Panics in produktiven Pfaden.

# ARCHITECTURE: Compaction & System-Pressure (Komponente D)

## 1. Übersicht & Datenfluss
Das Compaction- und System-Pressure-Subsystem von `contextra-store` steuert die Bereinigung, Zusammenführung und Drosselung persistenter SSTables im LSM-Tree.

### Ablauf: Auswahl → Merge → Installation → Aufräumen
1. **Auswahl (`select.rs` / `adaptive/planner.rs`)**:
   - `select_compaction_candidates`: Identifiziert zusammenhängende Kandidaten chronologisch sortiert nach `min_tx_id`.
   - `CostBasedAdaptivePlanner`: Passt die Compaction-Strategie dynamisch an (Read-optimiert, Write-optimiert oder Balanced) basierend auf dem `read_write_ratio`.
   - Invariante `INV-COMPACTION-ADAPTIVE-1`: Ein Tombstone darf niemals GC'd werden, wenn eine aktive Snapshot-Sequenznummer (`min_active_seqno`) ihn noch referenziert.
2. **Merge (`engine.rs` / `engine/throttle.rs`)**:
   - Multi-Way-Merge mit Min-Heap (`HeapItem`).
   - Priorisierung der neuesten Sequenznummer bei Duplikaten.
   - Anwenden des `MergeOperator` falls konfiguriert.
   - Drosselung über `calculate_throttle_delay` wenn `SystemPressure` auf `Elevated` oder `Critical` steht.
3. **Installation (`engine/install.rs`)**:
   - Eindeutige SSTable-Pfad-Generierung (`generate_sst_path`).
   - Atomares Schreiben und fsync der neuen SSTable via `Vfs`.
   - Atomarer Manifest-Edit zur Registrierung der neuen SSTable.
4. **Aufräumen**:
   - Sicheres Löschen alter SSTable-Dateien nach erfolgreichem Manifest-Commit.

## 2. Kostenmodell & Heuristik
Das adaptive Compaction-Modell orientiert sich an EcoTune / ArceKV:
- `read_ratio = read_count / (read_count + write_count)`
- Wenn `read_ratio >= threshold` (Standard 0.70): Aggressive Verdichtung (`ReadOptimizedAggressive`), um Leseverstärkung zu minimieren.
- Wenn `read_ratio <= 1.0 - threshold` (<= 0.30): Size-Tiered Verdichtung (`WriteOptimizedSTCS`), um Schreibverstärkung zu minimieren.
- Dazwischen: Ausgewogene Zusammenführung (`Balanced`).

## 3. System-Pressure Überwachung (`system_pressure/`)
- `SystemPressureMonitor` sammelt Signale:
  - `wal_queue_depth`: Tiefe der anstehenden Group-Commit Follower.
  - `scheduler_queue_depth`: Tokio-Scheduler Queue-Verhältnis.
  - `blocking_util`: Auslastung des Blocking-Pools.
- Pegel: `Normal` (0.0), `Elevated` (1.0), `Critical` (2.0) gemeldet an `MetricsSink`.

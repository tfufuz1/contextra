# LSM Subsystem Audit Report — `crates/memfuse-store/src/lsm.rs`

**Auditor:** Jules (Senior Rust Storage-Engine Engineer)
**Timestamp:** 2026-09-13T01:42:00Z
**Session:** b72f020d
**Target:** `crates/memfuse-store/src/lsm.rs` (XL file, ~4895 LOC)
**Mode:** AUDIT (Subsystem Audit — No functional code modifications)

---

## 1. LSM-Spezielles Invarianten-Audit

### C-1: Startup-Flush VOR WAL-Löschung (Durability)
- **Status:** **OK (VERIFIED)**
- **Befund / Code-Stelle:** `lsm.rs` Zeilen 598–630. Bei `LsmStorage::open()` wird bei `replayed_size > 0 && !wal_files.is_empty()` zwingend ein `storage.flush().await` ausgeführt, bevor alte replayed WAL-Dateien via `tokio::fs::remove_file()` gelöscht werden. Falls `flush()` fehlschlägt, bricht `open()` mit `MemFuseError::Storage` ab und behält die WAL-Dateien auf Disk.

### H-1: Per-Source-Limit statt globalem Limit in `scan_prefix_bounded` (Scan Memory Isolation)
- **Status:** **OK (VERIFIED)**
- **Befund / Code-Stelle:** `lsm.rs` Zeilen 1927–2050. `scan_prefix_bounded` beschränkt den Scan pro Storage-Source (aktive MemTable, immutable MemTables, SSTables) auf `candidate_limit = limit + 1` Einträge oberhalb des Paginierungs-Cursors. Dies verhindert unbegrenzte Memory-Materialisierung vor Tombstone-Filtering und schützt vor OOM bei großen SSTable-Iterationen.

### H-2: `flush_counter` Inkrementierung & Monotonie
- **Status:** **OK (VERIFIED)**
- **Befund / Code-Stelle:** `lsm.rs` Zeilen 589 (Initialisierung aus `max_wal_id + 1`), Zeilen 1698–1740 in `flush()`. Die Inkrementierung `self.flush_counter.fetch_add(1, Ordering::SeqCst)` erfolgt vor der Generierung des WAL-Dateipfads `wal-{:020}.log`. Monotonie ist über Instanz-Grenzen hinweg durch Parsem von `max_wal_id` beim Replay abgesichert.

### H-3: Compaction-Engine Instanz-Persistenz
- **Status:** **OK (VERIFIED)**
- **Befund / Code-Stelle:** `lsm.rs` Zeile 256 (`compaction_engine: Arc<CompactionEngine>`). `LsmStorage` hält eine persisente `CompactionEngine`-Instanz, sodass der interne SSTable-Dateinamen-Zähler nicht bei jedem `maybe_compact()` Aufruf zurückgesetzt wird. Dadurch sind SSTable-Namenskollisionen ausgeschlossen.

### H-7: Atomarität von Reads in `scan_prefix_at()` (Snapshot Isolation)
- **Status:** **OK (VERIFIED)**
- **Befund / Code-Stelle:** `lsm.rs` Zeile 1935. `last_committed_tx` wird strikt einmalig am Methodenanfang per `AtomicU64::load(Ordering::Acquire)` in eine lokale Variable geladen. Während der Iteration über MemTables und SSTables findet kein erneutes Auslesen von `last_committed_tx` statt, was Invariants gegen Split-Brain/Snapshot-Inversion garantiert.

### M-6: Memory-Budget Tracking & Drift-Akkumulierung
- **Status:** **OK (VERIFIED)**
- **Befund / Code-Stelle:** `lsm.rs` Zeilen 266, 736–740 (`budget_tracking_drift_bytes`). Wenn `consume_memory()` während `commit()` wegen überschrittenem RAM-Limit fehlschlägt, wird die Ungenauigkeit im atomaren Zähler `budget_tracking_drift_bytes` akkumuliert und über `budget_tracking_drift_bytes()` exportiert, bis beim nächsten Flush durch `release_memory()` wieder präziser Zustand hergestellt wird.

### M-9: `put_if_absent` Visibility-Isolation
- **Status:** **OK (VERIFIED)**
- **Befund / Code-Stelle:** `lsm.rs` Zeilen 1157–1200 & 4327–4350 (Test). `put_if_absent` stagt Einträge im Transaktionspuffer (`tx_buffer`). Konkurrierende Aufrufe von `put_if_absent` oder `get` aus anderen Transaktionen sehen uncommittete Einträge nicht, da Sichtbarkeitsprüfungen auf `last_committed_tx` basieren.

### APM-39: Read/Write Amplification Management
- **Status:** **OK (VERIFIED)**
- **Befund / Code-Stelle:** Compaction wird über `CompactionConfig` (Tiered Size Ratio) und `SystemPressureMonitor` gesteuert, um Read-/Write-Amplification im Zielbereich zu halten.

---

## 2. Zusammenfassung des Subsystem-Audits

| Invariante / Thema | Status | Befund / Anmerkung |
|---|---|---|
| C-1 Startup-Flush vor WAL-Löschung | **OK** | Garantierte Persistenz nach Replay |
| H-1 Per-Source Bounded Scan Limit | **OK** | Max. O(N * limit) Speicher |
| H-2 `flush_counter` Monotonie | **OK** | Safe Atomic fetch_add |
| H-3 CompactionEngine Counter Persistence | **OK** | Kein Reset bei `maybe_compact()` |
| H-7 `scan_prefix_at` Snapshot Single Load | **OK** | Atomic Acquire am Start |
| M-6 Budget-Drift Akkumulierung | **OK** | Atomic Drift Tracking |
| M-9 `put_if_absent` Sichtbarkeits-Isolation | **OK** | Staging im TxBuffer vor Commit |

---
*Ende des LSM-Subsystem-Audit-Reports vom 2026-09-13.*

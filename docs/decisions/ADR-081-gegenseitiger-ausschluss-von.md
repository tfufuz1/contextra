# ADR-081: gegenseitiger Ausschluss von MaintenanceScheduler und ConsolidationEngine

* **Datum:** 2026-09-12
* **Status:** ✅ Final
* **Target Path:** crates/memfuse-db/src/collection/mod.rs, crates/memfuse-db/src/consolidation_executor.rs, crates/memfuse-db/src/maintenance_scheduler.rs
* **Kontext / Auslöser:** Problem H-19: Bei künftiger oder paralleler Aktivierung von `MaintenanceScheduler` und `ConsolidationEngine` besteht das Risiko, dass beide Background-Pfade gleichzeitig einen `execute_consolidation_pass`-Aufruf auf derselben `Collection`-Instanz ausführen.

## Entscheidung
1. `MaintenanceScheduler` und `ConsolidationEngine` werden als sich gegenseitig ausschließende Ausführungspfade behandelt, niemals als gleichzeitig aktive Background-Tasks auf derselben `Collection`.
2. Mechanismus: Ein prozessweiter `consolidation_guard: Arc<tokio::sync::Mutex<()>>` pro `Collection`-Instanz, den beide Pfade vor Beginn eines Konsolidierungs-Durchlaufs erwerben müssen (`try_lock`, nicht blockierend).
3. Bei Kollision (`Err` von `try_lock()`) überspringt der unterlegene Pfad den aktuellen Tick mit einer `tracing::warn!`-Meldung statt zu blockieren, um Head-of-Line-Blocking im Scheduler zu vermeiden und P2 (Zero-Panic) einzuhalten.
4. Geltungsbereich: Gilt unbeschränkt auch dann, wenn `MaintenanceScheduler` zu einem späteren Zeitpunkt produktiv verdrahtet wird.

## Begründung
- **Nicht-blockierende Skip-Semantik (P2/P11):** Die Verwendung von `try_lock()` verhindert Head-of-Line-Blocking im periodischen Tick des MaintenanceSchedulers und schützt die Systemlatenz.
- **Doppelverarbeitungs-Schutz:** Verhindert TOCTOU-Rassen und doppelte Tombstone- / Graph-Kaskaden-Operationen auf derselben Collection.

## Konsequenzen
- `Collection` hält ein neues Feld `consolidation_guard: Arc<tokio::sync::Mutex<()>>`.
- `execute_consolidation_pass` erwirbt das Lock per `try_lock()`. Falls bereits gesperrt, liefert es `Ok(ConsolidationPhaseResult::default())` zurück und beendet den Pass ohne Fehler.

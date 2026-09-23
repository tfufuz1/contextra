# ADR-080: Weak<RouterEngine>-Injection in MemFuse für H-17 Stats Live-Daten

* **Datum:** 2026-09-12
* **Status:** Akzeptiert
* **Target Path:** crates/memfuse-db/src/lib.rs, crates/memfuse-router/src/router.rs
* **Kontext / Auslöser:** Problem H-17: `MemFuse::stats()` / `PyDbStats` liefert die Felder `drift_status`, `calibration_ece`, `last_calibration_at` und `pid_pool_size` derzeit als Platzhalterwerte, weil `memfuse-db` (Layer 5) keine Referenz auf `RouterEngine` (Layer 6) hält. Eine direkte starke Referenz (`Arc<RouterEngine>`) würde die DAG-Schichtung (P1) verletzen und einen zirkulären Bezug erzeugen.

## Entscheidung
1. `MemFuse` (Layer 5) erhält ein optionales Feld `router: Option<Weak<RouterEngine>>`.
2. Aufrufer auf Integrationsebene (z. B. Orchestrierung / `memfuse-py` / Bootstrap) injizieren nach der Initialisierung von `RouterEngine` eine schwache Referenz (`Weak<RouterEngine>`) in die `MemFuse`-Instanz.
3. Bei Aufruf von `MemFuse::stats()` wird versucht, die schwache Referenz via `Weak::upgrade()` hochzustufen:
   - Bei erfolgreichem Upgrade fragt `stats()` die Live-Daten über die bestehenden Getter-Funktionen ab:
     - `LyapunovDriftWatcher::status_str()` → `drift_status`
     - `IsotonicCalibrator::last_calibration_at()` → `last_calibration_at`
     - `PidController::current_pool_size()` → `pid_pool_size`
   - Bei fehlgeschlagenem Upgrade (`None` oder RouterEngine bereits dropped) werden weiterhin saubere Platzhalterwerte zurückgegeben — ohne Panic oder Error-Rückgabe.

## Begründung
- **DAG-Integrität (P1):** `memfuse-db` (Layer 5) übernimmt keine Eigentümerschaft oder starke Referenz auf `memfuse-router` (Layer 6). Es entsteht kein Kreisschluss zwischen Layer 5 und Layer 6.
- **Robustheit & Resilienz:** Das Fehlschlagen von `Weak::upgrade()` wird gracefully abgefangen. Das Verhalten im Unconnected State bleibt unverändert stabil.

## Konsequenzen
- Die tatsächliche physische Verdrahtung (Implementieren des `Weak`-Feldes in `MemFuse`, Aufrufen der Getter in `stats()` sowie Setzen des `Weak`-Felds beim MemFuse-Bootstrap) ist im Woche-3–4-Implementierungstask (H-17 Vollimplementierung) zu erledigen — dieser ADR dokumentiert ausschließlich die Architekturentscheidung.

---

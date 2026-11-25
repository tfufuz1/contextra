# ADR-079: Event-getriebene Drift-Erkennung (F-11) im Router statt im periodischen Maintenance-Tick

* **Datum:** 2026-09-12
* **Status:** ✅ Final
* **Target Path:** crates/contextra-db/src/maintenance_scheduler.rs, crates/contextra-router/src/router.rs
* **Kontext / Auslöser:** F-11 (`LyapunovDriftWatcher.update()`) schützt vor unbemerktem Verfall der Routing-Kalibrierung. Ursprüngliche Spezifikationsentwürfe deuten auf eine periodische Taktung hin.
* **Entscheidung:** F-11 (`LyapunovDriftWatcher.update()`) ist bewusst NICHT im periodischen 60s-Tick des `MaintenanceScheduler` enthalten. F-11 wird stattdessen reaktionsschnell & event-driven direkt nach jeder Routing-Entscheidung in `crates/contextra-router/src/router.rs` aufgerufen.
* **Begründung:** Eine Auslagerung der Drift-Erkennung in ein periodisches Intervall (z. B. 60s) würde zu verzögerten Reaktionen bei rascher Drift führen. Die direkte event-getriebene Auswertung sichert minimale Reaktionszeiten.
* **Konsequenzen:**
  - `MaintenanceScheduler` ruft F-11 im Hintergrund-Tick nicht auf.
  - Veraltete Spezifikationsreferenzen (§10.1) in Code-Kommentaren werden durch expliziten ADR-079-Verweis ersetzt.

---

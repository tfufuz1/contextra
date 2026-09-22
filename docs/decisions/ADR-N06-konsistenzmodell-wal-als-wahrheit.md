# ADR-N06: Konsistenzmodell — WAL als einzige Wahrheit vs. 2PC-Härtung

* **Status:** Proposed / Pending Product-Owner-Entscheidung (offen, Gesamtspezifikation §A2.4 Nr. 1)
* **Datum:** 2026-09-17
* **Kontext / Auslöser:**
  In heterogenen verteilten und mehrkomponentigen Speichersystemen wie MemFuse Cognitive OS stellt sich bei Multi-Engine-Transaktionen (LSM-Store, HNSW/DiskANN-Vektorindizes, CSR-Wissensgraph, BM25-Textindizes) die Frage nach dem primären Konsistenz- und Recovery-Modell.

  Bisher existiert ein zweiphasiges Commit-Protokoll (2PC) mit unvollständiger Intent-Key-Pessimisierung, das bei plötzlichen Prozess-Crashes oder I/O-Teilausfällen komplexe Invarianten-Verletzungen zwischen primärem Storage und Indizes auslösen kann.

  Gemäß Gesamtspezifikation §A2.4 Nr. 1 und §20.3 muss eine fundamentale Architektur-Entscheidung zwischen zwei Konsistenzmodellen getroffen werden, ohne den Beschluss vorab im Quellcode zu präjudizieren.

---

## 1. Neutrale Evaluierung der Optionen

### Option A: Zweiphasiges Commit-Protokoll (2PC) mit gehärteten Intent-Keys
* **Funktionsweise:**
  Jede Transaktion schreibt in Phase 1 Staging- bzw. Intent-Einträge ("Prepare") in alle beteiligten Subsysteme (LSM, Vektorindex, Graph, BM25) unter Verwendung expliziter Distributiv-Locks oder Intent-Schlüssel. Erst nach positiver Rückmeldung aller Subsysteme wird in Phase 2 ein globaler `Commit`-Marker im WAL und in den Systemen gesetzt.
* **Vorteile:**
  - Synchrones Lesen nach Commit spiegelt sofort den exakten In-Memory-Zustand aller Subsysteme wider.
  - Keine zeitliche Asynchronität zwischen Index und Hauptspeicher.
* **Nachteile / Risiken:**
  - Hohe Komplexität beim Handhaben von Teil-Crashes während Phase 2 (Distributed Deadlocks, verwaiste Intents).
  - Hoher Performance-Overhead durch Multi-Rundtrip-Protokolle und Koordinationssperren.
  - Erhöhte Anfälligkeit für TOCTOU-Gefahren und Distributed Deadlocks unter hoher Nebenläufigkeit.

### Option B: WAL als einzige Wahrheit (Derived State via `applied_lsn`) — *Spezifikations-Empfehlung*
* **Funktionsweise:**
  Nur das Write-Ahead Log (`memfuse-store::wal`) gilt als unumstößliche primäre Datenquelle ("Single Source of Truth"). Transaktionen schreiben ausschließlich einen sequentiellen, HMAC-gesicherten WAL-Record mit `WalOp::TxEnd { committed: true }`.
  Secondary Indizes (HNSW, CSR-Graph, BM25) sind reine abgeleitete Sichten ("Derived State"), die Änderungen asynchron oder synchron-gepuffert konsumieren und ihren Fortschritt über eine monotonically steigende `applied_lsn` nachhalten. Bei einem Crash wird der abgeleitete Zustand ausgehend vom letzten validen `applied_lsn`-Checkpoint aus dem WAL deterministisch replayed und rekonstruiert.
* **Vorteile:**
  - Drastische Vereinfachung des Recovery-Pfads und Vermeidung verteilter Transaktionszustände.
  - Höhere Schreib-Performanz durch Beseitigung von Multi-Engine 2PC-Locks.
  - Garantierte Deterministik im Crash-Recovery-Fall.
* **Nachteile / Risiken:**
  - Potenziell längere Recovery-Zeiten beim Systemstart, wenn Indizes nach einem unsauberen Shutdown aus dem WAL nachgezogen werden müssen.
  - Lese-Sichten müssen bei asynchronem Index-Replay das `applied_lsn`-Wasserzeichen berücksichtigen (Read-Your-Writes Guarantees erfordern ggf. Catch-up-Waits).

---

## 2. Entscheidungskriterien & PO-Entscheidungsvorbehalt

Gemäß **Gesamtspezifikation §A2.4 Nr. 1** darf diese Entscheidung nicht durch Entwicklungsarbeiten präjudiziert werden. Die Entscheidung obliegt dem Product Owner basierend auf folgenden Grundlagen:

1. **PO Recovery-Zeit-Ziel (RTO / Recovery Time Objective):** Der Product Owner muss das akzeptable Zeitfenster für das Wiederanlaufen des Systems nach einem harten Crash (z.B. < 500 ms vs. < 5 s) definieren.
2. **Phase 3a Crash-Injektions-Spike (`memfuse-testkit` / Fault-VFS):**
   Als Entscheidungsgrundlage dient ein empirischer Benchmark und Crash-Simulationstest unter Verwendung der `Fault-VFS`-Infrastruktur im `memfuse-testkit` (Phase 3a der Migration). Der Spike misst:
   - Durchsatz- und Latenzunterschiede zwischen 2PC und WAL-Only im Regelbetrieb.
   - Replay-Dauer und Speicherverbrauch bei der WAL-Rekonstruktion nach simulierten Systemabstürzen.

---

## 3. Status & Exit-Kriterium

* **Aktueller Status:** `Proposed / Pending Product-Owner-Entscheidung`
* **Exit-Kriterium für Statusübergang zu "Beschlossen":**
  1. Durchführung des Phase 3a Crash-Injektions-Spikes mit `memfuse-testkit` (Fault-VFS).
  2. Vorgelegter Evaluierungsbericht zur Recovery-Zeit und Durchsatz-Metriken.
  3. Formeller Beschluss des Product Owners zur Festlegung von Option A oder Option B.

---

## 4. Konsequenzen

* Der Produktionscode darf vor dem PO-Beschluss keine Annahmen treffen, die eine der Optionen unmöglich machen.
* Das `memfuse-testkit` bereitet in Phase 0R/3a die Testwerkzeuge für die Fault-VFS Simulation vor.

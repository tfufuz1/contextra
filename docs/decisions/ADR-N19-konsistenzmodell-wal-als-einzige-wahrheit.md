# ADR-N19: F4 Konsistenzmodell — WAL/Store als einzige Wahrheit (Akzeptanz ADR-N06 Option B)

* **Status:** Accepted
* **Datum:** 2026-09-30
* **Kontext / Auslöser:**
  In Multi-Engine-Speichersystemen (LSM-Store, HNSW-Vektorindex, CSR-Graph, BM25-Textindex) führen koordinierte Multi-Store-Transaktionen bei unvollständigen Commit-Klammern oder System-Crashes zu komplexen Inkonsistenzen.
  ADR-023 beschrieb zuvor fälschlicherweise eine kompensierende Transaktion für `relate()`-Operationen unter der unzutreffenden Bezeichnung "2PC". In Wahrheit handelte es sich dabei nicht um ein verteiltes Zweiphasen-Commit-Protokoll, sondern um eine Saga mit Kompensation.
  Zur Vermeidung von verteilt-komplexen Transaktionszuständen, TOCTOU-Gefahren und unsauberen Invarianten erfordert das Gesamtsystem ein klares, vereinfachtes Konsistenzmodell (Gesamtspezifikation §F4, §A2.4 Nr. 1).

## Entscheidung

1. **Akzeptanz von ADR-N06 Option B ("WAL/Store als Single Source of Truth"):**
   Nur das Write-Ahead Log (`contextra-store::wal`) und der primäre LSM-Store gelten als unumstößliche primäre Datenquelle ("Single Source of Truth") [V].
   Sämtliche Sekundärindizes (HNSW, CSR-Graph, BM25) sind reine **abgeleitete Zustände ("Derived State / Cache")** [V].

2. **Deterministisches Replay & Sequenz-Wasserzeichen (`applied_seq`):**
   Indizes konsumieren Modifikationen aus dem WAL und halten ihren Fortschritt über einen monoton steigenden Sequenzzähler (`applied_seq` / `applied_lsn`) nach [V]. Nach einem unsauberen Shutdown werden Sekundärindizes beim Systemstart ausgehend vom letzten validen Checkpoint aus dem WAL deterministisch neu aufgebaut oder nachgezogen.

3. **Ersetzung von ADR-023:**
   ADR-023 wird vollumfänglich durch dieses ADR **superseded** [S]. Die Bezeichnung "2PC" im bisherigen Kontext wird als fachlich falsch korrigiert: Es handelt sich um eine Saga mit kompensierender Aktion, die durch das primäre WAL-Append-Modell abgelöst wird.

## Alternativen

### Option A: Zweiphasiges Commit-Protokoll (2PC) über alle Subsysteme
* **Vorteile:** Synchrones In-Memory-Lesen zeigt sofort den exakten Stand aller Indizes.
* **Nachteile:** Hohe Sperr-Contention; Distributed Deadlocks; extrem fehleranfälliges Crash-Recovery bei Teil-Crashes; verstoß gegen die Einfachheits-Invariante.
* **Bewertung:** Verworfen (ADR-N06 Option A) [S].

## Konsequenzen

* **Drastische Vereinfachung:** Keine verteilten Locks oder komplexen Multi-Engine 2PC-Transaktionen mehr notwendig [V].
* **Deterministisches Crash-Recovery:** Vollständige Wiederherstellbarkeit aller Indexzustände aus dem WAL [V].
* **Read-Your-Writes:** Lese-Pfade auf Sekundärindizes können bei Bedarf über `applied_seq`-Wasserzeichen warten, bis der Index den Stand der Transaktion erreicht hat [V].

## Reversibilitäts-Bedingung

Eine Umstellung erfordert Nachweise, dass der Start-Replay-Aufwand bei Multiterabyte-Datenmengen das Kaltstart-RTO-Ziel (< 100 ms) ohne Checkpointing dauerhaft verletzt [S].

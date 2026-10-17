# ADR-N06: Konsistenzmodell — WAL als einzige Wahrheit (Akzeptanz Option B)

* **Status:** Accepted (festgelegt durch F4 / ADR-N19)
* **Datum:** 2026-09-17 (Aktualisiert: 2026-09-30)
* **Kontext / Auslöser:**
  In heterogenen verteilten und mehrkomponentigen Speichersystemen wie Contextra Cognitive OS stellt sich bei Multi-Engine-Transaktionen (LSM-Store, HNSW/DiskANN-Vektorindizes, CSR-Wissensgraph, BM25-Textindizes) die Frage nach dem primären Konsistenz- und Recovery-Modell.
  Dieses ADR untersuchte zwei Alternativen: Option A (Zweiphasiges Commit-Protokoll 2PC) und Option B (WAL als einzige Wahrheit mit abgeleiteten Indizes).

## Entscheidung (Beschluss F4 / ADR-N19)

**Option B wurde formell akzeptiert** [V].
1. **Single Source of Truth:** Nur das Write-Ahead Log (`contextra-store::wal`) und der primäre LSM-Store gelten als unumstößliche primäre Datenquelle ("Single Source of Truth") [V].
2. **Abgeleiteter Zustand ("Derived State"):** Sämtliche Sekundärindizes (HNSW, CSR-Graph, BM25) sind reine abgeleitete Caches, die Änderungen asynchron oder synchron-gepuffert aus dem WAL konsumieren und ihren Fortschritt über einen monoton steigenden Sequenzzähler (`applied_seq` / `applied_lsn`) nachhalten [V].
3. **Deterministischer Replay:** Bei einem Systemstart nach einem unsauberen Shutdown werden die Sekundärindizes ausgehend vom letzten validen Checkpoint aus dem WAL deterministisch neu aufgebaut [V].

## Evaluierung der Alternativen

### Option A: Zweiphasiges Commit-Protokoll (2PC) mit Intent-Keys
* **Funktionsweise:** Staging/Intent-Schreibschritte über alle Subsysteme vor dem Commit.
* **Nachteile:** Hohe Komplexität; hohe Lock-Contention; Distributed Deadlocks; fehleranfälliges Crash-Recovery.
* **Bewertung:** Verworfen [S].

### Option B: WAL als einzige Wahrheit (Derived State via `applied_lsn`)
* **Funktionsweise:** Atomares WAL-Append als alleinige Commit-Garantie. Indizes konsumieren WAL-Einträge.
* **Vorteile:** Drastische Vereinfachung des Recovery-Pfads; Beseitigung verteilter Locks; deterministische Wiederherstellung.
* **Bewertung:** **Akzeptiert [V].**

## Konsequenzen

* **Querverweis:** Siehe auch ADR-N19 für die detaillierte Ausgestaltung des Konsistenzmodells und die Formalsupersedung von ADR-023 [S].
* **Entwickler-Vorgabe:** Der Produktionscode verwendet das WAL-Append-Replay-Muster ohne verteilte 2PC-Sperren [V].

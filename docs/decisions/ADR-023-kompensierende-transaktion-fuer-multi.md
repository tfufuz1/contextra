# ADR-023: Kompensierende Transaktion für Multi-Store relate() Operations

* **Status:** Superseded (ersetzt durch ADR-N19 und ADR-N06 Option B)
* **Datum:** 2026-08-28 (Aktualisiert: 2026-09-30)
* **Kontext / Auslöser:**
  `Collection::relate()` führt Operationen über heterogene Storage-Backends (`LsmStorage` und `CsrGraph`) aus. Nachdem `storage.commit(tx)` aufgerufen wurde, ist der `TxBuffer`-Eintrag für `tx` geleert und im WAL dauerhaft persistiert. Ein nachfolgender Fehler in `graph_index.commit(tx)` führte dazu, dass `rollback_relate(tx)` aufgerufen wurde, was wiederum `storage.rollback(tx)` aufrief.
  Dieses ADR skizzierte eine kompensierende Transaktion als Saga-Muster, benutzte im Text jedoch fälschlicherweise die Bezeichnung "2-Phase Commit Protocol (2PC)".

## Grund der Ersetzung (Superseded)

1. **Begriffs-Korrektur:** Die im ursprünglichen Text verwendete Bezeichnung "2PC" war fachlich inkorrekt. Es handelte sich nicht um ein zweiphasiges Commit-Protokoll mit verteilt-synchroner Prepare-Phase, sondern um ein Saga-Muster mit kompensierender Löschaktion.
2. **Neues Konsistenzmodell (ADR-N19 / ADR-N06 Option B):** Gemäß Beschluss F4 (ADR-N19) gilt das Write-Ahead Log (`contextra-store::wal`) als einzige unumstößliche Datenquelle ("Single Source of Truth"). Der `CsrGraph` und alle weiteren Sekundärindizes sind reine abgeleitete Zustände ("Derived State"), die Änderungen aus dem WAL nachziehen. Kompensierende Transaktionen über heterogene Indizes hinweg entfallen; bei Index-Fehlern oder Unterm-Strich-Crashes replayed der Graph-Index deterministisch aus dem WAL [V].

---

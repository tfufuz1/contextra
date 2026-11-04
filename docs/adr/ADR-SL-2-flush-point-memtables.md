# ADR-SL-2: Flush-Punkt bei mehreren eingefrorenen Memtables

## Status
Vorschlag (Änderungsantrag zu SPEC 5.1.0 Entscheidung D-127, WP-W.3, Gate G0).

## Kontext
SPEC 5.1.0 Entscheidung D-127 (Snapshot-Widerruf und WAL-Reihenfolge) spezifiziert in Absatz 2 die Purge-Reihenfolge am WAL:
1. Kurze exklusive Commit-Barriere: aktive Memtable wird eingefroren, das WAL-Segment an einer Sequenzgrenze versiegelt, ein neues Segment mit `prev_mac = tail_mac` beginnt. Die Barriere wird sofort wieder freigegeben. Haltezeit ≤ 1 ms.
2. Ziel-Einträge aus der eingefrorenen Memtable verwerfen (im RAM, noch kein Schreibvorgang).
3. Eingefrorene Memtable flushen, ohne Barriere.
4. Manifest-Swap über den Flush-Punkt (`WalFloor`, F.4).
5. WAL-Segmente bis zum Flush-Punkt als Ganzes entfernen (D-73).

In realen Betriebsszenarien mit hoher Schreibrate können jedoch zum Zeitpunkt der Commit-Barriere bereits eine oder mehrere zuvor eingefrorene Memtables (`immutable memtables`) in der Flush-Warteschlange existieren, die noch nicht vollständig auf Platte persistiert wurden.
Wenn die Purge-Routine nun lediglich die *gerade eben* eingefrorene Memtable betrachtet (`drain_range` nur auf die neueste Memtable anwendet oder den Flush-Punkt nur auf diese bezieht), entstehen zwei schwerwiegende Risiken:
1. Unbereinigte Zielrecords verbleiben in älteren, noch nicht geflushten Memtables und werden anschließend ungefiltert in neue SSTables geschrieben.
2. Der Flush-Punkt `F_seq` wird fälschlicherweise auf die Sequenz der neuesten Memtable gesetzt, obwohl ältere Memtable-Inhalte noch nicht auf Platte sind. Werden daraufhin WAL-Segmente bis zu dieser Sequenz via `WalFloor` entfernt, führt ein Absturz vor Fertigstellung der älteren Flushes zu permanentem Datenverlust unbeteiligter Transaktionen (oder im umgekehrten Fall zu Löschfehlern).

## Entscheidung (Vorschlag)
1. Der Flush-Punkt `F_seq` MUSS verbindlich definiert werden als die höchste Sequenznummer, unterhalb derer die Inhalte **aller** existierenden Memtables (aktive sowie sämtliche eingefrorenen/wartenden Memtables) entweder vollständig in SSTables auf Platte persistiert oder im Rahmen des Purge-Drains verworfen wurden.
2. Das Purge-Primitiv `drain_range` MUSS zwingend über **alle** im Speicher befindlichen Memtables ausgeführt werden (die neu eingefrorene sowie alle zuvor eingefrorenen Memtables).
3. Vor dem Manifest-Swap `WalFloor(F_seq)` MÜSSEN alle anstehenden Memtable-Flushes bis einschließlich `F_seq` vollständig abgeschlossen und per `sync_data` persistiert sein.
4. D-127 Absatz 2 wird um diese Klarstellung in den Schritten (1) bis (3) ergänzt.

## Alternativen
1. *Flush-Warteschlange vor der Commit-Barriere synchron leeren:*
   Verworfen. Ein synchrones Abwarten aller laufenden Flushes während oder vor der Barriere würde die Latenz des `erase`-Aufrufs unkalkulierbar vergrößern und gegen das Ziel verstoßen, die Commit-Barriere unter 1 ms zu halten (BM-PURGE-WAIT).
2. *Ablehnen von Purge-Operationen bei vorhandenen älteren Frozen Memtables:*
   Verworfen. Würde unter Dauerlast zu Deadlocks oder wiederholten `Pending(SlaExceeded)`-Fehlern führen, da bei hohem Ingest-Volumen fast immer Memtables im Flush-Prozess sind.
3. *Drain nur auf aktiver Memtable, Alt-Memtables nachträglich über SSTable-Purge bereinigen:*
   Verworfen. Führt zu unnötigem I/O-Aufwand (Write Amplification, Verstoß gegen BM-ERASE-WA / P11), da Records erst auf Platte geschrieben und danach sofort wieder umgeschrieben werden müssten, und verlängert das Fenster, in dem Daten physisch auf Platte verbleiben.

## Sicherheitsargument (Löschintegrität)
Gemäß SPEC 2 (Prinzipien P1, P4) und SPEC 11.2 ist das Löschziel nach Passieren des Gates (`gated_seq`) für alle Lese- und Schreibpfade isoliert. Befinden sich Zielrecords noch in einer älteren eingefrorenen Memtable im flüchtigen Speicher, stellt das Verwerfen im RAM (`drain_range`) vor dem Schreiben auf Platte die effizienteste und sicherste Bereinigung dar. Erst wenn garantiert ist, dass kein Zielrecord aus irgendeiner flüchtigen Memtable mehr auf Platte gelangt und alle legitimen Nicht-Ziel-Records bis `F_seq` dauerhaft in SSTables gespeichert sind, darf `WalFloor` das WAL abschneiden.

## Vorsichtigere Lesart (Fail-Closed)
Solange D-127 nicht explizit klarstellt, wie mit mehreren eingefrorenen Memtables umzugehen ist, gilt die vorsichtigere Lesart:
Befinden sich beim Eintritt in Schritt 5c von SPEC 11.2 weitere eingefrorene Memtables im Speicher, darf der Flush-Punkt `F_seq` NICHT vorrücken, solange diese älteren Memtables nicht ebenfalls vollständig bereinigt und geflusht wurden. Ein `WalFloor` darf unter keinen Umständen Segmente freigeben, deren Einträge noch in ungeflushten flüchtigen Memtables gehalten werden.

## Betroffene SPEC-Stellen und Formulierungsvorschlag

### SPEC Abschnitt 4.7 (Entscheidung D-127):
Absatz 2 wird um verbindliche Anforderungen an alle Memtables ergänzt:
```diff
--- a/docs/SPEC.md
+++ b/docs/SPEC.md
@@ -D-127
 2. **Purge-Reihenfolge am WAL:**
    - (1) Kurze exklusive Commit-Barriere: aktive Memtable wird eingefroren, das WAL-Segment an einer Sequenzgrenze versiegelt, ein neues Segment mit `prev_mac = tail_mac` beginnt. Die Barriere wird sofort wieder freigegeben. Haltezeit ≤ 1 ms **[A]**.
-   - (2) Ziel-Einträge aus der eingefrorenen Memtable verwerfen (im RAM, noch kein Schreibvorgang).
-   - (3) Eingefrorene Memtable flushen, ohne Barriere.
+   - (2) Ziel-Einträge aus ALLEN im Speicher befindlichen Memtables (aktive sowie alle zuvor eingefrorenen) verwerfen (`drain_range` im RAM, noch kein Schreibvorgang).
+   - (3) Alle eingefrorenen Memtables flushen, ohne Barriere. Der Flush-Punkt `F_seq` ist die höchste Sequenz, unterhalb der ALLE Memtable-Inhalte in SSTables liegen oder verworfen wurden.
    - (4) Manifest-Swap über den Flush-Punkt (`WalFloor`, F.4).
    - (5) WAL-Segmente bis zum Flush-Punkt als Ganzes entfernen (D-73).
```

### SPEC Abschnitt 11.2 (Saga):
Schritt 5c wird textlich angeglichen:
„c. Kurze exklusive Commit-Barriere: aktive Memtable einfrieren, WAL an Sequenzgrenze rotieren. Ziel-Einträge aus allen eingefrorenen Memtables verwerfen (`drain_range`), alle eingefrorenen Memtables flushen, WAL-Segmente bis zum gemeinsamen Flush-Punkt `F_seq` entfernen (`WalFloor`, D-73, D-127).“

## Betroffene Komponenten und Tests
- `crates/contextra-store`: `LsmStorage::drain_range` und Flush-Orchestrierung (`purge/lsm_backend.rs`, `lsm/engine.rs`).
- Tests:
  - `T-WAL-3`: Testen der Purge-Reihenfolge bei mehreren gleichzeitigen / kaskadierenden Memtables unter Schreiblast.
  - `T-CRASH-6`: Absturz während des Flushes bei mehreren Memtables; sicherstellen, dass `WalFloor` nur bis zu verifizierten Flush-Punkten fortgeschritten ist.
  - `BM-PURGE-WAIT`: Bestätigung, dass die Commit-Barriere auch bei mehreren Memtables O(1) bleibt und die Haltezeit nicht beeinflusst wird.

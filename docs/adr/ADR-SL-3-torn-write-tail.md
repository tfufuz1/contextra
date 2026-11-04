# ADR-SL-3: Torn-Write-Regel und umsortierte Sektorpersistierung im WAL

## Status
Vorschlag (Änderungsantrag zu SPEC 5.1.0 Anhang F.4 und Abschnitt 10.4, WP-W.3, Gate G0).

## Kontext
In SPEC 5.1.0 Anhang F.4 (WAL und Manifest) und Abschnitt 10.4 (WAL-Integrität) ist die Behandlung von fehlerhaften Frames bei der WAL-Recovery definiert:
- F.4: „Ein unvollständiger oder ungültiger letzter Frame wird verworfen, wenn alle folgenden Bytes der Datei 0 sind; sonst `Corrupt`.“
- 10.4: „Recovery verwirft ab erstem fehlerhaften Frame nur dann automatisch, wenn der Rest der Datei Nullen/unbeschrieben ist (Torn Write); sonst `Corrupt` und Start verweigert (T-WAL-1).“

Diese Regel („alle folgenden Bytes sind 0“) geht von einer strikt sequenziellen Sektor-Persistierung des Speichermediums aus. Moderne Dateisysteme und NVMe-/SSD-Controller garantieren bei einem plötzlichen Stromausfall jedoch keine strikte lineare Schreibreihenfolge der Sektoren innerhalb eines Dateisystemblocks oder unvollständigen Gruppen-Schreibvorgangs. Werden mehrere Sektoren eines noch unbestätigten Gruppen-Commits (`GROUP_MAX_BYTES`, Standard 1 MiB) an das Betriebssystem übergeben, können spätere Sektoren des Writes bereits auf den Flash-Zellen dauerhaft geworden sein, während frühere Sektoren noch Nullen oder unvollständige Daten enthalten.

Das Auftreten von Nicht-Null-Bytes hinter einer Null- oder Müll-Lücke am Dateiende ist somit eine legitime physikalische Crash-Konsequenz eines unbestätigten Schreibvorgangs. Unter der bisherigen Formulierung wertet die Recovery diesen Zustand zwingend als bösartige oder fatale Korruption (`Corrupt`) und verweigert den Start vollständig. Dies führt zu einem gravierenden Verfügbarkeitsfehler, obwohl alle vor dem Crash bestätigten Commits kryptografisch intakt und vollständig sind.

## Entscheidung (Vorschlag)
1. Es wird eine präzisierte Torn-Tail-Regel für das WAL eingeführt:
   Liegt der erste ungültige Frame an Dateioffset $o$ und gilt für das verbleibende Dateiende:
   $$\text{file\_len} - o \le \text{GROUP\_MAX\_BYTES}$$
   (wobei $\text{GROUP\_MAX\_BYTES}$ intern auf standardmäßig 1 MiB festgelegt ist), so wird der gesamte Bereich ab Offset $o$ bis zum Dateiende als unbestätigter Torn-Tail gewertet.
2. Dieser Bereich darf bei der Recovery abgeschnitten (trunkiert auf Offset $o$) werden.
3. Voraussetzung: Ein Frame hinter $o$, der syntaktisch und kryptografisch gültig wäre, darf NIE übersprungen oder verworfen werden, wenn er an die HMAC-Kette anschließen würde. Die Kette bricht definitionsgemäß bei $o$ ab.
4. Das Abschneiden eines Torn-Tails MUSS ein persistentes `HealthReport`-Ereignis mit Grund `TornTailDiscarded { offset: o, discarded_bytes: file_len - o }` und Zustand `Degraded` auslösen, damit der Vorfall im Systemstatus nachvollziehbar dokumentiert ist.
5. Liegt der ungültige Frame tiefer in der Datei ($\text{file\_len} - o > \text{GROUP\_MAX\_BYTES}$), gilt weiterhin unverändert `Corrupt` und der Start MUSS verweigert werden (Schutz gegen Korruption mitten in der Datei).

## Alternativen
1. *Strikte Beibehaltung der „Nur Nullen“-Regel:*
   Verworfen. Führt bei realen Stromausfällen auf modernen NVMe-SSDs zu Fehlalarmen und unbrauchbaren Datenbankinstanzen bei unbestätigten Transaktionen.
2. *Beliebig langes Abschneiden bis zum letzten gültigen Frame ohne Größenbegrenzung:*
   Verworfen. Begünstigt Angreifer, die gezielt frühere Segmente oder größere Dateiteile überschreiben oder beschädigen könnten, um bestätigte Transaktionen verschwinden zu lassen.
3. *Ausschließliche Erkennung über feste Sektormuster:*
   Verworfen. Hängt von Hardware- und Dateisystemparametern ab (Blockgröße 512 B vs. 4 KiB vs. COW-Extents) und ist im Ring-1-Port nicht portabel spezifizierbar.

## Sicherheits- und Angriffsmodell-Analyse
Begünstigt die Begrenzung auf `GROUP_MAX_BYTES` einen Angreifer?
- **Nein:** Der Gruppen-Commit-Leader bestätigt einen Commit erst an die aufrufende Anwendung per `oneshot`-Kanal, **nachdem** das Segment bis einschließlich dieses Frames per `sync_data` dauerhaft synchronisiert wurde (SPEC 14.2, PLAN-WAL-ERASURE 4.4).
- Alle bestätigten Commits befinden sich zwingend vor dem Offset $o$.
- Ein manipulierter Frame innerhalb der letzten `GROUP_MAX_BYTES` kann keine bereits bestätigten Daten vortäuschen, da er keine gültige HMAC-Signatur mit dem geheimen `K_wal` besitzen kann.
- Kann ein Angreifer durch Einbringen von Müll am Dateiende das Abschneiden erzwingen? Er kann höchstens Daten abschneiden lassen, die noch gar nicht bestätigt waren. Da der maximale Verlust auf die unbestätigte Puffergröße der laufenden Gruppe beschränkt ist, entspricht das genau der Semantik eines Stromausfalls während des Schreibens.
- Bestätigte Daten vor $o$ sind durch die HMAC-Kette vollständig geschützt.

## Vorsichtigere Lesart (Fail-Closed)
Bis zur Verankerung dieser Regel in SPEC Anhang F.4 und 10.4 gilt:
Findet die Recovery Nicht-Null-Bytes hinter einem ungültigen Frame vor, muss sie strikt nach dem aktuellen SPEC-Wortlaut den Zustand als `Corrupt` einstufen und den Start verweigern. Ein eigenmächtiges Abschneiden von Datenbereichen ohne Spezifikationsgrundlage ist im Produktionscode verboten.

## Betroffene SPEC-Stellen und Formulierungsvorschlag

### 1. SPEC Anhang F.4 (WAL und Manifest):
Im Abschnitt **WAL-Segment (`kind = 1`)** wird der Absatz zu Torn Write wie folgt gefasst:
```diff
--- a/docs/SPEC.md
+++ b/docs/SPEC.md
@@ -F.4
-**Torn Write:** Ein unvollständiger oder ungültiger letzter Frame wird verworfen, wenn alle folgenden Bytes der Datei 0 sind; sonst `Corrupt`.
+**Torn Write:** Ein unvollständiger oder ungültiger Frame ab Offset `o` wird als unbestätigter Torn-Tail verworfen und auf Offset `o` trunkiert, wenn entweder alle folgenden Bytes der Datei 0 sind oder `file_len − o ≤ GROUP_MAX_BYTES` (Standard 1 MiB) gilt. In letzterem Fall setzt das System den Zustand im `HealthReport` auf `Degraded` mit `TornTailDiscarded`. Liegt der Fehler bei `file_len − o > GROUP_MAX_BYTES`, gilt zwingend `Corrupt` und der Start wird verweigert. Ein gültiger Frame hinter `o` darf nie verworfen werden.
```

### 2. SPEC Abschnitt 10.4 (WAL-Integrität):
Der Text wird angepasst:
„Recovery verwirft ab dem ersten fehlerhaften Frame am Dateiende (Torn Write), wenn der Rest der Datei Nullen enthält oder der Abstand zum Dateiende höchstens `GROUP_MAX_BYTES` beträgt (F.4); sonst `Corrupt` und Start verweigert (T-WAL-1).“

## Betroffene Komponenten und Tests
- `crates/contextra-store`: WAL-Replay-Logik (`wal/replay.rs`).
- `crates/contextra-testkit`: `FaultVfs` zur Simulation umsortierter Sektorpersistierung bei Stromausfall.
- Tests:
  - `T-WAL-1 Fall a`: Torn Tail im letzten Segment mit nachfolgenden Nullen.
  - `T-WAL-1 Fall b`: Korruption mitten in der Datei (`> GROUP_MAX_BYTES`) $\rightarrow$ Start verweigert (`Corrupt`).
  - `T-WAL-1 Fall c`: Umsortierte Sektoren / Nicht-Null-Bytes bei `file_len - o <= GROUP_MAX_BYTES` $\rightarrow$ Truncation auf $o$, Start erfolgreich, `HealthReport` meldet `Degraded`.

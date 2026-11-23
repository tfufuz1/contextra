# `contextra-store` → R8 Migrationswerkzeug

Besitzer: SP-CUTOVER. Das Werkzeug ist ein einmaliger, resumierbarer Export-/Importpfad, kein Runtime-Adapter und kein zweiter Schreibkern. Es wird erst nach stabilen R8-Formaten und freigegebener Bridge implementiert. Keine Legacy- oder R8-Änderung darf stillschweigend Daten verwerfen oder eine bestätigte Mutation verlieren.

## Phasen und Artefakte

1. **Inventar:** versionierte Liste aller Legacy-Namespaces, Datenräume, Verbraucher, Schlüssel-/Clusterabbildungen, Löschklassen, Indizes und bekannten Referenzen. Personenbezogene Daten erhalten eine explizite verschlüsselte R8-Schutzdomäne.
2. **Snapshot:** konsistenter Export mit eingefrorenen Writes oder beweisbarem Change-Capture. Manifest enthält Source-Revision, Cut-Watermark, Objekt-/Bytecounts und kryptografische Digests; keine Klartext-Personenkennungen in Diagnose-Logs.
3. **Import:** begrenzte Chunks, checkpointbar und `IdemToken`-gesichert. Jeder Chunk dokumentiert Input-Range, Count/Digest, Ziel-Space/Cluster, Durable-Stamp und Status. Bei Retry wird derselbe Token und Inhalt verwendet.
4. **Index-Rebuild:** HNSW/BM25/CSR/KV-Cache werden aus dem bestätigten R8-Quellstand neu aufgebaut oder mit explizit versioniertem Importformat übernommen. Kein Index-Watermark darf den höchsten vollständig angewandten `durable_stamp` überschreiten.
5. **Differentialphase:** Legacy und R8 werden read-only verglichen: Werte/Counts, Tombstones, Retrieval-Kandidaten/Recall mit definierter Rankingtoleranz, Graphbeziehungen, Gate-Sichtbarkeit und Fehlerklassen.
6. **Cutover:** Write-Stop oder finaler Change-Capture-Fence; letzte Differenz muss null bzw. explizit genehmigt sein. Danach existiert genau ein Writer. Kein Dual-Write.
7. **Rollback/Retirement:** Vor erstem R8-Write Rückkehr zum unveränderten Legacy-Stand. Danach ausschließlich getesteter Forward-Export/Replay oder Restore. Legacy-Code/Dateien erst nach Soak, Restore-Probe, Erasure-Probe und G-CUTOVER archivieren.

## Betriebs- und Sicherheitsregeln

- Migration ist im Ergebnis idempotent, arbeitet mit bounded memory/disk batches und meldet Backpressure statt unbounded buffering.
- Fehlerbericht enthält Objekt-/Chunk-IDs nur pseudonymisiert und nie Payload, Schlüssel oder rohe personenbezogene Kennungen.
- Quell- und Zielmanifest bleiben getrennt. Ein erfolgreicher Prozess-Exit bedeutet nicht Cutover-Freigabe.
- Restore der Migration erhält die R8-`InstanceId`; Schreiböffnung verlangt denselben externen Fencing-Nachweis wie jede andere Restore-Kopie.
- Nicht abbildbare Altsemantik landet in einer versionierten Unsupported-/Conversion-Liste mit Besitzer und Freigabe, nie in stiller Approximation.

## Abnahme

Crash-/Restart-Sweep an jeder Chunkgrenze, Duplikat-/Tokenkonflikt, Quelländerung während Snapshot, DiskFull, Indexlag, Differenzbericht und Forward-Rollback werden reproduzierbar simuliert. G-CUTOVER bleibt offen, bis alle produktiven Verbraucher, Datenräume und Erasure-Witnesses abgedeckt sind.

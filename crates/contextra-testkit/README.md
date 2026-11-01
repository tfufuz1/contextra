# contextra-testkit

Test-Fixtures, Mock-Engines, Referenzmodelle und Fehlerinjektion.

- **Ring:** dev (Test-Infrastruktur)
- **Normativ:** [docs/SPEC.md §8.15](../../docs/SPEC.md)
- **Arbeitsanweisungen:** [AGENTS.md](AGENTS.md)

## Rolle & Zweck
`contextra-testkit` stellt wiederverwendbare Testbausteine ausschließlich für Testumgebungen bereit:
- `FaultVfs`: Hochentwickelte I/O-Fehlerinjektion (Absturz nach $n$ Operationen, Torn-Writes an Byte $b$, `ENOSPC`, `EIO` bei `sync_*`, Seitencache-Verzögerung).
- `ReferenceModel`: Unabhängiges BTreeMap-Orakel zur Validierung von Transaktionen, MVCC-Zeitreisen und Löschsemantiken.
- `ManualClock`: Deterministisch steuerbare Testuhr für zeitabhängige Tests.
- `InMemoryStore`: Schneller In-Memory Key-Value-Speicher für Unit-Tests.
- `RecordingStorage`: Protokolliert alle geschriebenen Schlüssel zur Vollständigkeitsprüfung (T-COV-1).
- `NeighborDigest`: BLAKE3-Hash über alle Nicht-Ziel-Paare zum Nachweis unberührter Nachbardaten.
- `LeakyLayer`: Absichtlich fehlerhafte `ErasureLayer`-Implementierung als Positivkontrolle für das Erasure Lab (T-RAW-2).
- `CanaryFactory`: Generiert hochentropische Needles für Messungen.
- `MemVfs`, `MemLedger`: Reine RAM-Ports.

## Invarianten
- `#![forbid(unsafe_code)]`
- **Ausschließlich als `dev-dependency` zugelassen (INV-EMBED-1):** Darf niemals im Produktions-Release eingebunden werden.
- Veraltetes `in_memory_graph` ist entfernt.

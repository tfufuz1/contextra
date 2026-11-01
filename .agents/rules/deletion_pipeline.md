# Governance Rule: Deletion Pipeline & DSGVO Compliance (`rules/deletion_pipeline.md`)

> **Normative Vorgabe nach docs/SPEC.md §11, §12 und ADR-001 (D-01).**

## Gilt für
- `crates/contextra-erasure`
- `crates/contextra-engine`
- `crates/contextra-store`
- `crates/contextra-vector`
- `crates/contextra-text`
- `crates/contextra`

## Pflichtregeln
1. **Löschmodell v1 (ADR-001, D-01)**:
   - Die sogenannte Ebene B (kryptografische DocDek-Schlüsselvernichtung) entfällt in v1 vollständig.
   - Löschung in v1 ist die physische Bereinigung (Ebene A) kombiniert mit fail-closed `ReadGate` (`ErasedSet`), automatischer Residuen-Messung (E1 Rohscan, E2 Decoder-Scan, E3 Schlüsselabwesenheit) und dem kryptografisch Ed25519-signierten Löschbericht (`ctx-erasure-report/1`).
2. **Lösch-Saga Zustandsmaschine (SPEC §8.2, §11.2)**:
   - `Requested → Gated → Drained → Rewritten → Cleaned → Measured → Completed`
   - Fehlerfall: `Pending { attempt, next_retry_at }` mit exponentiellem Backoff (1 s, 2 s, 4 s … max. 15 min), niemals stilles Ignorieren.
3. **Fail-Closed Lesesperre (`ReadGate`)**:
   - Unmittelbar nach Erreichen des Zustands `Gated` blockiert das Gate (`ErasedSet`) alle Lese- und Suchzugriffe auf die Zielschlüssel.
   - Aktive Snapshots, die Zielversionen sehen könnten, werden widerrufen (`SnapshotRevoked`).
4. **Physisches Purging (Ebene A)**:
   - **Store**: `drain` versiegelt WAL; `rewrite_file` erzeugt neue SSTables ohne Zieldaten (Originalsequenzen bleiben erhalten, I-2); atomarer `swap` via Manifest; Block-Cache Eviction mit Zeroize.
   - **Vector**: `zero_slot` überschreibt RAM-Bytes mit `zeroize`, markiert Tombstone; betroffene Snapshot-Chunks werden neu geschrieben; Rebuild ab Schwelle.
   - **Text**: Textpuffer leeren; betroffene Segmente ohne Zieldokumente neu schreiben; Alt-Segmente löschen; Term-Statistik dynamisch neu aggregieren.
5. **Residuenmessung & Zeugen (SPEC §12)**:
   - Vor jedem Scan MUSS eine Positivkontrolle (Canary) belegen, dass der Scanner funktionsfähig ist.
   - Messungen umfassen E1 (Rohscan), E2 (Decoder-Scan) und E3 (Schlüssel-/Slot-Abwesenheit).
6. **Externe Prüfbarkeit (`ctx-erasure-report/1`)**:
   - Berichte werden per Ed25519 asymmetrisch signiert und können mit der unabhängigen Binärdatei `ctx-verify` geprüft werden.
   - Der Bericht enthält niemals Klartext-IDs, personenbezogene Daten oder Chunk-Inhalte.

## Verweise
- `docs/SPEC.md` §11, §12
- `PLAN.md` Phase 3 (WP-3.1 bis WP-3.12)
- `crates/contextra-erasure/AGENTS.md`

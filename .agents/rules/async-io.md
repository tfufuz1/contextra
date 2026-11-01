# Governance Rule: Asynchrones I/O, Laufzeitmodell & Cancel-Safety (`rules/async-io.md`)

> **Normative Vorgabe nach docs/SPEC.md §5.4 (Laufzeitmodell), §8.2 und I-14.**

## Gilt für
- `crates/contextra-engine`
- `crates/contextra-store`
- `crates/contextra-erasure`
- `crates/contextra`

## Pflichtregeln
1. **Laufzeitmodell & Tokio-Handle (SPEC §5.4)**:
   - Die Bibliothek startet keine eigene Tokio-Runtime, sondern verlangt bei `EngineBuilder::open` zwingend die Übergabe eines `tokio::runtime::Handle`.
   - Alle Hintergrundtasks laufen auf diesem Handle.
2. **Cancel-Safety bei mutierenden Operationen (I-14, D-C01)**:
   - Jede mutierende Operation (Commit, Write, Purge, Erase) läuft in einer vom Aufrufer unabhängigen Task (`spawn`).
   - Der Aufrufer wartet auf das Ergebnis über einen `oneshot`-Kanal. Der Abbruch des Aufruferfutures (Drop des Futures) verändert den Ausgang des Commits nicht.
3. **Lebenszyklus von Hintergrundtasks (SPEC §11.3)**:
   - WAL-Flusher, Memtable-Flush, Kompaktion, Purge-Scheduler, Vektor-Rebuild, Textsegment-Merge und Ledger-Rotation besitzen explizite Stop-Flags und `JoinHandle`s.
   - `close().await?` beendet alle Hintergrundtasks geordnet und synchronisiert die Daten.
   - **Kein Blockieren in `Drop`** (SPEC §14.3).
4. **Sperren & Lock-Reihenfolge**:
   - Dokumentlock: Pro Dokument ein asynchroner Lock, der Insert, Update, Delete und Erase serialisiert (behebt Race-Befund B-13).
   - Purge-Barrier (`RwLock`): Shared zwischen Kompaktionen, exklusiv für Purge.
   - **Verbot von Sync-Locks über `.await`**: Keine Sperren über fremde asynchrone Wartepunkte halten.

## Verweise
- `docs/SPEC.md` §5.4
- `PLAN.md` WP-2.7, WP-2.8

# ADR-Index — `contextra-store`

Dieses Verzeichnis enthält die Architecture Decision Records (ADRs) des Speichertriebwerks `contextra-store`.

## Status-Definitionen
- **Proposed:** Vorschlag eingereicht, wartet auf Prüfung und Genehmigung.
- **Accepted:** Architekturentscheidung akzeptiert und normativ umgesetzt.
- **Deprecated:** Durch spätere Entscheidung abgelöst oder dekommissioniert.

## Index der Architekturentscheidungen

| ADR-ID | Titel | Status | Datum | Relevante Module | Invariante / Bezug |
|---|---|---|---|---|---|
| [`ADR-0XX-P0A`](ADR-0XX-P0A-Lock-Handoff.md) | Lock-Handoff Invariant for Group-Commit WAL Serialization | Accepted | 2026-09-16 | `lsm/group_commit.rs`, `wal/flusher.rs` | P06 / P0-A, totale Schreibordnung vor Commit-Freigabe |
| `ADR-P01` | WAL Tail Truncation & Fail-Stop Poisoning | Accepted | 2026-10-06 | `wal/flusher.rs`, `wal/replay.rs` | INV-STORE-1 / INV-STORE-2, F-01 Behebung |
| `ADR-P03` | Persistence Enforcement & WAL Fail-Stop Semantics | Accepted | 2026-10-07 | `fsync.rs`, `wal/flusher.rs`, `manifest/` | INV-STORE-1 / INV-FSYNC-1, `durable-fs` Persistenz-Semantik |
| `ADR-P06` | Persistent Key Shredding & Lock Hierarchy | Accepted | 2026-10-08 | `lsm/group_commit.rs`, `kv/` | Sperrenordnung `commit_mutex` vor `truncate_lock` |
| `ADR-P07` | Deterministic Memory Ordering & Write Fencing | Accepted | 2026-10-07 | `lsm/engine.rs`, `memtable.rs` | INV-STORE-3 / INV-STORE-5 |

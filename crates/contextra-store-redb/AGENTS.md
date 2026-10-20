# AGENTS.md — contextra-store-redb
> Ring 1 · experimental · Quelle: capabilities.toml · Spec: §4.1 / §18 · Stand: 2026-10-21

## 1. Zweck
`StorageEngine` Adapter auf Basis von `redb` (version 2.6.3) als alternative, transaktionale Key-Value Engine für Contextra.

## 2. Modul-Karte
| Datei/Verzeichnis | Verantwortung |
|---|---|
| `src/lib.rs` | Re-Exports, Crate-Dokumentation, Honest Deletion Semantics & Abweichungstabelle |
| `src/capabilities.rs` | `RedbDeletionCapabilities` Descriptor (`max_deletion_stage = S1`) |
| `src/engine.rs` | `RedbStorageEngine` Implementierung des `StorageEngine` Traits mit MVCC und Staging Buffer |

## 3. Invarianten
- `INV-REDB-NO-PANIC`: Zero-Panic in Produktionspfaden (`unwrap`/`expect`/`panic!` verboten).
- `INV-REDB-S1-LIMIT`: Strikte Deklaration von `max_deletion_stage = S1` (Logical free-listing), da redb gelöschte Schlüssel/Werte in Free-Lists belässt und keine physische Nullung auf Speichermedien unterstützt.
- `INV-REDB-SERIALIZED-COMMIT`: Commits und Rollbacks werden über ein internes Mutex serialisiert, sodass Sequenznummern-Zuordnung und commit-basierte B-Tree Mutation strikt atomar erfolgen.

## 4. Verboten / Anti-Patterns
- Keine Vortäuschung von physischem Purge / Secure Erasure (S2) ohne Crypto-Shredding.
- Keine async Runtime Dependencies innerhalb dieser Subsystem-Crate.

## 5. Nebenläufigkeit und Lock-Regeln
- `commit()` und `rollback_to_tx()` nutzen `commit_lock` Mutex zur Serialisierung von Write-Transaktionen.
- Staging-Puffer ist über `parking_lot::RwLock` geschützt.

## 6. Verifikation
```bash
cargo test -p contextra-store-redb
cargo clippy -p contextra-store-redb --all-targets -- -D warnings
```

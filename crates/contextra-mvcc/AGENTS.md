# AGENTS.md — `contextra-mvcc`

> **Ring 0** · Klasse `[K]` · Reifegrad `stable` · Unsafe: nein · Rolle: Snapshots, Sequenzlog, Serializable Snapshot Isolation (SSI), Transaktionspuffer.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-mvcc --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `ports`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types` |
| Features | keine |
| Lints | `#![forbid(unsafe_code)]`, `deny(dead_code)`, `warn(missing_docs)` |

## 2. Abhängigkeiten & Schulden

- Ist = Soll, autark. Dev-Kante auf `testkit` (zulässig als Werkzeug).
- Externe Abhängigkeit `parking_lot`.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `seq_log.rs` | `SequenceLog`, `SeqLogEntry`, `SeqLogChange`: append-only Sichtbarkeit |
| `snapshot.rs` | `SnapshotRegistry`, `SnapshotGuard`: Pinning, Read-Isolation |
| `ssi.rs` | `ReadSet`, `SsiValidator`, `SequenceLogSsiValidator`, `PruningBlockerInfo` |
| `tx_buffer.rs` | `TxBuffer<T>` (sharded Staging, Timeout/Orphan-Reaper), `IndexOp<T>` |

## 4. Verträge & Invarianten

1. **SSI aktiv (INV-SSI-ACTIVE-1):** Read-Sets inkl. Prefix-/Range-Phantomschutz werden bei jedem Commit validiert.
2. **Index-Snapshot (INV-INDEX-SNAPSHOT-1):** Eintrag sichtbar nur für `born_seq ≤ S < died_seq`.
3. **Sperrenordnung (INV-LOCK-ORDER-1):** dokumentierte Reihenfolge, nie über `.await` (hier ohnehin synchron).
4. `TxId` kommt aus der Collection-Sequenz, nie aus der Uhr (P28). Zeit nur über `Clock` (`SequenceLogSsiValidator::new_with_clock_and_bounds`).
5. Begrenzte Strukturen: `max_tracked_keys`, Shard-Anzahl, Tx-Timeout sind Parameter, keine globalen Konstanten.
6. Die Snapshot-Lese-Sperre für gelöschte Betroffene setzt `contextra-erasure` (ReadGate); `mvcc` kennt sie nicht.

## 5. Ports

Konsumiert `Clock`. Implementiert keine Ports (reine Bausteine).

## 6. Nebenläufigkeit & Performance

- Sharded `TxBuffer` statt globalem Lock; Snapshot-Registry mit kurzem, einzelnem Mutex.
- Pflicht: `loom`-Modelle für Registry/Validator; Property-Tests gegen ein Referenzmodell (`testkit::reference_model`).
- Komplexität des Validators dokumentieren (Read-Set-Größe × aktive Schreiber) und per `benches/` belegen.

## 7. Anti-Patterns

- Systemzeit für Sequenzen.
- Unbegrenzte Read-Sets.
- Lock halten und anschließend Fremdcode (Callback) aufrufen.

## 8. Tests & Messung

```bash
cargo test -p contextra-mvcc --locked
cargo clippy -p contextra-mvcc --all-targets -- -D warnings
RUSTFLAGS="--cfg loom" cargo test -p contextra-mvcc --locked
cargo bench -p contextra-mvcc
```

- `tests/` + `benches/` vorhanden. Ziel Mutation-Score ≥ 85 % (`cargo mutants -p contextra-mvcc`).

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 4.x | Isolations-Test: Crate allein in Fremdprojekt bauen (Beispiel unter `examples/`, WP 6.1) |

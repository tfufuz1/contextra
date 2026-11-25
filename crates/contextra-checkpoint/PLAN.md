# Implementierungsplan: contextra-checkpoint

**Ebene:** L2 · **Ziel-Zustand:** Snapshot-Set-Register und Restore-Koordination. Kennt `storage` nicht (FIX-07). Definiert `SnapshotParticipant` und `RestoreStep` als Traits; `engine` bindet Teilnehmer.
**Basis:** SPEC R14 §2.1 (Regel 4), §9.3, §9.4, Anhang A („Restore“, „Ticket“), FIX-07, FIX-08, DEC-34; R15 §2.1, §2.3.

## 1. Ist-Befund (gemessen)

- 13 Dateien, 5 265 Zeilen. Module: `contextra_ports` (Stub), `contextra_types` (Stub, mit `TxId` in Zeile 68), `guard` (Verzeichnis + Datei), `hardlink_cloner`, `manifest`, `meta` (Verzeichnis + Datei), `orphan` (Verzeichnis + Datei, 815), `store` (Verzeichnis + Datei, 1008).
- **Abweichung:** `Cargo.toml` hat `tokio`, `blake3`, `serde`, `serde_json`, `parking_lot`, `tracing`. SPEC §2.7 verbietet `tokio`, `blake3`, `serde`. `parking_lot` ebenfalls.
- 70 Treffer verbotener Muster im Produktionscode.
- `benches/checkpoint_bench.rs` nutzt `block_on` (Benchmark, akzeptiert, aber Produktionscode in `store.rs` und `guard.rs` ebenfalls).
- **Befund:** `WorkflowState`, `StateCheckpoint`, `pin_checkpoint`, `hardlink_cloner` existieren laut R14 §13.3 als Altpfad. `hardlink_cloner.rs` ist vorhanden.

## 2. Entfernen

| Ziel | Begründung |
|---|---|
| `contextra_ports.rs`, `contextra_types.rs` | Stubs (R14 §13.2 Nr. 4) |
| `TxId` | ersetzt durch `Epoch` |
| `hardlink_cloner.rs` | Altpfad (§13.3) |
| `StateCheckpoint`, `WorkflowState`, `pin_checkpoint` | Altpfad (§13.3) |
| `blake3`-Manifest in `manifest.rs` | ersetzt durch kanonische Codierung + Mac |
| `tokio`, `blake3`, `serde`, `serde_json`, `parking_lot` in `Cargo.toml` | verboten bzw. nicht nötig |
| `block_on` in `store.rs`, `guard.rs` | nur `engine::kernel` darf `block_on` (§13.3) |

## 3. Ändern

- **`store.rs` (1008 Zeilen):** nach Neuzuschnitt in `register.rs` (Snapshot-Set-Register) und `coordinator.rs` (Restore-Koordination). `store.rs` bleibt, enthält nur Typen.
- **`guard.rs` (889):** Guard-Logik gehört zu `storage` (§5.3). Teile, die nur Checkpoint-Zustand prüfen, bleiben; Rest entfernen.
- **`orphan.rs` (815) und `orphan/`:** Orphan-Erkennung für Staging-Runs und Ingest (Publish-Record, §11.4). Inhalt auf Checkpoint-Sicht reduzieren, Ingest-Logik nach `engine::ingest`.
- **`meta.rs` und `meta/`:** Metadaten-Typen des Registers; kanonische Codierung über `types`.
- **`manifest.rs`:** Manifest mit Liste `(file_id, size, checksum, instance_id, epoch)`, Mac über den Port. Kein Inhalt.

## 4. Implementieren

1. **Traits (aus FIX-07):**
   - `SnapshotParticipant`: `pin(&self) -> PinHandle`, `manifest(&self, pin) -> Manifest`, `restore_step(&self, step) -> Result`.
   - `RestoreStep`: Name, Abhängigkeiten, Ausführung. Reihenfolge durch Abhängigkeitsgraph.
2. **Snapshot-Set-Register:**
   - `SnapshotSet = (instance_id, V)` plus je Partition `(partition, generation, applied_stamp)` (§0 Glossar).
   - Register ist ein Verzeichnis gepinnter Sets mit Lease-Ablauf.
   - Erase überstimmt Pins und SnapshotSets (FIX-08): Register liefert `Revoked`-Ereignis.
3. **Restore-Koordinator** mit Zustandsautomat aus Anhang A:
   - `RestoreQuarantine` → Fencing des alten Besitzers → Ledger-Kette prüfen → Anker prüfen → Intents anwenden → `RestoreReconciled` → `Running`.
   - Ohne externes Fencing kein Übergang aus `RestoreQuarantine`.
4. **Backup-Horizont und Tomb-Retire** (§9.4): Epochen-Register mit `Retired`-Zustand erst nach `backup_horizon` und Provider-Absent.
5. **Retention:** nur `KeepUntilFloor`, `history_retention` 7 d. Floor kommt aus `mvcc::EpochPinRegistry`.
6. **Checkpoint-Erzeugung** liefert `CheckpointManifest` mit Dateiliste und Mac, kein Inhalt.

## 5. Tests

- **Restore-Properties:** Restore eines Checkpoints liefert denselben Zustand wie Original zum Zeitpunkt (Modelltest).
- **Rollback-Test:** Anker höher als Katalog ⇒ `Err(Rollback)`, nur `ReadOnly` mit Diagnose (F-9).
- **Fencing-Test:** Restore ohne Fencing bleibt in `RestoreQuarantine`.
- **Zeitreise-Test:** `view_at(epoch)` nach Erase liefert `Erased`, nicht Inhalt (FIX-15).
- Migrations-Rehearsal (G-R5) mit echten Daten aus `contextra-store`.

## 6. Abhängigkeiten nach Umbau

Erlaubt: `contextra-types`, `contextra-ports` (nur Traits), `contextra-mvcc`, `thiserror`. Keine Serialisierung, keine Async-Laufzeit, keine Hash-Funktion außer über den `Mac`-Port.

## 7. Gate und Reihenfolge

1. Stubs löschen, Abhängigkeiten bereinigen. Build ohne `tokio`.
2. Altpfade (`StateCheckpoint`, `WorkflowState`, `hardlink_cloner`) entfernen.
3. Traits `SnapshotParticipant` und `RestoreStep`.
4. Snapshot-Set-Register und Restore-Koordinator.
5. **G-R5:** Restore-/Rollback-Properties, Migrations-Rehearsal.

**Risiko:** Restore ist die Stelle, an der Löschung nachträglich rückgängig gemacht werden könnte (AT-5). Der Anker-Test ist nicht optional.

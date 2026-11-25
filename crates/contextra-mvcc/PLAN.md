# Implementierungsplan: contextra-mvcc

**Ebene:** L2 · **Ziel-Zustand:** nur noch `EpochPinRegistry` (DEC-34). Keine eigene Transaktionslogik.
**Basis:** SPEC R14 §2.1 (Regel 4), §2.6, §7 (Pinning-Risiko), §13.3, NZ-3, DEC-34; R15 §2.1 (B-08 nicht betroffen).

## 1. Ist-Befund (gemessen)

- 7 Dateien, 4 890 Zeilen. Module: `contextra_ports` (Stub), `contextra_types` (Stub), `seq_log` (836), `snapshot` (952), `ssi` (1206), `tx_buffer` (1435).
- `TxId` ist definiert (`contextra_types.rs:212`), ein weiteres Duplikat zu `types`.
- 30 Treffer verbotener Muster im Produktionscode (`parking_lot`, `serde`, `tokio`-ähnliche Muster).
- Abhängigkeiten: `parking_lot`, `ahash`, `serde`, `bytes`, `thiserror`, `tracing`. Laut SPEC §2.7 darf `mvcc` nur an `std` und `types` hängen. `parking_lot` ist verboten, `serde` ebenso.
- `Cargo.toml` hat **keine** Abhängigkeit auf `contextra-types`, obwohl SPEC sie verlangt. Stub-Module ersetzen sie heute.

## 2. Entfernen

| Datei/Modul | Begründung |
|---|---|
| `seq_log.rs` (SequenceLog) | ersetzt durch Stamps im Sequencer von `storage` (§5.2) |
| `ssi.rs` (SSI) | NZ-3: SSI nicht im Kern |
| `tx_buffer.rs` (TxBuffer) | ersetzt durch Guards §5.3 |
| `snapshot.rs` | Transaktionslogik; nur die Pin-Teile bleiben (siehe Abschnitt 3) |
| `contextra_ports.rs`, `contextra_types.rs` | Stubs (R14 §13.2 Nr. 4) |
| `TxId` | ersetzt durch `Epoch` aus `types` |
| `parking_lot`, `serde`, `ahash`, `bytes`, `tracing` in `Cargo.toml` | verboten bzw. nicht nötig; `std::sync` genügt |

## 3. Ändern

- **Neuer Zuschnitt:** Einziges Modul `pin.rs` (oder `registry.rs`) mit `EpochPinRegistry`. `snapshot.rs` wird auf die Pin-Logik reduziert, der Rest gelöscht.
- **`lib.rs`:** Öffentliche API auf `EpochPinRegistry`, `PinGuard`, `Floor` begrenzen.
- **Abhängigkeiten:** `contextra-types`, `std`. `loom` nur als `cfg(loom)`.

## 4. Implementieren

1. **`EpochPinRegistry`:**
   - `pin(epoch) -> PinGuard`: Zähler je Epoche in einer `BTreeMap<Epoch, u32>` unter `std::sync::Mutex`.
   - `PinGuard::drop` gibt den Pin frei; kein Pin ohne Guard.
   - `floor() -> Option<Epoch>`: kleinster gepinnter Epoch. Grundlage für Compaction-Untergrenze und `history_floor`.
2. **Lease-Bindung:** Pins haben eine Lease (`view_lease` 60 s, `view_lease_max` 10 min). Lease-Ablauf setzt Pin zurück und liefert `SnapshotTooOld` an den Besitzer.
3. **Erase-Überstimmung:** Erase kann Pins brechen (FIX-08). Schnittstelle `revoke_below(epoch)` mit Ereignis an `Observer`.
4. **Metriken:** Anzahl Pins, ältester Pin (Branches, Risiko 1), Abgelaufene Leases. Keine IDs, nur Zähler.

## 5. Tests

- Pin-Freigabe bei Drop, auch bei Panic (`catch_unwind` im Test).
- Monotonie des `floor()` bei gleichzeitigem Pin und Unpin.
- `loom`-Test: Pin/Unpin-Verschränkung mit `Floor`-Lesern (Race-Freiheit, Z-8).
- Lease-Ablauf-Test mit `SimClock`.

## 6. Abhängigkeiten nach Umbau

Erlaubt: `std`, `contextra-types`, `loom` (nur Test). Keine weiteren.

## 7. Gate und Reihenfolge

1. Stub-Module löschen, `contextra-types` als Abhängigkeit eintragen.
2. `SequenceLog`, `SSI`, `TxBuffer` löschen. Konsumenten (`checkpoint`, `engine`) auf `EpochPinRegistry` umstellen.
3. `EpochPinRegistry` implementieren, `loom`-Test.
4. **G-R3:** `INV-RAG-4` (Race Gate × Query) mit `EpochPinRegistry`.

**Risiko:** Konsumenten in `engine` (44 Dateien referenzieren `TxId`/`SequenceLog`/`TxBuffer` gemeinsam mit `checkpoint`). Umstellung erst nach dem Neuzuschnitt von `engine` planen, sonst entsteht ein Zwischenzustand, der nicht baut.

# Extraktion fuer `acta-gate`

**Aufgabe:** ErasedView, GateWriter, Roaring-Mengen, ErasureLayer, LayerAck  
**Level:** L1 · **Budget:** 1200 LOC · **Spezifikation:** docs/SPEC.md Kap. 3.7, 5.3  
**Interne Abhaengigkeiten (Soll):** acta-base  
**Umfang des Archivs:** 12 Dateien, 2497 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `erasure` | ErasedView/GateWriter/Gate-Log (5.3); auf Roaring + HMAC-Kette umbauen |
| `ports` | Port-Vorlagen fuer ErasureLayer/ErasedView (3.7) |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| serde | 3 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| async | 1 | tokio/async (G-NOTOKIO: Kern synchron) |
| alt_spec | 21 | Alt-Referenzen (D-/ER-/INV-Kennungen): durch SPEC-Verweise ersetzen |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `parking_lot` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-gate/`) | LOC | Befunde |
|---|---:|---|
| `erasure/src/gate.rs` | 14 |  |
| `erasure/src/gate/batch.rs` | 262 | async=1 |
| `erasure/src/gate/erased_set.rs` | 341 |  |
| `erasure/src/gate/fence.rs` | 441 | async=1 |
| `erasure/src/gate/log.rs` | 202 | serde=1 |
| `erasure/src/gate/read_gate.rs` | 120 |  |
| `erasure/tests/gate_linearization_test.rs` | 230 | panic=2, async=9 |
| `erasure/tests/loom_gate.rs` | 157 | panic=6 |
| `erasure/tests/t_gate.rs` | 75 |  |
| `ports/src/erasure.rs` | 272 | serde=1 |
| `ports/src/gate.rs` | 224 | serde=1 |
| `types/src/tombstone.rs` | 159 | panic=1 |

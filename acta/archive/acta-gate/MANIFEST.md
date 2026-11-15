# Extraktion fuer `acta-gate`

**Aufgabe:** ErasedView, GateWriter, Roaring-Mengen, ErasureLayer, LayerAck  
**Level:** L1 · **Budget:** 1200 LOC · **Spezifikation:** docs/SPEC.md Kap. 3.7, 5.3  
**Interne Abhaengigkeiten (Soll):** acta-base  
**Umfang des Archivs:** 8 Dateien, 1884 Zeilen (inkl. Tests)  

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
| `erasure/src/gate.rs` | 15 |  |
| `erasure/src/gate/batch.rs` | 263 | async=1, alt_spec=9 |
| `erasure/src/gate/erased_set.rs` | 342 | alt_spec=7 |
| `erasure/src/gate/fence.rs` | 442 | alt_spec=3 |
| `erasure/src/gate/log.rs` | 203 | serde=1 |
| `erasure/src/gate/read_gate.rs` | 121 | alt_spec=2 |
| `ports/src/erasure.rs` | 273 | serde=1 |
| `ports/src/gate.rs` | 225 | serde=1 |

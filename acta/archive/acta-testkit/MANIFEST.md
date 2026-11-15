# Extraktion fuer `acta-testkit`

**Aufgabe:** Entwicklungs-Crate: ManualClock, SeededRng, Fehlerinjektion, Konformitaetstests.  
**Umfang des Archivs:** 13 Dateien, 2415 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `testkit` | ManualClock, Konformitaetstests |
| `env` | SeededRng (SplitMix64) und ManualClock |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| float | 31 | f32/f64 (Z7: Festkomma in Ranking-Pfaden) |
| async | 25 | tokio/async (G-NOTOKIO: Kern synchron) |
| unsafe | 3 | unsafe (G-UNSAFE: nur acta-sys) |
| zeit_zufall | 2 | Zeit/Zufall direkt (G3: nur ueber Ports Clock/CryptoRng) |
| hub | 21 | Hub-Import ex types/ports (HR-2/HR-3) |
| alt_spec | 2 | Alt-Referenzen (D-/ER-/INV-Kennungen): durch SPEC-Verweise ersetzen |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `bytes`, `parking_lot` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-testkit/`) | LOC | Befunde |
|---|---:|---|
| `env/src/manual_clock.rs` | 71 |  |
| `env/src/rng.rs` | 283 | float=8 |
| `testkit/src/conformance/clock.rs` | 48 | hub=1 |
| `testkit/src/conformance/failure.rs` | 62 |  |
| `testkit/src/conformance/macros.rs` | 48 |  |
| `testkit/src/conformance/mod.rs` | 104 | async=2 |
| `testkit/src/conformance/rng.rs` | 66 | hub=1 |
| `testkit/src/in_memory_text.rs` | 207 | float=2, async=9, hub=2 |
| `testkit/src/in_memory_vector.rs` | 232 | float=6, async=10, hub=2 |
| `testkit/src/lib.rs` | 146 | unsafe=1, hub=4, alt_spec=1 |
| `testkit/src/manual_clock.rs` | 173 | float=2, zeit_zufall=2, hub=1 |
| `testkit/src/mocks.rs` | 691 | float=13, async=4, unsafe=2, hub=10, alt_spec=1 |
| `testkit/src/reference_model.rs` | 284 |  |

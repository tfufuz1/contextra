# Extraktion fuer `acta-testkit`

**Aufgabe:** Entwicklungs-Crate: ManualClock, SeededRng, Fehlerinjektion, Konformitaetstests.  
**Umfang des Archivs:** 16 Dateien, 4235 Zeilen (inkl. Tests)  

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
| `env/src/manual_clock.rs` | 70 |  |
| `env/src/rng.rs` | 282 | float=8 |
| `ports/src/conformance.rs` | 278 | async=2, serde=4 |
| `ports/src/contract_tests.rs` | 632 | panic=40, async=3 |
| `testkit/src/conformance/clock.rs` | 47 |  |
| `testkit/src/conformance/failure.rs` | 61 |  |
| `testkit/src/conformance/macros.rs` | 47 |  |
| `testkit/src/conformance/mod.rs` | 103 | async=3 |
| `testkit/src/conformance/rng.rs` | 65 |  |
| `testkit/src/in_memory_text.rs` | 206 | async=9, float=2 |
| `testkit/src/in_memory_vector.rs` | 231 | async=10, float=6 |
| `testkit/src/lib.rs` | 145 | unsafe=1 |
| `testkit/src/manual_clock.rs` | 172 | float=2 |
| `testkit/src/mocks.rs` | 690 | async=10, float=13, unsafe=2 |
| `testkit/src/reference_model.rs` | 283 |  |
| `testkit/tests/in_memory_doubles.rs` | 923 |  |

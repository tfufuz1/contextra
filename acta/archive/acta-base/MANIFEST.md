# Extraktion fuer `acta-base`

**Aufgabe:** Newtypes (Ids, Ordinale), DataClass, ErrorClass/ClassifiedError, Ports Clock/CryptoRng, kanonische Kodierung, Festkomma fx  
**Level:** L0 · **Budget:** 2500 LOC · **Spezifikation:** docs/SPEC.md Kap. 2.2, 2.4, 3.5, 3.7, 4.6  
**Interne Abhaengigkeiten (Soll):** keine  
**Umfang des Archivs:** 10 Dateien, 1752 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `ids` | Newtypes/Ordinale (Kap. 2.4); Ordinale u32/u64 und Handles 16 B neu typisieren |
| `env` | Ports Clock/CryptoRng (3.7); CryptoRng muss fallibel werden (try_fill) |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| float | 8 | f32/f64 (Z7: Festkomma in Ranking-Pfaden) |
| serde | 1 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| zeit_zufall | 4 | Zeit/Zufall direkt (G3: nur ueber Ports Clock/CryptoRng) |
| panic | 13 | unwrap/expect/panic (G-LINT) |
| hr6 | 2 | include!/#[deprecated] (HR-6) |
| hub | 1 | Hub-Import ex types/ports (HR-2/HR-3) |
| liste_a | 1 | verbotene Formulierung (1.6, Liste A) |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `blake3` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-base/`) | LOC | Befunde |
|---|---:|---|
| `env/src/clock.rs` | 174 | zeit_zufall=4 |
| `env/src/env.rs` | 39 |  |
| `env/src/id_gen.rs` | 148 | panic=1 |
| `env/src/lib.rs` | 78 |  |
| `env/src/rng.rs` | 283 | float=8 |
| `env/tests/conformance_env.rs` | 52 | liste_a=1 |
| `ids/src/lib.rs` | 721 | serde=1, hr6=2 |
| `ids/tests/differential_ids.rs` | 70 | panic=7 |
| `ids/tests/golden_tombstone_bit.rs` | 109 | hub=1 |
| `ids/tests/ids_tests.rs` | 78 | panic=5 |

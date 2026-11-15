# Extraktion fuer `acta-pii`

**Aufgabe:** Personenbezug-Erkennung, N-Validator-Bausteine, Validated<T>  
**Level:** L1 · **Budget:** 4000 LOC · **Spezifikation:** docs/SPEC.md Kap. 7.5  
**Interne Abhaengigkeiten (Soll):** acta-base  
**Umfang des Archivs:** 11 Dateien, 3400 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `privacy` | Mustererkennung -> Validated<T> (7.5); nur Hinweise, nie Zuordnung (2.3) |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| float | 13 | f32/f64 (Z7: Festkomma in Ranking-Pfaden) |
| serde | 9 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| async | 2 | tokio/async (G-NOTOKIO: Kern synchron) |
| zeit_zufall | 4 | Zeit/Zufall direkt (G3: nur ueber Ports Clock/CryptoRng) |
| panic | 19 | unwrap/expect/panic (G-LINT) |
| std_fs | 1 | std::fs direkt (G-FS: nur acta-fs) |
| alt_spec | 26 | Alt-Referenzen (D-/ER-/INV-Kennungen): durch SPEC-Verweise ersetzen |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `tracing` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-pii/`) | LOC | Befunde |
|---|---:|---|
| `privacy/src/entity_recognizer.rs` | 1391 | float=1, serde=3 |
| `privacy/src/error.rs` | 190 |  |
| `privacy/src/guarded_payload.rs` | 128 |  |
| `privacy/src/lib.rs` | 80 |  |
| `privacy/src/types.rs` | 171 | serde=4 |
| `privacy/tests/evasion_hardening_tests.rs` | 53 |  |
| `privacy/tests/known_answer_tests.rs` | 114 | panic=2, serde=1, std_fs=1 |
| `privacy/tests/pattern_recognizer_and_property_tests.rs` | 398 | panic=11 |
| `privacy/tests/privacy_consolidated_tests.rs` | 398 | panic=20, async=6, serde=4 |
| `privacy/tests/recognizer_quality.rs` | 350 | panic=2, float=12, serde=1 |
| `privacy/tests/zeroize_and_policy_fail_closed.rs` | 127 | panic=4, async=3 |

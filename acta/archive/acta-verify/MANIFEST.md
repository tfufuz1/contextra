# Extraktion fuer `acta-verify`

**Aufgabe:** Unabhaengige Pruefung von Berichten und Ledgern; keine interne Abhaengigkeit  
**Level:** L9 · **Budget:** 1200 LOC · **Spezifikation:** docs/SPEC.md Kap. 17  
**Interne Abhaengigkeiten (Soll):** keine  
**Umfang des Archivs:** 6 Dateien, 3538 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `erasure` | Nur Pruefalgorithmik; acta-verify hat keine interne Abhaengigkeit (17.1) |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| serde | 34 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| zeit_zufall | 2 | Zeit/Zufall direkt (G3: nur ueber Ports Clock/CryptoRng) |
| panic | 159 | unwrap/expect/panic (G-LINT) |
| hr6 | 2 | include!/#[deprecated] (HR-6) |
| alt_spec | 12 | Alt-Referenzen (D-/ER-/INV-Kennungen): durch SPEC-Verweise ersetzen |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `bincode`, `blake3`, `rand` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-verify/`) | LOC | Befunde |
|---|---:|---|
| `erasure/src/deletion_proof.rs` | 2744 | serde=33, zeit_zufall=1, panic=117, hr6=2, alt_spec=12 |
| `erasure/tests/deletion_proof_external_verify_rejects_tamper.rs` | 172 | panic=8 |
| `erasure/tests/deletion_proof_hash_collision.rs` | 113 | panic=7 |
| `erasure/tests/deletion_proof_prefix_collision_proptest.rs` | 76 |  |
| `erasure/tests/deletion_proof_signature_version_serde.rs` | 245 | serde=1, zeit_zufall=1, panic=16 |
| `erasure/tests/deletion_proof_v3_validation.rs` | 188 | panic=11 |

# Extraktion fuer `acta-reports`

**Aufgabe:** Loeschbericht (JCS), Art.-30-Verzeichnis, DSFA-Entwurf, Betriebs-/Kryptoberichte, Wortfilter  
**Level:** L5 · **Budget:** 3500 LOC · **Spezifikation:** docs/SPEC.md Kap. 18  
**Interne Abhaengigkeiten (Soll):** acta-base, acta-crypto, acta-erasure, acta-ledger, acta-history  
**Umfang des Archivs:** 31 Dateien, 5272 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `erasure` | Loeschbericht JCS+Ed25519 (18.1); Domaenenpraefix acta/report/v1 |
| `audit-export` | Audit-/Verzeichnis-Export (18.2, 18.4) |
| `avv-generator` | AVV-Vorlagen (Art. 28; 20.5) |
| `privacy` | Verzeichnis von Verarbeitungstaetigkeiten (Art. 30, 18.2) |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| serde | 26 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| async | 3 | tokio/async (G-NOTOKIO: Kern synchron) |
| unsafe | 1 | unsafe (G-UNSAFE: nur acta-sys) |
| panic | 26 | unwrap/expect/panic (G-LINT) |
| std_fs | 3 | std::fs direkt (G-FS: nur acta-fs) |
| alt_spec | 41 | Alt-Referenzen (D-/ER-/INV-Kennungen): durch SPEC-Verweise ersetzen |
| liste_a | 1 | verbotene Formulierung (1.6, Liste A) |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `blake3`, `futures-util` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-reports/`) | LOC | Befunde |
|---|---:|---|
| `audit-export/src/bsi_mapping.rs` | 153 | serde=1 |
| `audit-export/src/csv_template.rs` | 143 |  |
| `audit-export/src/error.rs` | 32 |  |
| `audit-export/src/lib.rs` | 375 | serde=1 |
| `audit-export/src/markdown_template.rs` | 148 |  |
| `audit-export/src/testkit.rs` | 127 |  |
| `audit-export/tests/error_and_edge_cases.rs` | 69 |  |
| `audit-export/tests/golden_contract_test.rs` | 84 | panic=12, std_fs=1 |
| `audit-export/tests/golden_file_test.rs` | 123 |  |
| `audit-export/tests/golden_generator.rs` | 36 | panic=10, std_fs=1 |
| `audit-export/tests/property_tests.rs` | 128 |  |
| `audit-export/tests/register_rendering.rs` | 84 |  |
| `audit-export/tests/sanitization_tests.rs` | 81 |  |
| `audit-export/tests/validation_tests.rs` | 204 |  |
| `avv-generator/src/assistance.rs` | 64 |  |
| `avv-generator/src/audit.rs` | 46 |  |
| `avv-generator/src/confidentiality.rs` | 45 |  |
| `avv-generator/src/data_subject_rights.rs` | 45 |  |
| `avv-generator/src/error.rs` | 98 |  |
| `avv-generator/src/instructions.rs` | 54 |  |
| `avv-generator/src/lib.rs` | 359 | serde=10, unsafe=1 |
| `avv-generator/src/template.rs` | 309 |  |
| `avv-generator/src/termination.rs` | 67 |  |
| `avv-generator/tests/avv_rendering.rs` | 414 | std_fs=1, liste_a=1 |
| `erasure/src/report.rs` | 28 |  |
| `erasure/src/report/binding.rs` | 47 | alt_spec=3 |
| `erasure/src/report/build.rs` | 652 | serde=7, alt_spec=22 |
| `erasure/src/report/canonical.rs` | 93 | serde=1 |
| `erasure/src/report/risk.rs` | 342 | serde=1, alt_spec=12 |
| `erasure/src/report/signing.rs` | 399 | serde=1, async=1, alt_spec=4 |
| `privacy/src/processing_registry.rs` | 423 | serde=4, async=2, panic=4 |

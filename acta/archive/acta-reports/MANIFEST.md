# Extraktion fuer `acta-reports`

**Aufgabe:** Loeschbericht (JCS), Art.-30-Verzeichnis, DSFA-Entwurf, Betriebs-/Kryptoberichte, Wortfilter  
**Level:** L5 · **Budget:** 3500 LOC · **Spezifikation:** docs/SPEC.md Kap. 18  
**Interne Abhaengigkeiten (Soll):** acta-base, acta-crypto, acta-erasure, acta-ledger, acta-history  
**Umfang des Archivs:** 33 Dateien, 5566 Zeilen (inkl. Tests)  

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
| `audit-export/src/bsi_mapping.rs` | 152 | serde=1 |
| `audit-export/src/csv_template.rs` | 142 |  |
| `audit-export/src/error.rs` | 31 |  |
| `audit-export/src/lib.rs` | 374 | serde=1 |
| `audit-export/src/markdown_template.rs` | 147 |  |
| `audit-export/src/testkit.rs` | 126 |  |
| `audit-export/tests/error_and_edge_cases.rs` | 68 |  |
| `audit-export/tests/golden_contract_test.rs` | 83 | panic=12, std_fs=1 |
| `audit-export/tests/golden_file_test.rs` | 122 |  |
| `audit-export/tests/golden_generator.rs` | 35 | panic=10, std_fs=1 |
| `audit-export/tests/property_tests.rs` | 127 |  |
| `audit-export/tests/register_rendering.rs` | 83 |  |
| `audit-export/tests/sanitization_tests.rs` | 80 |  |
| `audit-export/tests/validation_tests.rs` | 203 |  |
| `avv-generator/src/assistance.rs` | 63 |  |
| `avv-generator/src/audit.rs` | 45 |  |
| `avv-generator/src/confidentiality.rs` | 44 |  |
| `avv-generator/src/data_subject_rights.rs` | 44 |  |
| `avv-generator/src/error.rs` | 97 |  |
| `avv-generator/src/instructions.rs` | 53 |  |
| `avv-generator/src/lib.rs` | 358 | serde=10, unsafe=1 |
| `avv-generator/src/template.rs` | 308 |  |
| `avv-generator/src/termination.rs` | 66 |  |
| `avv-generator/tests/avv_rendering.rs` | 413 | std_fs=1 |
| `erasure/src/report.rs` | 27 |  |
| `erasure/src/report/binding.rs` | 46 |  |
| `erasure/src/report/build.rs` | 651 | serde=7, alt_spec=1 |
| `erasure/src/report/canonical.rs` | 92 | serde=1 |
| `erasure/src/report/risk.rs` | 341 | serde=1, alt_spec=1 |
| `erasure/src/report/signing.rs` | 398 | async=4, serde=1, alt_spec=2 |
| `erasure/tests/report_canonical_spec_tests.rs` | 210 | panic=9, async=1 |
| `erasure/tests/report_spec_tests.rs` | 115 | panic=10 |
| `privacy/src/processing_registry.rs` | 422 | panic=4, async=3, serde=4 |

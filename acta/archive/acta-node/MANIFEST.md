# Extraktion fuer `acta-node`

**Aufgabe:** Composition Root der Kernports: Konfiguration, Verdrahtung, Selbsttest, Hintergrundaufgaben  
**Level:** L7 · **Budget:** 3000 LOC · **Spezifikation:** docs/SPEC.md Kap. 3.4 HR-7, 14.1  
**Interne Abhaengigkeiten (Soll):** acta-engine, acta-base, acta-crypto, acta-fs, acta-sys, acta-gate, acta-access, acta-infer, acta-ingest, acta-ledger, acta-vault, acta-seg, acta-pstore, acta-history, acta-text, acta-vec, acta-erasure, acta-decision, acta-rag, acta-reports  
**Umfang des Archivs:** 16 Dateien, 3425 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `license` | Offline signierte Lizenzdatei ohne Nutzerdaten (14.7, Z19) |
| `env` | Metriken ohne Kennungen (13.5, 14.3) |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| float | 18 | f32/f64 (Z7: Festkomma in Ranking-Pfaden) |
| serde | 18 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| unsafe | 1 | unsafe (G-UNSAFE: nur acta-sys) |
| zeit_zufall | 3 | Zeit/Zufall direkt (G3: nur ueber Ports Clock/CryptoRng) |
| panic | 13 | unwrap/expect/panic (G-LINT) |
| std_fs | 3 | std::fs direkt (G-FS: nur acta-fs) |
| alt_spec | 10 | Alt-Referenzen (D-/ER-/INV-Kennungen): durch SPEC-Verweise ersetzen |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `bincode`, `blake3`, `tracing` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-node/`) | LOC | Befunde |
|---|---:|---|
| `env/src/metrics.rs` | 407 | panic=1, float=18 |
| `license/src/lib.rs` | 44 | serde=2, unsafe=1 |
| `license/src/signed_gate.rs` | 799 | serde=8, std_fs=3 |
| `license/src/types.rs` | 205 | serde=1 |
| `license/tests/clock_expiration_boundary.rs` | 150 |  |
| `license/tests/crypto_hygiene_payload.rs` | 160 | panic=2, serde=3 |
| `license/tests/error_no_information_leak.rs` | 34 |  |
| `license/tests/installation_binding.rs` | 296 | panic=14, serde=7 |
| `license/tests/installation_binding_isolation.rs` | 110 | std_fs=1 |
| `license/tests/key_rotation.rs` | 220 | panic=2, serde=2 |
| `license/tests/license_gate_default.rs` | 34 |  |
| `license/tests/negative_matrix_and_from_activation.rs` | 318 | panic=4, serde=3 |
| `license/tests/signature_covers_all_fields.rs` | 65 |  |
| `license/tests/signed_activation_7_steps.rs` | 188 |  |
| `license/tests/signed_gate.rs` | 236 | panic=10, serde=5 |
| `ports/src/license.rs` | 159 | serde=4 |

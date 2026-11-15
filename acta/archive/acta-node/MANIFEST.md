# Extraktion fuer `acta-node`

**Aufgabe:** Composition Root der Kernports: Konfiguration, Verdrahtung, Selbsttest, Hintergrundaufgaben  
**Level:** L7 · **Budget:** 3000 LOC · **Spezifikation:** docs/SPEC.md Kap. 3.4 HR-7, 14.1  
**Interne Abhaengigkeiten (Soll):** acta-engine, acta-base, acta-crypto, acta-fs, acta-sys, acta-gate, acta-access, acta-infer, acta-ingest, acta-ledger, acta-vault, acta-seg, acta-pstore, acta-history, acta-text, acta-vec, acta-erasure, acta-decision, acta-rag, acta-reports  
**Umfang des Archivs:** 10 Dateien, 2358 Zeilen (inkl. Tests)  

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
| `env/src/metrics.rs` | 408 | float=18, panic=1 |
| `license/src/lib.rs` | 45 | serde=2, unsafe=1 |
| `license/src/signed_gate.rs` | 800 | serde=8, std_fs=3, alt_spec=4 |
| `license/src/types.rs` | 206 | serde=1, zeit_zufall=1, alt_spec=2 |
| `license/tests/clock_expiration_boundary.rs` | 151 |  |
| `license/tests/error_no_information_leak.rs` | 35 |  |
| `license/tests/key_rotation.rs` | 221 | serde=2, panic=2 |
| `license/tests/signature_covers_all_fields.rs` | 66 |  |
| `license/tests/signed_activation_7_steps.rs` | 189 | zeit_zufall=1, alt_spec=1 |
| `license/tests/signed_gate.rs` | 237 | serde=5, zeit_zufall=1, panic=10, alt_spec=3 |

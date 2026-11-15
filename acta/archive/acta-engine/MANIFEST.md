# Extraktion fuer `acta-engine`

**Aufgabe:** Anwendungsfaelle: Ingest, Antwort, Fachfunktionen, Loeschung, Szenario, Aufbewahrung, Wartung  
**Level:** L6 · **Budget:** 12000 LOC · **Spezifikation:** docs/SPEC.md Kap. 10, 19  
**Interne Abhaengigkeiten (Soll):** acta-base, acta-crypto, acta-fs, acta-sys, acta-gate, acta-access, acta-infer, acta-pii, acta-ingest, acta-ledger, acta-vault, acta-seg, acta-pstore, acta-history, acta-text, acta-vec, acta-erasure, acta-decision, acta-rag, acta-reports  
**Umfang des Archivs:** 19 Dateien, 7225 Zeilen (inkl. Tests)  

## Herkunft und Umbauauftrag

| Altquelle (Crate) | Umbauauftrag |
|---|---|
| `privacy` | Egress-Schicht (13.3, Z18): Positivliste, EgressGrant, Vier-Augen |
| `db` | Fristenjob/Orphan-Sweeper (14.2, L09), Export/Import (Art. 15/20) |
| `checkpoint` | Orphan-Bereinigung |

## Pruefhinweise (automatisch ermittelt)

| Befund | Treffer | Folge |
|---|---:|---|
| float | 7 | f32/f64 (Z7: Festkomma in Ranking-Pfaden) |
| serde | 16 | serde/bincode (G-NOSERDE-DISK: Plattenformate von Hand kodieren) |
| async | 149 | tokio/async (G-NOTOKIO: Kern synchron) |
| zeit_zufall | 5 | Zeit/Zufall direkt (G3: nur ueber Ports Clock/CryptoRng) |
| panic | 129 | unwrap/expect/panic (G-LINT) |
| std_fs | 4 | std::fs direkt (G-FS: nur acta-fs) |
| hr6 | 16 | include!/#[deprecated] (HR-6) |
| hub | 26 | Hub-Import ex types/ports (HR-2/HR-3) |

**Externe Abhaengigkeiten der Altquelle ausserhalb der Whitelist (3.8):** `blake3`, `bytes`, `parking_lot`, `rand`, `scc`, `tokio-util`, `tracing` — ersetzen oder per ADR aufnehmen.

## Dateien

| Datei (relativ zu `archive/acta-engine/`) | LOC | Befunde |
|---|---:|---|
| `checkpoint/src/orphan.rs` | 837 | serde=1, async=5, panic=3, std_fs=4, hr6=7 |
| `checkpoint/tests/checkpoint_systematic_crash.rs` | 257 | async=4, panic=10, hub=2 |
| `checkpoint/tests/crash_restore_proofs.rs` | 421 | async=7, panic=16 |
| `db/src/background_workers/config.rs` | 71 | hub=4 |
| `db/src/background_workers/expiry_workers.rs` | 122 | async=12, hr6=2, hub=2 |
| `db/src/background_workers/orphan_workers.rs` | 197 | async=9, hr6=1, hub=3 |
| `db/src/export.rs` | 396 | float=2, serde=9, async=6, panic=33, hub=8 |
| `db/src/import.rs` | 145 | serde=1, async=2, hub=7 |
| `privacy/src/audit_trace.rs` | 124 | zeit_zufall=1 |
| `privacy/src/bulk_exfiltration_detector.rs` | 459 | serde=1, async=4, panic=1 |
| `privacy/src/context_edit_audit.rs` | 381 | serde=1, zeit_zufall=1 |
| `privacy/src/egress_gateway.rs` | 871 | serde=1, async=19, hr6=6 |
| `privacy/src/egress_guard.rs` | 545 | float=5, serde=1, async=24, panic=3 |
| `privacy/src/egress_vault.rs` | 1010 | serde=1, async=27, zeit_zufall=3, panic=31 |
| `privacy/tests/audit_trace_tests.rs` | 172 | async=10 |
| `privacy/tests/context_edit_audit_chain.rs` | 148 |  |
| `privacy/tests/egress_gateway_and_recognizer_tests.rs` | 905 | async=16, panic=19 |
| `privacy/tests/pii_vault_forces_crypto_shred.rs` | 32 |  |
| `privacy/tests/tenant_scoped_egress_isolation.rs` | 132 | async=4, panic=13 |

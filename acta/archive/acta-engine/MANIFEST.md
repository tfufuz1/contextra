# Extraktion fuer `acta-engine`

**Aufgabe:** Anwendungsfaelle: Ingest, Antwort, Fachfunktionen, Loeschung, Szenario, Aufbewahrung, Wartung  
**Level:** L6 · **Budget:** 12000 LOC · **Spezifikation:** docs/SPEC.md Kap. 10, 19  
**Interne Abhaengigkeiten (Soll):** acta-base, acta-crypto, acta-fs, acta-sys, acta-gate, acta-access, acta-infer, acta-pii, acta-ingest, acta-ledger, acta-vault, acta-seg, acta-pstore, acta-history, acta-text, acta-vec, acta-erasure, acta-decision, acta-rag, acta-reports  
**Umfang des Archivs:** 22 Dateien, 8139 Zeilen (inkl. Tests)  

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
| `checkpoint/src/orphan.rs` | 836 | panic=3, async=6, serde=1, std_fs=4 |
| `checkpoint/tests/checkpoint_systematic_crash.rs` | 256 | panic=10, async=9 |
| `checkpoint/tests/crash_restore_proofs.rs` | 420 | panic=16, async=20 |
| `db/src/background_workers/config.rs` | 70 | async=1 |
| `db/src/background_workers/expiry_workers.rs` | 121 | async=14 |
| `db/src/background_workers/orphan_workers.rs` | 196 | async=10 |
| `db/src/export.rs` | 395 | panic=33, async=6, float=2, serde=9 |
| `db/src/import.rs` | 144 | async=2, serde=1 |
| `engine/tests/metrics_instrumentation.rs` | 248 |  |
| `engine/tests/tx_fault_injection.rs` | 337 |  |
| `engine/tests/tx_recovery_and_erasure.rs` | 348 |  |
| `privacy/src/audit_trace.rs` | 123 | async=1 |
| `privacy/src/bulk_exfiltration_detector.rs` | 458 | panic=1, async=4, serde=1 |
| `privacy/src/context_edit_audit.rs` | 380 | serde=1 |
| `privacy/src/egress_gateway.rs` | 870 | async=21, serde=1 |
| `privacy/src/egress_guard.rs` | 544 | panic=3, async=26, float=5, serde=1 |
| `privacy/src/egress_vault.rs` | 1009 | panic=31, async=27, serde=1 |
| `privacy/tests/audit_trace_tests.rs` | 171 | async=10 |
| `privacy/tests/context_edit_audit_chain.rs` | 147 |  |
| `privacy/tests/egress_gateway_and_recognizer_tests.rs` | 904 | panic=19, async=19 |
| `privacy/tests/pii_vault_forces_crypto_shred.rs` | 31 |  |
| `privacy/tests/tenant_scoped_egress_isolation.rs` | 131 | panic=13, async=4 |

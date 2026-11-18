# Deletion & GDPR Domain Leak Diagnostic Report

> **Werkzeug:** `erasure_leak_prober.py`  
> **Erstellungszeit:** `2026-11-18T18:00:58.785732+00:00`  
> **Status/Befunde:** `161 Befunde erfasst`  

## Beschreibung
Audits data-holding storage kernels against rule M7 (no GDPR, Saga, DeletionProof, or report logic outside contextra-erasure).

## Metriken-Übersicht
| Metrik | Wert |
|---|---|
| `kernel_violations` | `21` |
| `allowed_mentions` | `140` |
| `crates_with_leaks` | `4` |

## Löschketten-Entkopplungs-Status
| Crate | Schicht/Rolle | Status | Löschdomänen-Bezüge |
|---|---|---|---|
| `contextra-crypto` | foundation | **VERLETZUNG (M7)** | 21 |
| `contextra-erasure` | compliance | Zulässig (Löschdomäne) | 133 |
| `contextra-ports` | contracts | Zulässig (Löschdomäne) | 1 |
| `contextra-types` | contracts | Zulässig (Löschdomäne) | 6 |

## Detaillierte Befunde
| Crate | Datei | Zeile | Art / Beschreibung |
|---|---|---|---|
| `contextra-crypto` | `crates/contextra-crypto/benches/deletion_proof_latency_bench.rs` | `3` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/benches/deletion_proof_latency_bench.rs` | `13` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/benches/deletion_proof_latency_bench.rs` | `45` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/benches/deletion_proof_latency_bench.rs` | `86` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/benches/deletion_proof_latency_bench.rs` | `133` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs` | `10` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs` | `36` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs` | `55` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs` | `74` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs` | `105` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs` | `127` | Saga orchestration in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs` | `131` | Saga orchestration in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs` | `138` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_verify_external_fuzz.rs` | `10` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_verify_external_fuzz.rs` | `15` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_verify_external_fuzz.rs` | `19` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_verify_external_fuzz.rs` | `35` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_verify_external_fuzz.rs` | `39` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_verify_external_fuzz.rs` | `79` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_verify_external_fuzz.rs` | `106` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_verify_external_fuzz.rs` | `123` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `2` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `3` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `23` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `230` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `247` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `275` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `432` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `433` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `465` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `498` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `499` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `507` | Saga orchestration in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `707` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `712` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `732` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `797` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `822` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `849` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `876` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1151` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1242` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1318` | Saga orchestration in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1322` | Saga orchestration in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1326` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1343` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1361` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1379` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1395` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1416` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1434` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1445` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1456` | Saga orchestration in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1460` | Saga orchestration in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1469` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1491` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1546` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1573` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1594` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1617` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1674` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1705` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1727` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1755` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1767` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1790` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1825` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1836` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1875` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1886` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1906` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1932` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1941` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1946` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1964` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `1990` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `2018` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `2064` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `2100` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `2126` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `2162` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `2218` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `2268` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `2296` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `2316` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `2331` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `2369` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `2391` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `2408` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `2439` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `2468` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `2504` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `2602` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `2700` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `2702` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `2729` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/deletion_proof.rs` | `2731` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/ed25519_proof.rs` | `49` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/ed25519_proof.rs` | `127` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/kv_shredding.rs` | `1` | Legacy kv_shredding reference (DEBT-01) (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/kv_shredding.rs` | `21` | Legacy revocation_log reference (DEBT-01) (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/kv_shredding.rs` | `231` | Legacy revocation_log reference (DEBT-01) (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/kv_shredding.rs` | `256` | Legacy revocation_log reference (DEBT-01) (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/kv_shredding.rs` | `280` | Legacy revocation_log reference (DEBT-01) (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/kv_shredding.rs` | `327` | Legacy revocation_log reference (DEBT-01) (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/kv_shredding.rs` | `398` | Legacy revocation_log reference (DEBT-01) (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/kv_shredding.rs` | `484` | Legacy revocation_log reference (DEBT-01) (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/kv_shredding.rs` | `515` | Legacy revocation_log reference (DEBT-01) (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/ledger.rs` | `16` | Erasure ledger reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/ledger.rs` | `17` | Erasure ledger reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/lib.rs` | `40` | Legacy kv_shredding reference (DEBT-01) (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/lib.rs` | `44` | Legacy revocation_log reference (DEBT-01) (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/orchestrator.rs` | `6` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/orchestrator.rs` | `11` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/orchestrator.rs` | `27` | Proof key provider reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/orchestrator.rs` | `32` | Proof key provider reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/orchestrator.rs` | `93` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/orchestrator.rs` | `94` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/restricted.rs` | `1` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/restricted.rs` | `21` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/restricted.rs` | `79` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/restricted.rs` | `162` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/revocation_log.rs` | `1` | Legacy revocation_log reference (DEBT-01) (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/revocation_log.rs` | `12` | Legacy revocation_log reference (DEBT-01) (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/revocation_log.rs` | `27` | Legacy revocation_log reference (DEBT-01) (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/supersede.rs` | `26` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/types_local.rs` | `1714` | Saga orchestration in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/types_local.rs` | `1719` | Saga orchestration in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/types_local.rs` | `1720` | Saga orchestration in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/vault/key_store.rs` | `11` | Legacy kv_shredding reference (DEBT-01) (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/vault/key_store.rs` | `200` | Legacy kv_shredding reference (DEBT-01) (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/vault/key_store.rs` | `206` | Legacy kv_shredding reference (DEBT-01) (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/vault/key_store.rs` | `209` | Legacy kv_shredding reference (DEBT-01) (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/vault/key_store.rs` | `212` | Legacy kv_shredding reference (DEBT-01) (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/vault/key_store.rs` | `213` | Legacy kv_shredding reference (DEBT-01) (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/saga/machine.rs` | `40` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/ledger/durable.rs` | `1` | Erasure ledger reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/ledger/durable.rs` | `37` | Erasure ledger reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/ledger/durable.rs` | `45` | Erasure ledger reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/ledger/durable.rs` | `47` | Erasure ledger reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/ledger/durable.rs` | `55` | Erasure ledger reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/ledger/durable.rs` | `56` | Erasure ledger reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/ledger/persistent.rs` | `1` | Erasure ledger reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/ledger/persistent.rs` | `15` | Erasure ledger reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/ledger/persistent.rs` | `24` | Erasure ledger reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/ledger/persistent.rs` | `25` | Erasure ledger reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/ledger/persistent.rs` | `170` | Erasure ledger reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/ledger/persistent.rs` | `175` | Erasure ledger reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/ledger/persistent.rs` | `181` | Erasure ledger reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/ledger/sink.rs` | `6` | Erasure ledger reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/ledger/sink.rs` | `16` | Erasure ledger reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/ledger/sink.rs` | `24` | Erasure ledger reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/ledger/sink.rs` | `25` | Erasure ledger reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/ledger/sink.rs` | `57` | Erasure ledger reference in data kernel (Rule M7) |
| `contextra-ports` | `crates/contextra-ports/src/lib.rs` | `27` | Saga orchestration in data kernel (Rule M7) |
| `contextra-types` | `crates/contextra-types/src/lib.rs` | `76` | Saga orchestration in data kernel (Rule M7) |
| `contextra-types` | `crates/contextra-types/src/witness.rs` | `13` | Saga orchestration in data kernel (Rule M7) |
| `contextra-types` | `crates/contextra-types/src/witness.rs` | `16` | Saga orchestration in data kernel (Rule M7) |
| `contextra-types` | `crates/contextra-types/src/witness.rs` | `84` | Saga orchestration in data kernel (Rule M7) |
| `contextra-types` | `crates/contextra-types/src/witness.rs` | `89` | Saga orchestration in data kernel (Rule M7) |
| `contextra-types` | `crates/contextra-types/src/witness.rs` | `91` | Saga orchestration in data kernel (Rule M7) |

---
*Automatisch generiert durch `erasure_leak_prober.py` für Agentic Engineering & Context Auditing.*
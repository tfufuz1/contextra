# Deletion & GDPR Domain Leak Diagnostic Report

> **Werkzeug:** `erasure_leak_prober.py`  
> **Erstellungszeit:** `2026-11-07T23:03:39.571477+00:00`  
> **Status/Befunde:** `181 Befunde erfasst`  

## Beschreibung
Audits data-holding storage kernels against rule M7 (no GDPR, Saga, DeletionProof, or report logic outside contextra-erasure).

## Metriken-Übersicht
| Metrik | Wert |
|---|---|
| `kernel_violations` | `113` |
| `allowed_mentions` | `68` |
| `crates_with_leaks` | `8` |

## Löschketten-Entkopplungs-Status
| Crate | Schicht/Rolle | Status | Löschdomänen-Bezüge |
|---|---|---|---|
| `contextra-audit-export` | Kernel | **VERLETZUNG (M7)** | 20 |
| `contextra-avv-generator` | Kernel | Zulässig (Löschdomäne) | 56 |
| `contextra-crypto` | Kernel | **VERLETZUNG (M7)** | 66 |
| `contextra-engine` | Kernel | **VERLETZUNG (M7)** | 25 |
| `contextra-erasure` | Kernel | Zulässig (Löschdomäne) | 6 |
| `contextra-ports` | Kernel | Zulässig (Löschdomäne) | 1 |
| `contextra-store` | Kernel | **VERLETZUNG (M7)** | 2 |
| `contextra-types` | Kernel | Zulässig (Löschdomäne) | 5 |

## Detaillierte Befunde
| Crate | Datei | Zeile | Art / Beschreibung |
|---|---|---|---|
| `contextra-audit-export` | `crates/contextra-audit-export/examples/audit_export_standalone.rs` | `2` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-audit-export` | `crates/contextra-audit-export/src/bsi_mapping.rs` | `5` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-audit-export` | `crates/contextra-audit-export/src/bsi_mapping.rs` | `43` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-audit-export` | `crates/contextra-audit-export/src/error.rs` | `12` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-audit-export` | `crates/contextra-audit-export/src/lib.rs` | `1` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-audit-export` | `crates/contextra-audit-export/src/lib.rs` | `5` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-audit-export` | `crates/contextra-audit-export/src/lib.rs` | `23` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-audit-export` | `crates/contextra-audit-export/src/lib.rs` | `53` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-audit-export` | `crates/contextra-audit-export/src/lib.rs` | `79` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-audit-export` | `crates/contextra-audit-export/src/lib.rs` | `118` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-audit-export` | `crates/contextra-audit-export/src/lib.rs` | `131` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-audit-export` | `crates/contextra-audit-export/src/lib.rs` | `152` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-audit-export` | `crates/contextra-audit-export/src/lib.rs` | `163` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-audit-export` | `crates/contextra-audit-export/src/lib.rs` | `256` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-audit-export` | `crates/contextra-audit-export/src/markdown_template.rs` | `1` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-audit-export` | `crates/contextra-audit-export/src/markdown_template.rs` | `20` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-audit-export` | `crates/contextra-audit-export/src/markdown_template.rs` | `47` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-audit-export` | `crates/contextra-audit-export/src/markdown_template.rs` | `49` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-audit-export` | `crates/contextra-audit-export/src/testkit.rs` | `56` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-audit-export` | `crates/contextra-audit-export/src/testkit.rs` | `89` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/examples/avv_generator_standalone.rs` | `2` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/assistance.rs` | `1` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/assistance.rs` | `5` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/assistance.rs` | `7` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/assistance.rs` | `9` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/assistance.rs` | `10` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/assistance.rs` | `12` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/assistance.rs` | `22` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/assistance.rs` | `30` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/assistance.rs` | `31` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/assistance.rs` | `32` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/assistance.rs` | `39` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/assistance.rs` | `44` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/assistance.rs` | `49` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/assistance.rs` | `54` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/assistance.rs` | `59` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/audit.rs` | `1` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/audit.rs` | `7` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/audit.rs` | `36` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/confidentiality.rs` | `1` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/confidentiality.rs` | `8` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/data_subject_rights.rs` | `1` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/data_subject_rights.rs` | `5` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/data_subject_rights.rs` | `7` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/data_subject_rights.rs` | `21` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/data_subject_rights.rs` | `35` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/error.rs` | `35` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/error.rs` | `42` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/error.rs` | `70` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/error.rs` | `81` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/instructions.rs` | `1` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/instructions.rs` | `8` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/instructions.rs` | `49` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/lib.rs` | `3` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/lib.rs` | `7` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/lib.rs` | `9` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/lib.rs` | `10` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/lib.rs` | `44` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/lib.rs` | `78` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/lib.rs` | `229` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/lib.rs` | `239` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/lib.rs` | `285` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/lib.rs` | `290` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/lib.rs` | `295` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/lib.rs` | `300` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/template.rs` | `1` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/template.rs` | `7` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/template.rs` | `10` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/template.rs` | `11` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/template.rs` | `141` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/template.rs` | `232` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/template.rs` | `246` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/template.rs` | `273` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/termination.rs` | `1` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/termination.rs` | `7` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-avv-generator` | `crates/contextra-avv-generator/src/termination.rs` | `36` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/audit_chain.rs` | `28` | Legacy kv_shredding reference (DEBT-01) (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/ed25519_proof.rs` | `39` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/ed25519_proof.rs` | `134` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/kv_cipher.rs` | `106` | Legacy kv_shredding reference (DEBT-01) (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/kv_shredding.rs` | `1` | Legacy kv_shredding reference (DEBT-01) (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/kv_shredding.rs` | `22` | Legacy revocation_log reference (DEBT-01) (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/kv_shredding.rs` | `125` | Legacy revocation_log reference (DEBT-01) (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/kv_shredding.rs` | `134` | Legacy revocation_log reference (DEBT-01) (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/kv_shredding.rs` | `157` | Legacy revocation_log reference (DEBT-01) (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/kv_shredding.rs` | `187` | Legacy revocation_log reference (DEBT-01) (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/kv_shredding.rs` | `267` | Legacy revocation_log reference (DEBT-01) (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/lib.rs` | `54` | Legacy kv_shredding reference (DEBT-01) (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/lib.rs` | `55` | Legacy revocation_log reference (DEBT-01) (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/lib.rs` | `72` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/lib.rs` | `85` | Legacy kv_shredding reference (DEBT-01) (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/lib.rs` | `86` | Legacy revocation_log reference (DEBT-01) (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/revocation_log.rs` | `1` | Legacy revocation_log reference (DEBT-01) (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/revocation_log.rs` | `12` | Legacy revocation_log reference (DEBT-01) (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/revocation_log.rs` | `26` | Legacy revocation_log reference (DEBT-01) (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/revocation_log.rs` | `327` | Legacy kv_shredding reference (DEBT-01) (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/chain.rs` | `8` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/chain.rs` | `12` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/creation.rs` | `15` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/creation.rs` | `16` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/creation.rs` | `20` | Saga orchestration in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/creation.rs` | `212` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/creation.rs` | `232` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/creation.rs` | `296` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/creation.rs` | `321` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/creation.rs` | `348` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/creation.rs` | `375` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/key_derivation.rs` | `11` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/mod.rs` | `7` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/mod.rs` | `48` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/types.rs` | `321` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/verification.rs` | `17` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/verification.rs` | `34` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/verifier.rs` | `5` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/verifier.rs` | `8` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/verifier.rs` | `89` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/verifier.rs` | `96` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/verifier.rs` | `114` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/verifier.rs` | `116` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/verifier.rs` | `143` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/src/deletion_proof/verifier.rs` | `145` | Usage of DeletionProof in data kernel (Rule M7) |
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
| `contextra-crypto` | `crates/contextra-crypto/benches/deletion_proof_latency_bench.rs` | `3` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/benches/deletion_proof_latency_bench.rs` | `13` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/benches/deletion_proof_latency_bench.rs` | `45` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/benches/deletion_proof_latency_bench.rs` | `86` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-crypto` | `crates/contextra-crypto/benches/deletion_proof_latency_bench.rs` | `133` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `21` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `393` | Proof key provider reference in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/lib.rs` | `397` | Proof key provider reference in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/collections.rs` | `303` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/collections.rs` | `305` | Proof key provider reference in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/collections.rs` | `327` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/erasure.rs` | `11` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/erasure.rs` | `22` | Saga orchestration in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/erasure.rs` | `37` | Proof key provider reference in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/erasure.rs` | `98` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/erasure.rs` | `99` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/erasure.rs` | `113` | Proof key provider reference in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/erasure.rs` | `114` | Proof key provider reference in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/erasure.rs` | `118` | Proof key provider reference in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/erasure.rs` | `119` | Proof key provider reference in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/erasure.rs` | `147` | Proof key provider reference in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/erasure.rs` | `314` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/erasure.rs` | `315` | Saga orchestration in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/erasure.rs` | `316` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/erasure.rs` | `337` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/erasure.rs` | `356` | Proof key provider reference in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/erasure.rs` | `573` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/erasure.rs` | `574` | Saga orchestration in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/erasure.rs` | `575` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-engine` | `crates/contextra-engine/src/contextra_impl/erasure.rs` | `599` | Usage of DeletionProof in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/restricted.rs` | `1` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/restricted.rs` | `21` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/restricted.rs` | `79` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/restricted.rs` | `162` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/supersede.rs` | `26` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-erasure` | `crates/contextra-erasure/src/saga/machine.rs` | `40` | GDPR/DSGVO legal reference in data kernel (Rule M7) |
| `contextra-ports` | `crates/contextra-ports/src/lib.rs` | `26` | Saga orchestration in data kernel (Rule M7) |
| `contextra-store` | `crates/contextra-store/src/kv/segment.rs` | `7` | Legacy kv_shredding reference (DEBT-01) (Rule M7) |
| `contextra-store` | `crates/contextra-store/src/kv/segment.rs` | `15` | Legacy kv_shredding reference (DEBT-01) (Rule M7) |
| `contextra-types` | `crates/contextra-types/src/lib.rs` | `77` | Saga orchestration in data kernel (Rule M7) |
| `contextra-types` | `crates/contextra-types/src/witness.rs` | `13` | Saga orchestration in data kernel (Rule M7) |
| `contextra-types` | `crates/contextra-types/src/witness.rs` | `80` | Saga orchestration in data kernel (Rule M7) |
| `contextra-types` | `crates/contextra-types/src/witness.rs` | `85` | Saga orchestration in data kernel (Rule M7) |
| `contextra-types` | `crates/contextra-types/src/witness.rs` | `87` | Saga orchestration in data kernel (Rule M7) |

---
*Automatisch generiert durch `erasure_leak_prober.py` für Agentic Engineering & Context Auditing.*
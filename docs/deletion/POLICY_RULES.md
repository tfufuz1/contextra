# Deletion Proof v4 Policy Rules Mapping Specification (C7 / ADR-13)

Diese Dokumentation beschreibt die Zuordnung zwischen den Spezifikationsregeln (C7 / ADR-13) und den im Verifier-Engine (`crates/contextra-crypto/src/deletion_proof/policy.rs:1`) implementierten Code-Regel-IDs.

## Abbildungstabelle: Spezifikation ↔ Code-Regel-ID ↔ Test

| Spezifikations-Regel (C7) | Code-Regel-ID | Beschreibung | Code-Anker (Verifier / Test) |
|---|---|---|---|
| Signatur-Validität | **R1** | Ed25519-Signatur über `PROOV_V4_DOMAIN_TAG \|\| CANONICAL_TLV` ist gültig für den öffentlichen Schlüssel. | `crates/contextra-crypto/src/deletion_proof/policy.rs:104`, `test_r1_signature_verification_passes_and_fails` |
| Format-Version | **R2** | `proof.version == 4`. Legacy-Versionen (1..3) werden nicht als v4 evaluiert. | `crates/contextra-crypto/src/deletion_proof/policy.rs:115`, `test_r2_version_verification` |
| Pflicht-Layer | **R3** | Alle von der Policy geforderten Layer (`required_layers`) sind in den Claims enthalten. | `crates/contextra-crypto/src/deletion_proof/policy.rs:135`, `test_r3_required_layers_coverage` |
| Evidence-Invarianten | **R4** | Der optionale `EvidenceInvariant`-Verifier bestätigt die Evidence-Payloads für jeden Claim. | `crates/contextra-crypto/src/deletion_proof/policy.rs:165`, `test_r4_evidence_invariant_evaluation` |
| Pflicht-Exclusions | **R5** | Alle von der Policy geforderten ausgeschlossenen Scopes (`mandatory_excluded_scopes`) sind in `excluded_scopes` präsent. | `crates/contextra-crypto/src/deletion_proof/policy.rs:185`, `test_r5_mandatory_excluded_scopes` |
| Timestamp & Monotonic Counter | **R6** | `issued_wall_ms > 0` und `issued_mono_ctr > 0`. | `crates/contextra-crypto/src/deletion_proof/policy.rs:205`, `test_r6_timestamp_and_monotonic_counter` |
| Ketten-Anker | **R7** | `chain.prev_hash` stimmt bei vertrauenswürdigem Anker (`trusted_anchor`) mit dem erwarteten Vorläufer-Hash überein. | `crates/contextra-crypto/src/deletion_proof/policy.rs:225`, `test_r7_chain_hash_anchor_verification` |
| Assurance & Text-IDs | **R8** | Jeder Claim hat `assurance <= 4` (S0..S4) und ein gültiges `claim_text_id` (1..=8). | `crates/contextra-crypto/src/deletion_proof/policy.rs:240`, `test_r8_assurance_and_text_id_validity` |
| Erlaubte Text-IDs | **R9** | `claim_text_id` jedes Claims ist in der Policy-Menge `allowed_claim_text_ids` enthalten. | `crates/contextra-crypto/src/deletion_proof/policy.rs:250`, `test_r9_allowed_claim_text_ids` |
| Signer Key ID | **R10** | `proof.signer_key_id == BLAKE3(public_key)`. | `crates/contextra-crypto/src/deletion_proof/policy.rs:258`, `test_r10_signer_key_id_match` |
| Supersedes-Referenz | **R11** | Wenn `supersedes` gesetzt ist, darf es weder Null-Hash noch die eigene `proof_id` sein. | `crates/contextra-crypto/src/deletion_proof/policy.rs:271`, `test_r11_supersedes_reference_check` |
| Empty-State Consistency | **R12** | `deleted_keys_hash != [0; 32]` ODER `observed_empty == true`. | `crates/contextra-crypto/src/deletion_proof/policy.rs:286`, `test_r12_deleted_keys_hash_vs_observed_empty` |
| Kritische TLV-Tags | **R13** | Keine unbekannten TLV-Tags mit gesetztem Critical-Bit (oberstes Bit `0x8000`) in der Payload. | `crates/contextra-crypto/src/deletion_proof/policy.rs:298`, `test_r13_unknown_critical_tlv_tags_rejection` |
| Claim Scan Tx Boundaries | **R14** | Für jeden Claim muss `scanned_at_tx >= deleted_after_tx` gelten. | `crates/contextra-crypto/src/deletion_proof/policy.rs:311`, `test_r14_claim_scanned_at_tx_boundary` |
| Stage & Supersedes Consistency | **R15** | Stage 1 (Interim) oder Stage 2 (Final). Bei `supersedes` muss Stage konsistent verweisen. | `crates/contextra-crypto/src/deletion_proof/policy.rs:326`, `test_r15_stage_and_supersedes_consistency` |
| Signer Trust Set | **R16** | Wenn `trusted_signer_key_ids` in der Policy gesetzt ist, muss `signer_key_id` enthalten sein. | `crates/contextra-crypto/src/deletion_proof/policy.rs:349`, `test_r16_signer_key_id_trust_set` |
| Non-Critical Unknown Tags | **R17** | Unbekannte nicht-kritische TLV-Tags (Critical-Bit `0x8000` NICHT gesetzt) werden ignoriert ohne Fehler/Panic. | `crates/contextra-crypto/src/deletion_proof/policy.rs:365`, `test_r17_non_critical_unknown_tlv_tags_ignored` |

## Stabilitätsgarantie
Die Code-Regel-IDs R1..R13 bleiben im Code unverändert. Neue Prüfungen werden ausschließlich mit neuen fortlaufenden IDs ab R14 ergänzt.

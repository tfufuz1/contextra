# PENDING_API_CHANGES (WP-12 / P30)

Format: Symbol | betroffene Crates | gewünschte Änderung

| Symbol | betroffene Crates | gewünschte Änderung |
|---|---|---|
| `contextra_crypto::DeletionProofKeyPair` | `contextra-store` (`tests/campaign_j42_audit.rs`) | Auf `contextra_erasure::ed25519_proof::DeletionProofKeyPair` umstellen oder Test isolieren |
| `contextra_crypto::deletion_proof::*` | `contextra-db` (`tests/deletion_proof_integration.rs`, `tests/e2e_compliance_scenario.rs`) | Auf `contextra_erasure::deletion_proof::*` umstellen |
| `contextra_crypto::deletion_proof::*` | `contextra-engine` (`tests/erasure_report_honesty.rs`, `tests/proof_verify_not_constant.rs`) | Auf `contextra_erasure::deletion_proof::*` umstellen |

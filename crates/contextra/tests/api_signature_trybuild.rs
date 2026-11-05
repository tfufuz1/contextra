//! # Integrationstest: `api_signature_trybuild` (T-API-1, T-API-2)
//!
//! Absicherung der stabilen öffentlichen API-Signatur und Pflichtparameter per Typsystem.
//! Compile-Fail-Tests erzwingen:
//! - Pflichtparameter Passphrase (SPEC 5.7)
//! - Pflichtparameter Runtime-Handle (SPEC 5.7)
//! - Pflichtparameter Backup-Retention (SPEC 5.7, D-74)
//! - Pflichtparameter TenantContext (SPEC 7.1, D-07)
//! - Pflichtparameter RetentionAttestation (SPEC 7.3, D-15)

#[test]
fn test_public_api_compile_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/trybuild/01_public_api_surface.rs");
}

#[test]
fn test_missing_mandatory_params_compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/trybuild/02_missing_passphrase_compile_fail.rs");
    t.compile_fail("tests/trybuild/03_missing_runtime_compile_fail.rs");
    t.compile_fail("tests/trybuild/04_missing_tenant_context_compile_fail.rs");
    t.compile_fail("tests/trybuild/05_missing_backup_retention_compile_fail.rs");
    t.compile_fail("tests/trybuild/06_missing_retention_attestation_compile_fail.rs");
    t.compile_fail("tests/trybuild/07_missing_dimension_compile_fail.rs");
}

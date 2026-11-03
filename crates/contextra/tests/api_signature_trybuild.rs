//! # Integrationstest: `api_signature_trybuild` (T-API-1)
//!
//! Absicherung der stabilen öffentlichen API-Signatur und Pflichtparameter per Typsystem.
//! Compile-Fail-Tests erzwingen:
//! - Pflichtparameter Passphrase
//! - Pflichtparameter Runtime-Handle
//! - Pflichtparameter TenantContext (kein Zugriff auf Collection ohne Tenant)

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
}

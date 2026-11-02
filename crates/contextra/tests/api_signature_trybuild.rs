//! # Integrationstest: `api_signature_trybuild` (T-API-1)
//!
//! Absicherung der stabilen öffentlichen API-Signatur gegen Regressionen.
//! Trybuild-Basis-Setup für Kompiliertests. Die Compile-Fail-Tests für
//! fehlende Pflichtparameter werden in Stufe 2 (K10) aktiviert.

// ANCHOR[contextra:TEST-TRY-001]
#[test]
fn test_public_api_compile_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/trybuild/01_public_api_surface.rs");
}

// FILE-CONTEXT
// STAND: 2026-10-06T00:00:00Z
// ZWECK: Integration tests for WASM Module Provenance Policy (P05 / F-02 HIGH)
// INVARIANTEN: Untrusted WASM binaries are rejected before compilation with SandboxError::InvalidModule

use contextra_sandbox::{
    capabilities::ModulePolicy, AdmittedModule, SandboxError, WasmCapabilities, WasmExecutor,
    WasmMergeFunction,
};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_untrusted_wasm_module_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let wat = r#"
        (module
            (func (export "_start"))
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;
    let admitted = AdmittedModule::new(wasm_bytes, [0u8; 32]);

    let executor = WasmExecutor::new()?;

    // Hash allowlist with a dummy non-matching hash
    let dummy_hash = [0xAA; 32];
    let caps_restricted = WasmCapabilities {
        module_policy: ModulePolicy::HashAllowlist(vec![dummy_hash]),
        ..Default::default()
    };

    // Attempting execution of untrusted binary must be rejected with SandboxError::InvalidModule
    let res = executor
        .execute_admitted(&admitted, b"", &caps_restricted, Duration::from_secs(1))
        .await;

    assert!(
        matches!(res, Err(SandboxError::InvalidModule(ref msg)) if msg.contains("provenance")),
        "Expected InvalidModule error due to module provenance violation, got: {:?}",
        res
    );

    Ok(())
}

#[tokio::test]
async fn test_trusted_wasm_module_in_allowlist_allowed() -> Result<(), Box<dyn std::error::Error>>
{
    let wat = r#"
        (module
            (func (export "_start"))
        )
    "#;
    let wasm_bytes = wat::parse_str(wat)?;

    let executor = WasmExecutor::new()?;

    // Calculate actual hash of wasm_bytes using helper or capabilities verify
    let hash = WasmCapabilities::compute_sha256(&wasm_bytes);
    let admitted = AdmittedModule::new(wasm_bytes, hash);

    let caps_allowed = WasmCapabilities {
        module_policy: ModulePolicy::HashAllowlist(vec![hash]),
        ..Default::default()
    };

    let res = executor
        .execute_admitted(&admitted, b"", &caps_allowed, Duration::from_secs(1))
        .await;

    assert!(
        res.is_ok(),
        "Expected execution to succeed for trusted module in allowlist, got: {:?}",
        res
    );

    Ok(())
}

#[test]
fn test_wasm_merge_function_new_with_allowlist() -> Result<(), Box<dyn std::error::Error>> {
    let wat = r#"
        (module
            (func (export "_start"))
        )
    "#;
    let wasm_bytes: Arc<[u8]> = wat::parse_str(wat)?.into();

    let actual_hash = WasmCapabilities::compute_sha256(&wasm_bytes);

    // Fail case: wrong hash
    let wrong_hash = [0xBB; 32];
    let res_err = WasmMergeFunction::new_with_allowlist(wasm_bytes.clone(), vec![wrong_hash]);
    assert!(
        matches!(res_err, Err(SandboxError::InvalidModule(_))),
        "Expected InvalidModule error on wrong hash allowlist"
    );

    // Success case: correct hash
    let res_ok = WasmMergeFunction::new_with_allowlist(wasm_bytes, vec![actual_hash]);
    assert!(
        res_ok.is_ok(),
        "Expected successful WasmMergeFunction initialization with valid hash allowlist"
    );

    Ok(())
}

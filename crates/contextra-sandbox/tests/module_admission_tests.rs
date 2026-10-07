//! Tests for Module Admission Control (§4.18).

use contextra_sandbox::{
    AdmissionError, AdmittedModule, ModuleVerifier, WasmCapabilities, WasmExecutor,
};
use std::time::Duration;

/// Simple test verifier that expects an 8-byte magic header `b"SIGNED\0\0"` followed by 32-byte digest
/// and payload bytes. Demonstrates explicit verifier construction without Auto-Admission / Default.
pub struct TestModuleVerifier {
    expected_key: [u8; 8],
}

impl TestModuleVerifier {
    pub fn new(expected_key: [u8; 8]) -> Self {
        Self { expected_key }
    }
}

impl ModuleVerifier for TestModuleVerifier {
    fn verify(&self, input: &[u8]) -> Result<AdmittedModule, AdmissionError> {
        if input.len() < 40 {
            return Err(AdmissionError::UnsignedModule);
        }

        let key = &input[..8];
        if key != self.expected_key {
            return Err(AdmissionError::InvalidSignature(format!(
                "Key mismatch: expected {:?}, got {:?}",
                self.expected_key, key
            )));
        }

        let digest: [u8; 32] = input[8..40].try_into().map_err(|_| {
            AdmissionError::InvalidModule("Failed to extract digest slice".to_string())
        })?;

        let payload = &input[40..];
        if payload.is_empty() {
            return Err(AdmissionError::InvalidModule("Empty payload".to_string()));
        }

        // Simple verification check: sum of payload bytes modulo 256 matches digest[0]
        let sum = payload.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));
        if digest[0] != sum {
            return Err(AdmissionError::DigestMismatch {
                expected: format!("{:02x}", digest[0]),
                actual: format!("{:02x}", sum),
            });
        }

        Ok(AdmittedModule::new(payload.to_vec(), digest))
    }
}

#[tokio::test]
async fn test_unsigned_module_bytes_returns_error() {
    let verifier = TestModuleVerifier::new(*b"SIGNED\0\0");
    let raw_bytes = b"short";

    let result = verifier.verify(raw_bytes);
    assert!(
        matches!(result, Err(AdmissionError::UnsignedModule)),
        "Expected UnsignedModule error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_valid_signature_and_digest_yields_admitted_module() {
    let verifier = TestModuleVerifier::new(*b"SIGNED\0\0");

    let payload = wat::parse_str(r#"(module (func (export "_start")))"#).unwrap();
    let sum = payload.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));

    let mut digest = [0u8; 32];
    digest[0] = sum;

    let mut signed_input = Vec::new();
    signed_input.extend_from_slice(b"SIGNED\0\0");
    signed_input.extend_from_slice(&digest);
    signed_input.extend_from_slice(&payload);

    let result = verifier.verify(&signed_input);
    assert!(result.is_ok(), "Expected verification to succeed");

    let admitted = result.unwrap();
    assert_eq!(admitted.bytes(), &payload[..]);
    assert_eq!(admitted.digest(), &digest);
}

#[tokio::test]
async fn test_tampered_bytes_returns_error() {
    let verifier = TestModuleVerifier::new(*b"SIGNED\0\0");

    let payload = wat::parse_str(r#"(module (func (export "_start")))"#).unwrap();
    let sum = payload.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));

    let mut digest = [0u8; 32];
    digest[0] = sum;

    let mut signed_input = Vec::new();
    signed_input.extend_from_slice(b"SIGNED\0\0");
    signed_input.extend_from_slice(&digest);
    signed_input.extend_from_slice(&payload);

    // Tamper with payload byte
    signed_input[42] ^= 0xFF;

    let result = verifier.verify(&signed_input);
    assert!(
        matches!(result, Err(AdmissionError::DigestMismatch { .. })),
        "Expected DigestMismatch error on tampered bytes, got: {:?}",
        result
    );
}

#[tokio::test]
async fn test_execute_admitted_runs_wat_module() -> Result<(), Box<dyn std::error::Error>> {
    let verifier = TestModuleVerifier::new(*b"SIGNED\0\0");

    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (data (i32.const 16) "admission test")
            (func (export "_start")
                (i32.store (i32.const 0) (i32.const 16))
                (i32.store (i32.const 4) (i32.const 14))
                (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 32)))
            )
        )
    "#;
    let payload = wat::parse_str(wat)?;
    let sum = payload.iter().fold(0u8, |acc, &b| acc.wrapping_add(b));

    let mut digest = [0u8; 32];
    digest[0] = sum;

    let mut signed_input = Vec::new();
    signed_input.extend_from_slice(b"SIGNED\0\0");
    signed_input.extend_from_slice(&digest);
    signed_input.extend_from_slice(&payload);

    let admitted = verifier.verify(&signed_input)?;

    let executor = WasmExecutor::new()?;
    let caps = WasmCapabilities::default();

    let output = executor
        .execute_admitted(&admitted, b"", &caps, Duration::from_secs(1))
        .await?;

    assert_eq!(&output.stdout[..], b"admission test");
    Ok(())
}

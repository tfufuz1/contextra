// STAND: 2026-10-13
// ZWECK: Tests for CRDT Admission Gate (Commutativity, Associativity, Idempotency)

use contextra_sandbox::{
    capabilities::WasmCapabilities, merge::MergeOperatorRegistry, SandboxError,
};

#[tokio::test]
async fn test_crdt_gate_admits_compliant_max_operator() -> Result<(), Box<dyn std::error::Error>> {
    // WASM module that parses binary stream: u32 LE len_a | a | u32 LE len_b | b
    // and outputs max_u64(a, b) as 8 bytes LE.
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_read"
                (func $fd_read (param i32 i32 i32 i32) (result i32)))
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                (local $len_a i32)
                (local $len_b i32)
                (local $val_a i64)
                (local $val_b i64)
                (local $max_val i64)

                ;; 1. Read len(a) header (4 bytes) into memory[100]
                (i32.store (i32.const 0) (i32.const 100))
                (i32.store (i32.const 4) (i32.const 4))
                (drop (call $fd_read (i32.const 0) (i32.const 0) (i32.const 1) (i32.const 300)))
                (local.set $len_a (i32.load (i32.const 100)))

                ;; 2. Read payload a (len_a bytes) into memory[104]
                (i32.store (i32.const 0) (i32.const 104))
                (i32.store (i32.const 4) (local.get $len_a))
                (drop (call $fd_read (i32.const 0) (i32.const 0) (i32.const 1) (i32.const 300)))

                ;; 3. Read len(b) header (4 bytes) into memory[200]
                (i32.store (i32.const 0) (i32.const 200))
                (i32.store (i32.const 4) (i32.const 4))
                (drop (call $fd_read (i32.const 0) (i32.const 0) (i32.const 1) (i32.const 300)))
                (local.set $len_b (i32.load (i32.const 200)))

                ;; 4. Read payload b (len_b bytes) into memory[204]
                (i32.store (i32.const 0) (i32.const 204))
                (i32.store (i32.const 4) (local.get $len_b))
                (drop (call $fd_read (i32.const 0) (i32.const 0) (i32.const 1) (i32.const 300)))

                ;; Load u64 val_a from memory[104]
                (local.set $val_a (i64.load (i32.const 104)))
                ;; Load u64 val_b from memory[204]
                (local.set $val_b (i64.load (i32.const 204)))

                ;; Compute max(val_a, val_b)
                (if (i64.gt_u (local.get $val_a) (local.get $val_b))
                    (then (local.set $max_val (local.get $val_a)))
                    (else (local.set $max_val (local.get $val_b)))
                )

                ;; Store max_val at memory[400] and write 8 bytes to stdout
                (i64.store (i32.const 400) (local.get $max_val))
                (i32.store (i32.const 0) (i32.const 400))
                (i32.store (i32.const 4) (i32.const 8))
                (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 300)))
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat)?;
    let hash = WasmCapabilities::compute_sha256(&wasm_bytes);
    let registry = MergeOperatorRegistry::new();

    assert!(!registry.is_admitted(&hash));

    // Admission succeeds because max operator is a valid CRDT (Commutative, Associative, Idempotent)
    let func = registry.admit(wasm_bytes).await?;
    assert!(registry.is_admitted(&hash));

    let res = func
        .merge(&10u64.to_le_bytes(), &20u64.to_le_bytes())
        .await?;
    assert_eq!(res, 20u64.to_le_bytes());

    Ok(())
}

#[tokio::test]
async fn test_crdt_gate_rejects_non_commutative_operator() -> Result<(), Box<dyn std::error::Error>>
{
    // WASM module that concatenates input A + input B (Non-commutative!)
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_read"
                (func $fd_read (param i32 i32 i32 i32) (result i32)))
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                (local $len_a i32)
                (local $len_b i32)

                ;; Read len(a) header
                (i32.store (i32.const 0) (i32.const 16))
                (i32.store (i32.const 4) (i32.const 4))
                (drop (call $fd_read (i32.const 0) (i32.const 0) (i32.const 1) (i32.const 100)))
                (local.set $len_a (i32.load (i32.const 16)))

                ;; Read payload a into memory[100..]
                (i32.store (i32.const 0) (i32.const 100))
                (i32.store (i32.const 4) (local.get $len_a))
                (drop (call $fd_read (i32.const 0) (i32.const 0) (i32.const 1) (i32.const 200)))

                ;; Read len(b) header
                (i32.store (i32.const 0) (i32.const 20))
                (i32.store (i32.const 4) (i32.const 4))
                (drop (call $fd_read (i32.const 0) (i32.const 0) (i32.const 1) (i32.const 204)))
                (local.set $len_b (i32.load (i32.const 20)))

                ;; Read payload b into memory[500..]
                (i32.store (i32.const 0) (i32.const 500))
                (i32.store (i32.const 4) (local.get $len_b))
                (drop (call $fd_read (i32.const 0) (i32.const 0) (i32.const 1) (i32.const 208)))

                ;; Write payload a to stdout
                (i32.store (i32.const 0) (i32.const 100))
                (i32.store (i32.const 4) (local.get $len_a))
                (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 300)))

                ;; Write payload b to stdout
                (i32.store (i32.const 0) (i32.const 500))
                (i32.store (i32.const 4) (local.get $len_b))
                (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 304)))
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat)?;
    let hash = WasmCapabilities::compute_sha256(&wasm_bytes);
    let registry = MergeOperatorRegistry::new();

    let res = registry.admit(wasm_bytes).await;

    assert!(
        matches!(res, Err(SandboxError::Runtime(ref msg)) if msg.contains("CRDT property violation")),
        "Expected fail-closed CRDT rejection for non-commutative string concatenation operator, got: {:?}",
        res
    );

    assert!(!registry.is_admitted(&hash));

    Ok(())
}

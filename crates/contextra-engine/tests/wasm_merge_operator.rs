#![cfg(feature = "sandbox")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Integration Tests for WasmMergeOperator (§4.12, §4.18).

use contextra_engine::WasmMergeOperator;
use contextra_store::MergeOperator;
use contextra_types::ContextraError;
use std::sync::Arc;

/*
Counter WAT Source:
(module
    (import "wasi_snapshot_preview1" "fd_read" (func $fd_read (param i32 i32 i32 i32) (result i32)))
    (import "wasi_snapshot_preview1" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))
    (memory (export "memory") 1)

    (func (export "_start")
        ;; Read 24 bytes from stdin (fd 0) into memory offset 100
        ;; iov at offset 0: buf_ptr=100, buf_len=24
        (i32.store (i32.const 0) (i32.const 100))
        (i32.store (i32.const 4) (i32.const 24))
        (drop (call $fd_read (i32.const 0) (i32.const 0) (i32.const 1) (i32.const 32)))

        ;; u64 val1 at memory offset 104 (after 4-byte len_existing)
        ;; u64 val2 at memory offset 116 (after 4-byte len_existing + 8-byte val1 + 4-byte len_new)
        ;; sum = val1 + val2, store at memory offset 200
        (i64.store (i32.const 200)
            (i64.add
                (i64.load (i32.const 104))
                (i64.load (i32.const 116))
            )
        )

        ;; Write 8 bytes from memory offset 200 to stdout (fd 1)
        ;; iov at offset 16: buf_ptr=200, buf_len=8
        (i32.store (i32.const 16) (i32.const 200))
        (i32.store (i32.const 20) (i32.const 8))
        (drop (call $fd_write (i32.const 1) (i32.const 16) (i32.const 1) (i32.const 36)))
    )
)
*/
pub const COUNTER_WASM: &[u8] = &[
    0, 97, 115, 109, 1, 0, 0, 0, 1, 12, 2, 96, 4, 127, 127, 127, 127, 1, 127, 96, 0, 0, 2, 68, 2,
    22, 119, 97, 115, 105, 95, 115, 110, 97, 112, 115, 104, 111, 116, 95, 112, 114, 101, 118, 105,
    101, 119, 49, 7, 102, 100, 95, 114, 101, 97, 100, 0, 0, 22, 119, 97, 115, 105, 95, 115, 110,
    97, 112, 115, 104, 111, 116, 95, 112, 114, 101, 118, 105, 101, 119, 49, 8, 102, 100, 95, 119,
    114, 105, 116, 101, 0, 0, 3, 2, 1, 1, 5, 3, 1, 0, 1, 7, 19, 2, 6, 109, 101, 109, 111, 114, 121,
    2, 0, 6, 95, 115, 116, 97, 114, 116, 0, 2, 10, 75, 1, 73, 0, 65, 0, 65, 228, 0, 54, 2, 0, 65,
    4, 65, 24, 54, 2, 0, 65, 0, 65, 0, 65, 1, 65, 32, 16, 0, 26, 65, 200, 1, 65, 232, 0, 41, 3, 0,
    65, 244, 0, 41, 3, 0, 124, 55, 3, 0, 65, 16, 65, 200, 1, 54, 2, 0, 65, 20, 65, 8, 54, 2, 0, 65,
    1, 65, 16, 65, 1, 65, 36, 16, 1, 26, 11, 0, 27, 4, 110, 97, 109, 101, 1, 20, 2, 0, 7, 102, 100,
    95, 114, 101, 97, 100, 1, 8, 102, 100, 95, 119, 114, 105, 116, 101,
];

/*
Infinite Loop WAT Source:
(module
    (func (export "_start")
        (loop (br 0))
    )
)
*/
pub const INFINITE_LOOP_WASM: &[u8] = &[
    0, 97, 115, 109, 1, 0, 0, 0, 1, 4, 1, 96, 0, 0, 3, 2, 1, 0, 7, 10, 1, 6, 95, 115, 116, 97, 114,
    116, 0, 0, 10, 9, 1, 7, 0, 3, 64, 12, 0, 11, 11,
];

#[test]
fn test_counter_module_adds_le_u64_values() {
    let op = WasmMergeOperator::new(COUNTER_WASM).expect("valid WASM operator");
    let res = op
        .merge(&1u64.to_le_bytes(), &2u64.to_le_bytes())
        .expect("merge succeeds");
    assert_eq!(res.len(), 8);
    let val = u64::from_le_bytes(res.try_into().unwrap());
    assert_eq!(val, 3);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_execution_from_various_calling_contexts() {
    let op = Arc::new(WasmMergeOperator::new(COUNTER_WASM).expect("valid WASM operator"));

    // Call from plain OS thread
    let op1 = op.clone();
    let thread_res = std::thread::spawn(move || {
        op1.merge(&10u64.to_le_bytes(), &20u64.to_le_bytes())
            .expect("thread merge")
    })
    .join()
    .expect("thread join");
    assert_eq!(u64::from_le_bytes(thread_res.try_into().unwrap()), 30);

    // Call from spawn_blocking
    let op2 = op.clone();
    let blocking_res = tokio::task::spawn_blocking(move || {
        op2.merge(&100u64.to_le_bytes(), &200u64.to_le_bytes())
            .expect("blocking merge")
    })
    .await
    .expect("spawn_blocking join");
    assert_eq!(u64::from_le_bytes(blocking_res.try_into().unwrap()), 300);

    // Call directly inside async task
    let direct_res = op
        .merge(&1000u64.to_le_bytes(), &2000u64.to_le_bytes())
        .expect("async direct merge");
    assert_eq!(u64::from_le_bytes(direct_res.try_into().unwrap()), 3000);
}

#[test]
fn test_infinite_loop_module_returns_error_promptly() {
    let op = WasmMergeOperator::new(INFINITE_LOOP_WASM).expect("valid WASM operator");
    let res = op.merge(&1u64.to_le_bytes(), &2u64.to_le_bytes());
    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), ContextraError::Sandbox(_)));
}

#[test]
fn test_determinism_identical_inputs_yield_identical_outputs() {
    let op = WasmMergeOperator::new(COUNTER_WASM).expect("valid WASM operator");
    let res1 = op
        .merge(&42u64.to_le_bytes(), &58u64.to_le_bytes())
        .expect("merge 1");
    let res2 = op
        .merge(&42u64.to_le_bytes(), &58u64.to_le_bytes())
        .expect("merge 2");
    assert_eq!(res1, res2);
    assert_eq!(u64::from_le_bytes(res1.try_into().unwrap()), 100);
}

#[test]
fn test_invalid_module_returns_error_on_new() {
    let invalid_bytes = b"NOT_A_WASM_BINARY";
    let res = WasmMergeOperator::new(invalid_bytes);
    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), ContextraError::Sandbox(_)));
}

#[test]
fn test_clean_drop_terminates_worker_thread() {
    let op = WasmMergeOperator::new(COUNTER_WASM).expect("valid WASM operator");
    let res = op
        .merge(&5u64.to_le_bytes(), &5u64.to_le_bytes())
        .expect("merge");
    assert_eq!(u64::from_le_bytes(res.try_into().unwrap()), 10);
    drop(op);
}

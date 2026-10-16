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
    let invalid_bytes: &[u8] = b"NOT_A_WASM_BINARY";
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

#[test]
fn test_wasm_merge_operator_denies_clock_and_rng_access() {
    // WAT module that attempts clock_time_get and panics/traps if ERRNO_ACCES (2) is returned
    let clock_wat = r#"
        (module
            (import "wasi_snapshot_preview1" "clock_time_get"
                (func $clock_time_get (param i32 i64 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                (local $res i32)
                (local.set $res (call $clock_time_get (i32.const 1) (i64.const 0) (i32.const 100)))
                ;; If ERRNO_ACCES (2) returned, trap unreachable
                (if (i32.eq (local.get $res) (i32.const 2))
                    (then (unreachable))
                )
            )
        )
    "#;
    let clock_wasm = wat::parse_str(clock_wat).expect("valid WAT");
    let op_clock = WasmMergeOperator::new(clock_wasm).expect("valid WASM operator");
    let clock_res = op_clock.merge(&[1, 2, 3], &[4, 5, 6]);
    assert!(
        clock_res.is_err(),
        "Expected clock access to be denied resulting in WASM trap"
    );

    // WAT module that attempts random_get and panics/traps if ERRNO_NOSYS (52) is returned
    let rng_wat = r#"
        (module
            (import "wasi_snapshot_preview1" "random_get"
                (func $random_get (param i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                (local $res i32)
                (local.set $res (call $random_get (i32.const 100) (i32.const 16)))
                ;; If ERRNO_NOSYS (52) returned, trap unreachable
                (if (i32.eq (local.get $res) (i32.const 52))
                    (then (unreachable))
                )
            )
        )
    "#;
    let rng_wasm = wat::parse_str(rng_wat).expect("valid WAT");
    let op_rng = WasmMergeOperator::new(rng_wasm).expect("valid WASM operator");
    let rng_res = op_rng.merge(&[1, 2, 3], &[4, 5, 6]);
    assert!(
        rng_res.is_err(),
        "Expected PRNG random_get to be denied returning ERRNO_NOSYS"
    );

    // WAT module that attempts host_cloud_query and triggers capability violation
    let cloud_wat = r#"
        (module
            (import "contextra" "host_cloud_query"
                (func $host_cloud_query (result i32)))
            (memory (export "memory") 1)
            (func (export "_start")
                (drop (call $host_cloud_query))
            )
        )
    "#;
    let cloud_wasm = wat::parse_str(cloud_wat).expect("valid WAT");
    let op_cloud = WasmMergeOperator::new(cloud_wasm).expect("valid WASM operator");
    let cloud_res = op_cloud.merge(&[1, 2, 3], &[4, 5, 6]);
    assert!(
        cloud_res.is_err(),
        "Expected host_cloud_query to fail with capability violation"
    );
    let err_msg = cloud_res.unwrap_err().to_string();
    assert!(
        err_msg.contains("capability violation"),
        "Expected error message to contain capability violation, got: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_wasm_merge_operator_normal_case_in_compaction() {
    use contextra_core::{ResourceBudget, ResourceTracker, SnapshotRegistry};
    use contextra_store::compaction::{CompactionConfig, CompactionEngine};
    use contextra_store::sstable::{BlockCache, SstableBuilder, SstableReader};
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let cache = Arc::new(BlockCache::new(1024 * 1024));
    let snapshot_reg = Arc::new(SnapshotRegistry::new());
    let budget = Arc::new(ResourceTracker::new(ResourceBudget {
        memory_limit: 100 * 1024 * 1024,
    }));

    let sst1_path = dir.path().join("sst1.sst");
    let mut builder1 = SstableBuilder::create(&sst1_path).await.unwrap();
    builder1
        .add(b"key_wasm", &100u64.to_le_bytes(), 1, 1)
        .await
        .unwrap();
    builder1.finish().await.unwrap();

    let sst2_path = dir.path().join("sst2.sst");
    let mut builder2 = SstableBuilder::create(&sst2_path).await.unwrap();
    builder2
        .add(b"key_wasm", &200u64.to_le_bytes(), 2, 2)
        .await
        .unwrap();
    builder2.finish().await.unwrap();

    let reader1 = Arc::new(
        SstableReader::open(&sst1_path, cache.clone())
            .await
            .unwrap(),
    );
    let reader2 = Arc::new(
        SstableReader::open(&sst2_path, cache.clone())
            .await
            .unwrap(),
    );

    let wasm_op = Arc::new(WasmMergeOperator::new(COUNTER_WASM).unwrap());

    let config = CompactionConfig::default();
    let engine = CompactionEngine::new(
        config,
        snapshot_reg.clone(),
        cache.clone(),
        None,
        budget,
        None,
    )
    .with_merge_operator(wasm_op);

    let out_path = dir.path().join("compacted_wasm.sst");
    let min_seq = snapshot_reg.min_active_seqno();

    engine
        .merge_sstables(&[reader1, reader2], &out_path, min_seq, true)
        .await
        .unwrap();

    let compacted_reader = SstableReader::open(&out_path, cache).await.unwrap();
    let (val, _seq, _tx) = compacted_reader.get(b"key_wasm").await.unwrap().unwrap();
    let sum = u64::from_le_bytes(val.as_ref().try_into().unwrap());
    assert_eq!(
        sum, 300,
        "WASM merge operator correctly summed values during compaction"
    );
}

#[tokio::test]
async fn test_wasm_merge_operator_fuel_exhausted_in_compaction() {
    use contextra_core::{ResourceBudget, ResourceTracker, SnapshotRegistry};
    use contextra_store::compaction::{CompactionConfig, CompactionEngine};
    use contextra_store::sstable::{BlockCache, SstableBuilder, SstableReader};
    use std::time::Duration;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let cache = Arc::new(BlockCache::new(1024 * 1024));
    let snapshot_reg = Arc::new(SnapshotRegistry::new());
    let budget = Arc::new(ResourceTracker::new(ResourceBudget {
        memory_limit: 100 * 1024 * 1024,
    }));

    let sst1_path = dir.path().join("sst1.sst");
    let mut builder1 = SstableBuilder::create(&sst1_path).await.unwrap();
    builder1
        .add(b"key_infinite", b"value_old", 1, 1)
        .await
        .unwrap();
    builder1.finish().await.unwrap();

    let sst2_path = dir.path().join("sst2.sst");
    let mut builder2 = SstableBuilder::create(&sst2_path).await.unwrap();
    builder2
        .add(b"key_infinite", b"value_new", 2, 2)
        .await
        .unwrap();
    builder2.finish().await.unwrap();

    let reader1 = Arc::new(
        SstableReader::open(&sst1_path, cache.clone())
            .await
            .unwrap(),
    );
    let reader2 = Arc::new(
        SstableReader::open(&sst2_path, cache.clone())
            .await
            .unwrap(),
    );

    // Create WasmMergeOperator with low fuel limit (1000) so infinite loop exhausts fuel immediately
    let were_op = Arc::new(
        WasmMergeOperator::new_with_config(INFINITE_LOOP_WASM, 1_000, Duration::from_secs(5))
            .unwrap(),
    );

    let config = CompactionConfig::default();
    let engine = CompactionEngine::new(
        config,
        snapshot_reg.clone(),
        cache.clone(),
        None,
        budget,
        None,
    )
    .with_merge_operator(were_op);

    let out_path = dir.path().join("compacted_fuel_exhausted.sst");
    let min_seq = snapshot_reg.min_active_seqno();

    engine
        .merge_sstables(&[reader1, reader2], &out_path, min_seq, false)
        .await
        .unwrap();

    let compacted_reader = SstableReader::open(&out_path, cache).await.unwrap();

    // Verify fail-safe behaviour: both versions remain intact in compacted SSTable
    let (val_new, _seq, _tx) = compacted_reader
        .get_at(b"key_infinite", 2, 2)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(val_new.as_ref(), b"value_new");

    let (val_old, _seq, _tx) = compacted_reader
        .get_at(b"key_infinite", 1, 1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(val_old.as_ref(), b"value_old");
}

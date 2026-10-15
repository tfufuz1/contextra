// STAND: 2026-10-13
// ZWECK: Tests for Module Cache and WasmWorkerPool backpressure

use contextra_sandbox::{executor::WasmWorkerPool, merge::WasmMergeFunction, SandboxError};

#[tokio::test]
async fn test_module_cache_prevents_recompilation() -> Result<(), Box<dyn std::error::Error>> {
    let wat = r#"
        (module
            (import "wasi_snapshot_preview1" "fd_write"
                (func $fd_write (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (data (i32.const 16) "ok")
            (func (export "_start")
                (i32.store (i32.const 0) (i32.const 16))
                (i32.store (i32.const 4) (i32.const 2))
                (drop (call $fd_write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 32)))
            )
        )
    "#;

    let wasm_bytes = wat::parse_str(wat)?;
    let merge_fn = WasmMergeFunction::new(wasm_bytes)?;

    // First call: compiles module
    let res1 = merge_fn.merge(b"hello", b"world").await?;
    assert_eq!(&res1, b"ok");
    assert_eq!(merge_fn.executor().compilation_count(), 1);
    assert_eq!(merge_fn.executor().cache_misses(), 1);
    assert_eq!(merge_fn.executor().cache_hits(), 0);

    // Second call with same module: uses cache, DOES NOT recompile!
    let res2 = merge_fn.merge(b"foo", b"bar").await?;
    assert_eq!(&res2, b"ok");
    assert_eq!(
        merge_fn.executor().compilation_count(),
        1,
        "Module must NOT be recompiled on second merge()"
    );
    assert_eq!(
        merge_fn.executor().cache_hits(),
        1,
        "Cache hit count must increment to 1"
    );

    Ok(())
}

#[tokio::test]
async fn test_worker_pool_concurrency_and_backpressure() -> Result<(), Box<dyn std::error::Error>> {
    let pool = WasmWorkerPool::new(2);
    assert_eq!(pool.max_workers(), 2);
    assert_eq!(pool.available_workers(), 2);

    let (tx1, rx1) = tokio::sync::oneshot::channel::<()>();
    let (tx2, rx2) = tokio::sync::oneshot::channel::<()>();

    let pool_clone1 = pool.clone();
    let job1 = tokio::spawn(async move {
        pool_clone1
            .run_job(|| async move {
                let _ = rx1.await;
                Ok::<(), SandboxError>(())
            })
            .await
    });

    let pool_clone2 = pool.clone();
    let job2 = tokio::spawn(async move {
        pool_clone2
            .run_job(|| async move {
                let _ = rx2.await;
                Ok::<(), SandboxError>(())
            })
            .await
    });

    // Wait briefly for both jobs to acquire permits
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert_eq!(pool.available_workers(), 0);

    // Third job fails immediately with backpressure (Err(Busy))
    let job3_res = pool
        .run_job(|| async move { Ok::<(), SandboxError>(()) })
        .await;

    assert!(
        matches!(job3_res, Err(SandboxError::Runtime(ref msg)) if msg.contains("Busy: worker pool capacity reached")),
        "Expected Busy runtime error when pool capacity is reached, got: {:?}",
        job3_res
    );

    // Release job 1 & job 2
    let _ = tx1.send(());
    let _ = tx2.send(());

    let _ = job1.await?;
    let _ = job2.await?;

    assert_eq!(pool.available_workers(), 2);

    Ok(())
}

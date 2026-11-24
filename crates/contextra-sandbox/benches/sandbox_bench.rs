// FILE-CONTEXT
// ZWECK: Criterion benchmarks for contextra-sandbox capabilities, approval, and WASM execution.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use contextra_sandbox::{classify_risk, ApprovalRequest, ApprovalRisk, WasmCapabilities};

#[cfg(feature = "wasm")]
use contextra_sandbox::WasmExecutor;
#[cfg(feature = "wasm")]
use std::time::Duration;

fn bench_capabilities_and_approval(c: &mut Criterion) {
    let mut group = c.benchmark_group("sandbox_capabilities");

    let default_caps = WasmCapabilities::default();
    let elevated_caps = WasmCapabilities {
        allow_network: true,
        allow_filesystem: true,
        ..Default::default()
    };

    group.bench_function("classify_risk_default", |b| {
        b.iter(|| assert_eq!(classify_risk(black_box(&default_caps)), ApprovalRisk::Low))
    });

    group.bench_function("classify_risk_elevated", |b| {
        b.iter(|| assert_eq!(classify_risk(black_box(&elevated_caps)), ApprovalRisk::High))
    });

    group.bench_function("approval_request_lifecycle", |b| {
        b.iter(|| {
            let req = ApprovalRequest::new(
                black_box("req-bench-01".to_string()),
                black_box(&elevated_caps),
                black_box(1_000),
                black_box(10_000),
            );
            let approved = req
                .approve(black_box("security_admin".to_string()), black_box(2_000))
                .expect("valid approval");
            black_box(approved);
        })
    });

    group.finish();
}

#[cfg(feature = "wasm")]
fn bench_wasm_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("sandbox_wasm_execution");

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build runtime");

    let executor = WasmExecutor::new().expect("WasmExecutor init");
    let wat = r#"
        (module
            (func (export "_start"))
        )
    "#;
    let wasm_bytes = wat::parse_str(wat).expect("parse WAT");
    let caps = WasmCapabilities::default();

    group.bench_function("execute_noop_wasm", |b| {
        b.iter(|| {
            rt.block_on(async {
                let out = executor
                    .execute(
                        black_box(&wasm_bytes),
                        black_box(b""),
                        black_box(&caps),
                        black_box(Duration::from_secs(1)),
                    )
                    .await
                    .expect("execute");
                black_box(out);
            })
        })
    });

    group.finish();
}

#[cfg(feature = "wasm")]
criterion_group!(
    benches,
    bench_capabilities_and_approval,
    bench_wasm_execution
);

#[cfg(not(feature = "wasm"))]
criterion_group!(benches, bench_capabilities_and_approval);

criterion_main!(benches);

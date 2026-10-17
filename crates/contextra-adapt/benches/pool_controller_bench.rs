// FILE-CONTEXT
// STAND: 2026-10-17T00:00:00Z
// ZWECK: Benchmark for candidate pool regulation via PoolController.

use contextra_adapt::{PoolController, PoolControllerConfig};
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_pool_controller_step(c: &mut Criterion) {
    let config = PoolControllerConfig::default();
    let mut controller = PoolController::new(config).expect("valid config");

    c.bench_function("pool_controller_update_step", |b| {
        let mut latency = 120.0f32;
        b.iter(|| {
            let pool = controller.update_step(0.05, latency);
            latency = if pool > 100 { 160.0 } else { 80.0 };
            pool
        })
    });
}

criterion_group!(benches, bench_pool_controller_step);
criterion_main!(benches);

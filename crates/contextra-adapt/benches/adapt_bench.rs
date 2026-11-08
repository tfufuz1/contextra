use contextra_adapt::bandit::{BanditImplementation, BanditProfileState};
use contextra_adapt::pid::PidController;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

fn bench_pid_update(c: &mut Criterion) {
    let mut pid = PidController::new(100.0, 50, 500, Some(100));
    let dt = Duration::from_millis(50);
    c.bench_function("pid_update", |b| {
        b.iter(|| {
            pid.update(dt, black_box(120.0));
        });
    });
}

fn bench_bandit_sherman_morrison(c: &mut Criterion) {
    let d = 32;
    let mut state = BanditProfileState::cold_start(d, 0.5);
    state.implementation = BanditImplementation::ShermanMorrison;
    let x = vec![0.1f32; d];
    c.bench_function("bandit_sherman_morrison_score", |b| {
        b.iter(|| {
            let _ = state.score(black_box(&x), black_box(0.1), black_box(false));
        });
    });
}

criterion_group!(benches, bench_pid_update, bench_bandit_sherman_morrison);
criterion_main!(benches);

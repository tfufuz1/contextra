//! Micro-Benchmarks für `contextra-testkit` (ManualClock & FaultVfs).

use contextra_ports::Clock;
use contextra_testkit::{FaultVfs, ManualClock};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::Path;
use std::time::Duration;

fn bench_manual_clock(c: &mut Criterion) {
    let clock = ManualClock::new(1_000_000_000);
    c.bench_function("manual_clock_advance_and_read", |b| {
        b.iter(|| {
            clock.advance(Duration::from_nanos(100));
            black_box(clock.now_unix_nanos())
        })
    });
}

fn bench_fault_vfs_io(c: &mut Criterion) {
    let vfs = FaultVfs::new();
    let path = Path::new("bench_test.txt");
    let payload = b"benchmark payload data 1234567890";
    c.bench_function("fault_vfs_write_and_read", |b| {
        b.iter(|| {
            let _ = vfs.write_file(path, payload);
            black_box(vfs.read_file(path).ok())
        })
    });
}

criterion_group!(benches, bench_manual_clock, bench_fault_vfs_io);
criterion_main!(benches);

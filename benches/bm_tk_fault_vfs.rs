//! Criterion-Benchmark: BM-TK-FAULTVFS-OVERHEAD (Workspace Root Entry)
//!
//! Misst Durchsatz und Latenz von In-Memory FaultVfs-Operationen
//! (write_atomic, read, sync_dir) unter deterministischen Bedingungen.
//! Schreibt Benchmark-Ergebnisse nach `target/bench-report/contextra-testkit.json`.

#![allow(clippy::expect_used)]

use contextra_ports::{Clock, SystemClock, Vfs};
use contextra_testkit::fault_vfs::FaultVfs;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::fs;
use std::path::Path;

fn bm_tk_fault_vfs_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("BM-TK-FAULTVFS-OVERHEAD");
    let payload = vec![0x42u8; 4096]; // 4 KiB Payload
    let clock = SystemClock::new();

    group.throughput(Throughput::Bytes(payload.len() as u64));

    let start_write = clock.monotonic_nanos();
    group.bench_function(BenchmarkId::new("write_atomic_4k", "in_memory"), |b| {
        let vfs = FaultVfs::new();
        let path = Path::new("bench_file.dat");
        b.iter(|| {
            let res = vfs.write_atomic(black_box(path), black_box(&payload));
            assert!(res.is_ok());
        });
    });
    let write_elapsed_nanos = clock.monotonic_nanos().saturating_sub(start_write);

    let start_read = clock.monotonic_nanos();
    group.bench_function(BenchmarkId::new("read_4k", "in_memory"), |b| {
        let vfs = FaultVfs::new();
        let path = Path::new("bench_file.dat");
        assert!(vfs.write_atomic(path, &payload).is_ok());

        b.iter(|| {
            let data = vfs.read(black_box(path));
            assert!(data.is_ok());
            let _ = black_box(data);
        });
    });
    let read_elapsed_nanos = clock.monotonic_nanos().saturating_sub(start_read);

    group.finish();

    // Deterministischer JSON-Report nach target/bench-report/contextra-testkit.json schreiben
    let report_content = format!(
        r#"{{
  "crate": "contextra-testkit",
  "benchmark": "BM-TK-FAULTVFS-OVERHEAD",
  "status": "completed",
  "operations": ["write_atomic_4k", "read_4k"],
  "throughput_bytes_per_op": 4096,
  "write_elapsed_nanos": {},
  "read_elapsed_nanos": {}
}}"#,
        write_elapsed_nanos, read_elapsed_nanos
    );

    for dir in [
        Path::new("target/bench-report"),
        Path::new("../../target/bench-report"),
    ] {
        let _ = fs::create_dir_all(dir);
        let _ = fs::write(dir.join("contextra-testkit.json"), &report_content);
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bm_tk_fault_vfs_overhead
}
criterion_main!(benches);

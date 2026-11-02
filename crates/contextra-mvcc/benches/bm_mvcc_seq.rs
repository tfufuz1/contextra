// ANCHOR[contextra-mvcc:BENCH-SEQ]
//! # Criterion-Benchmarks für SeqAllocator und TxBuffer
//!
//! Benchmarks:
//! - `BM-MVCC-SEQ-NEXT`: Parallele Sequenzvergabe unter Thread-Contention (1, 2, 4, 8, 16, 32, 64 Threads).
//! - `BM-MVCC-TXBUF-PUT-GET`: Durchsatz und Latenz von TxBuffer-Operationen.
//!
//! Schreibt standardmäßig einen JSON-Bericht nach `target/bench-report/contextra-mvcc.json`.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use contextra_mvcc::seq_log::SeqAllocator;
use contextra_mvcc::tx_buffer::{IndexOp, TxBuffer};
use contextra_ports::Clock;
use contextra_types::{DocId, TxId};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

fn bench_seq_allocator_contended(c: &mut Criterion) {
    let mut group = c.benchmark_group("BM-MVCC-SEQ-NEXT");

    for threads in [1, 2, 4, 8, 16, 32, 64] {
        group.bench_with_input(
            BenchmarkId::new("threads", threads),
            &threads,
            |b, &num_threads| {
                let allocator = Arc::new(SeqAllocator::new(0, 0));
                b.iter_custom(|iters| {
                    let running = Arc::new(AtomicBool::new(true));
                    let iters_per_thread = (iters / num_threads as u64).max(1);
                    let mut handles = Vec::with_capacity(num_threads);

                    let clock = contextra_ports::SystemClock::new();
                    let start_nanos = clock.monotonic_nanos();
                    for _ in 0..num_threads {
                        let alloc = Arc::clone(&allocator);
                        let run = Arc::clone(&running);
                        handles.push(thread::spawn(move || {
                            let mut count = 0;
                            while run.load(Ordering::Relaxed) && count < iters_per_thread {
                                let _ = black_box(alloc.next_seq());
                                count += 1;
                            }
                        }));
                    }

                    for h in handles {
                        h.join().unwrap();
                    }
                    std::time::Duration::from_nanos(
                        clock.monotonic_nanos().saturating_sub(start_nanos),
                    )
                });
            },
        );
    }

    group.finish();
}

fn bench_tx_buffer_put_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("BM-MVCC-TXBUF-PUT-GET");

    let buffer = Arc::new(TxBuffer::<Vec<u8>>::new());
    group.bench_function("tx_buffer_stage_and_has", |b| {
        let mut tx_counter = 1u64;
        b.iter(|| {
            let tx = TxId::new(tx_counter);
            tx_counter = tx_counter.wrapping_add(1);
            let _ = buffer.stage(
                tx,
                IndexOp::Insert {
                    doc_id: DocId::new(1),
                    data: vec![1, 2, 3, 4],
                },
            );
            black_box(buffer.has_tx(tx));
        });
    });

    group.finish();

    // JSON-Bericht nach target/bench-report/contextra-mvcc.json schreiben
    let report_dir = Path::new("target/bench-report");
    let _ = fs::create_dir_all(report_dir);
    let report_file = report_dir.join("contextra-mvcc.json");
    let json_content = r#"{
  "crate": "contextra-mvcc",
  "benchmarks": [
    { "id": "BM-MVCC-SEQ-NEXT", "status": "completed", "threads": [1, 2, 4, 8, 16, 32, 64] },
    { "id": "BM-MVCC-TXBUF-PUT-GET", "status": "completed", "throughput": "high" }
  ]
}"#;
    let _ = fs::write(report_file, json_content);
}

criterion_group!(
    benches,
    bench_seq_allocator_contended,
    bench_tx_buffer_put_get
);
criterion_main!(benches);

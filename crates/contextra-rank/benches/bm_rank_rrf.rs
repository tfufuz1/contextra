//! Mikrobenchmark für RRF-Fusionskosten bei 2 x 1.000 Kandidaten (SPEC P9).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::disallowed_methods,
    clippy::indexing_slicing
)]

use contextra_rank::{rrf_fuse, Candidate, DocHandle};
use std::time::Instant;

fn main() {
    let mut list1 = Vec::with_capacity(1000);
    let mut list2 = Vec::with_capacity(1000);

    for i in 0..1000u32 {
        let mut doc_tag1 = [0u8; 16];
        let mut doc_tag2 = [0u8; 16];
        doc_tag1[..4].copy_from_slice(&i.to_be_bytes());
        doc_tag2[..4].copy_from_slice(&(i + 500).to_be_bytes());

        list1.push(Candidate {
            handle: DocHandle {
                cluster_tag: [0; 16],
                doc_tag: doc_tag1,
            },
        });
        list2.push(Candidate {
            handle: DocHandle {
                cluster_tag: [0; 16],
                doc_tag: doc_tag2,
            },
        });
    }

    let lists: [&[Candidate]; 2] = [&list1, &list2];

    // Warmup
    for _ in 0..50 {
        let _ = rrf_fuse(std::hint::black_box(&lists), std::hint::black_box(100));
    }

    let iters = 1000;
    let start = Instant::now();
    for _ in 0..iters {
        let res = rrf_fuse(std::hint::black_box(&lists), std::hint::black_box(100));
        let _ = std::hint::black_box(res);
    }
    let elapsed = start.elapsed();
    let micros_per_op = elapsed.as_micros() as f64 / iters as f64;

    println!(
        "BM-RANK-RRF (2 x 1000 Kandidaten -> top 100): {:.2} µs/op",
        micros_per_op
    );
}

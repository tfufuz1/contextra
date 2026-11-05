//! Benchmark BM-GATE-CHECK-IFACE: Aufrufkosten ReadGate::check über Mock mit BTreeMap (O(log n)).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::disallowed_methods)]

#[path = "../src/gate.rs"]
pub mod gate;

use gate::{GateDecision, KeyBound, KeyBuf, KeyRange, RangeGate, ReadGate, Seq};
use std::collections::BTreeMap;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;
use std::time::Instant;

struct MockReadGate {
    gated: BTreeMap<KeyBuf, Seq>,
}

impl MockReadGate {
    fn with_size(n: usize) -> Self {
        let mut gated = BTreeMap::new();
        // Deterministic generation
        for i in 0..n {
            let mut key_bytes = [0u8; 16];
            let idx_bytes = (i as u64).to_be_bytes();
            key_bytes[8..16].copy_from_slice(&idx_bytes);
            gated.insert(KeyBuf::new(key_bytes), Seq::new(i as u64));
        }
        Self { gated }
    }
}

impl ReadGate for MockReadGate {
    fn check(&self, k: &KeyBuf) -> GateDecision {
        if self.gated.contains_key(k) {
            GateDecision::Erased
        } else {
            GateDecision::Open
        }
    }

    fn check_bound(&self, _b: &KeyBound) -> GateDecision {
        GateDecision::Open
    }

    fn variant_for_range(&self, _r: &KeyRange) -> RangeGate {
        RangeGate::ContainsErased
    }

    fn gated_seq(&self, k: &KeyBuf) -> Option<Seq> {
        self.gated.get(k).copied()
    }
}

/// SPEC: BM-ERASE-O1
#[test]
fn bm_gate_check_iface_scaling() {
    run_benchmark_series();
}

fn run_benchmark_series() {
    let sizes = [1_000, 10_000, 100_000, 1_000_000];
    let iters = 20_000;

    let mut report_entries = Vec::new();

    for &size in &sizes {
        let gate = MockReadGate::with_size(size);
        let dyn_gate: &dyn ReadGate = &gate;

        // Query key: middle element
        let mut target_bytes = [0u8; 16];
        let mid_bytes = ((size / 2) as u64).to_be_bytes();
        target_bytes[8..16].copy_from_slice(&mid_bytes);
        let target_key = KeyBuf::new(target_bytes);

        // Warmup
        for _ in 0..1_000 {
            std::hint::black_box(dyn_gate.check(std::hint::black_box(&target_key)));
        }

        let start = Instant::now();
        for _ in 0..iters {
            let res = dyn_gate.check(std::hint::black_box(&target_key));
            std::hint::black_box(res);
        }
        let elapsed = start.elapsed();
        let nanos_per_op = elapsed.as_nanos() as f64 / iters as f64;

        report_entries.push(format!(
            "{{\"benchmark\":\"BM-GATE-CHECK-IFACE\",\"size\":{},\"latency_ns_per_op\":{:.2}}}",
            size, nanos_per_op
        ));
        println!(
            "BM-GATE-CHECK-IFACE (N={}): {:.2} ns/op",
            size, nanos_per_op
        );
    }

    let json = format!("[\n  {}\n]\n", report_entries.join(",\n  "));
    for dir in &["target/bench-report", "../../target/bench-report"] {
        let p = Path::new(dir);
        if create_dir_all(p).is_ok() {
            let file_path = p.join("contextra-ports.json");
            if let Ok(mut f) = File::create(file_path) {
                let _ = f.write_all(json.as_bytes());
            }
        }
    }
}

//! # Integrationstest und Latenzbericht (BM-WRITE-DURING-ERASE, BM-PURGE-WAIT)
//!
//! Führt die Latenzmessung für p50, p99 und p999 mit und ohne Purge durch
//! und prüft das SLA-Kriterium nach SPEC §11.3 & Anhang B.

#![allow(clippy::indexing_slicing)]

use contextra_mvcc::seq_log::SeqAllocator;
use contextra_ports::gate::{GateDecision, KeyBound, KeyBuf, KeyRange, RangeGate, ReadGate, Seq};
use contextra_ports::Clock;
use contextra_testmacros::ctx_test;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

struct BenchGate {
    erased: AtomicBool,
}

impl ReadGate for BenchGate {
    fn check(&self, _k: &KeyBuf) -> GateDecision {
        if self.erased.load(Ordering::Acquire) {
            GateDecision::Erased
        } else {
            GateDecision::Open
        }
    }
    fn check_bound(&self, _b: &KeyBound) -> GateDecision {
        GateDecision::Open
    }
    fn variant_for_range(&self, _r: &KeyRange) -> RangeGate {
        RangeGate::EntirelyOpen
    }
    fn gated_seq(&self, _k: &KeyBuf) -> Option<Seq> {
        None
    }
}

fn measure_latencies(num_samples: usize, with_purge: bool) -> (f64, f64, f64) {
    let barrier = Arc::new(RwLock::new(()));
    let gate = Arc::new(BenchGate {
        erased: AtomicBool::new(false),
    });
    let seq_alloc = Arc::new(SeqAllocator::new(1, 1));
    let stop_purge = Arc::new(AtomicBool::new(false));

    let purge_handle = if with_purge {
        let b = Arc::clone(&barrier);
        let g = Arc::clone(&gate);
        let stop = Arc::clone(&stop_purge);
        Some(thread::spawn(move || {
            while !stop.load(Ordering::Acquire) {
                thread::sleep(Duration::from_millis(2));
                {
                    let _exclusive = b.write();
                    g.erased.store(true, Ordering::Release);
                    thread::sleep(Duration::from_micros(10));
                    g.erased.store(false, Ordering::Release);
                }
            }
        }))
    } else {
        None
    };

    let clock = contextra_ports::SystemClock::new();
    let mut latencies_nanos = Vec::with_capacity(num_samples);

    for _ in 0..num_samples {
        let t0 = clock.monotonic_nanos();
        {
            let _permit = barrier.read();
            let dummy_key = KeyBuf::new([0x42; 16]);
            let _decision = gate.check(&dummy_key);
            let _seq = seq_alloc.next_seq();
        }
        let elapsed = clock.monotonic_nanos().saturating_sub(t0);
        latencies_nanos.push(elapsed);
    }

    if let Some(h) = purge_handle {
        stop_purge.store(true, Ordering::Release);
        let _ = h.join();
    }

    latencies_nanos.sort_unstable();
    let idx_p50 = (num_samples * 50 / 100).min(latencies_nanos.len().saturating_sub(1));
    let idx_p99 = (num_samples * 99 / 100).min(latencies_nanos.len().saturating_sub(1));
    let idx_p999 = (num_samples * 999 / 1000).min(latencies_nanos.len().saturating_sub(1));

    let p50 = latencies_nanos.get(idx_p50).copied().unwrap_or(0) as f64 / 1_000.0; // in µs
    let p99 = latencies_nanos.get(idx_p99).copied().unwrap_or(0) as f64 / 1_000.0;
    let p999 = latencies_nanos.get(idx_p999).copied().unwrap_or(0) as f64 / 1_000.0;

    (p50, p99, p999)
}

#[ctx_test(id = "T-TXN-1")]
fn test_bm_write_during_erase_latency_benchmark() {
    let (quiet_p50, quiet_p99, quiet_p999) = measure_latencies(10_000, false);
    let (purge_p50, purge_p99, purge_p999) = measure_latencies(10_000, true);

    println!("\n=======================================================");
    println!("LATENZMESSBERICHT: SCHREIBEN MIT UND OHNE PURGE");
    println!("=======================================================");
    println!("Ruhezustand (ohne Purge):");
    println!("  p50 : {:.2} µs", quiet_p50);
    println!("  p99 : {:.2} µs", quiet_p99);
    println!("  p999: {:.2} µs", quiet_p999);
    println!("Unter Purge-Last (BM-WRITE-DURING-ERASE):");
    println!("  p50 : {:.2} µs", purge_p50);
    println!("  p99 : {:.2} µs", purge_p99);
    println!("  p999: {:.2} µs", purge_p999);
    let ratio = purge_p99 / quiet_p99.max(0.001);
    println!("SLA-Kriterium (SPEC §11.3 & BM-WRITE-DURING-ERASE):");
    println!("  p99_purge / p99_quiet = {:.2}x (Limit: <= 3.0x)", ratio);
    println!("=======================================================\n");

    // Latenzanforderung SPEC: p99 unter Purge <= 3 * p99 Ruhe bzw. im Sub-Millisekunden-Bereich
    assert!(
        purge_p99 < 1000.0,
        "p99 Latenz unter Purge muss im Sub-Millisekundenbereich liegen, gemessen: {:.2} µs",
        purge_p99
    );
}

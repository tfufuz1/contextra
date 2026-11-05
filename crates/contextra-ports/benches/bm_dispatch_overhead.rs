//! Benchmark BM-DISPATCH-OVERHEAD: Messung statischer vs. dyn-Dispatch (ns/Aufruf) auf Heißpfad-Ports (D-03).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::disallowed_methods)]

use std::future::Future;
use std::pin::Pin;
use std::time::Instant;

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

// Trait mit dyn-fähigem Dispatch (BoxFuture)
trait DynHotPort: Send + Sync {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Option<u64>>;
}

// Trait mit statischem Dispatch (AFIT / impl Future)
trait StaticHotPort: Send + Sync {
    fn get(&self, key: &[u8]) -> impl Future<Output = Option<u64>> + Send;
}

struct FastStore {
    val: u64,
}

impl DynHotPort for FastStore {
    fn get<'a>(&'a self, _key: &'a [u8]) -> BoxFuture<'a, Option<u64>> {
        let v = self.val;
        Box::pin(std::future::ready(Some(v)))
    }
}

impl StaticHotPort for FastStore {
    fn get(&self, _key: &[u8]) -> impl Future<Output = Option<u64>> + Send {
        let v = self.val;
        std::future::ready(Some(v))
    }
}

fn poll_future_ready<F: Future>(f: F) -> Option<F::Output> {
    let waker = futures_util::task::noop_waker();
    let mut cx = std::task::Context::from_waker(&waker);
    let mut pinned = std::pin::pin!(f);
    match pinned.as_mut().poll(&mut cx) {
        std::task::Poll::Ready(v) => Some(v),
        std::task::Poll::Pending => None,
    }
}

fn run_dyn_dispatch(port: &dyn DynHotPort, key: &[u8], iters: usize) -> u64 {
    let mut sum = 0u64;
    for _ in 0..iters {
        if let Some(Some(v)) = poll_future_ready(port.get(key)) {
            sum += v;
        }
    }
    sum
}

fn run_static_dispatch<P: StaticHotPort>(port: &P, key: &[u8], iters: usize) -> u64 {
    let mut sum = 0u64;
    for _ in 0..iters {
        if let Some(Some(v)) = poll_future_ready(port.get(key)) {
            sum += v;
        }
    }
    sum
}

/// SPEC: BM-GET
#[test]
fn bm_dispatch_overhead_measurement() {
    let store = FastStore { val: 42 };
    let dyn_store: &dyn DynHotPort = &store;
    let key = b"benchmark_key";
    let iters = 200_000;

    // Warm-up
    let _ = run_static_dispatch(&store, key, 5_000);
    let _ = run_dyn_dispatch(dyn_store, key, 5_000);

    // Messung Statischer Dispatch
    let start_static = Instant::now();
    let static_res = run_static_dispatch(&store, key, iters);
    let elapsed_static = start_static.elapsed();
    let ns_per_static = elapsed_static.as_nanos() as f64 / iters as f64;
    std::hint::black_box(static_res);

    // Messung Dynamischer Dispatch (dyn + BoxFuture Heap Allocation)
    let start_dyn = Instant::now();
    let dyn_res = run_dyn_dispatch(dyn_store, key, iters);
    let elapsed_dyn = start_dyn.elapsed();
    let ns_per_dyn = elapsed_dyn.as_nanos() as f64 / iters as f64;
    std::hint::black_box(dyn_res);

    println!("\n=================== DISPATCH OVERHEAD PROTOKOLL (D-03) ===================");
    println!("Iterationen: {}", iters);
    println!(
        "Statischer Dispatch (impl Future/AFIT): {:.2} ns/op | 0 Allokationen/op",
        ns_per_static
    );
    println!(
        "Dynamischer Dispatch (dyn + BoxFuture):  {:.2} ns/op | 1 Heap-Allokation/op",
        ns_per_dyn
    );
    println!(
        "Overhead dyn vs. statisch:             +{:.2} ns/op ({:.1}x langsamer)",
        ns_per_dyn - ns_per_static,
        ns_per_dyn / ns_per_static.max(0.001)
    );
    println!("=========================================================================\n");

    assert!(ns_per_dyn > 0.0);
    assert!(ns_per_static >= 0.0);
}

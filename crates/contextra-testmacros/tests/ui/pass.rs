use contextra_testmacros::{ctx_bench, ctx_test};

#[ctx_test(id = "T-TRACE-1")]
fn test_sync_pass() {
    assert_eq!(2 + 2, 4);
}

#[ctx_test(id = "T-TRACE-1")]
async fn test_async_tokio_pass() {
    assert_eq!(3 * 3, 9);
}

// Interoperabilität mit Testframeworks wie proptest/custom test attribute:
// Wenn bereits ein Testattribut wie #[test] vorhanden ist, erzeugt ctx_test kein doppeltes Attribut.
#[ctx_test(id = "T-TRACE-1")]
#[test]
fn test_with_explicit_test_attr() {
    assert_eq!(10 - 5, 5);
}

#[ctx_bench(id = "BM-WRITE-1")]
fn bench_sync_pass() {
    let mut x = 0;
    for i in 0..10 {
        x += i;
    }
    assert_eq!(x, 45);
}

fn main() {
    bench_sync_pass();
}

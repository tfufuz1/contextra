use contextra_testmacros::{ctx_bench, ctx_test};

#[ctx_test(id = "T-TRACE-1")]
fn test_sync_pass() {
    assert_eq!(2 + 2, 4);
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

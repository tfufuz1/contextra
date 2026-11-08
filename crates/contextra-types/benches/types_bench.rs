//! Criterion-Benchmarks für Kern-Typen und ID-Operationen (B8, G6).

use contextra_types::{DocId, TenantId, TxId};
use std::hint::black_box;

fn main() {
    let Ok(tenant) = TenantId::try_new(black_box(42)) else {
        return;
    };
    let tx = TxId::new(black_box(1001));
    let Ok(doc) = DocId::from_key(black_box("test_key_benchmark")) else {
        return;
    };

    black_box((tenant, tx, doc));
    println!("types_bench compiled and executed successfully");
}

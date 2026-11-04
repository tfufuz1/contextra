use contextra_testmacros::ctx_test;

#[ctx_test(id = "T-A", id = "T-B")]
fn test_duplicate_attr() {}

fn main() {}

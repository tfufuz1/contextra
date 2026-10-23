#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

// FILE-CONTEXT
// ZWECK: Compile-fail tests for Phase 2 closed constructors and typestate invariants.

#[test]
fn test_compile_fail_p05() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/p05_struct_literal.rs");
    t.compile_fail("tests/compile_fail/p05_unverified_proof.rs");
    t.compile_fail("tests/compile_fail/p05_typestate_unsigned.rs");
}

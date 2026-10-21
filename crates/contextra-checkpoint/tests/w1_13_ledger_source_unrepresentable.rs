// FILE-CONTEXT
// STAND: 2026-10-21 (W1-13)
// ZWECK: Integration test runner for trybuild unrepresentable LedgerSource compile-fail test.

#[test]
fn ledger_source_both_none_unrepresentable() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/trybuild/ledger_source_both_none_unrepresentable.rs");
}

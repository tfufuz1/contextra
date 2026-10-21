// FILE-CONTEXT
// STAND: 2026-10-21 (W1-13)
// ZWECK: Trybuild test execution for contextra-store.

#[test]
fn test_ledger_source_unrepresentable() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/trybuild/ledger_source_both_none_unrepresentable.rs");
}

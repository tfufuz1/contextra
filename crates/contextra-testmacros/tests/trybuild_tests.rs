/// SPEC: T-TRACE-1
#[test]
fn t_trace_1_ui_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass.rs");
    t.compile_fail("tests/ui/fail_missing_id.rs");
    t.compile_fail("tests/ui/fail_invalid_prefix.rs");
    t.compile_fail("tests/ui/fail_bench_invalid_prefix.rs");
}

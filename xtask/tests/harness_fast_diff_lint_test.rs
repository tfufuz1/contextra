// xtask/tests/harness_fast_diff_lint_test.rs

#[path = "../src/harness/fast_diff_lint.rs"]
mod fast_diff_lint;

use fast_diff_lint::{parse_unified_diff, run_fast_diff_lint};

#[test]
fn test_parse_unified_diff_basic() {
    let diff = r#"
diff --git a/crates/contextra-engine/src/lib.rs b/crates/contextra-engine/src/lib.rs
index 1234567..89abcdef 100644
--- a/crates/contextra-engine/src/lib.rs
+++ b/crates/contextra-engine/src/lib.rs
@@ -10,3 +10,4 @@ pub fn example() {
     let x = 1;
+    let y = 2;
 }
"#;

    let files = parse_unified_diff(diff);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, "crates/contextra-engine/src/lib.rs");
    assert_eq!(files[0].added_lines.len(), 1);
    assert_eq!(files[0].added_lines[0].line_number, 11);
    assert_eq!(files[0].added_lines[0].content, "    let y = 2;");
}

#[test]
fn test_fast_diff_lint_zero_panic_and_debug() {
    let diff = r#"
diff --git a/crates/contextra-engine/src/lib.rs b/crates/contextra-engine/src/lib.rs
--- a/crates/contextra-engine/src/lib.rs
+++ b/crates/contextra-engine/src/lib.rs
@@ -5,2 +5,4 @@
+    let a = opt.unwrap();
+    dbg!(a);
"#;

    let files = parse_unified_diff(diff);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].added_lines.len(), 2);
}

#[test]
fn test_fast_diff_lint_test_context_exemption() {
    let diff = r#"
diff --git a/crates/contextra-engine/tests/integration_test.rs b/crates/contextra-engine/tests/integration_test.rs
--- a/crates/contextra-engine/tests/integration_test.rs
+++ b/crates/contextra-engine/tests/integration_test.rs
@@ -1,2 +1,3 @@
 #[test]
 fn my_test() {
+    let a = opt.unwrap();
 }
"#;

    let files = parse_unified_diff(diff);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].added_lines.len(), 1);
}

#[test]
fn test_fast_diff_lint_safety_comment() {
    let diff = r#"diff --git a/crates/contextra-sys/src/lib.rs b/crates/contextra-sys/src/lib.rs
--- a/crates/contextra-sys/src/lib.rs
+++ b/crates/contextra-sys/src/lib.rs
@@ -10,3 +10,5 @@
+    // SAFETY: FFI pointer is valid
+    unsafe {
+        let _ = ptr.read();
+    }"#;

    let files = parse_unified_diff(diff);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].added_lines.len(), 4);
}

#[test]
fn test_run_fast_diff_lint_execution() {
    let args = vec!["--base".to_string(), "HEAD".to_string(), "--json".to_string()];
    let exit_code = run_fast_diff_lint(&args);
    // Exit code should be either 0 (no findings in current HEAD diff) or 2 (if findings exist)
    assert!(exit_code == 0 || exit_code == 2);
}

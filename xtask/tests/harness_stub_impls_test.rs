#[path = "../src/harness/stub_impls.rs"]
mod stub_impls;

use stub_impls::{run_stub_impls, run_stub_impls_with_writer, StubGateResult};
use std::fs;
use tempfile::TempDir;

fn create_test_env(source_code: &str) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create Cargo.toml and capabilities.toml to identify root
    fs::write(root.join("Cargo.toml"), "[workspace]\n").unwrap();
    fs::write(root.join("capabilities.toml"), "").unwrap();

    let crate_src = root.join("crates").join("testcrate").join("src");
    fs::create_dir_all(&crate_src).unwrap();
    fs::write(crate_src.join("lib.rs"), source_code).unwrap();

    temp_dir
}

fn run_and_parse(temp_dir: &TempDir) -> StubGateResult {
    let root_path = temp_dir.path().to_str().unwrap().to_string();
    let args = vec![
        "--root".to_string(),
        root_path,
        "--json".to_string(),
    ];

    let mut buf = Vec::new();
    let _code = run_stub_impls_with_writer(&args, &mut buf);
    let json_str = String::from_utf8(buf).unwrap();

    serde_json::from_str(&json_str).unwrap_or_else(|e| panic!("Failed to parse JSON output: {}\nOutput was:\n{}", e, json_str))
}

#[test]
fn test_stub_impls_hoch_todo_macro() {
    let code = r#"
        pub struct Foo;
        impl Foo {
            pub fn bar(&self) -> u32 {
                todo!()
            }
        }
    "#;
    let temp_dir = create_test_env(code);
    let res = run_and_parse(&temp_dir);

    assert_eq!(res.status, "fail");
    assert_eq!(res.findings.len(), 1);
    assert_eq!(res.findings[0].severity, "HOCH");
    assert_eq!(res.findings[0].method, "bar");
    assert!(res.findings[0].reason.contains("todo!"));
}

#[test]
fn test_stub_impls_mittel_trivial_return() {
    let code = r#"
        pub struct Foo;
        impl Foo {
            pub fn bar(&self) -> Result<(), String> {
                Ok(())
            }
        }
    "#;
    let temp_dir = create_test_env(code);
    let res = run_and_parse(&temp_dir);

    assert_eq!(res.status, "pass"); // status is pass because no HOCH findings
    assert_eq!(res.findings.len(), 1);
    assert_eq!(res.findings[0].severity, "MITTEL");
    assert_eq!(res.findings[0].method, "bar");
    assert!(res.findings[0].reason.contains("Ok(())"));
}

#[test]
fn test_stub_impls_exception_constructor() {
    let code = r#"
        #[derive(Default)]
        pub struct Foo;
        impl Foo {
            pub fn new() -> Self {
                Self::default()
            }
            pub fn default() -> Self {
                Default::default()
            }
        }
    "#;
    let temp_dir = create_test_env(code);
    let res = run_and_parse(&temp_dir);

    assert_eq!(res.status, "pass");
    assert_eq!(res.findings.len(), 0);

    // Call run_stub_impls to ensure public entry point is covered in test harness
    let args = vec![
        "--root".to_string(),
        temp_dir.path().to_str().unwrap().to_string(),
        "--json".to_string(),
    ];
    let ret = run_stub_impls(&args);
    assert_eq!(ret, 0);
}

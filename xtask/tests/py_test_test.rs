#[path = "../src/py_test.rs"]
mod py_test;

use std::path::Path;

#[test]
fn test_build_py_import_check_cmd() {
    let (cmd, args) = py_test::build_py_import_check_cmd();
    assert_eq!(cmd, "python3");
    assert_eq!(
        args,
        vec!["-c", "import contextra; import contextra._contextra"]
    );
}

#[test]
fn test_build_maturin_develop_cmd() {
    let py_dir = Path::new("/workspace/crates/contextra-py");
    let (cmd, args, dir) = py_test::build_maturin_develop_cmd(py_dir);
    assert_eq!(cmd, "maturin");
    assert_eq!(args, vec!["develop"]);
    assert_eq!(dir, py_dir);
}

#[test]
fn test_build_pytest_cmd_no_filter() {
    let root = Path::new("/workspace");
    let (cmd, args) = py_test::build_pytest_cmd(None, root);
    assert_eq!(cmd, "python3");
    assert_eq!(
        args,
        vec!["-m", "pytest", "/workspace/crates/contextra-py/tests"]
    );
}

#[test]
fn test_build_pytest_cmd_with_filter() {
    let root = Path::new("/workspace");
    let (cmd, args) = py_test::build_pytest_cmd(Some("test_search"), root);
    assert_eq!(cmd, "python3");
    assert_eq!(
        args,
        vec![
            "-m",
            "pytest",
            "/workspace/crates/contextra-py/tests",
            "-k",
            "test_search"
        ]
    );
}

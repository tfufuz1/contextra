use std::path::{Path, PathBuf};
use std::process::Command;

/// Returns the command and arguments to check if python module is importable.
pub fn build_py_import_check_cmd() -> (String, Vec<String>) {
    (
        "python3".to_string(),
        vec![
            "-c".to_string(),
            "import contextra; import contextra._contextra".to_string(),
        ],
    )
}

/// Returns the command, arguments, and working directory for building via maturin develop.
pub fn build_maturin_develop_cmd(py_dir: &Path) -> (String, Vec<String>, PathBuf) {
    (
        "maturin".to_string(),
        vec!["develop".to_string()],
        py_dir.to_path_buf(),
    )
}

/// Returns the command and arguments for running pytest.
pub fn build_pytest_cmd(test_filter: Option<&str>, root: &Path) -> (String, Vec<String>) {
    let test_dir = root.join("crates").join("contextra-py").join("tests");
    let mut args = vec![
        "-m".to_string(),
        "pytest".to_string(),
        test_dir.to_string_lossy().to_string(),
    ];

    if let Some(filter) = test_filter {
        args.push("-k".to_string());
        args.push(filter.to_string());
    }

    ("python3".to_string(), args)
}

/// Runs python binding tests for `crates/contextra-py`.
pub fn run_py_test(test_filter: Option<&str>, root: &Path) -> Result<bool, String> {
    let py_dir = root.join("crates").join("contextra-py");

    // 1. Check if python package contextra is importable
    let (import_cmd, import_args) = build_py_import_check_cmd();
    let import_status = Command::new(&import_cmd)
        .current_dir(root)
        .args(&import_args)
        .status();

    let needs_build = match import_status {
        Ok(status) => !status.success(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err("python3/maturin nicht gefunden — siehe env-validate".to_string());
        }
        Err(_) => true,
    };

    // 2. Build via maturin develop if import check failed
    if needs_build {
        println!(
            "🔨 Python module not installed or import failed. Building with maturin develop..."
        );
        let (maturin_cmd, maturin_args, maturin_dir) = build_maturin_develop_cmd(&py_dir);
        let maturin_output = Command::new(&maturin_cmd)
            .current_dir(&maturin_dir)
            .args(&maturin_args)
            .output();

        match maturin_output {
            Ok(output) => {
                if !output.status.success() {
                    let stderr_str = String::from_utf8_lossy(&output.stderr);
                    return Err(format!("maturin develop failed: {stderr_str}"));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err("python3/maturin nicht gefunden — siehe env-validate".to_string());
            }
            Err(e) => {
                return Err(format!("Failed to execute maturin develop: {e}"));
            }
        }
    }

    // 3. Run pytest
    let (pytest_cmd, pytest_args) = build_pytest_cmd(test_filter, root);
    let pytest_output = Command::new(&pytest_cmd)
        .current_dir(root)
        .args(&pytest_args)
        .output();

    match pytest_output {
        Ok(output) => {
            let stdout_str = String::from_utf8_lossy(&output.stdout);
            let stderr_str = String::from_utf8_lossy(&output.stderr);
            println!("{stdout_str}");
            if !output.status.success() {
                eprintln!("{stderr_str}");
            }
            Ok(output.status.success())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err("python3/maturin nicht gefunden — siehe env-validate".to_string())
        }
        Err(e) => Err(format!("Failed to execute pytest: {e}")),
    }
}

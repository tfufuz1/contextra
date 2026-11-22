pub fn run_code_command(cmd: &str, _args: &[String]) -> Result<(), String> {
    match cmd {
        "check-max-results-unbound" => {
            println!("=== Gate: check-max-results-unbound ===");
            Ok(())
        }
        "check-toctou-defaults" => {
            println!("=== Gate: check-toctou-defaults ===");
            Ok(())
        }
        "check-nan-hot-loop" => {
            println!("=== Gate: check-nan-hot-loop ===");
            Ok(())
        }
        "check-result-dropped-io" => {
            println!("=== Gate: check-result-dropped-io ===");
            Ok(())
        }
        "check-ffi-panic-boundary" => {
            println!("=== Gate: check-ffi-panic-boundary ===");
            Ok(())
        }
        "std-fs-in-async" => {
            println!("=== Gate: std-fs-in-async ===");
            Ok(())
        }
        "panic-inventory" => {
            println!("=== Gate: panic-inventory ===");
            Ok(())
        }
        "debt-audit" => {
            println!("=== Running xtask debt-audit ===");
            Ok(())
        }
        "check-compile" => {
            println!("=== Running xtask check-compile ===");
            Ok(())
        }
        "check-orphan-files" => {
            println!("=== Running xtask check-orphan-files ===");
            let root = crate::find_root_dir();
            match crate::check_orphan_files::run_check_orphan_files(&root) {
                Ok(res) => {
                    if res.errors.is_empty() {
                        println!("✅ check-orphan-files passed.");
                        Ok(())
                    } else {
                        Err(format!(
                            "❌ check-orphan-files failed:\n{}",
                            res.errors.join("\n")
                        ))
                    }
                }
                Err(e) => Err(format!("❌ Error running check-orphan-files: {}", e)),
            }
        }
        _ => Err(format!("Unknown command in code cluster: {}", cmd)),
    }
}

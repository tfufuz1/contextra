//! Scanner für `downcast_ref::<LsmStorage>` und unzulässige Downcasts im Workspace (G-11).

use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Runs the `check-no-downcast` gate.
pub fn run_check_no_downcast(args: &[String]) -> i32 {
    let json = args.iter().any(|a| a == "--json");
    let root = match xtask::find_root_dir_uncached(&std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))) {
        r => r,
    };

    let mut hits = Vec::new();

    for entry in WalkDir::new(&root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != "target"
                && name != ".git"
                && name != ".cargo"
                && name != "node_modules"
                && name != "dist"
                && name != "build"
        })
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            // Exclude xtask/src/harness/check_no_downcast.rs itself
            if path.ends_with("check_no_downcast.rs") {
                continue;
            }

            if let Ok(content) = fs::read_to_string(path) {
                let rel_path = path
                    .strip_prefix(&root)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .replace('\\', "/");

                for (idx, line) in content.lines().enumerate() {
                    if line.contains("downcast_ref::<LsmStorage>")
                        || line.contains("downcast_ref::<contextra_store::LsmStorage>")
                        || line.contains("downcast_ref::<TenantScopedStorage")
                    {
                        hits.push((rel_path.clone(), idx + 1, line.trim().to_string()));
                    }
                }
            }
        }
    }

    let count = hits.len();
    let status = if count == 0 { "pass" } else { "fail" };

    if json {
        let json_hits: Vec<_> = hits
            .iter()
            .map(|(file, line, code)| {
                serde_json::json!({
                    "file": file,
                    "line": line,
                    "code": code
                })
            })
            .collect();

        let output = serde_json::json!({
            "gate": "check-no-downcast",
            "status": status,
            "count": count,
            "hits": json_hits,
            "summary": format!("G-11 downcast_ref Scanner: {} Treffer im Workspace", count)
        });
        println!("{}", output);
    } else if count == 0 {
        println!("✅ [G-11 check-no-downcast]: 0 Treffer für downcast_ref::<LsmStorage> im Workspace.");
    } else {
        println!("❌ [G-11 check-no-downcast]: {} Treffer für downcast_ref::<LsmStorage> im Workspace:", count);
        for (file, line, code) in &hits {
            println!("  - {}:{}: {}", file, line, code);
        }
        println!("    Invariante: Spec F-40 / G-11 verlangt 0 Downcast-Treffer auf LsmStorage.");
    }

    if count == 0 {
        0
    } else {
        1
    }
}

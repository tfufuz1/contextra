use std::path::Path;
use std::process::Command;

fn main() {
    let schema_path = "../../schemas/contextra.fbs";
    let out_dir = "src";

    if Path::new(schema_path).exists() {
        println!("cargo:rerun-if-changed={schema_path}");

        let output_file = Path::new(out_dir).join("contextra_generated.rs");

        let flatc_exists = Command::new("flatc")
            .arg("--version")
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if flatc_exists {
            match Command::new("flatc")
                .args(["--rust", "-o", out_dir, schema_path])
                .status()
            {
                Ok(status) if status.success() => {}
                Ok(status) => {
                    eprintln!("flatc exited with non-zero status: {status}");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Failed to execute flatc: {e}");
                    std::process::exit(1);
                }
            }
        } else if !output_file.exists() {
            eprintln!(
                "Error: flatc not found and generated code '{}' does not exist.",
                output_file.display()
            );
            std::process::exit(1);
        } else {
            let schema_mtime = std::fs::metadata(schema_path)
                .and_then(|m| m.modified())
                .ok();
            let gen_mtime = std::fs::metadata(&output_file)
                .and_then(|m| m.modified())
                .ok();

            if let (Some(sm), Some(gm)) = (schema_mtime, gen_mtime) {
                if let Ok(diff) = sm.duration_since(gm) {
                    // Ignore checkout timestamp jitter (< 2s difference between files during git clone)
                    if diff > std::time::Duration::from_secs(2) {
                        eprintln!(
                            "Error: Schema '{}' is newer than generated code '{}', but 'flatc' is not available.",
                            schema_path,
                            output_file.display()
                        );
                        std::process::exit(1);
                    }
                }
            }
            println!("cargo:warning=flatc not found, using existing generated code.");
        }
    }
}

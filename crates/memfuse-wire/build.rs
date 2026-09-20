use std::path::Path;
use std::process::Command;

fn main() {
    let schema_path = "../../schemas/memfuse.fbs";
    let out_dir = "src";

    if Path::new(schema_path).exists() {
        println!("cargo:rerun-if-changed={schema_path}");

        let output_file = Path::new(out_dir).join("memfuse_generated.rs");

        let flatc_exists = Command::new("flatc")
            .arg("--version")
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if flatc_exists {
            if let Ok(status) = Command::new("flatc")
                .args(["--rust", "-o", out_dir, schema_path])
                .status()
            {
                assert!(status.success(), "flatc failed to generate code");
            }
        } else if !output_file.exists() {
            eprintln!("flatc not found and generated code does not exist. Please install flatbuffers compiler.");
            std::process::exit(1);
        } else {
            println!("cargo:warning=flatc not found, using existing generated code.");
        }
    }
}

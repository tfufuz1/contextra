//! CI Gate: FlatBuffers Schema Drift Checker & Code Regenerator.
//! Validates whether `schemas/memfuse.fbs` matches `crates/memfuse-core-ipc-gen/src/memfuse_generated.rs`.

use std::fs;
use std::process::Command;
use tempfile::tempdir;

use crate::find_root_dir;

/// Checks if `flatc` is available in PATH.
fn check_flatc_available() -> Result<(), String> {
    match Command::new("flatc").arg("--version").output() {
        Ok(output) if output.status.success() => Ok(()),
        _ => Err("❌ Gate failed: 'flatc' binary not found in PATH. Please install FlatBuffers compiler (flatc) to run the flatbuffers drift gate.".to_string()),
    }
}

/// Checks if the generated FlatBuffers code matches the schema in `schemas/memfuse.fbs`.
pub fn check_flatbuffers_drift() -> Result<(), String> {
    println!("=== Gate: Check FlatBuffers Schema Drift ===");

    check_flatc_available()?;

    let root = find_root_dir();
    let schema_path = root.join("schemas/memfuse.fbs");
    let existing_generated_path = root.join("crates/memfuse-core-ipc-gen/src/memfuse_generated.rs");

    if !schema_path.exists() {
        return Err(format!(
            "❌ Gate failed: FlatBuffers schema file not found at '{}'",
            schema_path.display()
        ));
    }

    if !existing_generated_path.exists() {
        return Err(format!(
            "❌ Gate failed: Existing generated file not found at '{}'. Run 'cargo xtask regenerate-flatbuffers'.",
            existing_generated_path.display()
        ));
    }

    let temp_dir = tempdir().map_err(|e| format!("Failed to create temporary directory: {}", e))?;
    let temp_out_dir = temp_dir.path();

    let output = Command::new("flatc")
        .args([
            "--rust",
            "-o",
            temp_out_dir.to_str().unwrap_or("."),
            schema_path.to_str().unwrap_or(""),
        ])
        .output()
        .map_err(|e| format!("Failed to execute 'flatc': {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "❌ Gate failed: 'flatc' code generation failed with status {}:\n{}",
            output.status, stderr
        ));
    }

    let newly_generated_path = temp_out_dir.join("memfuse_generated.rs");
    if !newly_generated_path.exists() {
        return Err(
            "❌ Gate failed: 'flatc' completed successfully but output 'memfuse_generated.rs' was not found.".to_string(),
        );
    }

    let new_content = fs::read_to_string(&newly_generated_path)
        .map_err(|e| format!("Failed to read generated output file: {}", e))?;
    let existing_content = fs::read_to_string(&existing_generated_path)
        .map_err(|e| format!("Failed to read existing file '{}': {}", existing_generated_path.display(), e))?;

    let norm_new = normalize_code(&new_content);
    let norm_existing = normalize_code(&existing_content);

    if norm_new == norm_existing {
        println!("✅ Gate passed: FlatBuffers generated Rust code is in sync with 'schemas/memfuse.fbs'.");
        Ok(())
    } else {
        let err_msg = "❌ Gate failed: FlatBuffers schema drift detected! 'schemas/memfuse.fbs' does not match 'crates/memfuse-core-ipc-gen/src/memfuse_generated.rs'.\n💡 Run 'cargo xtask regenerate-flatbuffers' to update the generated Rust code.".to_string();
        eprintln!("{}", err_msg);
        Err(err_msg)
    }
}

/// Regenerates `crates/memfuse-core-ipc-gen/src/memfuse_generated.rs` directly from `schemas/memfuse.fbs`.
pub fn regenerate_flatbuffers() -> Result<(), String> {
    println!("=== XTask: Regenerate FlatBuffers Rust Code ===");

    check_flatc_available()?;

    let root = find_root_dir();
    let schema_path = root.join("schemas/memfuse.fbs");
    let out_dir = root.join("crates/memfuse-core-ipc-gen/src");

    if !schema_path.exists() {
        return Err(format!(
            "❌ FlatBuffers schema file not found at '{}'",
            schema_path.display()
        ));
    }

    let output = Command::new("flatc")
        .args([
            "--rust",
            "-o",
            out_dir.to_str().unwrap_or("."),
            schema_path.to_str().unwrap_or(""),
        ])
        .output()
        .map_err(|e| format!("Failed to execute 'flatc': {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "❌ 'flatc' code generation failed with status {}:\n{}",
            output.status, stderr
        ));
    }

    println!("✅ FlatBuffers Rust code successfully regenerated at '{}'.", out_dir.join("memfuse_generated.rs").display());
    Ok(())
}

/// Normalizes code strings for comparison (trims line endings and whitespace).
fn normalize_code(content: &str) -> String {
    content
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<&str>>()
        .join("\n")
        .trim()
        .to_string()
}

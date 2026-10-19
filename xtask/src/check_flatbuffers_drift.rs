//! CI Gate: FlatBuffers Schema Drift Checker & Code Regenerator.
//!
//! Wann ausführen:
//! - `cargo xtask check-flatbuffers-drift`: Im CI-Gate und vor Commits, um zu prüfen,
//!   dass `crates/contextra-wire/src/contextra_generated.rs` mit `schemas/contextra.fbs` übereinstimmt.
//! - `cargo xtask regenerate-flatbuffers`: Bei Änderungen an `schemas/contextra.fbs`,
//!   um den generierten Rust-Code in `crates/contextra-wire/src/contextra_generated.rs` zu aktualisieren.
//!
//! Erwartete flatc-Version:
//! - FlatBuffers compiler (`flatc`) Version 24.12.23.
//!
//! Einchecken des Ergebnisses:
//! - Das Generierungsergebnis in `crates/contextra-wire/src/contextra_generated.rs` ist
//!   im Repository committet (P01), sodass `cargo build -p contextra-wire` ohne lokales
//!   `flatc` gebaut werden kann und der Quellbaum während des Builds niemals verändert wird.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;

use crate::find_root_dir;

const REQUIRED_FLATC_VERSION: &str = "24.12.23";

/// Validates whether a candidate `flatc` binary exists and matches `REQUIRED_FLATC_VERSION`.
fn check_flatc_candidate(candidate: &Path) -> Result<(), String> {
    let output = Command::new(candidate)
        .arg("--version")
        .output()
        .map_err(|e| format!("Fehler beim Ausführen von '{:?} --version': {}", candidate, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "flatc --version fehlgeschlagen (Status {}): {}",
            output.status, stderr
        ));
    }

    let version_str = String::from_utf8_lossy(&output.stdout);
    let version_stderr = String::from_utf8_lossy(&output.stderr);
    let combined_version = format!("{} {}", version_str, version_stderr);

    if combined_version.contains(REQUIRED_FLATC_VERSION) {
        Ok(())
    } else {
        Err(format!(
            "Inkompatible flatc-Version '{}'. Erwartet wird Version {}.",
            version_str.trim(),
            REQUIRED_FLATC_VERSION
        ))
    }
}

/// Finds or fetches `flatc` binary, enforcing `REQUIRED_FLATC_VERSION`.
fn find_or_fetch_flatc() -> Result<PathBuf, String> {
    // 1. Check if 'flatc' in PATH is valid
    let path_flatc = PathBuf::from("flatc");
    if check_flatc_candidate(&path_flatc).is_ok() {
        return Ok(path_flatc);
    }

    // 2. Check ~/.local/bin/flatc or /tmp/flatc
    if let Ok(home) = std::env::var("HOME") {
        let local_flatc = PathBuf::from(home).join(".local/bin/flatc");
        if local_flatc.exists() && check_flatc_candidate(&local_flatc).is_ok() {
            return Ok(local_flatc);
        }
    }

    let tmp_flatc = PathBuf::from("/tmp/flatc");
    if tmp_flatc.exists() && check_flatc_candidate(&tmp_flatc).is_ok() {
        return Ok(tmp_flatc);
    }

    // 3. Try downloading flatc binary from GitHub release
    let download_url = "https://github.com/google/flatbuffers/releases/download/v24.12.23/Linux.flatc.binary.g%2B%2B-13.zip";
    let zip_path = PathBuf::from("/tmp/flatc_dl.zip");
    if let Ok(st) = Command::new("curl")
        .args([
            "-sL",
            download_url,
            "-o",
            zip_path.to_str().unwrap_or("/tmp/flatc_dl.zip"),
        ])
        .status()
    {
        if st.success() {
            if let Ok(ust) = Command::new("unzip")
                .args([
                    "-o",
                    zip_path.to_str().unwrap_or("/tmp/flatc_dl.zip"),
                    "-d",
                    "/tmp/",
                ])
                .status()
            {
                if ust.success() {
                    let _ = Command::new("chmod").args(["+x", "/tmp/flatc"]).status();
                    if check_flatc_candidate(&tmp_flatc).is_ok() {
                        return Ok(tmp_flatc);
                    }
                }
            }
        }
    }

    Err(format!(
        "❌ Gate fehlgeschlagen: 'flatc' Binary Version {} nicht gefunden. Bitte installieren Sie FlatBuffers Compiler (flatc {}).",
        REQUIRED_FLATC_VERSION, REQUIRED_FLATC_VERSION
    ))
}

/// Wraps `.get` calls in `unsafe { ... }` blocks for FlatBuffers generated code.
fn wrap_tab_get_calls_in_unsafe(line: &str) -> String {
    let pattern = "self._tab.get";
    if !line.contains(pattern) {
        return line.to_string();
    }

    let mut result = String::with_capacity(line.len() + 32);
    let mut search_from = 0;

    while let Some(rel_idx) = line[search_from..].find(pattern) {
        let idx = search_from + rel_idx;

        // Check if this occurrence is already preceded by `unsafe` on the same line
        let prefix = &line[..idx];
        if prefix.trim_end().ends_with("unsafe {") || prefix.contains("unsafe") {
            // Already wrapped
            result.push_str(&line[search_from..idx + pattern.len()]);
            search_from = idx + pattern.len();
            continue;
        }

        // Find the opening parenthesis of `get::<...>(...`
        if let Some(rel_paren_open) = line[idx + pattern.len()..].find('(') {
            let paren_open = idx + pattern.len() + rel_paren_open;

            // Find matching closing parenthesis
            let mut depth = 0;
            let mut close_paren = None;
            for (i, ch) in line[paren_open..].char_indices() {
                if ch == '(' {
                    depth += 1;
                } else if ch == ')' {
                    depth -= 1;
                    if depth == 0 {
                        close_paren = Some(paren_open + i);
                        break;
                    }
                }
            }

            if let Some(end_idx) = close_paren {
                result.push_str(&line[search_from..idx]);
                result.push_str("unsafe { ");
                result.push_str(&line[idx..=end_idx]);
                result.push_str(" }");
                search_from = end_idx + 1;
                continue;
            }
        }

        result.push_str(&line[search_from..idx + pattern.len()]);
        search_from = idx + pattern.len();
    }

    result.push_str(&line[search_from..]);
    result
}

/// Post-processes generated Rust file by wrapping `.get` calls in `unsafe`.
fn post_process_generated_code(output_file: &Path) -> Result<(), String> {
    let content = fs::read_to_string(output_file)
        .map_err(|e| format!("Failed to read generated code at '{}': {}", output_file.display(), e))?;

    let mut modified = false;
    let new_lines: Vec<String> = content
        .lines()
        .map(|line| {
            let processed = wrap_tab_get_calls_in_unsafe(line);
            if processed != line {
                modified = true;
            }
            processed
        })
        .collect();

    if modified {
        let mut new_content = new_lines.join("\n");
        if content.ends_with('\n') {
            new_content.push('\n');
        }
        fs::write(output_file, new_content)
            .map_err(|e| format!("Failed to write post-processed code to '{}': {}", output_file.display(), e))?;
    }
    Ok(())
}

/// Checks if the generated FlatBuffers code matches the schema in `schemas/contextra.fbs`.
pub fn check_flatbuffers_drift() -> Result<(), String> {
    println!("=== Gate: Check FlatBuffers Schema Drift ===");

    let root = find_root_dir();
    let schema_path = root.join("schemas/contextra.fbs");
    let existing_generated_path = root.join("crates/contextra-wire/src/contextra_generated.rs");

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

    let flatc_bin = find_or_fetch_flatc()?;

    let temp_dir = tempdir().map_err(|e| format!("Failed to create temporary directory: {}", e))?;
    let temp_out_dir = temp_dir.path();

    let output = Command::new(&flatc_bin)
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

    let newly_generated_path = temp_out_dir.join("contextra_generated.rs");
    if !newly_generated_path.exists() {
        return Err(
            "❌ Gate failed: 'flatc' completed successfully but output 'contextra_generated.rs' was not found.".to_string(),
        );
    }

    post_process_generated_code(&newly_generated_path)?;

    let new_bytes = fs::read(&newly_generated_path)
        .map_err(|e| format!("Failed to read generated output file: {}", e))?;
    let existing_bytes = fs::read(&existing_generated_path).map_err(|e| {
        format!(
            "Failed to read existing file '{}': {}",
            existing_generated_path.display(),
            e
        )
    })?;

    if new_bytes == existing_bytes {
        println!("✅ Gate passed: FlatBuffers generated Rust code is in sync with 'schemas/contextra.fbs'.");
        Ok(())
    } else {
        let err_msg = format!(
            "❌ Gate failed: FlatBuffers schema drift detected! 'schemas/contextra.fbs' does not match '{}'.\n💡 Run 'cargo xtask regenerate-flatbuffers' to update the generated Rust code.",
            existing_generated_path.display()
        );
        eprintln!("{}", err_msg);
        Err(err_msg)
    }
}

/// Regenerates `crates/contextra-wire/src/contextra_generated.rs` directly from `schemas/contextra.fbs`.
pub fn regenerate_flatbuffers() -> Result<(), String> {
    println!("=== XTask: Regenerate FlatBuffers Rust Code ===");

    let root = find_root_dir();
    let schema_path = root.join("schemas/contextra.fbs");
    let target_generated_path = root.join("crates/contextra-wire/src/contextra_generated.rs");

    if !schema_path.exists() {
        return Err(format!(
            "❌ FlatBuffers schema file not found at '{}'",
            schema_path.display()
        ));
    }

    let flatc_bin = find_or_fetch_flatc()?;

    let temp_dir = tempdir().map_err(|e| format!("Failed to create temporary directory: {}", e))?;
    let temp_out_dir = temp_dir.path();

    let output = Command::new(&flatc_bin)
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
            "❌ 'flatc' code generation failed with status {}:\n{}",
            output.status, stderr
        ));
    }

    let newly_generated_path = temp_out_dir.join("contextra_generated.rs");
    if !newly_generated_path.exists() {
        return Err(
            "❌ 'flatc' completed successfully but output 'contextra_generated.rs' was not found.".to_string(),
        );
    }

    post_process_generated_code(&newly_generated_path)?;

    let content = fs::read(&newly_generated_path)
        .map_err(|e| format!("Failed to read newly generated code: {}", e))?;

    fs::write(&target_generated_path, content)
        .map_err(|e| format!("Failed to write regenerated code to '{}': {}", target_generated_path.display(), e))?;

    println!(
        "✅ FlatBuffers Rust code successfully regenerated at '{}'.",
        target_generated_path.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_tab_get_calls() {
        let input1 = "    self._tab.get::<u32>(RoleId::VT_ID, Some(0)).unwrap()";
        let expected1 = "    unsafe { self._tab.get::<u32>(RoleId::VT_ID, Some(0)) }.unwrap()";
        assert_eq!(wrap_tab_get_calls_in_unsafe(input1), expected1);

        let input2 =
            "    self._tab.get::<flatbuffers::ForwardsUOffset<&str>>(ScoredDocument::VT_ID, None)";
        let expected2 = "    unsafe { self._tab.get::<flatbuffers::ForwardsUOffset<&str>>(ScoredDocument::VT_ID, None) }";
        assert_eq!(wrap_tab_get_calls_in_unsafe(input2), expected2);

        let input3 = "    unsafe { self._tab.get::<i8>(Embedding::VT_METRIC, Some(0)).unwrap()}";
        assert_eq!(wrap_tab_get_calls_in_unsafe(input3), input3);
    }
}

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

pub mod artifact_header;
pub mod check_duplicate_core_primitives;
pub mod gates;
pub mod generate_diagnostics;
pub mod harness;
pub mod reproducible_build;
pub mod session_history;

pub fn find_root_dir() -> PathBuf {
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut dir = PathBuf::from(manifest_dir);
        loop {
            let cargo_path = dir.join("Cargo.toml");
            if cargo_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&cargo_path) {
                    if content.contains("[workspace]") && content.contains("members") {
                        return dir;
                    }
                }
            }
            if !dir.pop() {
                break;
            }
        }
    }
    if let Ok(curr) = std::env::current_dir() {
        let mut dir = curr;
        loop {
            let cargo_path = dir.join("Cargo.toml");
            if cargo_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&cargo_path) {
                    if content.contains("[workspace]") && content.contains("members") {
                        return dir;
                    }
                }
            }
            if !dir.pop() {
                break;
            }
        }
    }
    if Path::new("Cargo.toml").exists()
        && std::fs::read_to_string("Cargo.toml")
            .unwrap_or_default()
            .contains("members")
    {
        PathBuf::from(".")
    } else if Path::new("../Cargo.toml").exists() {
        PathBuf::from("..")
    } else {
        PathBuf::from(".")
    }
}

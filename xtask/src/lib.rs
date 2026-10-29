#![forbid(unsafe_code)]
#![allow(clippy::disallowed_methods)]
#![allow(clippy::disallowed_types)]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::panic)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::question_mark)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::unnecessary_map_or)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::manual_strip)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::useless_format)]
#![allow(clippy::exit)]
#![allow(clippy::unreachable)]
#![allow(clippy::needless_borrow)]

use std::path::{Path, PathBuf};

pub mod agent_lifecycle;
pub mod artifact_header;
pub mod audit_integrity_check;
pub mod bench_compile;
pub mod bench_download;
pub mod check_adr_deadlines;
pub mod check_commit_diff_integrity;
pub mod check_duplicate_core_primitives;
pub mod check_duplicate_symbols_cross_file;
pub mod check_mutation_score_gate;
pub mod check_phantom_files;
pub mod check_ring0_async_purity;
pub mod check_ring_capabilities_consistency;
pub mod check_ring_layering;
pub mod check_stale_tags;
pub mod check_type_registry;
pub mod check_unsafe_islands;
pub mod check_vetoes;
pub mod claim;
pub mod commit_health;
pub mod context_pack;
pub mod core;
pub mod crate_context;
pub mod env_validate;
pub mod feature_matrix;
pub mod gate_check;
pub mod gates;
pub mod generate_diagnostics;
pub mod generate_markers;
pub mod harness;
pub mod hotspot_report;
pub mod legacy;
pub mod loom_run;
pub mod panic_inventory;
pub mod py_test;
pub mod reproducible_build;
pub mod security_scan;
pub mod session_history;
pub mod session_init;
pub mod shell_commit_audit;
pub mod tag_health;
pub mod veto_deadline_gate;
pub mod workspace_verify;

pub fn find_root_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("CONTEXTRA_ROOT_DIR") {
        return PathBuf::from(dir);
    }
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

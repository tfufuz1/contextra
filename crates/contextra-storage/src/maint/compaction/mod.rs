//! # Kompaktions-Modul (Leveled Compaction)
//!
//! SPEC §12.2, §12.3, §16.4, §16.5:
//! Deterministische Leveled-Kompaktion mit reiner Planung und Versionenaufbewahrung.

pub mod exec;
pub mod install;
pub mod plan;
pub mod retain;

pub use exec::{execute_compaction, run_file_name, CompactionExecutionResult};
pub use install::{install_compaction, CompactionInstallError};
pub use plan::{plan, Claims, CompactionParams, CompactionPlan};
pub use retain::{retain_key_versions, CompactionEntry, RetainResult};

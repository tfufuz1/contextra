//! `doctor` CLI subcommand inspecting system environment and platform bounds.

use contextra_types::ContextraError;
use std::path::Path;

/// Result status of an individual doctor check.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DoctorCheckStatus {
    Ok(String),
    Warning(String),
    NotCheckable(String),
    Failed(String),
}

/// Trait for extensible doctor checkers.
pub trait DoctorChecker: Send + Sync {
    fn name(&self) -> &'static str;
    fn check(&self, db_path: &Path) -> DoctorCheckStatus;
}

/// System doctor evaluator running all registered checkers.
pub struct SystemDoctor {
    checkers: Vec<Box<dyn DoctorChecker>>,
}

impl Default for SystemDoctor {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemDoctor {
    /// Creates a default `SystemDoctor` pre-populated with standard checkers.
    pub fn new() -> Self {
        let checkers: Vec<Box<dyn DoctorChecker>> = vec![
            Box::new(DataDirChecker),
            Box::new(EncryptionStateChecker),
            Box::new(DurabilityChecker),
        ];
        Self { checkers }
    }

    /// Registers a custom `DoctorChecker`.
    pub fn with_checker(mut self, checker: Box<dyn DoctorChecker>) -> Self {
        self.checkers.push(checker);
        self
    }

    /// Runs all doctor checks and returns results.
    pub fn run_diagnostics(&self, db_path: &Path) -> Vec<(&'static str, DoctorCheckStatus)> {
        self.checkers
            .iter()
            .map(|c| (c.name(), c.check(db_path)))
            .collect()
    }
}

/// Data directory checker.
struct DataDirChecker;
impl DoctorChecker for DataDirChecker {
    fn name(&self) -> &'static str {
        "DataDirectory"
    }
    fn check(&self, db_path: &Path) -> DoctorCheckStatus {
        if db_path.exists() {
            DoctorCheckStatus::Ok(format!("Directory exists at {}", db_path.display()))
        } else {
            DoctorCheckStatus::Warning(format!("Directory does not exist at {}", db_path.display()))
        }
    }
}

/// Encryption state checker.
struct EncryptionStateChecker;
impl DoctorChecker for EncryptionStateChecker {
    fn name(&self) -> &'static str {
        "EncryptionState"
    }
    fn check(&self, _db_path: &Path) -> DoctorCheckStatus {
        DoctorCheckStatus::NotCheckable(
            "Verschlüsselungsstatus ohne aktiven KeyManager nicht prüfbar".to_string(),
        )
    }
}

/// Durability primitive checker.
struct DurabilityChecker;
impl DoctorChecker for DurabilityChecker {
    fn name(&self) -> &'static str {
        "DurabilityPrimitives"
    }
    fn check(&self, _db_path: &Path) -> DoctorCheckStatus {
        DoctorCheckStatus::NotCheckable(
            "Dateisystem-Durability und Sync-Klasse ohne sys::fs_class Anbindung nicht prüfbar"
                .to_string(),
        )
    }
}

/// Executes system doctor diagnostics.
pub fn execute_doctor(db_path: &Path) -> Result<(), ContextraError> {
    let doctor = SystemDoctor::new();
    let results = doctor.run_diagnostics(db_path);

    let mut failed = false;
    for (name, status) in results {
        match status {
            DoctorCheckStatus::Ok(msg) => {
                tracing::info!(check = name, status = "OK", details = %msg);
            }
            DoctorCheckStatus::Warning(msg) => {
                tracing::warn!(check = name, status = "WARN", details = %msg);
            }
            DoctorCheckStatus::NotCheckable(msg) => {
                tracing::info!(check = name, status = "NICHT PRÜFBAR", details = %msg);
            }
            DoctorCheckStatus::Failed(msg) => {
                tracing::error!(check = name, status = "FAIL", details = %msg);
                failed = true;
            }
        }
    }

    if failed {
        Err(ContextraError::InvalidInput(
            "Doctor diagnostic failed on one or more checks.".to_string(),
        ))
    } else {
        Ok(())
    }
}

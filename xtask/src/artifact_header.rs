//! Diagnostic Artifact Header Infrastructure (§A.4)
//! Provides standard metadata fields (`commit`, `generated_by`, `generated_at`, `toolchain`)
//! for test reports, lint reports, audit documents, and marker tables.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::process::Command;

#[derive(Debug)]
pub enum ArtifactHeaderError {
    CommandFailed { command: &'static str, details: String },
    Utf8Error(std::string::FromUtf8Error),
    JsonError(serde_json::Error),
    IoError(std::io::Error),
}

impl fmt::Display for ArtifactHeaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArtifactHeaderError::CommandFailed { command, details } => {
                write!(f, "Command '{}' failed: {}", command, details)
            }
            ArtifactHeaderError::Utf8Error(e) => write!(f, "UTF-8 conversion error: {}", e),
            ArtifactHeaderError::JsonError(e) => write!(f, "JSON error: {}", e),
            ArtifactHeaderError::IoError(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for ArtifactHeaderError {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactHeader {
    pub commit: String,       // volle Git-SHA von HEAD
    pub generated_by: String, // exakter aufgerufener xtask-Befehl inkl. Argumente
    pub generated_at: String, // UTC, ISO-8601
    pub toolchain: String,    // Ausgabe von `rustc --version`
}

impl ArtifactHeader {
    pub fn capture(generated_by: impl Into<String>) -> Result<Self, ArtifactHeaderError> {
        let commit_output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .map_err(ArtifactHeaderError::IoError)?;

        if !commit_output.status.success() {
            return Err(ArtifactHeaderError::CommandFailed {
                command: "git rev-parse HEAD",
                details: String::from_utf8_lossy(&commit_output.stderr).to_string(),
            });
        }

        let commit = String::from_utf8(commit_output.stdout)
            .map_err(ArtifactHeaderError::Utf8Error)?
            .trim()
            .to_string();

        let toolchain_output = Command::new("rustc")
            .arg("--version")
            .output()
            .map_err(ArtifactHeaderError::IoError)?;

        if !toolchain_output.status.success() {
            return Err(ArtifactHeaderError::CommandFailed {
                command: "rustc --version",
                details: String::from_utf8_lossy(&toolchain_output.stderr).to_string(),
            });
        }

        let toolchain = String::from_utf8(toolchain_output.stdout)
            .map_err(ArtifactHeaderError::Utf8Error)?
            .trim()
            .to_string();

        let generated_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        Ok(Self {
            commit,
            generated_by: generated_by.into(),
            generated_at,
            toolchain,
        })
    }

    pub fn render_markdown_frontmatter(&self) -> String {
        format!(
            "---\ncommit: {}\ngenerated_by: {}\ngenerated_at: {}\ntoolchain: {}\n---\n",
            self.commit, self.generated_by, self.generated_at, self.toolchain
        )
    }

    pub fn render_json(&self) -> Result<String, ArtifactHeaderError> {
        serde_json::to_string_pretty(self).map_err(ArtifactHeaderError::JsonError)
    }
}

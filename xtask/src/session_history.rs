// TODO: weekly digest, separater Auftrag
//! Lokales, rotierendes Session-History-Log für MemFuse.
//!
//! Dieses Modul verwaltet ein chronologisches, git-ignoriertes Protokoll
//! aller Agenten-Aktionen unter `.jules/session_history.jsonl`.
//!
//! # Invarianten
//! - **Zero-Panic-Doctrine**: Keine `.unwrap()` oder `.expect()` in Produktions-Pfade.
//! - **Append-Only File I/O**: Verwendet append-mode ohne vorheriges Einlesen der gesamten Datei.
//! - **Metadata-based Rotation**: Rotationsprüfung nutzt reine Metadaten-Abfragen (`fs::metadata`).

use chrono::Utc;
use memfuse_types::MemFuseError;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Standard-Schwellenwert für die Log-Rotation (5 MB).
pub const DEFAULT_MAX_LOG_SIZE_BYTES: u64 = 5 * 1024 * 1024;

/// Konfiguration für die Log-Rotation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RotationConfig {
    /// Maximale Dateigröße in Bytes vor der Rotation.
    pub max_bytes: u64,
    /// Anzahl aufzubewahrender Backup-Dateien (z. B. 1 für `.1`).
    pub max_backups: usize,
}

impl Default for RotationConfig {
    fn default() -> Self {
        Self {
            max_bytes: DEFAULT_MAX_LOG_SIZE_BYTES,
            max_backups: 1,
        }
    }
}

/// Repräsentiert einen einzelnen Eintrag im Session-History-Log.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionHistoryEntry {
    /// ISO-8601 UTC-Zeitstempel der Aktion.
    pub timestamp: String,
    /// Eindeutige Session-ID des ausführenden Agenten.
    pub session_id: String,
    /// Bezeichnung der ausgeführten Aktion (z. B. "claim", "release", "preflight").
    pub action: String,
    /// Crate-Name, falls kontextuell zutreffend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub krate: Option<String>,
    /// Issue- oder Task-ID, falls zutreffend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue: Option<String>,
    /// Freie strukturierte Details im JSON-Format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl SessionHistoryEntry {
    /// Erstellt einen neuen Log-Eintrag mit aktuellem UTC-Zeitstempel.
    pub fn new(session_id: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now().to_rfc3339(),
            session_id: session_id.into(),
            action: action.into(),
            krate: None,
            issue: None,
            details: None,
        }
    }

    /// Setzt das Crate-Feld.
    pub fn with_crate(mut self, krate: impl Into<String>) -> Self {
        self.krate = Some(krate.into());
        self
    }

    /// Setzt das Issue-Feld.
    pub fn with_issue(mut self, issue: impl Into<String>) -> Self {
        self.issue = Some(issue.into());
        self
    }

    /// Setzt strukturierte Details.
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

/// Ermittelt das Root-Verzeichnis des Repositorys.
pub fn find_root_dir() -> PathBuf {
    if Path::new("Cargo.toml").exists()
        && fs::read_to_string("Cargo.toml")
            .unwrap_or_default()
            .contains("[workspace]")
    {
        PathBuf::from(".")
    } else if Path::new("../Cargo.toml").exists() {
        PathBuf::from("..")
    } else {
        PathBuf::from(".")
    }
}

/// Ermittelt den Standard-Pfad `.jules/session_history.jsonl` relativ zum Repository-Root.
pub fn default_session_history_path() -> PathBuf {
    find_root_dir().join(".jules/session_history.jsonl")
}

/// Hängt einen Eintrag an das globale Session-History-Log `.jules/session_history.jsonl` an.
/// Führt bei Überschreitung der Standardgrößenschwelle (5 MB) eine automatische Rotation durch.
pub fn append_entry(entry: &SessionHistoryEntry) -> Result<(), MemFuseError> {
    let path = default_session_history_path();
    let config = RotationConfig::default();
    append_entry_with_config(entry, &config, &path)
}

/// Hängt einen Eintrag an eine definierte Log-Datei an und wendet die angegebene Rotationskonfiguration an.
pub fn append_entry_with_config(
    entry: &SessionHistoryEntry,
    config: &RotationConfig,
    path: &Path,
) -> Result<(), MemFuseError> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    // Rotationsprüfung via Metadaten-Abfrage (zero-copy / kein Einlesen des Dateiinhalts)
    if path.is_file() {
        let meta = fs::metadata(path)?;
        if meta.len() >= config.max_bytes && config.max_backups > 0 {
            rotate_logs(path, config.max_backups)?;
        }
    }

    let serialized = serde_json::to_string(entry)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;

    writeln!(file, "{}", serialized)?;
    Ok(())
}

/// Führt die Dateirotation durch: benennt `path` zu `path.1` um (und ältere Backups entsprechend `max_backups`).
fn rotate_logs(path: &Path, max_backups: usize) -> Result<(), MemFuseError> {
    for i in (1..max_backups).rev() {
        let src = path.with_extension(format!("jsonl.{}", i));
        let dst = path.with_extension(format!("jsonl.{}", i + 1));
        if src.exists() {
            fs::rename(&src, &dst)?;
        }
    }

    let target = path.with_extension("jsonl.1");
    if path.exists() {
        fs::rename(path, target)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_entry_builder_and_serialization() {
        let entry = SessionHistoryEntry::new("session-123", "claim")
            .with_crate("memfuse-core")
            .with_issue("ISSUE-42")
            .with_details(serde_json::json!({ "status": "ok" }));

        assert_eq!(entry.session_id, "session-123");
        assert_eq!(entry.action, "claim");
        assert_eq!(entry.krate.as_deref(), Some("memfuse-core"));
        assert_eq!(entry.issue.as_deref(), Some("ISSUE-42"));

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: SessionHistoryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, deserialized);
    }

    #[test]
    fn test_append_single_entry() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("session_history.jsonl");

        let entry = SessionHistoryEntry::new("sess-1", "preflight")
            .with_crate("xtask");

        let config = RotationConfig::default();
        append_entry_with_config(&entry, &config, &log_path).unwrap();

        assert!(log_path.exists());
        let content = fs::read_to_string(&log_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 1);

        let parsed: SessionHistoryEntry = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(parsed.session_id, "sess-1");
        assert_eq!(parsed.action, "preflight");
        assert_eq!(parsed.krate.as_deref(), Some("xtask"));
    }

    #[test]
    fn test_append_multiple_entries_append_mode() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("session_history.jsonl");
        let config = RotationConfig::default();

        let e1 = SessionHistoryEntry::new("sess-1", "claim").with_crate("memfuse-store");
        let e2 = SessionHistoryEntry::new("sess-1", "preflight").with_crate("memfuse-store");

        append_entry_with_config(&e1, &config, &log_path).unwrap();
        append_entry_with_config(&e2, &config, &log_path).unwrap();

        let content = fs::read_to_string(&log_path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);

        let p1: SessionHistoryEntry = serde_json::from_str(lines[0]).unwrap();
        let p2: SessionHistoryEntry = serde_json::from_str(lines[1]).unwrap();

        assert_eq!(p1.action, "claim");
        assert_eq!(p2.action, "preflight");
    }

    #[test]
    fn test_log_rotation_on_size_threshold() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("session_history.jsonl");

        // Set low size limit (50 bytes) so rotation triggers after first write
        let config = RotationConfig {
            max_bytes: 50,
            max_backups: 1,
        };

        let e1 = SessionHistoryEntry::new("sess-1", "action-1")
            .with_details(serde_json::json!({ "data": "larger_payload_to_exceed_limit" }));

        // First append creates log_path
        append_entry_with_config(&e1, &config, &log_path).unwrap();
        assert!(log_path.exists());
        let meta1 = fs::metadata(&log_path).unwrap();
        assert!(meta1.len() >= 50);

        let e2 = SessionHistoryEntry::new("sess-2", "action-2");

        // Second append triggers rotation because size >= 50 bytes
        append_entry_with_config(&e2, &config, &log_path).unwrap();

        let rotated_path = dir.path().join("session_history.jsonl.1");
        assert!(rotated_path.exists(), "Rotated log file .1 must exist");
        assert!(log_path.exists(), "New active log file must exist");

        let rotated_content = fs::read_to_string(&rotated_path).unwrap();
        let active_content = fs::read_to_string(&log_path).unwrap();

        assert!(rotated_content.contains("sess-1"));
        assert!(active_content.contains("sess-2"));
        assert!(!active_content.contains("sess-1"));
    }
}

use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Struktur für Sicherheits-Audit-Logeinträge bei Auslösung des `Escalate`-Modus.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecurityAuditRecord {
    pub timestamp: String,
    pub event_type: String,
    pub doc_id: String,
    pub collection: String,
    pub pattern_matched: String,
    pub action_taken: String,
}

/// Separater Audit-Logger für Sicherheitsvorfälle (vollständig isoliert vom Vektor-Index).
#[derive(Clone, Debug, Default)]
pub struct SecurityAuditLogger {
    file_path: Option<PathBuf>,
    in_memory_records: Arc<Mutex<Vec<SecurityAuditRecord>>>,
}

impl SecurityAuditLogger {
    pub fn new(file_path: Option<PathBuf>) -> Self {
        Self {
            file_path,
            in_memory_records: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn log_event(&self, record: SecurityAuditRecord) {
        // 1. In-Memory Puffer für Tests und Auditing
        if let Ok(mut records) = self.in_memory_records.lock() {
            records.push(record.clone());
        }

        // 2. Tracing Log
        tracing::warn!(
            target: "security_audit",
            doc_id = %record.doc_id,
            collection = %record.collection,
            pattern = %record.pattern_matched,
            action = %record.action_taken,
            "Security Audit: Prompt injection attempt detected and escalated"
        );

        // 3. Datei-Sicherheits-Log schreiben, falls konfiguriert
        if let Some(path) = &self.file_path {
            if let Ok(json_line) = serde_json::to_string(&record) {
                if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                    if let Err(err) = writeln!(file, "{json_line}") {
                        tracing::warn!(?err, "Failed to write security audit record to file");
                    }
                }
            }
        }
    }

    pub fn get_recorded_events(&self) -> Vec<SecurityAuditRecord> {
        self.in_memory_records
            .lock()
            .map(|r| r.clone())
            .unwrap_or_default()
    }
}

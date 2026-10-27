use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub gate: String,
    pub cmd: Vec<String>,
    pub toolchain: String,
    pub commit: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub tests_run: u64,
    pub tests_skipped: u64,
    pub log_sha256: String,
    pub started_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceIndexEntry {
    pub gate: String,
    pub exit_code: i32,
    pub record_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceIndex {
    pub index_hash: String,
    pub entries: Vec<EvidenceIndexEntry>,
}

pub fn save_evidence_with_log(
    evidence_dir: &Path,
    record: &EvidenceRecord,
    raw_log: &str,
) -> Result<(), String> {
    fs::create_dir_all(evidence_dir)
        .map_err(|e| format!("Failed to create evidence dir {}: {}", evidence_dir.display(), e))?;

    let file_path = evidence_dir.join(format!("{}.json", record.gate));
    let json_bytes = serde_json::to_vec_pretty(record)
        .map_err(|e| format!("Failed to serialize evidence record for {}: {}", record.gate, e))?;

    fs::write(&file_path, &json_bytes)
        .map_err(|e| format!("Failed to write evidence record to {}: {}", file_path.display(), e))?;

    let log_path = evidence_dir.join(format!("{}.log", record.gate));
    fs::write(&log_path, raw_log.as_bytes())
        .map_err(|e| format!("Failed to write raw log file to {}: {}", log_path.display(), e))?;

    update_index(evidence_dir)?;
    Ok(())
}

pub fn save_evidence(
    evidence_dir: &Path,
    record: &EvidenceRecord,
) -> Result<(), String> {
    save_evidence_with_log(evidence_dir, record, "")
}

fn update_index(evidence_dir: &Path) -> Result<(), String> {
    let index_path = evidence_dir.join("index.json");
    let mut entries = Vec::new();

    let read_dir = fs::read_dir(evidence_dir)
        .map_err(|e| format!("Failed to read evidence dir {}: {}", evidence_dir.display(), e))?;

    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
            let filename = match path.file_name().and_then(|s| s.to_str()) {
                Some(name) => name,
                None => continue,
            };
            if filename == "index.json" {
                continue;
            }

            let content = match fs::read(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let rec: EvidenceRecord = match serde_json::from_slice(&content) {
                Ok(r) => r,
                Err(_) => continue,
            };

            let hash = blake3::hash(&content).to_hex().to_string();
            entries.push(EvidenceIndexEntry {
                gate: rec.gate,
                exit_code: rec.exit_code,
                record_sha256: hash,
            });
        }
    }

    entries.sort_by(|a, b| a.gate.cmp(&b.gate));

    let index_json_for_hash = serde_json::to_vec(&entries).unwrap_or_default();
    let index_hash = blake3::hash(&index_json_for_hash).to_hex().to_string();

    let index = EvidenceIndex {
        index_hash,
        entries,
    };

    let index_bytes = serde_json::to_vec_pretty(&index)
        .map_err(|e| format!("Failed to serialize evidence index: {}", e))?;

    fs::write(&index_path, index_bytes)
        .map_err(|e| format!("Failed to write evidence index {}: {}", index_path.display(), e))?;

    Ok(())
}

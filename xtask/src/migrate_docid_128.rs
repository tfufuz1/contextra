//! Offline migration tool to upgrade MemFuse data exports and manifests from 64-bit DocIds (v1)
//! to 128-bit DocIds (v2, ADR-082) via deterministic BLAKE3 16-byte key re-derivation.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Migration configuration options.
#[derive(Debug, Clone, Default)]
pub struct MigrationConfig {
    /// Source input file or directory path.
    pub input_path: PathBuf,
    /// Destination output file or directory path.
    pub output_path: PathBuf,
    /// If true, performs validation and calculation without writing output files.
    pub dry_run: bool,
}

/// Progress report summary returned after migration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationReport {
    /// Number of records successfully processed and migrated.
    pub records_processed: usize,
    /// Number of DocId collisions detected (should be 0 for BLAKE3-128).
    pub collisions_detected: usize,
    /// Target schema version set post-migration.
    pub target_schema_version: String,
    /// Dry run flag status.
    pub is_dry_run: bool,
}

/// Generic JSON record representing a document export frame.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DocumentRecord {
    /// Original string key.
    pub key: String,
    /// Document metadata or raw payload content.
    #[serde(default)]
    pub content: serde_json::Value,
    /// Associated schema version tag.
    #[serde(default = "default_v1_version")]
    pub schema_version: String,
    /// Calculated 128-bit hex string DocId.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_id_128_hex: Option<String>,
}

fn default_v1_version() -> String {
    "1.0".to_string()
}

/// Derives a 128-bit DocId hex string deterministically from a string key via 16-byte BLAKE3 truncation.
pub fn derive_doc_id_128_hex(key: &str) -> Result<String, String> {
    if key.is_empty() {
        return Err("Document key cannot be empty".to_string());
    }
    let hash = blake3::hash(key.as_bytes());
    let bytes = &hash.as_bytes()[..16];
    let mut hex = String::with_capacity(32);
    for b in bytes {
        use std::fmt::Write;
        if write!(&mut hex, "{:02x}", b).is_err() {
            return Err("Formatting hex failed".to_string());
        }
    }
    Ok(hex)
}

/// Runs the offline DocId migration from v1 to v2.
pub fn run_migration(config: &MigrationConfig) -> Result<MigrationReport, String> {
    if !config.input_path.exists() {
        return Err(format!(
            "Input path does not exist: {}",
            config.input_path.display()
        ));
    }

    let input_content = fs::read_to_string(&config.input_path)
        .map_err(|e| format!("Failed to read input file {}: {e}", config.input_path.display()))?;

    let mut records: Vec<DocumentRecord> = serde_json::from_str(&input_content)
        .map_err(|e| format!("Failed to parse document JSON records from {}: {e}", config.input_path.display()))?;

    let mut seen_ids = std::collections::HashSet::new();
    let mut collisions = 0;
    let records_count = records.len();

    for record in &mut records {
        let hex_id = derive_doc_id_128_hex(&record.key)?;
        if !seen_ids.insert(hex_id.clone()) {
            collisions += 1;
        }
        record.doc_id_128_hex = Some(hex_id);
        record.schema_version = "2.0".to_string();
    }

    if !config.dry_run {
        if let Some(parent) = config.output_path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    format!("Failed to create output parent dir {}: {e}", parent.display())
                })?;
            }
        }

        let output_json = serde_json::to_string_pretty(&records)
            .map_err(|e| format!("Failed to serialize migrated records: {e}"))?;

        fs::write(&config.output_path, output_json).map_err(|e| {
            format!("Failed to write migrated output to {}: {e}", config.output_path.display())
        })?;
    }

    Ok(MigrationReport {
        records_processed: records_count,
        collisions_detected: collisions,
        target_schema_version: "2.0".to_string(),
        is_dry_run: config.dry_run,
    })
}

/// Parses CLI arguments for `cargo xtask migrate-docid-128` and runs migration.
pub fn run_cli(args: &[String]) -> bool {
    println!("=== xtask migrate-docid-128 ===");

    let mut input = None;
    let mut output = None;
    let mut dry_run = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                if i + 1 < args.len() {
                    input = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--output" => {
                if i + 1 < args.len() {
                    output = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--dry-run" => {
                dry_run = true;
            }
            _ => {
                if let Some(val) = args[i].strip_prefix("--input=") {
                    input = Some(PathBuf::from(val));
                } else if let Some(val) = args[i].strip_prefix("--output=") {
                    output = Some(PathBuf::from(val));
                }
            }
        }
        i += 1;
    }

    let input_path = match input {
        Some(p) => p,
        None => {
            eprintln!("❌ Missing required parameter `--input <path>`");
            return false;
        }
    };

    let output_path = output.unwrap_or_else(|| {
        let mut p = input_path.clone();
        p.set_extension("v2.json");
        p
    });

    let config = MigrationConfig {
        input_path,
        output_path,
        dry_run,
    };

    match run_migration(&config) {
        Ok(report) => {
            println!(
                "✅ Migration completed: {} records processed, {} collisions, dry_run={}",
                report.records_processed, report.collisions_detected, report.is_dry_run
            );
            true
        }
        Err(e) => {
            eprintln!("❌ Migration failed: {e}");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_derive_doc_id_128_hex_determinism() {
        let key = "test_document_key_123";
        let hex1 = derive_doc_id_128_hex(key).unwrap();
        let hex2 = derive_doc_id_128_hex(key).unwrap();
        assert_eq!(hex1, hex2);
        assert_eq!(hex1.len(), 32);
    }

    #[test]
    fn test_derive_doc_id_128_empty_key_error() {
        assert!(derive_doc_id_128_hex("").is_err());
    }

    #[test]
    fn test_run_migration_roundtrip() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("export_v1.json");
        let output_path = dir.path().join("export_v2.json");

        let sample_records = vec![
            DocumentRecord {
                key: "doc_a".to_string(),
                content: serde_json::json!({"text": "hello"}),
                schema_version: "1.0".to_string(),
                doc_id_128_hex: None,
            },
            DocumentRecord {
                key: "doc_b".to_string(),
                content: serde_json::json!({"text": "world"}),
                schema_version: "1.0".to_string(),
                doc_id_128_hex: None,
            },
        ];

        fs::write(&input_path, serde_json::to_string(&sample_records).unwrap()).unwrap();

        let config = MigrationConfig {
            input_path: input_path.clone(),
            output_path: output_path.clone(),
            dry_run: false,
        };

        let report = run_migration(&config).unwrap();
        assert_eq!(report.records_processed, 2);
        assert_eq!(report.collisions_detected, 0);

        let output_content = fs::read_to_string(&output_path).unwrap();
        let migrated_records: Vec<DocumentRecord> = serde_json::from_str(&output_content).unwrap();

        assert_eq!(migrated_records.len(), 2);
        assert_eq!(migrated_records[0].schema_version, "2.0");
        assert!(migrated_records[0].doc_id_128_hex.is_some());
    }
}

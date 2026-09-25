use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MutationScoreThreshold {
    pub crate_name: String,
    pub minimum_percent: f64,
}

pub fn get_mutation_threshold_for_crate(crate_name: &str) -> Option<f64> {
    // Tier 0/1 (§22.1 der Spec): mindestens 70.0
    match crate_name {
        "contextra-crypto" => Some(70.0),
        "contextra-store" => Some(70.0),
        "contextra-checkpoint" => Some(70.0),
        // Tier 2/3 (Spec: "bewusst niedriger") — kein hartes Gate in diesem PR,
        // nur Tier 0/1 wird durchgesetzt, um Scope-Kriechen zu vermeiden
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MutationGateReport {
    pub crate_name: String,
    pub total_mutants: u64,
    pub caught_mutants: u64,
    pub score_pct: f64,
    pub required_threshold_pct: Option<f64>,
    pub informational: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MutationGateError {
    NoHistoryFound(String),
    ThresholdNotMet {
        crate_name: String,
        actual_pct: f64,
        required_pct: f64,
        caught: u64,
        total: u64,
    },
    IoError(String),
    ParseError(String),
}

impl fmt::Display for MutationGateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MutationGateError::NoHistoryFound(krate) => {
                write!(
                    f,
                    "No mutation score history found for crate '{}' in docs/mutation_score_history.jsonl",
                    krate
                )
            }
            MutationGateError::ThresholdNotMet {
                crate_name,
                actual_pct,
                required_pct,
                caught,
                total,
            } => {
                write!(
                    f,
                    "Mutation score gate failed for '{}': actual score {:.2}% ({}/{}) is below required threshold {:.2}%",
                    crate_name, actual_pct, caught, total, required_pct
                )
            }
            MutationGateError::IoError(e) => write!(f, "IO Error: {}", e),
            MutationGateError::ParseError(e) => write!(f, "Parse Error: {}", e),
        }
    }
}

impl std::error::Error for MutationGateError {}

#[derive(Debug, Deserialize)]
struct HistoryEntry {
    #[serde(rename = "crate")]
    crate_name: String,
    total_mutants: u64,
    caught: u64,
}

pub fn check_gate(crate_name: &str, root: &Path) -> Result<MutationGateReport, MutationGateError> {
    let history_file = root.join("docs/mutation_score_history.jsonl");
    if !history_file.exists() {
        return Err(MutationGateError::NoHistoryFound(crate_name.to_string()));
    }

    let file = fs::File::open(&history_file).map_err(|e| {
        MutationGateError::IoError(format!(
            "Failed to open {}: {}",
            history_file.display(),
            e
        ))
    })?;
    let reader = BufReader::new(file);

    let mut last_matching: Option<HistoryEntry> = None;

    for (line_num, line_res) in reader.lines().enumerate() {
        let line = line_res.map_err(|e| {
            MutationGateError::IoError(format!(
                "Failed to read line {} in {}: {}",
                line_num + 1,
                history_file.display(),
                e
            ))
        })?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let entry: HistoryEntry = serde_json::from_str(trimmed).map_err(|e| {
            MutationGateError::ParseError(format!(
                "Invalid JSON on line {} in {}: {}",
                line_num + 1,
                history_file.display(),
                e
            ))
        })?;
        if entry.crate_name == crate_name {
            last_matching = Some(entry);
        }
    }

    let entry = last_matching
        .ok_or_else(|| MutationGateError::NoHistoryFound(crate_name.to_string()))?;

    let score_pct = if entry.total_mutants > 0 {
        (entry.caught as f64 / entry.total_mutants as f64) * 100.0
    } else {
        0.0
    };

    let threshold = get_mutation_threshold_for_crate(crate_name);

    if let Some(req_pct) = threshold {
        if score_pct < req_pct {
            return Err(MutationGateError::ThresholdNotMet {
                crate_name: crate_name.to_string(),
                actual_pct: score_pct,
                required_pct: req_pct,
                caught: entry.caught,
                total: entry.total_mutants,
            });
        }
        Ok(MutationGateReport {
            crate_name: crate_name.to_string(),
            total_mutants: entry.total_mutants,
            caught_mutants: entry.caught,
            score_pct,
            required_threshold_pct: Some(req_pct),
            informational: false,
        })
    } else {
        Ok(MutationGateReport {
            crate_name: crate_name.to_string(),
            total_mutants: entry.total_mutants,
            caught_mutants: entry.caught,
            score_pct,
            required_threshold_pct: None,
            informational: true,
        })
    }
}

pub fn run_check_mutation_score_gate(args: &[String], root: &Path) -> Result<(), String> {
    let mut crate_name = String::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--crate" && i + 1 < args.len() {
            crate_name = args[i + 1].clone();
            i += 1;
        } else if let Some(val) = args[i].strip_prefix("--crate=") {
            crate_name = val.to_string();
        }
        i += 1;
    }

    if crate_name.is_empty() {
        return Err("Missing required argument --crate <name>".to_string());
    }

    match check_gate(&crate_name, root) {
        Ok(report) => {
            if report.informational {
                println!(
                    "ℹ️ Mutation score gate for '{}' is informational (no threshold configured): {}/{} ({:.2}%)",
                    report.crate_name, report.caught_mutants, report.total_mutants, report.score_pct
                );
            } else {
                println!(
                    "✅ Mutation score gate PASSED for '{}': {}/{} ({:.2}%) >= required threshold {:.2}%",
                    report.crate_name,
                    report.caught_mutants,
                    report.total_mutants,
                    report.score_pct,
                    report.required_threshold_pct.unwrap_or(0.0)
                );
            }
            Ok(())
        }
        Err(err) => Err(err.to_string()),
    }
}

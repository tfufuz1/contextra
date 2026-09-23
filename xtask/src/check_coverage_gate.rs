use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub struct CrateCoverageResult {
    pub crate_name: String,
    pub actual_coverage: f64,
    pub threshold: f64,
    pub passed: bool,
}

#[derive(Deserialize)]
struct LlvmCovReport {
    data: Vec<LlvmCovData>,
}

#[derive(Deserialize)]
struct LlvmCovData {
    files: Vec<LlvmCovFile>,
}

#[derive(Deserialize)]
struct LlvmCovFile {
    filename: String,
    summary: LlvmCovSummary,
}

#[derive(Deserialize)]
struct LlvmCovSummary {
    lines: LlvmCovMetric,
}

#[derive(Deserialize)]
struct LlvmCovMetric {
    count: u64,
    covered: u64,
}

pub fn get_threshold_for_crate(crate_name: &str) -> f64 {
    match crate_name {
        "contextra-store" => 80.0,
        "contextra-db" => 75.0,
        "contextra-index" => 75.0,
        "contextra-graph" => 70.0,
        "contextra-text" => 70.0,
        "contextra-core" => 85.0,
        _ => 50.0,
    }
}

#[allow(dead_code)]
pub fn run_check_coverage_gate(root: &Path) -> Result<Vec<CrateCoverageResult>, String> {
    run_check_coverage_gate_file(&root.join("coverage.json"))
}

pub fn run_check_coverage_gate_file(cov_path: &Path) -> Result<Vec<CrateCoverageResult>, String> {
    if !cov_path.exists() {
        return Err(format!(
            "coverage.json not found at {}. Please run llvm-cov first.",
            cov_path.display()
        ));
    }

    let content = fs::read_to_string(cov_path)
        .map_err(|e| format!("Failed to read coverage json {}: {}", cov_path.display(), e))?;

    let report: LlvmCovReport = serde_json::from_str(&content).map_err(|e| {
        format!(
            "Failed to parse coverage json {}: {}",
            cov_path.display(),
            e
        )
    })?;

    let mut crate_stats: BTreeMap<String, (u64, u64)> = BTreeMap::new();

    for data in &report.data {
        for file in &data.files {
            let filename = file.filename.replace('\\', "/");
            let crate_name = if let Some(idx) = filename.find("crates/") {
                let rest = &filename[idx + 7..];
                rest.split('/').next().unwrap_or("unknown")
            } else if filename.starts_with("xtask") {
                "xtask"
            } else {
                continue;
            };

            let entry = crate_stats.entry(crate_name.to_string()).or_insert((0, 0));
            entry.0 += file.summary.lines.count;
            entry.1 += file.summary.lines.covered;
        }
    }

    let mut results = Vec::new();

    for (crate_name, (total, covered)) in crate_stats {
        let coverage = if total == 0 {
            0.0
        } else {
            (covered as f64 / total as f64) * 100.0
        };

        let threshold = get_threshold_for_crate(&crate_name);
        let passed = coverage >= threshold;

        results.push(CrateCoverageResult {
            crate_name,
            actual_coverage: coverage,
            threshold,
            passed,
        });
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_coverage_thresholds() {
        assert_eq!(get_threshold_for_crate("contextra-store"), 80.0);
        assert_eq!(get_threshold_for_crate("contextra-db"), 75.0);
        assert_eq!(get_threshold_for_crate("contextra-index"), 75.0);
        assert_eq!(get_threshold_for_crate("contextra-graph"), 70.0);
        assert_eq!(get_threshold_for_crate("contextra-text"), 70.0);
        assert_eq!(get_threshold_for_crate("contextra-core"), 85.0);
    }

    #[test]
    fn test_parses_coverage_json_correctly() {
        let dir = tempdir().unwrap();
        let cov_file = dir.path().join("coverage.json");
        fs::write(
            &cov_file,
            r#"{
  "data": [
    {
      "files": [
        {
          "filename": "crates/contextra-store/src/lib.rs",
          "summary": {
            "lines": { "count": 100, "covered": 80 }
          }
        },
        {
          "filename": "crates/contextra-db/src/lib.rs",
          "summary": {
            "lines": { "count": 100, "covered": 50 }
          }
        }
      ]
    }
  ]
}"#,
        )
        .unwrap();

        let results = run_check_coverage_gate(dir.path()).unwrap();
        assert_eq!(results.len(), 2);

        let store_res = results
            .iter()
            .find(|r| r.crate_name == "contextra-store")
            .unwrap();
        assert_eq!(store_res.actual_coverage, 80.0);
        assert!(store_res.passed);

        let db_res = results
            .iter()
            .find(|r| r.crate_name == "contextra-db")
            .unwrap();
        assert_eq!(db_res.actual_coverage, 50.0);
        assert!(!db_res.passed);
    }
}

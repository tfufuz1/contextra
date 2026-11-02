//! Einheitlicher Verifikations-Report (JSON und Markdown) für CI und lokale Audits.

use std::fs;
use std::path::Path;

/// Ein einzelnes Testergebnis.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TestResultEntry {
    pub test_name: String,
    pub passed: bool,
    pub duration_ms: f64,
}

/// Eine Zusammenfassung eines Benchmark-Ergebnisses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct BenchmarkSummaryEntry {
    pub id: String,
    pub p50_ns: u128,
    pub p99_ns: u128,
    pub ops_per_sec: f64,
}

/// Aggregierter Verifikations-Report.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct VerifyReport {
    pub tests: Vec<TestResultEntry>,
    pub benchmarks: Vec<BenchmarkSummaryEntry>,
    pub coverage_lines_percent: Option<f64>,
    pub coverage_branches_percent: Option<f64>,
    pub miri_passed: Option<bool>,
    pub loom_passed: Option<bool>,
    pub asan_passed: Option<bool>,
    pub spec_coverage_passed: Option<bool>,
}

impl VerifyReport {
    pub fn add_test_result(&mut self, entry: TestResultEntry) {
        self.tests.push(entry);
    }

    pub fn add_benchmark_summary(&mut self, entry: BenchmarkSummaryEntry) {
        self.benchmarks.push(entry);
    }

    pub fn write_json(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json_str = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        fs::write(path, json_str)
    }

    pub fn write_markdown(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut md = String::new();
        md.push_str("# Contextra Verification Report\n\n");

        md.push_str("## Übersicht\n\n");
        let tests_total = self.tests.len();
        let tests_passed = self.tests.iter().filter(|t| t.passed).count();
        md.push_str(&format!(
            "- **Tests:** {} / {} bestanden\n",
            tests_passed, tests_total
        ));
        if let Some(cov) = self.coverage_lines_percent {
            md.push_str(&format!("- **Coverage (Zeilen):** {:.2} %\n", cov));
        }
        if let Some(miri) = self.miri_passed {
            md.push_str(&format!(
                "- **Miri:** {}\n",
                if miri { "PASS" } else { "FAIL" }
            ));
        }
        if let Some(loom) = self.loom_passed {
            md.push_str(&format!(
                "- **Loom:** {}\n",
                if loom { "PASS" } else { "FAIL" }
            ));
        }
        if let Some(asan) = self.asan_passed {
            md.push_str(&format!(
                "- **ASan:** {}\n",
                if asan { "PASS" } else { "FAIL" }
            ));
        }
        if let Some(spec) = self.spec_coverage_passed {
            md.push_str(&format!(
                "- **Spec Coverage:** {}\n",
                if spec { "PASS" } else { "FAIL" }
            ));
        }

        md.push_str("\n## Benchmarks\n\n");
        md.push_str("| Benchmark-ID | p50 (ns) | p99 (ns) | Durchsatz (ops/s) |\n");
        md.push_str("|---|---|---|---|\n");
        for b in &self.benchmarks {
            md.push_str(&format!(
                "| {} | {} | {} | {:.1} |\n",
                b.id, b.p50_ns, b.p99_ns, b.ops_per_sec
            ));
        }

        fs::write(path, md)
    }
}

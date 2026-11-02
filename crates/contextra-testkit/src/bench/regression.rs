//! Regressionserkennung für Benchmarks gegen Baselines.

use super::harness::BenchmarkResult;
use std::collections::BTreeMap;
use std::path::Path;

/// Konfiguration für Schwellenwerte bei der Regressionsprüfung.
#[derive(Debug, Clone)]
pub struct RegressionConfig {
    /// Maximal zulässige Latenzzunahme in Prozent (Standard: 10.0 %).
    pub latency_threshold_percent: f64,
    /// Maximal zulässiger Durchsatzabfall in Prozent (Standard: 10.0 %).
    pub throughput_threshold_percent: f64,
}

impl Default for RegressionConfig {
    fn default() -> Self {
        Self {
            latency_threshold_percent: 10.0,
            throughput_threshold_percent: 10.0,
        }
    }
}

/// Baseline-Report für den Abgleich.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BaselineReport {
    pub benchmarks: Vec<BenchmarkResult>,
}

impl BaselineReport {
    /// Lädt eine Baseline aus einer JSON-Datei.
    pub fn load_from_file(path: &Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }
}

/// Mögliche Fehler bei festgestellter Leistungsregression.
#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum RegressionError {
    #[error("Latenz-Regression bei {id}: Baseline p50 = {baseline_ns} ns, Aktuell = {current_ns} ns (+{diff_percent:.1} % > {threshold:.1} %)")]
    LatencyRegression {
        id: String,
        baseline_ns: u128,
        current_ns: u128,
        diff_percent: f64,
        threshold: f64,
    },
    #[error("Durchsatz-Regression bei {id}: Baseline = {baseline_ops:.1} ops/s, Aktuell = {current_ops:.1} ops/s (-{diff_percent:.1} % > {threshold:.1} %)")]
    ThroughputRegression {
        id: String,
        baseline_ops: f64,
        current_ops: f64,
        diff_percent: f64,
        threshold: f64,
    },
}

/// Vergleicht aktuelle Benchmark-Ergebnisse mit einer Baseline.
pub fn check_regression(
    baseline: &BaselineReport,
    current: &[BenchmarkResult],
    config: &RegressionConfig,
) -> Result<(), Vec<RegressionError>> {
    let mut baseline_map: BTreeMap<&str, &BenchmarkResult> = BTreeMap::new();
    for b in &baseline.benchmarks {
        baseline_map.insert(&b.id, b);
    }

    let mut errors = Vec::new();

    for curr in current {
        if let Some(base) = baseline_map.get(curr.id.as_str()) {
            let base_p50 = base.p50.as_nanos();
            let curr_p50 = curr.p50.as_nanos();

            if base_p50 > 0 && curr_p50 > base_p50 {
                let diff_pct = ((curr_p50 - base_p50) as f64 / base_p50 as f64) * 100.0;
                if diff_pct > config.latency_threshold_percent {
                    errors.push(RegressionError::LatencyRegression {
                        id: curr.id.clone(),
                        baseline_ns: base_p50,
                        current_ns: curr_p50,
                        diff_percent: diff_pct,
                        threshold: config.latency_threshold_percent,
                    });
                }
            }

            if base.throughput_ops_per_sec > 0.0
                && curr.throughput_ops_per_sec < base.throughput_ops_per_sec
            {
                let drop_pct = ((base.throughput_ops_per_sec - curr.throughput_ops_per_sec)
                    / base.throughput_ops_per_sec)
                    * 100.0;
                if drop_pct > config.throughput_threshold_percent {
                    errors.push(RegressionError::ThroughputRegression {
                        id: curr.id.clone(),
                        baseline_ops: base.throughput_ops_per_sec,
                        current_ops: curr.throughput_ops_per_sec,
                        diff_percent: drop_pct,
                        threshold: config.throughput_threshold_percent,
                    });
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

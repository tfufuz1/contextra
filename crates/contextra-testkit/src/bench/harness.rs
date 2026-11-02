//! Benchmark-Harness mit deterministischer Seed-Verwaltung, BM-Registry und JSON-Export.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::Duration;

/// Metadaten der ausführenden Plattform (OS, Architektur, CPU, Kernel).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct HardwareMetadata {
    pub os: String,
    pub arch: String,
    pub hostname: String,
}

impl Default for HardwareMetadata {
    fn default() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            hostname: std::env::var("HOSTNAME").unwrap_or_else(|_| "localhost".to_string()),
        }
    }
}

/// Ergebnis einer einzelnen Benchmark-Ausführung nach BM-ID.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct BenchmarkResult {
    /// Benchmark-ID (z. B. `BM-TK-HARNESS-OVERHEAD`, `BM-WRITE`, `BM-GET`).
    pub id: String,
    /// 50. Perzentil (Median) Latenz.
    pub p50: Duration,
    /// 95. Perzentil Latenz.
    pub p95: Duration,
    /// 99. Perzentil Latenz.
    pub p99: Duration,
    /// Gemessener Durchsatz in Ops/Sekunde.
    pub throughput_ops_per_sec: f64,
    /// Zusätzliche domainspezifische Kennzahlen (z. B. Bytes, Chunks, Cache-Hits).
    #[serde(default)]
    pub extra_metrics: BTreeMap<String, f64>,
}

/// Sammel-Harness für Benchmark-Ergebnisse eines Crates.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkHarness {
    /// Name des Crates (z. B. `contextra-testkit`, `contextra-store`).
    pub crate_name: String,
    /// Fester Seed für Reproduzierbarkeit.
    pub seed: u64,
    /// Plattform- und Hardware-Metadaten.
    pub metadata: HardwareMetadata,
    /// Registrierte Benchmark-Ergebnisse.
    pub results: Vec<BenchmarkResult>,
}

impl BenchmarkHarness {
    /// Initialisiert ein neues Harness mit festem Seed.
    pub fn new(crate_name: impl Into<String>, seed: u64) -> Self {
        Self {
            crate_name: crate_name.into(),
            seed,
            metadata: HardwareMetadata::default(),
            results: Vec::new(),
        }
    }

    /// Registriert ein Benchmark-Ergebnis.
    pub fn record(&mut self, result: BenchmarkResult) {
        self.results.push(result);
    }

    /// Schreibt den Benchmark-Report als JSON nach `path`.
    pub fn write_json(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json_str = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        fs::write(path, json_str)
    }
}

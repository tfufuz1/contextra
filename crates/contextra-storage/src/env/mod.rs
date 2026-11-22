//! # Env Modul
//!
//! Produktive Umgebungskomponenten: Dateisystem, Uhrzeit, Entropie, Laufzeit und Ressourcenverwaltung.

pub mod atomic;
pub mod clock;
pub mod entropy;
pub mod governor;
pub mod metrics;
pub mod real_fs;
pub mod runtime;

pub use atomic::write_atomic;
pub use clock::RealClock;
pub use entropy::RealEntropy;
pub use governor::{FileQuota, Governor, InstanceBudget, InstanceFileQuota, TokenBucket};
pub use metrics::{MetricsAdapter, NoopMetrics};
pub use real_fs::RealFs;
pub use runtime::{Lane, LaneBusyError, Runtime};

pub use contextra_storage_ports::FsPolicy;
use contextra_storage_ports::{Clock, Entropy, Fs, PressureSource};
use std::sync::Arc;

/// Prüft, ob ein Dateisystem die Anforderungen einer Richtlinie erfüllt (INV-FS-1).
pub fn check_fs_policy(
    policy: FsPolicy,
    caps: &contextra_storage_ports::FsCaps,
) -> std::io::Result<()> {
    caps.verify_policy(policy)
}

/// Bündelung aller Umgebungskomponenten gemäß SPEC §4.
#[derive(Clone)]
pub struct Env {
    /// Dateisystem-Port.
    pub fs: Arc<dyn Fs>,
    /// Uhrzeit-Port.
    pub clock: Arc<dyn Clock>,
    /// Entropie-Port.
    pub entropy: Arc<dyn Entropy>,
    /// Geteilte Laufzeit.
    pub runtime: Arc<Runtime>,
    /// Ressourcen-Governor.
    pub governor: Arc<Governor>,
    /// Metriken-Adapter.
    pub metrics: Arc<MetricsAdapter>,
    /// Optionaler Systemdruck-Eingang.
    pub pressure: Option<Arc<dyn PressureSource>>,
}

impl Env {
    /// Erstellt eine Produktivumgebung mit realem Dateisystem und Ports.
    ///
    /// Öffnet nur, wenn `root_fs` die Anforderungen von `policy` erfüllt; sonst `Unsupported` (INV-FS-1).
    pub fn real(root_fs: Arc<dyn Fs>, policy: FsPolicy) -> std::io::Result<Self> {
        check_fs_policy(policy, &root_fs.caps())?;
        let clock = Arc::new(RealClock::new());
        let entropy = Arc::new(RealEntropy);
        let runtime = Arc::new(Runtime::new(4));
        let governor = Arc::new(Governor::new(
            Arc::clone(&clock) as Arc<dyn Clock>,
            None,
            512,
            64 * 1024 * 1024,
            100 * 1024 * 1024,
            100 * 1024 * 1024,
        ));
        let metrics = Arc::new(MetricsAdapter::noop());

        Ok(Self {
            fs: root_fs,
            clock,
            entropy,
            runtime,
            governor,
            metrics,
            pressure: None,
        })
    }
}

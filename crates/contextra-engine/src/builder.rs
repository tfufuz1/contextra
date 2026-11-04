//! EngineBuilder für die Contextra-Engine (SPEC §5.7).

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use contextra_ports::{Clock, Rng, Vfs};
use contextra_types::{ContextraError, Result};
use crate::instance::Contextra;

/// Konfigurations- und Initialisierungs-Builder für Contextra nach SPEC §5.7.
pub struct EngineBuilder {
    pub(crate) data_dir: Option<PathBuf>,
    pub(crate) passphrase: Option<Vec<u8>>,
    pub(crate) erase_sla: Option<Duration>,
    pub(crate) backup_retention: Option<Duration>,
    pub(crate) runtime: Option<tokio::runtime::Handle>,
    pub(crate) vfs: Option<Arc<dyn Vfs>>,
    pub(crate) clock: Option<Arc<dyn Clock>>,
    pub(crate) rng: Option<Arc<dyn Rng>>,
    pub(crate) batch_window: Option<Duration>,
    pub(crate) erase_return_timeout: Duration,
    pub(crate) history_horizon: Duration,
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineBuilder {
    /// Erzeugt einen neuen leeren `EngineBuilder`.
    pub fn new() -> Self {
        Self {
            data_dir: None,
            passphrase: None,
            erase_sla: None,
            backup_retention: None,
            runtime: None,
            vfs: None,
            clock: None,
            rng: None,
            batch_window: None,
            erase_return_timeout: Duration::from_secs(60),
            history_horizon: Duration::from_secs(24 * 3600),
        }
    }

    /// Setzt das Datenverzeichnis (Pflicht).
    pub fn data_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.data_dir = Some(path.into());
        self
    }

    /// Setzt die Master-Passphrase (Pflicht, ≥ 12 Bytes).
    pub fn passphrase(mut self, passphrase: impl AsRef<[u8]>) -> Self {
        self.passphrase = Some(passphrase.as_ref().to_vec());
        self
    }

    /// Setzt das Lösch-SLA (Pflicht, 1 min bis 30 Tage).
    pub fn erase_sla(mut self, sla: Duration) -> Self {
        self.erase_sla = Some(sla);
        self
    }

    /// Setzt die Aufbewahrungsdauer für Backups (Pflicht, 1 Tag bis 30 Jahre).
    pub fn backup_retention(mut self, retention: Duration) -> Self {
        self.backup_retention = Some(retention);
        self
    }

    /// Setzt das Tokio-Runtime-Handle (Pflicht, SPEC §5.4).
    pub fn runtime(mut self, handle: tokio::runtime::Handle) -> Self {
        self.runtime = Some(handle);
        self
    }

    /// Setzt das VFS (Pflicht, SPEC D-04).
    pub fn vfs(mut self, vfs: Arc<dyn Vfs>) -> Self {
        self.vfs = Some(vfs);
        self
    }

    /// Setzt die Systemuhr (Pflicht).
    pub fn clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = Some(clock);
        self
    }

    /// Setzt den Zufallszahlengenerator (Pflicht).
    pub fn rng(mut self, rng: Arc<dyn Rng>) -> Self {
        self.rng = Some(rng);
        self
    }

    /// Setzt das Batch-Fenster für Purges (Standard: 60 s, muss ≤ erase_sla / 2 sein, D-39).
    pub fn batch_window(mut self, window: Duration) -> Self {
        self.batch_window = Some(window);
        self
    }

    /// Setzt das Timeout für Erase-Rückgaben (Standard 60 s, 1 s bis 10 min).
    pub fn erase_return_timeout(mut self, timeout: Duration) -> Self {
        self.erase_return_timeout = timeout;
        self
    }

    /// Setzt den Historien-Horizont für Versionen (Standard 24 h, 0 bis 365 d).
    pub fn history_horizon(mut self, horizon: Duration) -> Self {
        self.history_horizon = horizon;
        self
    }

    /// Validiert alle Pflichtparameter und öffnet die Datenbankinstanz (SPEC §5.7, T-CFG-3).
    pub async fn open(self) -> Result<Contextra> {
        let data_dir = self.data_dir.ok_or_else(|| {
            ContextraError::InvalidInput("Pflichtparameter data_dir fehlt".to_string())
        })?;

        let passphrase = self.passphrase.ok_or_else(|| {
            ContextraError::InvalidInput("Pflichtparameter passphrase fehlt".to_string())
        })?;
        if passphrase.len() < 12 {
            return Err(ContextraError::InvalidInput(
                "Passphrase muss mindestens 12 Bytes lang sein (SPEC §5.7)".to_string(),
            ));
        }

        let erase_sla = self.erase_sla.ok_or_else(|| {
            ContextraError::InvalidInput("Pflichtparameter erase_sla fehlt".to_string())
        })?;
        let min_sla = Duration::from_secs(60);
        let max_sla = Duration::from_secs(30 * 24 * 3600);
        if erase_sla < min_sla || erase_sla > max_sla {
            return Err(ContextraError::InvalidInput(
                "erase_sla muss zwischen 1 Minute und 30 Tagen liegen".to_string(),
            ));
        }

        let backup_retention = self.backup_retention.ok_or_else(|| {
            ContextraError::InvalidInput("Pflichtparameter backup_retention fehlt (SPEC §5.7, D-74)".to_string())
        })?;
        let min_ret = Duration::from_secs(24 * 3600);
        let max_ret = Duration::from_secs(30 * 365 * 24 * 3600);
        if backup_retention < min_ret || backup_retention > max_ret {
            return Err(ContextraError::InvalidInput(
                "backup_retention muss zwischen 1 Tag und 30 Jahren liegen".to_string(),
            ));
        }

        let runtime = self.runtime.ok_or_else(|| {
            ContextraError::InvalidInput("Pflichtparameter runtime fehlt (SPEC §5.4)".to_string())
        })?;

        let vfs = self.vfs.ok_or_else(|| {
            ContextraError::InvalidInput("Pflichtparameter vfs fehlt".to_string())
        })?;

        let clock = self.clock.ok_or_else(|| {
            ContextraError::InvalidInput("Pflichtparameter clock fehlt".to_string())
        })?;

        let rng = self.rng.ok_or_else(|| {
            ContextraError::InvalidInput("Pflichtparameter rng fehlt".to_string())
        })?;

        let batch_window = self.batch_window.unwrap_or_else(|| Duration::from_secs(60));
        if batch_window > erase_sla / 2 {
            return Err(ContextraError::InvalidInput(
                "batch_window darf höchstens erase_sla / 2 betragen (SPEC D-39, T-CFG-3)".to_string(),
            ));
        }

        if self.erase_return_timeout < Duration::from_secs(1) || self.erase_return_timeout > Duration::from_secs(600) {
            return Err(ContextraError::InvalidInput(
                "erase_return_timeout muss zwischen 1 s und 10 min liegen".to_string(),
            ));
        }

        if self.history_horizon > Duration::from_secs(365 * 24 * 3600) {
            return Err(ContextraError::InvalidInput(
                "history_horizon darf höchstens 365 Tage betragen".to_string(),
            ));
        }

        crate::startup::execute_startup_sequence(data_dir.clone(), vfs.clone(), clock.clone(), rng.clone()).await?;

        Contextra::new_instance(
            data_dir,
            passphrase,
            erase_sla,
            backup_retention,
            runtime,
            vfs,
            clock,
            rng,
            batch_window,
        )
    }
}

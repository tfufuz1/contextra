//! Instanz-Koordination von Contextra (SPEC §8.13, D-90).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use contextra_ports::{Clock, Rng, Vfs};
use contextra_types::{Result, TenantId};
use crate::collection::{Collection, CollectionOptions};
use crate::health::{EngineState, HealthReport};

/// Zentrales Instanz-Handle von Contextra.
pub struct Contextra {
    pub(crate) data_dir: PathBuf,
    pub(crate) passphrase: Vec<u8>,
    pub(crate) erase_sla: Duration,
    pub(crate) backup_retention: Duration,
    pub(crate) runtime: tokio::runtime::Handle,
    pub(crate) vfs: Arc<dyn Vfs>,
    pub(crate) clock: Arc<dyn Clock>,
    pub(crate) rng: Arc<dyn Rng>,
    pub(crate) batch_window: Duration,
    pub(crate) closed: AtomicBool,
}

impl Contextra {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_instance(
        data_dir: PathBuf,
        passphrase: Vec<u8>,
        erase_sla: Duration,
        backup_retention: Duration,
        runtime: tokio::runtime::Handle,
        vfs: Arc<dyn Vfs>,
        clock: Arc<dyn Clock>,
        rng: Arc<dyn Rng>,
        batch_window: Duration,
    ) -> Result<Self> {
        Ok(Self {
            data_dir,
            passphrase,
            erase_sla,
            backup_retention,
            runtime,
            vfs,
            clock,
            rng,
            batch_window,
            closed: AtomicBool::new(false),
        })
    }

    /// Datenverzeichnis der Instanz.
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// Lösch-SLA der Instanz.
    pub fn erase_sla(&self) -> Duration {
        self.erase_sla
    }

    /// Backup-Retention der Instanz.
    pub fn backup_retention(&self) -> Duration {
        self.backup_retention
    }

    /// Batch-Fenster der Instanz.
    pub fn batch_window(&self) -> Duration {
        self.batch_window
    }

    /// Runtime-Handle der Instanz.
    pub fn runtime_handle(&self) -> &tokio::runtime::Handle {
        &self.runtime
    }

    /// VFS der Instanz.
    pub fn vfs(&self) -> Arc<dyn Vfs> {
        self.vfs.clone()
    }

    /// Systemuhr der Instanz.
    pub fn clock(&self) -> Arc<dyn Clock> {
        self.clock.clone()
    }

    /// Zufallszahlengenerator der Instanz.
    pub fn rng(&self) -> Arc<dyn Rng> {
        self.rng.clone()
    }

    /// Passphrase der Instanz (Bytes).
    pub fn passphrase(&self) -> &[u8] {
        &self.passphrase
    }

    /// Öffnet oder erstellt eine Collection für den angegebenen Mandanten.
    pub async fn collection_for_tenant(
        &self,
        name: &str,
        tenant_id: TenantId,
        opts: CollectionOptions,
    ) -> Result<Arc<Collection>> {
        let coll = Collection::new_collection(name, tenant_id, opts);
        Ok(Arc::new(coll))
    }

    /// Liefert einen schnellen Gesundheitsbericht ohne blockierende Datei-I/O (D-64).
    pub fn health(&self) -> HealthReport {
        HealthReport {
            state: if self.closed.load(Ordering::SeqCst) {
                EngineState::Degraded
            } else {
                EngineState::Ok
            },
            ..Default::default()
        }
    }

    /// Schließt die Instanz geordnet (SPEC §14.3).
    pub async fn close(&self) -> Result<()> {
        self.closed.store(true, Ordering::SeqCst);
        Ok(())
    }
}

impl Drop for Contextra {
    fn drop(&mut self) {
        // Drop blockiert nie (SPEC §5.4, §14.3)
        self.closed.store(true, Ordering::SeqCst);
    }
}

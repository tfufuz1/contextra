//! contextra-storage: Speicherkern. Fassade und Modulwurzel (SP-FACADE).
#![forbid(unsafe_code)]

pub use contextra_storage_ports as ports;

pub mod catalog;
pub mod checkpoint;
pub mod codec;
pub mod config;
pub mod engine;
pub mod env;
pub mod erase;
pub mod keys;
pub mod maint;
pub mod metrics;
pub mod recovery;
pub mod spaces;
pub mod store;
pub mod sync;
#[cfg(feature = "testing")]
pub mod testing;

use config::Config;
use ports::Error;
use std::path::PathBuf;
use std::sync::Arc;

/// Öffentliche Fassade. Rümpfe folgen den Besitzer-SPs; bis dahin NotImplemented.
pub struct Storage {
    _fs: Arc<dyn ports::traits::Fs>,
    catalog: catalog::file::CatalogFile,
}

impl Storage {
    /// Validates configuration before opening the durable Catalog.
    pub fn open(config: Config) -> Result<Storage, Error> {
        config.validate()?;
        let fs = config.fs.ok_or(Error::Limit { limit: "storage filesystem is required" })?;
        let root: PathBuf = config.root.ok_or(Error::Limit { limit: "storage root is required" })?;
        let authenticated = config.catalog_integrity == catalog::CatalogIntegrity::Authenticated;
        if authenticated && (config.catalog_key.is_none() || config.catalog_mac.is_none()) {
            return Err(Error::Limit { limit: "catalog key and MAC must be supplied together" });
        }
        if !authenticated && (config.catalog_key.is_some() || config.catalog_mac.is_some()) {
            return Err(Error::Limit { limit: "catalog keys require authenticated integrity mode" });
        }
        let catalog = catalog::file::CatalogFile::open_current(
            fs.as_ref(),
            &root,
            catalog::file::OpenMode::ReadWrite,
            config.catalog_integrity,
            config.catalog_key.as_deref(),
            config.catalog_mac.as_deref(),
        )?;
        Ok(Storage { _fs: fs, catalog })
    }

    pub fn catalog(&self) -> &catalog::state::CatalogState { self.catalog.state() }
}

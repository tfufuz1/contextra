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

/// Öffentliche Fassade. Rümpfe folgen den Besitzer-SPs; bis dahin NotImplemented.
pub struct Storage {
    _private: (),
}

impl Storage {
    /// SP-CATALOG (WP-06.2). Validiert zuerst die Konfiguration.
    pub fn open(config: Config) -> Result<Storage, Error> {
        config.validate()?;
        Err(Error::NotImplemented { sp: "CATALOG open" })
    }
}

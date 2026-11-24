//! Storage Bridge: Adapter for synchronous and legacy callers (SP-BRIDGE 18, SPEC §13).
#![forbid(unsafe_code)]

use contextra_storage::Storage;
use contextra_storage_ports::Error;

pub struct StorageBridge {
    storage: Storage,
}

impl StorageBridge {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub fn inner(&self) -> &Storage {
        &self.storage
    }

    pub fn catalog_state(&self) -> &contextra_storage::catalog::state::CatalogState {
        self.storage.catalog()
    }

    pub fn check_healthy(&self) -> Result<(), Error> {
        Ok(())
    }
}

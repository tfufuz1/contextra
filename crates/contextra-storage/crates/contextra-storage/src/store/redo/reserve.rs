//! Durabler Writer für vorallokierte Emergency-Redo-Reserven (INV-LOG-7).

use contextra_storage_ports::traits::{File, FileRole, IoCtx};
use contextra_storage_ports::Error;

const IO: IoCtx = IoCtx { role: FileRole::Redo, purpose: "redo_reserve" };

pub struct ReserveWriter {
    file: Box<dyn File>,
    capacity: u64,
    used: u64,
    poisoned: bool,
}

impl ReserveWriter {
    /// Initialisiert die Reserve mit Nullen und macht die Vorallokation durabel.
    /// Der Aufrufer erzeugt/öffnet exklusiv die zugehörige Reservendatei.
    pub fn initialize(file: Box<dyn File>, capacity: u64) -> Result<Self, Error> {
        if capacity == 0 { return Err(Error::Limit { limit: "redo reserve capacity" }); }
        file.preallocate(capacity, IO)?;
        file.sync_all(IO)?;
        Ok(Self { file, capacity, used: 0, poisoned: false })
    }

    pub fn remaining(&self) -> u64 { self.capacity.saturating_sub(self.used) }

    /// Emergency-Datensatz durabel anhängen; Fehler vergiften diesen Handle.
    pub fn append_durable(&mut self, record: &[u8]) -> Result<u64, Error> {
        if self.poisoned { return Err(Error::Poisoned { cause: "redo reserve is poisoned" }); }
        let length = u64::try_from(record.len()).map_err(|_| Error::Limit { limit: "redo reserve record" })?;
        let next = self.used.checked_add(length).ok_or(Error::Limit { limit: "redo reserve offset" })?;
        if next > self.capacity { return Err(Error::DiskFull); }
        if let Err(error) = self.file.append_prealloc(record, IO) {
            if matches!(error, Error::Io { .. } | Error::Poisoned { .. }) {
                self.poisoned = true;
                return Err(Error::Poisoned { cause: "redo reserve append failed" });
            }
            return Err(error);
        }
        if self.file.sync_data(IO).is_err() {
            self.poisoned = true;
            return Err(Error::Poisoned { cause: "redo reserve sync failed" });
        }
        self.used = next;
        Ok(self.used)
    }
}

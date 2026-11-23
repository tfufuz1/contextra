//! In-memory monotone Anchor for deterministic tests and recovery models.

use contextra_storage_ports::traits::{Anchor, AnchorStrength, AnchorValue};
use contextra_storage_ports::Error;
use std::sync::Mutex;

pub struct MemAnchor { value: Mutex<AnchorValue> }

impl MemAnchor {
    pub fn new(initial: AnchorValue) -> Self { Self { value: Mutex::new(initial) } }
}

impl Default for MemAnchor {
    fn default() -> Self { Self::new(AnchorValue::default()) }
}

impl Anchor for MemAnchor {
    fn read(&self) -> Result<AnchorValue, Error> {
        self.value.lock().map(|value| *value)
            .map_err(|_| Error::Poisoned { cause: "memory anchor poisoned" })
    }

    fn advance(&self, expect: AnchorValue, next: AnchorValue) -> Result<(), Error> {
        let mut current = self.value.lock().map_err(|_| Error::Poisoned { cause: "memory anchor poisoned" })?;
        if *current != expect { return Err(Error::CasLost); }
        if next.erase_epoch < expect.erase_epoch
            || next.ledger_head < expect.ledger_head
            || next.catalog_epoch < expect.catalog_epoch
        {
            return Err(Error::Limit { limit: "anchor value must be monotone" });
        }
        *current = next;
        Ok(())
    }

    fn strength(&self) -> AnchorStrength { AnchorStrength::Weak }
}

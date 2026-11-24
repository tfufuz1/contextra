//! Weight-budgeted pagination (SPEC §7, SP-READ).
#![forbid(unsafe_code)]

use contextra_storage_ports::{Error, Stamp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageBudget {
    pub max_items: usize,
    pub max_weight: u64,
    pub max_bytes: usize,
}

impl Default for PageBudget {
    fn default() -> Self {
        Self {
            max_items: 1024,
            max_weight: 65_536,
            max_bytes: 4 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageEntry {
    pub key: Vec<u8>,
    pub header: Vec<u8>,
    pub body: Option<Vec<u8>>,
    pub weight: u64,
    pub stamp: Stamp,
}

pub struct PageCollector {
    budget: PageBudget,
    entries: Vec<PageEntry>,
    total_weight: u64,
    total_bytes: usize,
}

impl PageCollector {
    pub fn new(budget: PageBudget) -> Self {
        Self {
            budget,
            entries: Vec::new(),
            total_weight: 0,
            total_bytes: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn total_weight(&self) -> u64 {
        self.total_weight
    }

    pub fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    /// Checks if a candidate entry fits within the remaining page budget.
    pub fn can_accept(&self, entry: &PageEntry) -> bool {
        if self.entries.len() >= self.budget.max_items {
            return false;
        }
        let entry_bytes = entry.key.len()
            + entry.header.len()
            + entry.body.as_ref().map(|b| b.len()).unwrap_or(0);

        if !self.entries.is_empty() {
            if self.total_weight.saturating_add(entry.weight) > self.budget.max_weight {
                return false;
            }
            if self.total_bytes.saturating_add(entry_bytes) > self.budget.max_bytes {
                return false;
            }
        }
        true
    }

    /// Appends an entry if it fits. If the collector was empty, always accepts at least
    /// one entry to prevent deadlocks on single large records.
    pub fn try_push(&mut self, entry: PageEntry) -> Result<bool, Error> {
        if !self.can_accept(&entry) {
            return Ok(false);
        }
        let entry_bytes = entry.key.len()
            + entry.header.len()
            + entry.body.as_ref().map(|b| b.len()).unwrap_or(0);

        self.total_weight = self.total_weight.saturating_add(entry.weight);
        self.total_bytes = self.total_bytes.saturating_add(entry_bytes);
        self.entries.push(entry);
        Ok(true)
    }

    /// Finalizes the page, returning the gathered entries and the continuation key if truncated.
    pub fn finish(self, next_key: Option<Vec<u8>>) -> (Vec<PageEntry>, Option<Vec<u8>>) {
        (self.entries, next_key)
    }
}

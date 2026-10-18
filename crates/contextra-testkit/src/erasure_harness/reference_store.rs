// FILE-CONTEXT
// STAND: 2026-10-15
// ZWECK: In-Memory BTreeMap Referenzmodell fuer Contextra Store Operationen (Put, Delete, Purge).
// INVARIANTEN: #![forbid(unsafe_code)], Determinismus, exakt selftest-konform.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyState {
    Active,
    Tombstone,
    Purged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Put(Vec<u8>, Vec<u8>),
    Delete(Vec<u8>),
    Purge(Vec<u8>),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReferenceStore {
    entries: BTreeMap<Vec<u8>, (Vec<u8>, KeyState)>,
}

impl ReferenceStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.active_count() == 0
    }

    pub fn put(&mut self, key: impl AsRef<[u8]>, val: impl AsRef<[u8]>) {
        self.entries.insert(
            key.as_ref().to_vec(),
            (val.as_ref().to_vec(), KeyState::Active),
        );
    }

    pub fn delete(&mut self, key: impl AsRef<[u8]>) {
        let k = key.as_ref().to_vec();
        self.entries.insert(k, (Vec::new(), KeyState::Tombstone));
    }

    pub fn purge(&mut self, key: impl AsRef<[u8]>) -> usize {
        let k = key.as_ref().to_vec();
        let existed = self.entries.contains_key(&k);
        self.entries.insert(k, (Vec::new(), KeyState::Purged));
        if existed {
            1
        } else {
            0
        }
    }

    pub fn purge_prefix(&mut self, prefix: &[u8]) -> usize {
        let matching_keys: Vec<_> = self
            .entries
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        let count = matching_keys.len();
        for k in matching_keys {
            self.entries.insert(k, (Vec::new(), KeyState::Purged));
        }
        count
    }

    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        match self.entries.get(key) {
            Some((val, KeyState::Active)) => Some(val.as_slice()),
            _ => None,
        }
    }

    pub fn state_of(&self, key: &[u8]) -> Option<&KeyState> {
        self.entries.get(key).map(|(_, state)| state)
    }

    pub fn active_count(&self) -> usize {
        self.entries
            .values()
            .filter(|(_, st)| *st == KeyState::Active)
            .count()
    }

    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }

    pub fn scan_prefix(&self, prefix: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.entries
            .iter()
            .filter(|(k, (_, st))| k.starts_with(prefix) && *st == KeyState::Active)
            .map(|(k, (v, _))| (k.clone(), v.clone()))
            .collect()
    }

    pub fn active_snapshot(&self) -> BTreeMap<Vec<u8>, Vec<u8>> {
        self.entries
            .iter()
            .filter(|(_, (_, st))| *st == KeyState::Active)
            .map(|(k, (v, _))| (k.clone(), v.clone()))
            .collect()
    }

    pub fn apply(&mut self, op: Op) {
        match op {
            Op::Put(k, v) => self.put(k, v),
            Op::Delete(k) => self.delete(k),
            Op::Purge(k) => {
                self.purge(k);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_store_operations() {
        let mut store = ReferenceStore::new();
        assert!(store.is_empty());

        store.put(b"k1", b"v1");
        assert_eq!(store.get(b"k1"), Some(&b"v1"[..]));
        assert_eq!(store.state_of(b"k1"), Some(&KeyState::Active));

        store.delete(b"k1");
        assert_eq!(store.get(b"k1"), None);
        assert_eq!(store.state_of(b"k1"), Some(&KeyState::Tombstone));

        store.purge(b"k1");
        assert_eq!(store.state_of(b"k1"), Some(&KeyState::Purged));
    }
}

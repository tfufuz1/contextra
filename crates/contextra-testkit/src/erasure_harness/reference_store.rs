// FILE-CONTEXT
// STAND: 2026-10-15
// ZWECK: BTreeMap-basierte Referenzimplementierung fuer Loeschtests (ReferenceStore).
// INVARIANTEN: Rein std, deterministisch, bildet Put, Delete, Purge exakt ab.

use std::collections::BTreeMap;

/// Representing the state of a key in the reference store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyState {
    /// Key is present and active with value.
    Active(Vec<u8>),
    /// Key has been deleted logically (tombstone).
    Tombstone,
    /// Key has been purged physically (purged / forgotten).
    Purged,
}

/// BTreeMap-based reference store model for verifying erasure and storage invariants.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReferenceStore {
    store: BTreeMap<Vec<u8>, KeyState>,
}

impl ReferenceStore {
    /// Creates a new empty `ReferenceStore`.
    pub fn new() -> Self {
        Self {
            store: BTreeMap::new(),
        }
    }

    /// Sets or updates a key with a value (`Put`).
    pub fn put(&mut self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) {
        self.store.insert(
            key.as_ref().to_vec(),
            KeyState::Active(value.as_ref().to_vec()),
        );
    }

    /// Marks a key as logically deleted (`Delete` / Tombstone).
    pub fn delete(&mut self, key: impl AsRef<[u8]>) {
        self.store
            .insert(key.as_ref().to_vec(), KeyState::Tombstone);
    }

    /// Marks a key as physically purged (`Purge` / Forgotten).
    pub fn purge(&mut self, key: impl AsRef<[u8]>) {
        self.store.insert(key.as_ref().to_vec(), KeyState::Purged);
    }

    /// Removes a key completely from the reference store map.
    pub fn remove_completely(&mut self, key: impl AsRef<[u8]>) -> Option<KeyState> {
        self.store.remove(key.as_ref())
    }

    /// Retrieves active value for `key` if present and active.
    pub fn get(&self, key: impl AsRef<[u8]>) -> Option<&[u8]> {
        match self.store.get(key.as_ref()) {
            Some(KeyState::Active(v)) => Some(v.as_slice()),
            _ => None,
        }
    }

    /// Returns the exact `KeyState` of `key` if present.
    pub fn state_of(&self, key: impl AsRef<[u8]>) -> Option<&KeyState> {
        self.store.get(key.as_ref())
    }

    /// Returns `true` if `key` exists and is in `Active` state.
    pub fn contains_key(&self, key: impl AsRef<[u8]>) -> bool {
        matches!(self.store.get(key.as_ref()), Some(KeyState::Active(_)))
    }

    /// Scans keys starting with `prefix` and returns active key-value pairs.
    pub fn scan_prefix(&self, prefix: impl AsRef<[u8]>) -> Vec<(Vec<u8>, Vec<u8>)> {
        let pref = prefix.as_ref();
        let mut results = Vec::new();
        for (k, state) in self.store.range(pref.to_vec()..) {
            if !k.starts_with(pref) {
                break;
            }
            if let KeyState::Active(ref v) = state {
                results.push((k.clone(), v.clone()));
            }
        }
        results
    }

    /// Returns number of entries in the store (including tombstones/purged markers).
    pub fn total_entries(&self) -> usize {
        self.store.len()
    }

    /// Returns number of active key-value pairs.
    pub fn active_count(&self) -> usize {
        self.store
            .values()
            .filter(|s| matches!(s, KeyState::Active(_)))
            .count()
    }

    /// Returns `true` if no active entries exist.
    pub fn is_empty(&self) -> bool {
        self.active_count() == 0
    }

    /// Clears all entries from the store.
    pub fn clear(&mut self) {
        self.store.clear();
    }

    /// Returns a snapshot map of active key-value pairs.
    pub fn active_snapshot(&self) -> BTreeMap<Vec<u8>, Vec<u8>> {
        self.store
            .iter()
            .filter_map(|(k, state)| match state {
                KeyState::Active(v) => Some((k.clone(), v.clone())),
                _ => None,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_store_basic_lifecycle() {
        let mut store = ReferenceStore::new();
        assert!(store.is_empty());

        store.put(b"k1", b"v1");
        store.put(b"k2", b"v2");
        assert_eq!(store.get(b"k1"), Some(&b"v1"[..]));
        assert_eq!(store.active_count(), 2);

        store.delete(b"k1");
        assert_eq!(store.get(b"k1"), None);
        assert_eq!(store.state_of(b"k1"), Some(&KeyState::Tombstone));
        assert_eq!(store.active_count(), 1);

        store.purge(b"k1");
        assert_eq!(store.state_of(b"k1"), Some(&KeyState::Purged));

        let scan = store.scan_prefix(b"k");
        assert_eq!(scan, vec![(b"k2".to_vec(), b"v2".to_vec())]);
    }
}

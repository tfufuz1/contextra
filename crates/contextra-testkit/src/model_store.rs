// FILE-CONTEXT
// STAND: 2026-10-22
// ZWECK: Differential oracle ModelStore backed by BTreeMap sequence/transaction history.
// INVARIANTEN: Monotonic MVCC sequence numbers, zero panic, std-only pure Rust.

use crate::reference_model::RefOp;
use std::collections::BTreeMap;

/// MVCC Reference Oracle ModelStore backed by a sequence/transaction history map.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModelStore {
    #[allow(clippy::type_complexity)]
    history: BTreeMap<Vec<u8>, Vec<(u64, Option<Vec<u8>>)>>,
    current_seq: u64,
}

impl ModelStore {
    /// Creates a new empty `ModelStore`.
    pub fn new() -> Self {
        Self {
            history: BTreeMap::new(),
            current_seq: 0,
        }
    }

    /// Returns the current high-water mark sequence number.
    pub fn current_seq(&self) -> u64 {
        self.current_seq
    }

    /// Retrieves the value of `key` as visible at `seq`.
    pub fn get_at(&self, seq: u64, key: &[u8]) -> Option<Vec<u8>> {
        let versions = self.history.get(key)?;
        let latest = versions
            .iter()
            .rev()
            .find(|(version_seq, _)| *version_seq <= seq)?;
        latest.1.clone()
    }

    /// Appends a batch of operations and commits under a new sequence number.
    pub fn apply_batch(&mut self, ops: Vec<RefOp>) -> u64 {
        if ops.is_empty() {
            return self.current_seq;
        }

        self.current_seq += 1;
        let commit_seq = self.current_seq;

        for op in ops {
            match op {
                RefOp::Put { key, value } => {
                    self.history
                        .entry(key)
                        .or_default()
                        .push((commit_seq, Some(value)));
                }
                RefOp::Delete { key } | RefOp::Forget { key } => {
                    self.history
                        .entry(key)
                        .or_default()
                        .push((commit_seq, None));
                }
            }
        }

        commit_seq
    }

    /// Purges MVCC history up to `as_of_seq` for keys matching `scope_prefix`.
    pub fn purge_scope(&mut self, as_of_seq: u64, scope_prefix: &[u8]) {
        let keys_to_process: Vec<Vec<u8>> = self
            .history
            .keys()
            .filter(|k| k.starts_with(scope_prefix))
            .cloned()
            .collect();

        for key in keys_to_process {
            if let Some(versions) = self.history.get_mut(&key) {
                let mut newer_versions: Vec<(u64, Option<Vec<u8>>)> = Vec::new();
                let mut latest_older_version: Option<(u64, Option<Vec<u8>>)> = None;

                for (seq, val) in versions.drain(..) {
                    if seq <= as_of_seq {
                        latest_older_version = Some((seq, val));
                    } else {
                        newer_versions.push((seq, val));
                    }
                }

                let mut new_history = Vec::new();
                if let Some(older) = latest_older_version {
                    if older.1.is_some() || !newer_versions.is_empty() {
                        new_history.push(older);
                    }
                }
                new_history.extend(newer_versions);

                if new_history.is_empty() {
                    self.history.remove(&key);
                } else {
                    *versions = new_history;
                }
            }
        }
    }
}

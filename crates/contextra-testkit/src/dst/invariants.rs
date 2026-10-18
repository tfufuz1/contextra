// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Post-recovery invariant checking for DST.
// INVARIANTEN: 1. Keine verlorenen Acks, 2. Kette gültig, 3. Nach forget nichts Gelöschtes lesbar.

use crate::dst::engine::DstEngine;
use std::collections::{BTreeMap, HashSet};

/// In-memory ground-truth reference model for acknowledged operations.
#[derive(Debug, Clone, Default)]
pub struct DstModel {
    ack_data: BTreeMap<Vec<u8>, Vec<u8>>,
    forgotten_keys: HashSet<Vec<u8>>,
    pending_puts: Vec<(Vec<u8>, Vec<u8>)>,
    pending_deletes: Vec<Vec<u8>>,
    ack_seq: u64,
}

impl DstModel {
    /// Creates a new empty `DstModel`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Stages a Put operation in pending transaction.
    pub fn stage_put(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.pending_puts.push((key, value));
    }

    /// Stages a Delete operation in pending transaction.
    pub fn stage_delete(&mut self, key: Vec<u8>) {
        self.pending_deletes.push(key);
    }

    /// Acknowledges a committed transaction batch.
    pub fn ack_commit(&mut self, seq: u64) {
        for (k, v) in self.pending_puts.drain(..) {
            self.forgotten_keys.remove(&k);
            self.ack_data.insert(k, v);
        }
        for k in self.pending_deletes.drain(..) {
            self.ack_data.remove(&k);
        }
        self.ack_seq = seq;
    }

    /// Acknowledges a Forget operation.
    pub fn ack_forget(&mut self, key: Vec<u8>) {
        self.ack_data.remove(&key);
        self.forgotten_keys.insert(key);
    }

    /// Clears unacknowledged pending transaction staged operations.
    pub fn rollback(&mut self) {
        self.pending_puts.clear();
        self.pending_deletes.clear();
    }

    /// Returns the last acknowledged sequence number.
    pub fn ack_seq(&self) -> u64 {
        self.ack_seq
    }

    /// Returns a reference to the acknowledged key-value map.
    pub fn ack_data(&self) -> &BTreeMap<Vec<u8>, Vec<u8>> {
        &self.ack_data
    }

    /// Returns a reference to the set of forgotten keys.
    pub fn forgotten_keys(&self) -> &HashSet<Vec<u8>> {
        &self.forgotten_keys
    }
}

/// Checker enforcing safety and durability invariants post-recovery.
pub struct InvariantChecker;

impl InvariantChecker {
    /// Verifies all three required DST invariants post-recovery.
    pub fn verify(engine: &DstEngine, model: &DstModel) -> Result<(), String> {
        // 1. Keine verlorenen Acks: Every acknowledged key-value must match in recovered engine
        for (key, expected_val) in model.ack_data() {
            match engine.get(key) {
                Some(actual_val) => {
                    if &actual_val != expected_val {
                        return Err(format!(
                            "INVARIANT VIOLATION (Mismatched Ack Value): Key {:?} expected {:?}, found {:?}",
                            key, expected_val, actual_val
                        ));
                    }
                }
                None => {
                    return Err(format!(
                        "INVARIANT VIOLATION (Lost Ack): Acknowledged key {:?} not found in recovered store!",
                        key
                    ));
                }
            }
        }

        // 2. Kette gültig: Sequence numbers in recovered engine must be >= model's last acked seq
        if engine.last_seq() < model.ack_seq() {
            return Err(format!(
                "INVARIANT VIOLATION (Broken Chain): Recovered seq {} is less than acknowledged seq {}",
                engine.last_seq(),
                model.ack_seq()
            ));
        }

        // 3. Nach forget nichts Gelöschtes lesbar: Shredded / forgotten keys must yield None
        for key in model.forgotten_keys() {
            if let Some(val) = engine.get(key) {
                return Err(format!(
                    "INVARIANT VIOLATION (Forgotten Data Readable): Key {:?} was forgotten but recovered value {:?}",
                    key, val
                ));
            }
        }

        Ok(())
    }
}

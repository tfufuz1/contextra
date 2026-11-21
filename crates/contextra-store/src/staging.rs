//! # Staging-Puffer für Schreibstapel (Ring 1)
//!
//! Hält die noch nicht committeten Operationen je Schreibstapel. Es gibt kein SSI und
//! kein ReadSet (SPEC ADR-06, REM-12): Isolation entsteht allein durch Sequenznummern
//! und den atomaren Commit unter dem Commit-Lock. Alle Grenzen sind hart und liefern
//! `LimitExceeded`, nie ein stilles Abschneiden.

use crate::error::{StoreError, StoreResult};
use contextra_ids::TxId;
use parking_lot::Mutex;
use std::collections::HashMap;

/// Pauschaler Verwaltungsaufwand je gestagter Operation (Bytes) für die Budgetierung.
pub const STAGING_ENTRY_OVERHEAD_BYTES: usize = 32;
/// Maximale Operationen je Schreibstapel.
pub const MAX_OPS_PER_TX: usize = 10_000;
/// Maximale gestagte Bytes je Schreibstapel.
pub const MAX_STAGED_BYTES_PER_TX: usize = 16 * 1024 * 1024;
/// Maximale gestagte Bytes über alle Schreibstapel.
pub const MAX_STAGED_BYTES_TOTAL: usize = 256 * 1024 * 1024;

/// Eine gestagte Operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StagedOp {
    /// Schreibt `value` unter `key`.
    Put { key: Vec<u8>, value: Vec<u8> },
    /// Löscht `key` (Tombstone).
    Delete { key: Vec<u8> },
}

impl StagedOp {
    fn bytes(&self) -> usize {
        match self {
            Self::Put { key, value } => key.len() + value.len() + STAGING_ENTRY_OVERHEAD_BYTES,
            Self::Delete { key } => key.len() + STAGING_ENTRY_OVERHEAD_BYTES,
        }
    }
}

#[derive(Default)]
struct TxEntry {
    ops: Vec<StagedOp>,
    bytes: usize,
}

#[derive(Default)]
struct Inner {
    txs: HashMap<TxId, TxEntry>,
    /// Anzahl aktiver Stapel, die `key` als Put gestaged haben (für `put_if_absent`).
    staged_puts: HashMap<Vec<u8>, u32>,
    total_bytes: usize,
}

/// Begrenzter Staging-Puffer. Alle kritischen Abschnitte sind kurz und enthalten kein `await`.
#[derive(Default)]
pub(crate) struct TxStaging {
    inner: Mutex<Inner>,
}

impl TxStaging {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Stagt eine Operation. Prüft alle Grenzen vor der Mutation (atomar: alles oder nichts).
    pub(crate) fn stage(&self, tx: TxId, op: StagedOp) -> StoreResult<()> {
        self.stage_many(tx, std::iter::once(op))
    }

    /// Stagt mehrere Operationen atomar.
    pub(crate) fn stage_many(
        &self,
        tx: TxId,
        ops: impl IntoIterator<Item = StagedOp>,
    ) -> StoreResult<()> {
        let ops: Vec<StagedOp> = ops.into_iter().collect();
        let add_bytes: usize = ops.iter().map(StagedOp::bytes).sum();
        let mut g = self.inner.lock();
        let (cur_ops, cur_bytes) = g.txs.get(&tx).map_or((0, 0), |e| (e.ops.len(), e.bytes));

        if cur_ops.saturating_add(ops.len()) > MAX_OPS_PER_TX {
            return Err(StoreError::LimitExceeded {
                limit: MAX_OPS_PER_TX,
                context: "Operationen je Schreibstapel".into(),
            });
        }
        if cur_bytes.saturating_add(add_bytes) > MAX_STAGED_BYTES_PER_TX {
            return Err(StoreError::LimitExceeded {
                limit: MAX_STAGED_BYTES_PER_TX,
                context: "gestagte Bytes je Schreibstapel".into(),
            });
        }
        if g.total_bytes.saturating_add(add_bytes) > MAX_STAGED_BYTES_TOTAL {
            return Err(StoreError::LimitExceeded {
                limit: MAX_STAGED_BYTES_TOTAL,
                context: "gestagte Bytes gesamt".into(),
            });
        }

        for op in &ops {
            if let StagedOp::Put { key, .. } = op {
                let c = g.staged_puts.entry(key.clone()).or_insert(0);
                *c = c.saturating_add(1);
            }
        }
        g.total_bytes += add_bytes;
        let entry = g.txs.entry(tx).or_default();
        entry.bytes += add_bytes;
        entry.ops.extend(ops);
        Ok(())
    }

    /// Entnimmt alle Operationen des Stapels in Einfügereihenfolge und gibt sein Budget frei.
    pub(crate) fn drain(&self, tx: TxId) -> Vec<StagedOp> {
        let mut g = self.inner.lock();
        Self::remove_locked(&mut g, tx)
    }

    /// Verwirft den Stapel.
    pub(crate) fn discard(&self, tx: TxId) {
        let mut g = self.inner.lock();
        drop(Self::remove_locked(&mut g, tx));
    }

    /// `true`, wenn irgendein aktiver Stapel `key` als Put gestaged hat.
    pub(crate) fn is_put_staged(&self, key: &[u8]) -> bool {
        self.inner.lock().staged_puts.get(key).is_some_and(|c| *c > 0)
    }

    /// Kopie der gestagten Operationen (nur Tests).
    #[cfg(test)]
    pub(crate) fn peek(&self, tx: TxId) -> Option<Vec<StagedOp>> {
        self.inner.lock().txs.get(&tx).map(|e| e.ops.clone())
    }

    fn remove_locked(g: &mut Inner, tx: TxId) -> Vec<StagedOp> {
        let Some(entry) = g.txs.remove(&tx) else {
            return Vec::new();
        };
        g.total_bytes = g.total_bytes.saturating_sub(entry.bytes);
        for op in &entry.ops {
            if let StagedOp::Put { key, .. } = op {
                if let Some(c) = g.staged_puts.get_mut(key) {
                    *c = c.saturating_sub(1);
                    if *c == 0 {
                        g.staged_puts.remove(key);
                    }
                }
            }
        }
        entry.ops
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put(k: &[u8], v: &[u8]) -> StagedOp {
        StagedOp::Put { key: k.to_vec(), value: v.to_vec() }
    }

    #[test]
    fn drain_preserves_order_and_releases_budget() {
        let s = TxStaging::new();
        let tx = TxId::new(1);
        s.stage(tx, put(b"a", b"1")).unwrap();
        s.stage(tx, StagedOp::Delete { key: b"b".to_vec() }).unwrap();
        assert!(s.is_put_staged(b"a"));
        let ops = s.drain(tx);
        assert_eq!(ops.len(), 2);
        assert_eq!(ops[0], put(b"a", b"1"));
        assert!(!s.is_put_staged(b"a"));
        assert_eq!(s.inner.lock().total_bytes, 0);
    }

    #[test]
    fn op_limit_is_atomic() {
        let s = TxStaging::new();
        let tx = TxId::new(2);
        let batch: Vec<_> = (0..=MAX_OPS_PER_TX).map(|i| put(&i.to_be_bytes(), b"")).collect();
        assert!(matches!(
            s.stage_many(tx, batch),
            Err(StoreError::LimitExceeded { .. })
        ));
        assert!(s.peek(tx).is_none(), "keine Teilwirkung bei Fehler");
        assert_eq!(s.inner.lock().total_bytes, 0);
    }

    #[test]
    fn discard_clears_put_index() {
        let s = TxStaging::new();
        let tx = TxId::new(3);
        s.stage(tx, put(b"k", b"v")).unwrap();
        s.discard(tx);
        assert!(!s.is_put_staged(b"k"));
    }
}

//! Zustandsloser Löschfilter für MemTable, Flush, Merge und Replay (§7.2).

use crate::erasure::gate::GateSnapshot;
use contextra_ports::gate::DiscardRule;
use contextra_types::Seq;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Map von Rohschlüssel auf minimale zu behaltende Sequenznummer (`CollapseVersions`).
pub(crate) type CollapseMap = BTreeMap<Vec<u8>, Seq>;

/// Entscheidung des [`EraseFilter`] für eine einzelne Schlüsselversion.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Decision {
    Keep,
    Drop,
}

/// Kombinierter Filter aus aktiven Gates (`DropKeys`) und Version-Collapse (`CollapseVersions`).
#[derive(Clone)]
pub(crate) struct EraseFilter {
    jobs: Arc<GateSnapshot>,
    collapse: Arc<CollapseMap>,
}

impl EraseFilter {
    pub fn new(jobs: Arc<GateSnapshot>, collapse: Arc<CollapseMap>) -> Self {
        Self { jobs, collapse }
    }

    /// Reine, I/O-freie Entscheidungsfunktion (§7.2).
    ///
    /// Reihenfolge: `DropKeys` vor `CollapseVersions`.
    #[inline]
    pub fn decide(&self, key: &[u8], raw_seq: u64, _tombstone: bool) -> Decision {
        if self.jobs.discard_rule(key, Seq(raw_seq)) == DiscardRule::MayDrop {
            return Decision::Drop;
        }
        if let Some(keep_from) = self.collapse.get(key) {
            if raw_seq < keep_from.0 {
                return Decision::Drop;
            }
        }
        Decision::Keep
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erasure::gate::{GatePhase, JobGate};
    use contextra_ports::erasure::{KeyPattern, KeyScope, Verdict};
    use contextra_types::ErasureId;

    #[test]
    fn test_erase_filter_drop_keys_and_collapse() {
        let scope = Arc::new(
            KeyScope::new(vec![(KeyPattern::Prefix(b"t:1:".to_vec()), Verdict::In)]).unwrap(),
        );
        let snap = Arc::new(GateSnapshot::from_jobs(
            1,
            vec![Arc::new(JobGate {
                id: ErasureId([1; 16]),
                scope,
                docs: Vec::new(),
                up_to: Seq(50),
                phase: GatePhase::Erased,
                reinsert_after_sealed: false,
                sealed: false,
            })],
        ));
        let mut collapse = CollapseMap::new();
        collapse.insert(b"i:term:foo".to_vec(), Seq(80));
        let filter = EraseFilter::new(snap, Arc::new(collapse));

        // DropKeys: <= 50 fälllt (auch Tombstone)
        assert_eq!(filter.decide(b"t:1:doc1", 50, false), Decision::Drop);
        assert_eq!(filter.decide(b"t:1:doc1", 50, true), Decision::Drop);
        assert_eq!(filter.decide(b"t:1:doc1", 51, false), Decision::Keep);

        // CollapseVersions: < 80 fällt
        assert_eq!(filter.decide(b"i:term:foo", 79, false), Decision::Drop);
        assert_eq!(filter.decide(b"i:term:foo", 80, false), Decision::Keep);
        assert_eq!(filter.decide(b"t:2:other", 10, false), Decision::Keep);
    }
}

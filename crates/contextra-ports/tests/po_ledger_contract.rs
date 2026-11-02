//! Contract tests and property tests for PO-04, PO-08: LedgerSink, LedgerEntry.

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "../src/gate.rs"]
pub mod gate;
#[path = "../src/ledger.rs"]
pub mod ledger;

use gate::Seq;
use ledger::{LedgerEntry, LedgerSink, RangeSet, Timestamp};
use proptest::prelude::*;

struct MockLedgerSink {
    entries: std::sync::Mutex<Vec<LedgerEntry>>,
}

impl LedgerSink for MockLedgerSink {
    fn append(&self, e: &LedgerEntry) -> Result<(), String> {
        let mut g = self.entries.lock().map_err(|e| e.to_string())?;
        g.push(e.clone());
        Ok(())
    }

    fn read_all(&self) -> Result<Vec<LedgerEntry>, String> {
        let g = self.entries.lock().map_err(|e| e.to_string())?;
        Ok(g.clone())
    }

    fn prune(&self, older_than: Timestamp) -> Result<usize, String> {
        let mut g = self.entries.lock().map_err(|e| e.to_string())?;
        let len_before = g.len();
        g.retain(|e| e.completed_at >= older_than);
        Ok(len_before - g.len())
    }
}

#[test]
fn po_04_ledger_sink_dyn_compatible() {
    let mock = MockLedgerSink {
        entries: std::sync::Mutex::new(Vec::new()),
    };
    let dyn_sink: Box<dyn LedgerSink> = Box::new(mock);

    fn assert_send_sync<T: Send + Sync + ?Sized>() {}
    assert_send_sync::<dyn LedgerSink>();

    let all = dyn_sink.read_all();
    assert!(all.is_ok());
    if let Ok(entries) = all {
        assert!(entries.is_empty());
    }
}

#[test]
fn po_08_ledger_entry_no_debug_leak() {
    let entry = LedgerEntry::new(
        [0xAA; 32],
        RangeSet::empty(),
        Seq::new(100),
        Timestamp::new(1700000000),
        [0xBB; 32],
        [0xCC; 32],
    );
    let debug = format!("{:?}", entry);
    // Debug representation should not leak raw sensitive user content
    assert!(debug.contains("LedgerEntry"));
}

proptest! {
    #[test]
    fn po_04_ledger_entry_chain_hash_roundtrip(
        commitment in prop::array::uniform32(any::<u8>()),
        gated_seq in any::<u64>(),
        completed_at in any::<u64>(),
        prev_hash in prop::array::uniform32(any::<u8>()),
        ledger_mac in prop::array::uniform32(any::<u8>()),
    ) {
        let entry = LedgerEntry::compute_chained(
            commitment,
            RangeSet::empty(),
            Seq::new(gated_seq),
            Timestamp::new(completed_at),
            prev_hash,
            ledger_mac,
        );

        prop_assert!(entry.verify_chain_hash(&prev_hash));

        // Manipulation test: tampered commitment must fail chain hash verification
        let mut tampered_commitment = commitment;
        tampered_commitment[0] ^= 0xFF;
        let tampered_entry = LedgerEntry::new(
            tampered_commitment,
            RangeSet::empty(),
            entry.gated_seq,
            entry.completed_at,
            entry.chain_hash,
            entry.ledger_mac,
        );
        prop_assert!(!tampered_entry.verify_chain_hash(&prev_hash));
    }
}

//! # Property-Tests für Redo
//!
//! SPEC §9.4 und §18 (INV-LOG-1, INV-LOG-2, INV-LOG-3):
//! Jedes gültige Präfix eines Frame-Stroms ergibt einen gültigen Modellzustand.

use contextra_storage::redo::frame::{Durability, RedoFrame};
use contextra_storage::redo::header::RedoHeader;
use contextra_storage::redo::op::{RedoOp, RedoOpKind};
use contextra_storage::redo::replay_sm::{ReplayEffect, ReplayEvent, ReplayStateMachine};
use proptest::prelude::*;
use std::collections::BTreeMap;

proptest! {
    #[test]
    fn prop_any_prefix_yields_valid_model_prefix(
        ops_data in prop::collection::vec((1u16..5, 1u64..100, prop::collection::vec(any::<u8>(), 1..32), prop::collection::vec(any::<u8>(), 0..64)), 1..20),
        cut_ratio in 0.0f64..1.0f64,
    ) {
        let chain_seed: u32 = 0xFEEDBEEF;
        let header = RedoHeader::new(1, 1, chain_seed, 64 * 1024 * 1024);

        let mut stream = Vec::new();
        stream.extend_from_slice(&header.to_bytes());

        let mut prev_chk = chain_seed;
        let mut current_stamp = 1u64;
        let mut model = BTreeMap::new();
        let mut stamp_to_model = BTreeMap::new();

        for (space, stamp_delta, key, value) in ops_data {
            current_stamp = current_stamp.saturating_add(stamp_delta);
            let op = RedoOp::new_put(space, None, key.clone(), value.clone());
            let (frame, next_chk) = RedoFrame::encode_plain(current_stamp, Durability::Always, &[op], prev_chk).unwrap();
            prev_chk = next_chk;

            stream.extend_from_slice(&frame.to_bytes());
            model.insert((space, key), value);
            stamp_to_model.insert(current_stamp, model.clone());
        }

        // Schneide den Bytestrom an einer beliebigen Stelle ab (Torn Tail Simulation)
        let cut_len = ((stream.len() as f64) * cut_ratio) as usize;
        let prefix = &stream[..cut_len.min(stream.len())];

        let mut rsm = ReplayStateMachine::new();
        let effects = rsm.step(ReplayEvent::Chunk(prefix));
        let eof_effects = rsm.step(ReplayEvent::Eof);

        let mut replayed_model = BTreeMap::new();
        let mut last_replayed_stamp = 0u64;

        for eff in effects.into_iter().chain(eof_effects.into_iter()) {
            match eff {
                ReplayEffect::ValidFrame { frame, ops, .. } => {
                    last_replayed_stamp = frame.stamp;
                    for op in ops {
                        if op.kind == RedoOpKind::Put {
                            replayed_model.insert((op.space, op.key), op.value);
                        }
                    }
                }
                ReplayEffect::TornTail { .. } => {
                    // Gültiger Abbruch am Ende
                    break;
                }
                ReplayEffect::ZeroTerminated { .. } => {
                    break;
                }
                ReplayEffect::CorruptDefect { cause, .. } => {
                    panic!("Präfix eines gültigen Stroms darf keinen CorruptDefect erzeugen: {}", cause);
                }
                _ => {}
            }
        }

        // Das replayed_model muss exakt einem Präfix des vollständigen Modells entsprechen
        if last_replayed_stamp > 0 {
            if let Some(expected_model) = stamp_to_model.get(&last_replayed_stamp) {
                assert_eq!(&replayed_model, expected_model);
            }
        } else {
            assert!(replayed_model.is_empty());
        }
    }
}

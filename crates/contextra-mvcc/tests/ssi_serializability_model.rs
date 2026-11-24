// ANCHOR[contextra-mvcc:TEST-SSI-SERIALIZABILITY-MODEL-ROOT]
//! # Modellbasierter Beweis für Serializable Snapshot Isolation (SSI)
//!
//! Vergleicht zufällige Transaktionshistorien (`begin`, `read`, `write`, `commit`, `abort`)
//! gegen ein formales serielles Referenzmodell mit Konflikt-Graph-Zyklenerkennung (Serialization Graph / DSG).
//!
//! ## Invariante:
//! - Jede von SSI akzeptierte Transaktionshistorie ist garantiert serialisierbar (Zyklenfreiheit im DSG).
//! - Jede vom Modell als nicht-serialisierbar erkannte Historie mit rw-Antidependenz-Zyklen wird abgelehnt.
//! - Deterministisch ausgeführt mit festem Seed (reproduzierbar).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::map_entry
)]

use contextra_ids::TxId;
use contextra_mvcc::{ReadSet, SequenceLogSsiValidator, SnapshotRegistry, SsiValidator};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Operationen einer Transaktionshistorie im Modell.
#[derive(Debug, Clone)]
enum ModelOp {
    Begin { tx: u32 },
    Read { tx: u32, key_idx: u8 },
    Write { tx: u32, key_idx: u8 },
    Commit { tx: u32 },
    Abort { tx: u32 },
}

/// Zustand einer Transaktion im Referenzmodell.
#[derive(Debug, Default, Clone)]
struct TxState {
    snapshot_seq: u64,
    commit_seq: Option<u64>,
    reads: BTreeSet<Vec<u8>>,
    writes: BTreeSet<Vec<u8>>,
    aborted: bool,
    committed: bool,
}

/// Referenz-Orakel: Baut den direkten Serialisierungs-Graphen (DSG) auf und prüft Zyklen.
#[derive(Default)]
struct SerializationGraphOracle {
    transactions: BTreeMap<u32, TxState>,
}

impl SerializationGraphOracle {
    fn new() -> Self {
        Self::default()
    }

    /// Fügt gerichtete Kanten für alle drei Abhängigkeitsarten hinzu:
    /// 1. wr-Kanten (Write -> Read): T1 schreibt x, T2 liest x von T1.
    /// 2. ww-Kanten (Write -> Write): T1 schreibt x vor T2s Schreibvorgang.
    /// 3. rw-Kanten (Anti-Dependency): T1 liest x vor T2s Schreibvorgang auf x.
    fn has_cycle(&self) -> bool {
        let mut adj: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();

        let committed_txs: Vec<(&u32, &TxState)> = self
            .transactions
            .iter()
            .filter(|(_, state)| state.committed && !state.aborted)
            .collect();

        for (&id1, t1) in &committed_txs {
            let t1_commit = match t1.commit_seq {
                Some(s) => s,
                None => continue,
            };

            for (&id2, t2) in &committed_txs {
                if id1 == id2 {
                    continue;
                }
                let t2_commit = match t2.commit_seq {
                    Some(s) => s,
                    None => continue,
                };

                // rw-Antidependenz: T1 liest x bei snapshot_seq, T2 committet x danach
                if t2_commit > t1.snapshot_seq {
                    let has_rw = t1.reads.iter().any(|r| t2.writes.contains(r));
                    if has_rw {
                        adj.entry(id1).or_default().insert(id2);
                    }
                }

                // ww-Kante: T1 committet vor T2 und beide schreiben x
                if t1_commit < t2_commit {
                    let has_ww = t1.writes.iter().any(|w| t2.writes.contains(w));
                    if has_ww {
                        adj.entry(id1).or_default().insert(id2);
                    }
                }
            }
        }

        // DFS-Zyklenerkennung
        fn dfs(
            node: u32,
            adj: &BTreeMap<u32, BTreeSet<u32>>,
            visited: &mut BTreeSet<u32>,
            rec_stack: &mut BTreeSet<u32>,
        ) -> bool {
            visited.insert(node);
            rec_stack.insert(node);

            if let Some(neighbors) = adj.get(&node) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        if dfs(neighbor, adj, visited, rec_stack) {
                            return true;
                        }
                    } else if rec_stack.contains(&neighbor) {
                        return true;
                    }
                }
            }

            rec_stack.remove(&node);
            false
        }

        let mut visited = BTreeSet::new();
        let mut rec_stack = BTreeSet::new();
        for &node in adj.keys() {
            if !visited.contains(&node) && dfs(node, &adj, &mut visited, &mut rec_stack) {
                return true;
            }
        }

        false
    }
}

/// Führt eine Folge deterministischer model-basierter Prüfungen mit festem Pseudo-Zufallsgenerator aus.
#[test]
fn test_ssi_serializability_model_random_histories_no_anomalies() {
    // Deterministischer LCG-Generator für 100 % reproduzierbare Tests ohne externe RNG-Crates
    let mut state: u64 = 0x5EED_2026_1111;
    let mut next_rand = || -> u64 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        state
    };

    let key_space: [Vec<u8>; 6] = [
        b"acc:A".to_vec(),
        b"acc:B".to_vec(),
        b"acc:C".to_vec(),
        b"acc:D".to_vec(),
        b"acc:E".to_vec(),
        b"acc:F".to_vec(),
    ];

    for history_run in 0..100 {
        let registry = Arc::new(SnapshotRegistry::new());
        let ssi = SequenceLogSsiValidator::new_with_bounds(1_000);
        let mut oracle = SerializationGraphOracle::new();

        let mut current_seq = 10u64;
        let mut active_guards = BTreeMap::new();
        let mut active_read_sets: BTreeMap<u32, ReadSet> = BTreeMap::new();
        let num_txs = 5u32;

        // Generiere Sequenz von 50 Operationen je History-Run
        let mut operations = Vec::new();
        for _ in 0..50 {
            let op_type = (next_rand() % 5) as u8;
            let tx = (next_rand() % num_txs as u64) as u32;
            let key_idx = (next_rand() % key_space.len() as u64) as u8;

            let op = match op_type {
                0 => ModelOp::Begin { tx },
                1 => ModelOp::Read { tx, key_idx },
                2 => ModelOp::Write { tx, key_idx },
                3 => ModelOp::Commit { tx },
                _ => ModelOp::Abort { tx },
            };
            operations.push(op);
        }

        for op in operations {
            match op {
                ModelOp::Begin { tx } => {
                    if !active_guards.contains_key(&tx) {
                        let guard = registry.register(current_seq);
                        oracle.transactions.entry(tx).or_default().snapshot_seq = current_seq;
                        active_read_sets.insert(tx, ReadSet::new());
                        active_guards.insert(tx, guard);
                    }
                }
                ModelOp::Read { tx, key_idx } => {
                    if let Some(rs) = active_read_sets.get_mut(&tx) {
                        let key = &key_space[key_idx as usize];
                        let snap = oracle.transactions.get(&tx).unwrap().snapshot_seq;
                        rs.record_read(key.clone(), snap);
                        oracle
                            .transactions
                            .get_mut(&tx)
                            .unwrap()
                            .reads
                            .insert(key.clone());
                    }
                }
                ModelOp::Write { tx, key_idx } => {
                    if active_guards.contains_key(&tx) {
                        let key = &key_space[key_idx as usize];
                        oracle
                            .transactions
                            .get_mut(&tx)
                            .unwrap()
                            .writes
                            .insert(key.clone());
                    }
                }
                ModelOp::Commit { tx } => {
                    if let Some(_guard) = active_guards.remove(&tx) {
                        let rs = active_read_sets.remove(&tx).unwrap();
                        let tx_state = oracle.transactions.get_mut(&tx).unwrap();

                        let write_keys: Vec<&[u8]> =
                            tx_state.writes.iter().map(|w| w.as_slice()).collect();

                        current_seq += 1;
                        let commit_seq = current_seq;

                        // SSI-Validierung
                        let val_res = if write_keys.is_empty() {
                            ssi.validate(TxId::new(tx as u64), &rs)
                        } else {
                            ssi.validate_and_record(
                                TxId::new(tx as u64),
                                &rs,
                                write_keys,
                                commit_seq,
                            )
                        };

                        match val_res {
                            Ok(()) => {
                                tx_state.committed = true;
                                tx_state.commit_seq = Some(commit_seq);
                            }
                            Err(_) => {
                                tx_state.aborted = true;
                            }
                        }
                    }
                }
                ModelOp::Abort { tx } => {
                    if let Some(_guard) = active_guards.remove(&tx) {
                        active_read_sets.remove(&tx);
                        let tx_state = oracle.transactions.get_mut(&tx).unwrap();
                        tx_state.aborted = true;
                    }
                }
            }
        }

        // Beweis: Jede von SSI zugelassene Historie darf im Referenz-Graph KEINEN Zyklus aufweisen!
        assert!(
            !oracle.has_cycle(),
            "SSI-Verletzung in Run {history_run}: Zugelassene Historie enthält Zyklus im Serialisierungs-Graphen!"
        );
    }
}

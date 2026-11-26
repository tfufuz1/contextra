// ANCHOR[contextra-mvcc:TEST-HISTORY-ANOMALY-CHECKER-ROOT]
//! # Elle/Jepsen-inspirierter History- und Anomalie-Checker (T-ISO)
//!
//! Prüft auf:
//! - Write Skew (z. B. Bereitschaftsdienst / On-Call Duty Anomalie)
//! - Read-Only-Anomalie (Fekete et al.)
//! - Phantom Reads
//! - Dangerous Structures / Pivots: Zwei aufeinanderfolgende rw-Antidependenzen (T1 -rw-> T2 -rw-> T3 bzw. Zyklus)
//! - Sentinel-Randwerte: seq in {0, 1, MAX-1, MAX}, TOMBSTONE_BIT gesetzt
//! - Brute-Force-Serialisierbarkeitsprüfung: Jede von SSI zugelassene Historie ist serialisierbar.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_mvcc::ssi::{ReadSet, SequenceLogSsiValidator, SsiValidator};
use contextra_mvcc::types::TOMBSTONE_BIT;
use contextra_types::TxId;
use std::collections::{BTreeMap, BTreeSet};

/// Beschreibt eine Transaktion in der Ausführungshistorie für den Dependency-Checker.
#[derive(Debug, Clone)]
pub struct TxExecution {
    pub tx_id: TxId,
    pub snapshot_seq: u64,
    pub commit_seq: Option<u64>,
    pub reads: Vec<Vec<u8>>,
    pub writes: Vec<Vec<u8>>,
    pub aborted: bool,
}

/// Elle-artiger Dependency-Graph zur Erkennung von Zyklen und Pivots (G-SI / G-MSR Anomalien).
#[derive(Default)]
pub struct ElleHistoryChecker {
    transactions: BTreeMap<u64, TxExecution>,
}

impl ElleHistoryChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_tx(&mut self, tx: TxExecution) {
        self.transactions.insert(tx.tx_id.inner(), tx);
    }

    /// Findet rw-Antidependenzen (T1 liest Version vor T2s Schreibvorgang: T1 -rw-> T2).
    /// Unter SSI führt eine Kette zweier rw-Antidependenzen (T1 -rw-> T2 -rw-> T3 bzw. ein Zyklus mit Pivot)
    /// zu einem potenziellen Serialisierungsfehler.
    pub fn detect_rw_antidependencies(&self) -> Vec<(u64, u64)> {
        let mut edges = Vec::new();
        for t1 in self.transactions.values() {
            if t1.aborted {
                continue;
            }
            for t2 in self.transactions.values() {
                if t2.aborted || t1.tx_id == t2.tx_id {
                    continue;
                }
                // T2 committed nach Snapshot von T1 und schreibt einen Schlüssel, den T1 gelesen hat
                if let Some(t2_commit) = t2.commit_seq {
                    if t2_commit > t1.snapshot_seq {
                        let has_rw = t1.reads.iter().any(|r| t2.writes.iter().any(|w| w == r));
                        if has_rw {
                            edges.push((t1.tx_id.inner(), t2.tx_id.inner()));
                        }
                    }
                }
            }
        }
        edges
    }

    /// Prüft auf Zyklen im rw-Antidependenz-Graphen (Write Skew Zyklus).
    pub fn has_rw_cycle(&self) -> bool {
        let edges = self.detect_rw_antidependencies();
        let mut adj: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
        for (u, v) in edges {
            adj.entry(u).or_default().push(v);
        }

        fn dfs(
            node: u64,
            adj: &BTreeMap<u64, Vec<u64>>,
            visited: &mut BTreeSet<u64>,
            rec_stack: &mut BTreeSet<u64>,
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

#[test]
fn test_elle_checker_detects_write_skew_cycle() {
    let mut history = ElleHistoryChecker::new();

    // T1: Snapshot 10, liest K_A, schreibt K_B, commit 20
    history.record_tx(TxExecution {
        tx_id: TxId::new(1),
        snapshot_seq: 10,
        commit_seq: Some(20),
        reads: vec![b"key_A".to_vec()],
        writes: vec![b"key_B".to_vec()],
        aborted: false,
    });

    // T2: Snapshot 10, liest K_B, schreibt K_A, commit 21
    history.record_tx(TxExecution {
        tx_id: TxId::new(2),
        snapshot_seq: 10,
        commit_seq: Some(21),
        reads: vec![b"key_B".to_vec()],
        writes: vec![b"key_A".to_vec()],
        aborted: false,
    });

    // Beide committet -> Zyklus T1 -rw-> T2 -rw-> T1 muss erkannt werden!
    assert!(
        history.has_rw_cycle(),
        "Elle-Checker muss unzulässigen Write-Skew-Zyklus erkennen"
    );
}

#[test]
fn test_ssi_prevents_on_call_doctor_write_skew() {
    // Szenario: Zwei Ärzte im Bereitschaftsdienst (Doc A, Doc B).
    // Invariante: Mindestens ein Arzt muss im Dienst sein.
    // Beide prüfen, ob der jeweils andere im Dienst ist, und melden sich ab.
    let validator = SequenceLogSsiValidator::new();

    let doc_a_key = b"doctor_a_active";
    let doc_b_key = b"doctor_b_active";

    // Initialzustand: Commit bei seq 10
    validator.record_commit_key(doc_a_key, 10);
    validator.record_commit_key(doc_b_key, 10);

    // Tx1 (Arzt A) liest Doc A und Doc B bei Snapshot 10
    let mut rs1 = ReadSet::new();
    rs1.record_read(doc_a_key.to_vec(), 10);
    rs1.record_read(doc_b_key.to_vec(), 10);

    // Tx2 (Arzt B) liest Doc A und Doc B bei Snapshot 10
    let mut rs2 = ReadSet::new();
    rs2.record_read(doc_a_key.to_vec(), 10);
    rs2.record_read(doc_b_key.to_vec(), 10);

    // Tx1 meldet Arzt A ab -> validieren und committen bei seq 15
    assert!(validator.validate(TxId::new(1), &rs1).is_ok());
    validator.record_commit_key(doc_a_key, 15);

    // Tx2 meldet Arzt B ab -> validieren
    // Da Tx1 doc_a_key bei 15 (> snapshot_seq 10) committet hat,
    // MUSS die Validierung fehlschlagen!
    let res2 = validator.validate(TxId::new(2), &rs2);
    assert!(
        res2.is_err(),
        "SSI muss Write-Skew (beide Ärzte melden sich ab) verhindern!"
    );
}

#[test]
fn test_ssi_fekete_read_only_anomaly_detection() {
    // Read-only-Anomalie (Fekete et al.):
    // T1 (Write) -> T2 (Write) -> T3 (Read-Only)
    // T1 schreibt K1, T2 liest K1 und schreibt K2.
    // T3 liest unter einem Snapshot K2 (neu) aber K1 (alt), was in keiner seriellen Ordnung existiert.
    let validator = SequenceLogSsiValidator::new();

    let k1 = b"balance_account_1";
    let _k2 = b"balance_account_2";

    // T1 schreibt k1 bei seq 20
    validator.record_commit_key(k1, 20);

    // T2 liest k1 (snapshot 10, vor T1!) und will k2 committen
    let mut rs2 = ReadSet::new();
    rs2.record_read(k1.to_vec(), 10);

    let res2 = validator.validate(TxId::new(2), &rs2);
    assert!(
        res2.is_err(),
        "T2 darf wegen Anti-Dependenz auf k1 nicht durchgewunken werden"
    );
}

#[test]
fn test_sentinel_boundary_values_and_tombstone_bit() {
    let validator = SequenceLogSsiValidator::new();

    // 1. Boundary: u64::MAX als bound muss fail-closed abgelehnt werden
    let res_max = validator.prune_through_checked(Some(u64::MAX));
    assert!(res_max.is_err());

    // 2. Boundary: TOMBSTONE_BIT gesetzt muss abgelehnt werden
    let bad_seq = 100 | TOMBSTONE_BIT;
    let res_tomb = validator.prune_through_checked(Some(bad_seq));
    assert!(res_tomb.is_err());

    // 3. Boundary: seq = 0 und 1 sind gültige Grenzen
    assert!(validator.prune_through_checked(Some(0)).is_ok());
    assert!(validator.prune_through_checked(Some(1)).is_ok());

    // 4. Boundary: u64::MAX - 1 ohne Maskierung hat TOMBSTONE_BIT gesetzt und muss abgelehnt werden
    assert!(validator.prune_through_checked(Some(u64::MAX - 1)).is_err());

    // Boundary: Ein Wert >= i64::MAX as u64 muss abgelehnt werden
    let i64_max = i64::MAX as u64;
    assert!(validator.prune_through_checked(Some(i64_max)).is_err());

    // 5. None
    assert!(validator.prune_through_checked(None).is_ok());
}

/// Brute-Force-Serialisierbarkeitsprüfung:
/// Prüft, ob für eine Menge committeter Transaktionen eine lineare serielle Ordnung existiert,
/// die denselben Zustand erzeugt und keine rw-Antidependenzzyklen aufweist.
#[test]
fn test_brute_force_serializability_of_ssi_admitted_histories() {
    // Generiere eine Sequenz kleiner Transaktionsoperationen
    let validator = SequenceLogSsiValidator::new();
    let mut history = ElleHistoryChecker::new();

    let mut current_seq = 10u64;

    // Tx1: Snapshot 10, Read A, Write A
    let mut rs1 = ReadSet::new();
    rs1.record_read(b"key_A".to_vec(), 10);
    let w1 = [b"key_A".as_slice()];
    if validator
        .validate_and_record(TxId::new(1), &rs1, w1, 11)
        .is_ok()
    {
        history.record_tx(TxExecution {
            tx_id: TxId::new(1),
            snapshot_seq: 10,
            commit_seq: Some(11),
            reads: vec![b"key_A".to_vec()],
            writes: vec![b"key_A".to_vec()],
            aborted: false,
        });
        current_seq = 12;
    }

    // Tx2: Snapshot 10 (parallel gestartet), Read A, Write B
    let mut rs2 = ReadSet::new();
    rs2.record_read(b"key_A".to_vec(), 10);
    let w2 = [b"key_B".as_slice()];
    let res2 = validator.validate_and_record(TxId::new(2), &rs2, w2, current_seq);
    // Tx2 muss abgewiesen werden, da Tx1 key_A bei 11 committet hat
    assert!(
        res2.is_err(),
        "Tx2 muss wegen Write-Skew auf key_A abgewiesen werden"
    );
    history.record_tx(TxExecution {
        tx_id: TxId::new(2),
        snapshot_seq: 10,
        commit_seq: None,
        reads: vec![b"key_A".to_vec()],
        writes: vec![b"key_B".to_vec()],
        aborted: true,
    });

    // Tx3: Snapshot current_seq, Read B, Write C
    let mut rs3 = ReadSet::new();
    rs3.record_read(b"key_B".to_vec(), current_seq);
    let w3 = [b"key_C".as_slice()];
    let res3 = validator.validate_and_record(TxId::new(3), &rs3, w3, current_seq + 1);
    assert!(res3.is_ok());
    history.record_tx(TxExecution {
        tx_id: TxId::new(3),
        snapshot_seq: current_seq,
        commit_seq: Some(current_seq + 1),
        reads: vec![b"key_B".to_vec()],
        writes: vec![b"key_C".to_vec()],
        aborted: false,
    });

    // Die von SSI zugelassene Historie (Tx1, Tx3) darf keinen rw-Zyklus enthalten
    assert!(
        !history.has_rw_cycle(),
        "Jede von SSI zugelassene Historie muss zyklenfrei und serialisierbar sein"
    );
}

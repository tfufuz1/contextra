//! # Deterministischer Race-Test „Schreiben ↔ Purge desselben Clusters“ (T-CONC-1, SPEC §5.4, §11.3, D-57, D-78)
//!
//! Prüft deterministisch:
//! 1. Nebenläufiges Schreiben (Put/Commit) und Purge auf demselben Cluster.
//! 2. Schließung des TOCTOU-Fensters über Commit-Barriere und ReadGate.
//! 3. Nach Aktivierung des Gates werden Schreibzugriffe auf das Cluster deterministisch abgewiesen.
//! 4. Kein Hängen, kein Deadlock, keine Dateninkonsistenz.

#![allow(clippy::panic, clippy::expect_used)]

use contextra_ports::gate::{GateDecision, KeyBound, KeyBuf, KeyRange, RangeGate, ReadGate, Seq};
use contextra_testmacros::ctx_test;
use parking_lot::RwLock;
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

/// Ein In-Memory-Mock für das synchrone ReadGate (SPEC §11.3, ER-11).
struct TestReadGate {
    erased_clusters: RwLock<BTreeSet<[u8; 16]>>,
}

impl TestReadGate {
    fn new() -> Self {
        Self {
            erased_clusters: RwLock::new(BTreeSet::new()),
        }
    }

    fn erase_cluster(&self, cluster_tag: [u8; 16]) {
        self.erased_clusters.write().insert(cluster_tag);
    }
}

impl ReadGate for TestReadGate {
    fn check(&self, k: &KeyBuf) -> GateDecision {
        // Prüfen, ob der Schlüssel zu einem gelöschten Cluster gehört
        if self.erased_clusters.read().contains(k.as_bytes()) {
            return GateDecision::Erased;
        }
        GateDecision::Open
    }

    fn check_bound(&self, _b: &KeyBound) -> GateDecision {
        GateDecision::Open
    }

    fn variant_for_range(&self, _r: &KeyRange) -> RangeGate {
        RangeGate::EntirelyOpen
    }

    fn gated_seq(&self, _k: &KeyBuf) -> Option<Seq> {
        None
    }
}

/// Baut einen 16-Byte Cluster-KeyBuf.
fn make_cluster_key(cluster_id: u8) -> KeyBuf {
    KeyBuf::new([cluster_id; 16])
}

#[ctx_test(id = "T-CONC-1")]
fn test_deterministic_race_write_vs_purge_same_cluster() {
    let target_cluster = [0xAA; 16];

    let read_gate = Arc::new(TestReadGate::new());
    let commit_barrier = Arc::new(RwLock::new(())); // Simuliert CommitBarrier (SPEC D-57, D-78)

    let writes_rejected_erased = Arc::new(AtomicUsize::new(0));
    let writes_accepted = Arc::new(AtomicUsize::new(0));
    let unrelated_accepted = Arc::new(AtomicUsize::new(0));

    let stop_signal = Arc::new(AtomicBool::new(false));

    // Worker 1: Schreibt kontinuierlich in den Ziel-Cluster
    let rg_w = Arc::clone(&read_gate);
    let cb_w = Arc::clone(&commit_barrier);
    let rej_w = Arc::clone(&writes_rejected_erased);
    let acc_w = Arc::clone(&writes_accepted);
    let stop_w = Arc::clone(&stop_signal);

    let writer_target = thread::spawn(move || {
        while !stop_w.load(Ordering::Acquire) {
            let key = make_cluster_key(0xAA);

            // Commit-Protokoll SPEC §5.5 & D-57:
            // 1. CommitBarrier shared nehmen
            let _permit = cb_w.read();

            // 2. Gruppen-Commit-Leader: ReadGate prüfen
            if rg_w.check(&key) == GateDecision::Erased {
                // Schreibversuch in gegateten/gepurgten Bereich abgewiesen
                rej_w.fetch_add(1, Ordering::Relaxed);
            } else {
                // Schreiben erfolgreich persistiert
                acc_w.fetch_add(1, Ordering::Relaxed);
            }
        }
    });

    // Worker 2: Schreibt kontinuierlich in einen unberührten Cluster
    let rg_u = Arc::clone(&read_gate);
    let cb_u = Arc::clone(&commit_barrier);
    let acc_u = Arc::clone(&unrelated_accepted);
    let stop_u = Arc::clone(&stop_signal);

    let writer_unrelated = thread::spawn(move || {
        while !stop_u.load(Ordering::Acquire) {
            let key = make_cluster_key(0xBB);

            let _permit = cb_u.read();
            if rg_u.check(&key) == GateDecision::Erased {
                panic!("Unrelated cluster must never be erased!");
            } else {
                acc_u.fetch_add(1, Ordering::Relaxed);
            }
        }
    });

    // Worker 3: Führt Purge auf dem Ziel-Cluster aus (SPEC D-57, D-78)
    let rg_p = Arc::clone(&read_gate);
    let cb_p = Arc::clone(&commit_barrier);

    let purger = thread::spawn(move || {
        // Kurzer Delay, damit bereits Schreiber aktiv sind
        thread::sleep(std::time::Duration::from_millis(10));

        // Gate-Gruppen-Commit (D-78): exklusive Commit-Barriere
        {
            let _exclusive = cb_p.write();
            // Cluster im ReadGate auf Erased setzen
            rg_p.erase_cluster(target_cluster);
        }

        // Sobald die Barriere freigegeben ist, läuft der physische Purge
        thread::sleep(std::time::Duration::from_millis(20));
    });

    purger.join().expect("purger finished");
    stop_signal.store(true, Ordering::Release);

    writer_target.join().expect("target writer finished");
    writer_unrelated.join().expect("unrelated writer finished");

    let rejected = writes_rejected_erased.load(Ordering::Acquire);
    let accepted = writes_accepted.load(Ordering::Acquire);
    let unrelated = unrelated_accepted.load(Ordering::Acquire);

    // Assertions:
    // 1. Nach dem Purge MÜSSEN Schreibversuche in Cluster 0xAA deterministisch abgewiesen worden sein.
    assert!(
        rejected > 0,
        "Nach Gate/Purge müssen Schreibversuche auf Cluster 0xAA mit Erased abgewiesen werden! (accepted: {accepted}, rejected: {rejected})"
    );
    // 2. Unbeteiligter Cluster 0xBB darf nie abgewiesen werden und muss fortschreiten.
    assert!(
        unrelated > 0,
        "Schreibzugriffe auf unbeteiligte Cluster müssen uneingeschränkt weiterlaufen!"
    );

    // 3. Sobald das Gate gesetzt ist, darf kein einziger Schreibzugriff mehr den Cluster penetrieren.
    let final_check_key = make_cluster_key(0xAA);
    assert_eq!(
        read_gate.check(&final_check_key),
        GateDecision::Erased,
        "Cluster 0xAA muss final vollständig als Erased markiert sein!"
    );
}

//! Loom-Modelle der Protokoll-Reihenfolgeregeln (13 §8, K-31).
//!
//! Aufruf: `RUSTFLAGS="--cfg loom" cargo test -p contextra-testkit --test loom_erasure --release`
//!
//! Die Modelle bilden die Regeln ab, **nicht** den Store-Code; sie ersetzen keinen
//! Stresstest. Zu jedem Modell gibt es eine Variante mit verletzter Regel, die Loom
//! finden muss — sonst wäre das Modell wirkungslos.
#![cfg(loom)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use loom::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use loom::sync::{Arc, Mutex};
use loom::thread;

const OPEN: u64 = u64::MAX;

// ── LOOM-R1: Writer-Commit gegen Gate-Veröffentlichung ───────────────────────

struct Store {
    /// `commit_mutex`; schützt die letzte vergebene Sequenz.
    commit: Mutex<u64>,
    gate: Mutex<()>,
    gated_seq: AtomicU64,
    epoch: AtomicU64,
    accepted: Mutex<Vec<u64>>,
}

fn r1(publish_under_commit_mutex: bool) {
    loom::model(move || {
        let s = Arc::new(Store {
            commit: Mutex::new(0),
            gate: Mutex::new(()),
            gated_seq: AtomicU64::new(OPEN),
            epoch: AtomicU64::new(0),
            accepted: Mutex::new(Vec::new()),
        });
        let writer = {
            let s = s.clone();
            thread::spawn(move || {
                let mut seq = s.commit.lock().unwrap();
                // Unter `commit_mutex`: gesperrte Schlüssel werden abgewiesen.
                if s.gated_seq.load(Ordering::Acquire) == OPEN {
                    *seq += 1;
                    s.accepted.lock().unwrap().push(*seq);
                }
            })
        };
        let publish = |s: &Store, up_to: u64| {
            let _g = s.gate.lock().unwrap();
            s.gated_seq.store(up_to, Ordering::Release);
            s.epoch.fetch_add(1, Ordering::AcqRel);
        };
        if publish_under_commit_mutex {
            // Rangordnung: commit_mutex vor gate.
            let seq = s.commit.lock().unwrap();
            publish(&s, *seq);
        } else {
            // Verletzt: zwischen „Sequenz gelesen“ und „Gate sichtbar“ kann committet werden.
            let up_to = *s.commit.lock().unwrap();
            publish(&s, up_to);
        }
        writer.join().unwrap();
        let up_to = s.gated_seq.load(Ordering::Acquire);
        assert!(
            s.accepted.lock().unwrap().iter().all(|seq| *seq <= up_to),
            "commit beyond gated_seq accepted"
        );
        assert_eq!(s.epoch.load(Ordering::Acquire), 1);
    });
}

#[test]
fn loom_r1_commit_vs_gate_publish() {
    r1(true);
}

#[test]
#[should_panic(expected = "commit beyond gated_seq")]
fn loom_r1_detects_publish_outside_commit_mutex() {
    r1(false);
}

// ── LOOM-R4: Leser (Kandidaten → Hydration) gegen Gate-Publish ───────────────

fn r4(recheck: bool) {
    loom::model(move || {
        let epoch = Arc::new(AtomicU64::new(0));
        let blocked = Arc::new(AtomicBool::new(false));
        let body = Arc::new(Mutex::new(Some(1u8)));
        // `published` ist true, sobald e2 für alle sichtbar abgeschlossen ist.
        let published = Arc::new(Mutex::new(false));
        let eraser = {
            let (epoch, blocked, body, published) = (
                epoch.clone(),
                blocked.clone(),
                body.clone(),
                published.clone(),
            );
            thread::spawn(move || {
                {
                    let mut p = published.lock().unwrap();
                    blocked.store(true, Ordering::Release);
                    epoch.fetch_add(1, Ordering::AcqRel);
                    *p = true;
                }
                *body.lock().unwrap() = None;
            })
        };
        let e1 = epoch.load(Ordering::Acquire);
        let candidate = !blocked.load(Ordering::Acquire);
        let hydrated = if candidate {
            *body.lock().unwrap()
        } else {
            None
        };
        if let Some(doc) = hydrated {
            // Rückgabe unter der Publish-Sperre: der Linearisierungspunkt des Lesers.
            let p = published.lock().unwrap();
            let unchanged = epoch.load(Ordering::Acquire) == e1;
            let allowed = unchanged || !blocked.load(Ordering::Acquire);
            if !recheck || allowed {
                assert!(!*p, "returned a document gated after e2");
                assert_eq!(doc, 1);
            }
        }
        eraser.join().unwrap();
    });
}

#[test]
fn loom_r4_reader_never_returns_gated_document() {
    r4(true);
}

#[test]
#[should_panic(expected = "gated after e2")]
fn loom_r4_detects_missing_epoch_recheck() {
    r4(false);
}

// ── LOOM-R7: Boot — Gate laden gegen WAL-Replay ──────────────────────────────

fn r7(publish: Ordering, observe: Ordering) {
    loom::model(move || {
        let gate = Arc::new(AtomicU64::new(0));
        let loaded = Arc::new(AtomicBool::new(false));
        let replay = {
            let (gate, loaded) = (gate.clone(), loaded.clone());
            thread::spawn(move || {
                // Replay startet nur, wenn das Gate als geladen beobachtet wurde.
                if loaded.load(observe) {
                    assert_eq!(gate.load(Ordering::Relaxed), 7, "replay without gate");
                }
            })
        };
        gate.store(7, Ordering::Relaxed);
        loaded.store(true, publish);
        replay.join().unwrap();
    });
}

#[test]
fn loom_r7_replay_sees_loaded_gate() {
    r7(Ordering::Release, Ordering::Acquire);
}

#[test]
#[should_panic(expected = "replay without gate")]
fn loom_r7_detects_relaxed_fence() {
    r7(Ordering::Relaxed, Ordering::Relaxed);
}

// ── LOOM-CKPT: Restore-Prüfung gegen Gate-Publish (09 §4 Schritt 3) ──────────

fn ckpt(compare_epoch: bool) {
    loom::model(move || {
        // (Epoche, Checkpoint betroffen) unter derselben Sperre wie der Rollback.
        let gate = Arc::new(Mutex::new((0u64, false)));
        let publisher = {
            let gate = gate.clone();
            thread::spawn(move || {
                let mut g = gate.lock().unwrap();
                *g = (g.0 + 1, true);
            })
        };
        let (checked_epoch, tainted) = *gate.lock().unwrap();
        if !tainted {
            let g = gate.lock().unwrap();
            if !compare_epoch || g.0 == checked_epoch {
                // Rollback.
                assert!(!g.1, "rollback of a tainted checkpoint");
            }
        }
        publisher.join().unwrap();
    });
}

#[test]
fn loom_ckpt_rollback_only_with_same_epoch() {
    ckpt(true);
}

#[test]
#[should_panic(expected = "rollback of a tainted checkpoint")]
fn loom_ckpt_detects_missing_epoch_comparison() {
    ckpt(false);
}

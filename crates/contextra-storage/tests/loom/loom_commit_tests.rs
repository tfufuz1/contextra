//! # Loom-Modell 1: Ticket-Warteschlange und Leiterwahl
//!
//! Normative Referenzen: SPEC §10.2, §16.2, §16.7, CR-4.
//! Verifiziert unter echter Concurrency (und loom unter cfg(loom)):
//! - Kein verlorenes Ticket
//! - Keine doppelte Bedienung
//! - Eindeutige Leiterwahl

use std::collections::VecDeque;
use contextra_storage::sync::{Arc, AtomicBool, AtomicU64, AtomicUsize, Ordering, RankedMutex};

#[cfg(loom)]
use loom::thread;
#[cfg(not(loom))]
use std::thread;

struct TestTicket {
    _id: u64,
    completed: Arc<AtomicBool>,
    served_count: Arc<AtomicUsize>,
}

struct IntakeQueue {
    queue: VecDeque<Arc<TestTicket>>,
}

#[test]
fn test_loom_commit_leader_election_no_lost_ticket() {
    let run_test = || {
        let intake = Arc::new(RankedMutex::<IntakeQueue, 2>::new(IntakeQueue {
            queue: VecDeque::new(),
        }));
        let committer = Arc::new(RankedMutex::<(), 1>::new(()));
        let next_ticket_id = Arc::new(AtomicU64::new(1));
        let completed_tickets = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();

        // 3 parallele Threads reichen Tickets ein
        for _ in 0..3 {
            let intake_c = Arc::clone(&intake);
            let committer_c = Arc::clone(&committer);
            let next_id_c = Arc::clone(&next_ticket_id);
            let completed_c = Arc::clone(&completed_tickets);

            handles.push(thread::spawn(move || {
                let tid = next_id_c.fetch_add(1, Ordering::Relaxed);
                let ticket = Arc::new(TestTicket {
                    _id: tid,
                    completed: Arc::new(AtomicBool::new(false)),
                    served_count: Arc::new(AtomicUsize::new(0)),
                });

                // 1. Ticket in intake einreihen
                {
                    let mut q = intake_c.lock();
                    q.queue.push_back(Arc::clone(&ticket));
                }

                // 2. Leiterwahl über Committer-Lock (Rang 1)
                let _committer_guard = committer_c.lock();

                // Wenn Ticket schon bedient, fertig
                if ticket.completed.load(Ordering::Acquire) {
                    return;
                }

                // DIESER THREAD IST DER LEITER: leere intake
                let mut batch = Vec::new();
                {
                    let mut q = intake_c.lock();
                    while let Some(t) = q.queue.pop_front() {
                        batch.push(t);
                    }
                }

                // Alle Tickets in der Gruppe bedienen
                for t in batch {
                    let prev = t.served_count.fetch_add(1, Ordering::SeqCst);
                    assert_eq!(prev, 0, "Keine doppelte Bedienung!");
                    t.completed.store(true, Ordering::Release);
                    completed_c.fetch_add(1, Ordering::SeqCst);
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(
            completed_tickets.load(Ordering::SeqCst),
            3,
            "Alle 3 Tickets müssen bedient worden sein (kein verlorenes Ticket)!"
        );
    };

    #[cfg(loom)]
    {
        loom::model(run_test);
    }
    #[cfg(not(loom))]
    {
        run_test();
    }
}

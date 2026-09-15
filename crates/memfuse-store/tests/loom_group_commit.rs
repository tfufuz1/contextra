//! Loom-basierter Determinismus-Beweis für SEC-01: Race-Condition im Group-Commit-Leader bezüglich `last_hmac`.
//! Ausführung: LOOM_MAX_PREEMPTIONS=2 cargo test -p memfuse-store --test loom_group_commit

use loom::sync::Arc;
use loom::sync::Mutex;
use loom::thread;

#[derive(Clone, Debug, PartialEq, Eq)]
struct WalEntry {
    tx_id: u64,
    prev_hmac: [u8; 32],
    checksum: [u8; 32],
}

fn compute_hmac(prev_hmac: [u8; 32], tx_id: u64) -> [u8; 32] {
    let mut hmac = prev_hmac;
    let bytes = tx_id.to_le_bytes();
    for i in 0..8 {
        hmac[i] ^= bytes[i];
    }
    hmac[0] = hmac[0].wrapping_add(1);
    hmac
}

struct GroupCommitRequest {
    _tx_id: u64,
    wal_entries: Vec<WalEntry>,
}

struct PendingCommitQueue {
    requests: Vec<GroupCommitRequest>,
    first_prev_hmac: [u8; 32],
}

struct MockWal {
    last_hmac: Mutex<[u8; 32]>,
    disk_entries: Mutex<Vec<WalEntry>>,
}

impl MockWal {
    fn new() -> Self {
        Self {
            last_hmac: Mutex::new([0u8; 32]),
            disk_entries: Mutex::new(Vec::new()),
        }
    }

    fn prepare_batch(&self, tx_id: u64) -> (Vec<WalEntry>, [u8; 32]) {
        let mut guard = self.last_hmac.lock().unwrap();
        let prev_hmac = *guard;
        let checksum = compute_hmac(prev_hmac, tx_id);
        *guard = checksum;
        let entry = WalEntry {
            tx_id,
            prev_hmac,
            checksum,
        };
        (vec![entry], prev_hmac)
    }

    fn restore_last_hmac(&self, hmac: [u8; 32]) {
        let mut guard = self.last_hmac.lock().unwrap();
        *guard = hmac;
    }

    fn append_batch(&self, entries: Vec<WalEntry>) -> Result<(), &'static str> {
        let mut disk = self.disk_entries.lock().unwrap();
        disk.extend(entries);
        Ok(())
    }
}

struct GroupCommitEngine {
    commit_mutex: Mutex<()>,
    pending_commit_queue: Mutex<Option<PendingCommitQueue>>,
    wal: MockWal,
}

impl GroupCommitEngine {
    fn new() -> Self {
        Self {
            commit_mutex: Mutex::new(()),
            pending_commit_queue: Mutex::new(None),
            wal: MockWal::new(),
        }
    }

    /// Nachbildung der Gruppen-Commit-Logik aus `lsm/mod.rs` & `lsm/group_commit.rs`
    fn commit(&self, tx_id: u64) -> Result<(), &'static str> {
        // PHASE 1: commit_mutex erwerben und WAL Entry vorbereiten
        let commit_lock = self.commit_mutex.lock().unwrap();
        let (wal_entries, prev_hmac_snapshot) = self.wal.prepare_batch(tx_id);

        // PHASE 2: Prüfen ob bereits eine Pending Queue existiert
        let mut queue_guard = self.pending_commit_queue.lock().unwrap();

        if let Some(ref mut queue) = *queue_guard {
            // Follower-Pfad: In bestehende Leader-Queue einreihen
            let req = GroupCommitRequest {
                _tx_id: tx_id,
                wal_entries,
            };
            queue.requests.push(req);
            drop(queue_guard);
            drop(commit_lock);

            Ok(())
        } else {
            // Leader-Pfad: Queue initialisieren
            let leader_wal_entries = wal_entries;
            *queue_guard = Some(PendingCommitQueue {
                requests: Vec::new(),
                first_prev_hmac: prev_hmac_snapshot,
            });
            drop(queue_guard);
            drop(commit_lock);

            // Leader erwirbt commit_mutex erneut um Queue zu konsolidieren
            let commit_lock = self.commit_mutex.lock().unwrap();
            let mut queue_guard = self.pending_commit_queue.lock().unwrap();
            let pending_queue = queue_guard.take().expect("pending commit queue must exist");
            drop(queue_guard);

            let mut all_wal_entries = leader_wal_entries;
            for r in &pending_queue.requests {
                all_wal_entries.extend(r.wal_entries.clone());
            }

            // SEC-01 RACE WINDOW: Leader gibt commit_mutex VOR dem physischen Disk I/O frei!
            drop(commit_lock);

            if let Err(e) = self.wal.append_batch(all_wal_entries) {
                let _commit_lock = self.commit_mutex.lock().unwrap();
                self.wal.restore_last_hmac(pending_queue.first_prev_hmac);
                return Err(e);
            }

            let _commit_lock = self.commit_mutex.lock().unwrap();
            Ok(())
        }
    }

    fn verify_disk_hmac_chain(&self) {
        let disk = self.wal.disk_entries.lock().unwrap().clone();
        let mut expected_prev = [0u8; 32];
        for (i, entry) in disk.iter().enumerate() {
            assert_eq!(
                entry.prev_hmac, expected_prev,
                "CRITICAL SEC-01 RACE TRIGGERED: HMAC Chain link broken on disk at index {i}! Entry tx_id={} has prev_hmac={:?}, expected={:?}",
                entry.tx_id, entry.prev_hmac, expected_prev
            );
            expected_prev = entry.checksum;
        }
    }
}

#[test]
fn test_loom_group_commit_last_hmac_race() {
    loom::model(|| {
        let engine = Arc::new(GroupCommitEngine::new());
        let mut handles = Vec::new();

        // Simuliere 3 konkurrierende Threads
        for i in 1..=3 {
            let engine_clone = Arc::clone(&engine);
            handles.push(thread::spawn(move || {
                let _ = engine_clone.commit(i as u64);
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // Validierung: Die HMAC-Kette auf Disk MUSS ohne Lücken oder out-of-order Writes durchgehend valide sein
        engine.verify_disk_hmac_chain();
    });
}

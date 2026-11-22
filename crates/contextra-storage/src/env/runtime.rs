//! # Runtime: Teilbarer Thread-Pool und deterministische Einplanung
//!
//! SPEC §4, §17.1:
//! - Teilbarer Thread-Pool mit `spawn(Box<dyn FnOnce + Send>)`.
//! - `Runtime::manual()` führt Jobs ausschließlich auf `run_pending()` aus, in fester Reihenfolge.
//! - Lanes: `urgent` und `background`, je Instanz höchstens ein Job pro Lane.

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

/// Wartungs-Lanes gemäß SPEC §12.0 und §17.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lane {
    /// Dringende Aufgaben (Erase, Flush, fsync).
    Urgent,
    /// Hintergrund-Aufgaben (Compaction, Blob-GC, Segment-Bau).
    Background,
}

/// Fehler, wenn eine Lane für eine Instanz bereits belegt ist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LaneBusyError {
    /// Instanz-Kennung.
    pub instance_id: u64,
    /// Belegte Lane.
    pub lane: Lane,
}

impl fmt::Display for LaneBusyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Lane {:?} für Instanz {} ist bereits belegt",
            self.lane, self.instance_id
        )
    }
}

impl std::error::Error for LaneBusyError {}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct SharedState {
    queue: VecDeque<Job>,
    active_lanes: HashMap<(u64, Lane), bool>,
    shutdown: bool,
}

struct Inner {
    state: Mutex<SharedState>,
    condvar: Condvar,
    manual: bool,
    workers: Mutex<Vec<thread::JoinHandle<()>>>,
}

fn lock_mutex<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    match m.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Geteilter Laufzeitpool für Hintergrund- und Wartungsaufgaben.
#[derive(Clone)]
pub struct Runtime {
    inner: Arc<Inner>,
}

impl Runtime {
    /// Erstellt eine nebenläufige Runtime mit bis zu `threads` Hintergrund-Threads.
    pub fn new(threads: usize) -> Self {
        let thread_count = threads.clamp(1, 4);
        let inner = Arc::new(Inner {
            state: Mutex::new(SharedState {
                queue: VecDeque::new(),
                active_lanes: HashMap::new(),
                shutdown: false,
            }),
            condvar: Condvar::new(),
            manual: false,
            workers: Mutex::new(Vec::new()),
        });

        let mut handles = Vec::with_capacity(thread_count);
        for _ in 0..thread_count {
            let inner_clone = Arc::clone(&inner);
            let handle = thread::spawn(move || {
                loop {
                    let job = {
                        let mut state = lock_mutex(&inner_clone.state);
                        while state.queue.is_empty() && !state.shutdown {
                            state = match inner_clone.condvar.wait(state) {
                                Ok(g) => g,
                                Err(p) => p.into_inner(),
                            };
                        }
                        if state.shutdown && state.queue.is_empty() {
                            break;
                        }
                        state.queue.pop_front()
                    };

                    if let Some(j) = job {
                        j();
                    }
                }
            });
            handles.push(handle);
        }

        let mut workers = lock_mutex(&inner.workers);
        *workers = handles;
        drop(workers);

        Self { inner }
    }

    /// Erstellt eine manuelle Runtime für deterministische Tests.
    ///
    /// Jobs werden ausschließlich über `run_pending()` in fester FIFO-Reihenfolge ausgeführt.
    pub fn manual() -> Self {
        let inner = Arc::new(Inner {
            state: Mutex::new(SharedState {
                queue: VecDeque::new(),
                active_lanes: HashMap::new(),
                shutdown: false,
            }),
            condvar: Condvar::new(),
            manual: true,
            workers: Mutex::new(Vec::new()),
        });
        Self { inner }
    }

    /// Prüft, ob diese Runtime im manuellen Modus läuft.
    pub fn is_manual(&self) -> bool {
        self.inner.manual
    }

    /// Gibt die Anzahl aktuell wartender Jobs zurück.
    pub fn pending_count(&self) -> usize {
        let state = lock_mutex(&self.inner.state);
        state.queue.len()
    }

    /// Prüft, ob auf einer bestimmten Lane für eine Instanz ein Job aktiv ist.
    pub fn is_lane_busy(&self, instance_id: u64, lane: Lane) -> bool {
        let state = lock_mutex(&self.inner.state);
        matches!(state.active_lanes.get(&(instance_id, lane)), Some(&true))
    }

    /// Übergibt einen Job an den Pool.
    pub fn spawn(&self, job: Box<dyn FnOnce() + Send + 'static>) {
        let mut state = lock_mutex(&self.inner.state);
        if !state.shutdown {
            state.queue.push_back(job);
            if !self.inner.manual {
                self.inner.condvar.notify_one();
            }
        }
    }

    /// Übergibt eine Funktion oder Closure an den Pool.
    pub fn spawn_fn<F: FnOnce() + Send + 'static>(&self, f: F) {
        self.spawn(Box::new(f));
    }

    /// Plant einen Job auf einer bestimmten Lane für eine Instanz ein.
    ///
    /// Garantiert die Strukturvorgabe: Je Instanz höchstens ein Job pro Lane.
    /// Gibt `Err(LaneBusyError)` zurück, falls die Lane für diese Instanz bereits belegt ist.
    pub fn spawn_lane(
        &self,
        instance_id: u64,
        lane: Lane,
        job: Box<dyn FnOnce() + Send + 'static>,
    ) -> Result<(), LaneBusyError> {
        let mut state = lock_mutex(&self.inner.state);
        let key = (instance_id, lane);
        if matches!(state.active_lanes.get(&key), Some(&true)) {
            return Err(LaneBusyError { instance_id, lane });
        }
        state.active_lanes.insert(key, true);

        let inner_clone = Arc::clone(&self.inner);
        let wrapped_job = Box::new(move || {
            job();
            let mut s = lock_mutex(&inner_clone.state);
            s.active_lanes.remove(&key);
        });

        if !state.shutdown {
            state.queue.push_back(wrapped_job);
            if !self.inner.manual {
                self.inner.condvar.notify_one();
            }
        }
        Ok(())
    }

    /// Führt alle ausstehenden Jobs in FIFO-Reihenfolge aus (nur im manuellen Modus sinnvoll).
    ///
    /// Liefert die Anzahl der ausgeführten Jobs zurück.
    pub fn run_pending(&self) -> usize {
        let mut count: usize = 0;
        loop {
            let next_job = {
                let mut state = lock_mutex(&self.inner.state);
                state.queue.pop_front()
            };
            match next_job {
                Some(job) => {
                    job();
                    count = count.saturating_add(1);
                }
                None => break,
            }
        }
        count
    }

    /// Beendet den Thread-Pool geordnet.
    pub fn shutdown(&self) {
        {
            let mut state = lock_mutex(&self.inner.state);
            state.shutdown = true;
            self.inner.condvar.notify_all();
        }
        let mut workers = lock_mutex(&self.inner.workers);
        for handle in workers.drain(..) {
            let _ = handle.join();
        }
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        let mut state = lock_mutex(&self.state);
        state.shutdown = true;
        self.condvar.notify_all();
    }
}

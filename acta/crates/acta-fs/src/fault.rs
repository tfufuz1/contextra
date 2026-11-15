//! `FaultVfs`: deterministische Fehlerinjektion über einem beliebigen `Vfs` (Spec 16.2, 16.4).
//!
//! Ein [`FaultPlan`] löst genau eine Störung aus, am `skip + 1`-ten Aufruf des gewählten
//! Operationspunkts. Nach [`FaultAction::Crash`] oder einem zerrissenen Schreibvorgang
//! gilt der Zustand als abgestürzt: jede weitere Operation scheitert. Damit gelangt
//! nach dem Absturzpunkt nichts mehr auf den Datenträger.
//!
//! Die Störung wirkt nur auf die Operationen dieses Wrappers. Der innere Port bleibt
//! unverändert, sodass ein Test nach dem Absturz mit einem frischen Wrapper oder direkt
//! mit dem inneren Port erneut öffnen und prüfen kann.

use crate::error::FsyncFailed;
use crate::vfs::{VFile, Vfs};
use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// Operationspunkte, an denen eine Störung ausgelöst werden kann.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FaultPoint {
    /// `create_new`.
    Create,
    /// `write_all_at` auf einem Dateihandle.
    Write,
    /// `sync_data` auf einem Dateihandle.
    Sync,
    /// `open_read`, `open_direct_read` und `read_exact_at`.
    Read,
    /// `rename_atomic`.
    Rename,
    /// `remove_scrub`.
    Remove,
}

/// Art der Störung.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultAction {
    /// Die Operation scheitert mit diesem Fehlerbild, etwa `StorageFull` für ENOSPC.
    /// Bei `Sync` wird die Datei entwertet (Spec 3.7, WP-7).
    Fail(io::ErrorKind),
    /// Zerrissener Schreibvorgang: die ersten `n` Bytes erreichen den Datenträger,
    /// danach tritt ein Absturz ein. An anderen Punkten wirkt dies wie `Crash`.
    TornWrite(usize),
    /// Absturz vor der Operation: sie erreicht den Datenträger nicht, alle späteren scheitern.
    Crash,
}

/// Ein einzelner Störungsplan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaultPlan {
    /// Operationspunkt, an dem gestört wird.
    pub point: FaultPoint,
    /// Anzahl der Aufrufe dieses Punkts, die unverändert bleiben.
    pub skip: u64,
    /// Art der Störung.
    pub action: FaultAction,
}

/// Zustand, den Wrapper und Dateihandles teilen.
#[derive(Debug, Default)]
struct Shared {
    plan: Mutex<Option<FaultPlan>>,
    counts: Mutex<BTreeMap<FaultPoint, u64>>,
    crashed: AtomicBool,
}

impl Shared {
    fn check_alive(&self) -> io::Result<()> {
        if self.crashed.load(Ordering::SeqCst) {
            Err(crash_error())
        } else {
            Ok(())
        }
    }

    /// Zählt den Aufruf und liefert die Störung, falls der Plan jetzt greift.
    fn decide(&self, point: FaultPoint) -> io::Result<Option<FaultAction>> {
        self.check_alive()?;
        let seen = {
            let mut counts = lock(&self.counts);
            let entry = counts.entry(point).or_insert(0);
            let before = *entry;
            *entry += 1;
            before
        };
        let mut plan = lock(&self.plan);
        if let Some(p) = *plan {
            if p.point == point && seen == p.skip {
                *plan = None;
                return Ok(Some(p.action));
            }
        }
        Ok(None)
    }

    fn crash(&self) {
        self.crashed.store(true, Ordering::SeqCst);
    }
}

/// Wrapper über einem `Vfs` mit Fehlerinjektion.
#[derive(Debug)]
pub struct FaultVfs<V: Vfs> {
    inner: V,
    shared: Arc<Shared>,
}

impl<V: Vfs> FaultVfs<V> {
    /// Umhüllt `inner` ohne aktiven Plan.
    pub fn new(inner: V) -> Self {
        Self {
            inner,
            shared: Arc::new(Shared::default()),
        }
    }

    /// Setzt den Plan. Ein zuvor gesetzter, noch nicht ausgelöster Plan wird ersetzt.
    pub fn arm(&self, plan: FaultPlan) {
        *lock(&self.shared.plan) = Some(plan);
    }

    /// Ist der Zustand abgestürzt? Nach `true` scheitert jede weitere Operation.
    pub fn is_crashed(&self) -> bool {
        self.shared.crashed.load(Ordering::SeqCst)
    }

    /// Anzahl der bisherigen Aufrufe eines Operationspunkts (ausgelöste eingeschlossen).
    pub fn count(&self, point: FaultPoint) -> u64 {
        lock(&self.shared.counts).get(&point).copied().unwrap_or(0)
    }

    fn wrap(&self, inner: Box<dyn VFile>) -> Box<dyn VFile> {
        Box::new(FaultFile {
            inner,
            shared: Arc::clone(&self.shared),
            failed: AtomicBool::new(false),
        })
    }

    fn gate(&self, point: FaultPoint) -> io::Result<()> {
        match self.shared.decide(point)? {
            None => Ok(()),
            Some(action) => Err(plain_fault(&self.shared, action)),
        }
    }
}

impl<V: Vfs> Vfs for FaultVfs<V> {
    fn create_new(&self, p: &Path) -> io::Result<Box<dyn VFile>> {
        self.gate(FaultPoint::Create)?;
        let inner = self.inner.create_new(p)?;
        Ok(self.wrap(inner))
    }

    fn open_read(&self, p: &Path) -> io::Result<Box<dyn VFile>> {
        self.gate(FaultPoint::Read)?;
        let inner = self.inner.open_read(p)?;
        Ok(self.wrap(inner))
    }

    fn open_direct_read(&self, p: &Path) -> io::Result<Box<dyn VFile>> {
        self.gate(FaultPoint::Read)?;
        let inner = self.inner.open_direct_read(p)?;
        Ok(self.wrap(inner))
    }

    fn rename_atomic(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.gate(FaultPoint::Rename)?;
        self.inner.rename_atomic(from, to)
    }

    fn remove_scrub(&self, p: &Path) -> io::Result<()> {
        self.gate(FaultPoint::Remove)?;
        self.inner.remove_scrub(p)
    }

    fn list(&self, dir: &Path) -> io::Result<Vec<PathBuf>> {
        self.shared.check_alive()?;
        self.inner.list(dir)
    }
}

/// Dateihandle unter Fehlerinjektion.
struct FaultFile {
    inner: Box<dyn VFile>,
    shared: Arc<Shared>,
    /// Fsync-Versagen durch Injektion (WP-7). Das innere Handle kennt es nicht.
    failed: AtomicBool,
}

impl VFile for FaultFile {
    fn write_all_at(&self, off: u64, buf: &[u8]) -> io::Result<()> {
        if self.failed.load(Ordering::SeqCst) {
            return Err(io::Error::other(
                "file refused: an earlier fsync failed (WP-7)",
            ));
        }
        match self.shared.decide(FaultPoint::Write)? {
            None => self.inner.write_all_at(off, buf),
            Some(FaultAction::Fail(kind)) => Err(io::Error::new(kind, "injected write fault")),
            Some(FaultAction::TornWrite(n)) => {
                let k = n.min(buf.len());
                if let Some(head) = buf.get(..k) {
                    self.inner.write_all_at(off, head)?;
                }
                self.shared.crash();
                Err(crash_error())
            }
            Some(FaultAction::Crash) => {
                self.shared.crash();
                Err(crash_error())
            }
        }
    }

    fn read_exact_at(&self, off: u64, buf: &mut [u8]) -> io::Result<()> {
        match self.shared.decide(FaultPoint::Read)? {
            None => self.inner.read_exact_at(off, buf),
            Some(action) => Err(plain_fault(&self.shared, action)),
        }
    }

    fn sync_data(&self) -> Result<(), FsyncFailed> {
        if self.failed.load(Ordering::SeqCst) {
            return Err(FsyncFailed);
        }
        match self.shared.decide(FaultPoint::Sync) {
            Ok(None) => self.inner.sync_data(),
            Ok(Some(action)) => {
                if matches!(action, FaultAction::Crash | FaultAction::TornWrite(_)) {
                    self.shared.crash();
                }
                self.failed.store(true, Ordering::SeqCst);
                Err(FsyncFailed)
            }
            Err(_) => {
                // Bereits abgestürzt: die Synchronisation hat den Datenträger nicht erreicht.
                self.failed.store(true, Ordering::SeqCst);
                Err(FsyncFailed)
            }
        }
    }

    fn len(&self) -> io::Result<u64> {
        self.shared.check_alive()?;
        self.inner.len()
    }
}

/// Wandelt eine ausgelöste Störung an einem Punkt ohne Schreibsemantik in einen Fehler.
fn plain_fault(shared: &Shared, action: FaultAction) -> io::Error {
    match action {
        FaultAction::Fail(kind) => io::Error::new(kind, "injected fault"),
        FaultAction::Crash | FaultAction::TornWrite(_) => {
            shared.crash();
            crash_error()
        }
    }
}

fn crash_error() -> io::Error {
    io::Error::other("injected crash: the system is down")
}

/// Sperre ohne Abbruch bei Vergiftung: ein früherer Panikpfad darf den Plan nicht unbrauchbar machen.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

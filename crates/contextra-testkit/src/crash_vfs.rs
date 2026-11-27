//! # Absturzfähiges virtuelles Dateisystem (`crash_vfs`)
//!
//! [`CrashVfs`] trennt für jede Datei und jeden Verzeichniseintrag den *flüchtigen* vom
//! *durablen* Zustand und kann nach jeder mutierenden Operation „abstürzen“.
//!
//! ## Modell (POSIX-Semantik)
//! - `write_at` / `set_len` ändern nur den flüchtigen Inhalt.
//! - `sync_data` / `sync_all` machen den Inhalt einer Datei durabel — **nicht** ihren Namen.
//! - `sync_dir(p)` macht alle Namensänderungen unterhalb von `p` durabel. Ein Rename über
//!   zwei Verzeichnisse wird erst durabel, wenn beide Namen abgedeckt wurden.
//! - Eine Datei überlebt einen Neustart nur, wenn auch alle Elternverzeichnisse überleben.
//! - `sync_dir(p)` macht **nicht** den Namen von `p` selbst durabel (dafür den Elternordner
//!   synchronisieren). Nach einem Verzeichnis-Rename bleiben ältere, noch ungesyncte
//!   Änderungen unter dem alten Pfad ungesynct (strenger als reale Dateisysteme).
//! - `write_atomic` ist **eine** gezählte Operation: danach gilt alt **oder** neu, nie gemischt.
//!   Zurückbleibende Temp-Dateien realer Implementierungen werden nicht modelliert.
//!
//! ## Zählung
//! Gezählt werden nur **erfolgreich zulässige** mutierende Operationen. Nach dem Crashpunkt
//! liefert **jede** Operation (auch lesende) `io::ErrorKind::Interrupted`. Code unter Test
//! darf diesen Fehler deshalb nicht in einer Endlosschleife wiederholen.
//!
//! ## Grenzen
//! Keine Controller-/Flash-Eigenschaften, keine CoW-Dateisysteme. Bewiesen wird die
//! Protokollreihenfolge Write-Ahead → Wirkung → Sync, nicht die Hardware.

use contextra_ports::{Vfs, VfsFile, VfsOpenOptions};
use contextra_types::{ContextraError, Result};
use parking_lot::Mutex;
use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Startwert aller seed-gesteuerten Entscheidungen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CrashSeed(pub u64);

/// Deterministischer xorshift64*-Generator (reine Testlogik).
#[derive(Debug, Clone)]
pub(crate) struct XorShift(u64);

impl XorShift {
    pub(crate) fn new(seed: u64) -> Self {
        // splitmix64-Schritt, damit benachbarte Seeds unabhängige Folgen liefern.
        let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        Self((z ^ (z >> 31)).max(1))
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Gleichverteilt (bis auf Modulo-Verzerrung) in `0..n`; `0` bei `n == 0`.
    pub(crate) fn below(&mut self, n: u64) -> u64 {
        if n == 0 {
            0
        } else {
            self.next_u64() % n
        }
    }
}

/// Art einer gezählten Operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpKind {
    /// Datei oder Verzeichnis angelegt.
    Create,
    /// `write_at`.
    Write,
    /// `set_len` (auch `truncate` beim Öffnen).
    SetLen,
    /// `rename`.
    Rename,
    /// `remove_file` / `unlink`.
    Remove,
    /// `sync_data`.
    SyncData,
    /// `sync_all`.
    SyncAll,
    /// `sync_dir`.
    SyncDir,
    /// `write_atomic`.
    WriteAtomic,
    /// Testhilfe [`CrashVfs::link`].
    Link,
}

/// Protokolleintrag einer ausgeführten Operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpRecord {
    /// Operationsart.
    pub kind: OpKind,
    /// FNV-1a-Hash des Pfades (bei `rename`/`link`: Zielpfad), siehe [`path_hash`].
    pub path_hash: u64,
}

/// Deterministischer Pfad-Hash für [`OpRecord`].
pub fn path_hash(path: &Path) -> u64 {
    path.as_os_str()
        .as_encoded_bytes()
        .iter()
        .fold(0xCBF2_9CE4_8422_2325, |h, b| {
            (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01B3)
        })
}

/// Injizierbare Fehlerart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrKind {
    /// `ENOSPC`.
    NoSpace,
    /// `EIO`.
    Io,
    /// `EROFS`.
    ReadOnly,
}

impl ErrKind {
    fn to_error(self) -> io::Error {
        match self {
            Self::NoSpace => io::Error::new(ErrorKind::StorageFull, "injected ENOSPC"),
            Self::Io => io::Error::other("injected EIO"),
            Self::ReadOnly => io::Error::new(ErrorKind::ReadOnlyFilesystem, "injected EROFS"),
        }
    }
}

/// Verhalten beim Neustart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrashPolicy {
    /// Nur Durables überlebt.
    DropUnsynced,
    /// Ungesyncte Schreibvorgänge überleben bis zu einem seed-gewählten Präfix je Datei.
    TornTail {
        /// Sektorgröße (1 = byte-genau, 512, 4096).
        sector: u32,
    },
    /// Ungesyncte Namensänderungen werden in seed-gewählter Teilmenge angewandt.
    PartialNamespace,
    /// Alles Flüchtige überlebt (Prozess-Kill ohne OS-Crash).
    KeepAll,
    /// Wie `DropUnsynced`, zusätzlich ein Bitflip in einer durablen Datei unter dem Präfix.
    BitFlip {
        /// Pfadpräfix der Kandidaten.
        path_prefix: PathBuf,
    },
}

/// Inhalt eines Pfades in beiden Zuständen (`None` = Name existiert dort nicht).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileState {
    /// Inhalt, falls der Name durabel ist.
    pub durable: Option<Vec<u8>>,
    /// Inhalt, falls der Name flüchtig sichtbar ist.
    pub volatile: Option<Vec<u8>>,
}

/// Abbild aller Dateien.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreeSnapshot {
    /// Pfad → Zustände.
    pub files: BTreeMap<PathBuf, FileState>,
}

impl TreeSnapshot {
    /// Pfade, deren durabler oder flüchtiger Inhalt `needle` enthält.
    pub fn find(&self, needle: &[u8]) -> Vec<PathBuf> {
        let hit = |c: &Option<Vec<u8>>| {
            !needle.is_empty()
                && c.as_ref()
                    .is_some_and(|c| c.windows(needle.len()).any(|w| w == needle))
        };
        self.files
            .iter()
            .filter(|(_, s)| hit(&s.durable) || hit(&s.volatile))
            .map(|(p, _)| p.clone())
            .collect()
    }
}

#[derive(Debug, Clone)]
enum DataOp {
    Write { offset: u64, data: Vec<u8> },
    SetLen(u64),
}

#[derive(Debug, Clone, Default)]
struct Inode {
    durable: Vec<u8>,
    volatile: Vec<u8>,
    unsynced: Vec<DataOp>,
}

impl Inode {
    fn synced(data: Vec<u8>) -> Self {
        Self {
            durable: data.clone(),
            volatile: data,
            unsynced: Vec::new(),
        }
    }

    fn sync(&mut self) {
        self.durable.clone_from(&self.volatile);
        self.unsynced.clear();
    }
}

#[derive(Debug, Clone, Default)]
struct Names {
    files: BTreeMap<PathBuf, u64>,
    dirs: BTreeSet<PathBuf>,
}

impl Names {
    fn apply(&mut self, op: &NsOp) {
        match op {
            NsOp::Mkdir(p) => {
                self.dirs.insert(p.clone());
            }
            NsOp::Link { path, ino } => {
                self.files.insert(path.clone(), *ino);
            }
            NsOp::Unlink(p) => {
                self.files.remove(p);
            }
            NsOp::Rename { from, to, ino } => {
                if self.files.get(from) == Some(ino) {
                    self.files.remove(from);
                }
                self.files.insert(to.clone(), *ino);
            }
            NsOp::RenameDir { from, to } => {
                let moved = |p: &Path| p.strip_prefix(from).ok().map(|rel| join_rel(to, rel));
                self.dirs = std::mem::take(&mut self.dirs)
                    .into_iter()
                    .map(|p| moved(&p).unwrap_or(p))
                    .collect();
                self.files = std::mem::take(&mut self.files)
                    .into_iter()
                    .map(|(p, i)| (moved(&p).unwrap_or(p), i))
                    .collect();
            }
        }
    }

    /// Entfernt Einträge, deren Elternkette nicht existiert.
    fn prune_orphans(&mut self) {
        // BTreeSet-Ordnung: Eltern stehen vor ihren Kindern.
        let mut kept = BTreeSet::new();
        for d in std::mem::take(&mut self.dirs) {
            if parent_exists(&kept, &d) {
                kept.insert(d);
            }
        }
        self.dirs = kept;
        let dirs = &self.dirs;
        self.files.retain(|p, _| parent_exists(dirs, p));
    }
}

fn join_rel(base: &Path, rel: &Path) -> PathBuf {
    if rel.as_os_str().is_empty() {
        base.to_path_buf()
    } else {
        base.join(rel)
    }
}

fn is_root(p: &Path) -> bool {
    p.as_os_str().is_empty() || p == Path::new("/") || p == Path::new(".")
}

fn parent_exists(dirs: &BTreeSet<PathBuf>, p: &Path) -> bool {
    p.parent()
        .is_none_or(|parent| is_root(parent) || dirs.contains(parent))
}

fn under(path: &Path, dir: &Path) -> bool {
    if dir == Path::new(".") {
        path.is_relative()
    } else {
        path.starts_with(dir)
    }
}

#[derive(Debug, Clone)]
enum NsOp {
    Mkdir(PathBuf),
    Link {
        path: PathBuf,
        ino: u64,
    },
    Unlink(PathBuf),
    Rename {
        from: PathBuf,
        to: PathBuf,
        ino: u64,
    },
    RenameDir {
        from: PathBuf,
        to: PathBuf,
    },
}

impl NsOp {
    fn names(&self) -> Vec<PathBuf> {
        match self {
            Self::Mkdir(p) | Self::Unlink(p) | Self::Link { path: p, .. } => vec![p.clone()],
            Self::Rename { from, to, .. } | Self::RenameDir { from, to } => {
                vec![from.clone(), to.clone()]
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Pending {
    op: NsOp,
    /// Noch nicht durch `sync_dir` abgedeckte Namen.
    uncovered: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
struct Fault {
    prefix: PathBuf,
    remaining: u64,
    kind: ErrKind,
}

#[derive(Debug)]
struct State {
    seed: CrashSeed,
    inodes: BTreeMap<u64, Inode>,
    next_ino: u64,
    volatile: Names,
    durable: Names,
    pending: Vec<Pending>,
    op_count: u64,
    crash_after: Option<u64>,
    crash_before: Option<u64>,
    crashed: bool,
    sync_faults: Vec<Fault>,
    write_faults: Vec<Fault>,
    log: Vec<OpRecord>,
    marks: Vec<(String, u64)>,
}

fn interrupted() -> io::Error {
    io::Error::new(ErrorKind::Interrupted, "simulated crash")
}

fn not_found(p: &Path) -> io::Error {
    io::Error::new(ErrorKind::NotFound, format!("not found: {}", p.display()))
}

fn invalid(msg: &'static str) -> io::Error {
    io::Error::new(ErrorKind::InvalidInput, msg)
}

fn splice(buf: &mut Vec<u8>, offset: usize, data: &[u8]) -> io::Result<()> {
    let end = offset
        .checked_add(data.len())
        .ok_or_else(|| invalid("write offset overflow"))?;
    if buf.len() < end {
        buf.resize(end, 0);
    }
    if let Some(dst) = buf.get_mut(offset..end) {
        dst.copy_from_slice(data);
    }
    Ok(())
}

/// Obergrenze der modellierten Dateigröße; schützt vor Speicherexplosion durch defekte Offsets.
const MAX_FILE: u64 = 1 << 30;

fn checked_size(n: u64) -> io::Result<usize> {
    if n > MAX_FILE {
        return Err(invalid("file size exceeds CrashVfs limit (1 GiB)"));
    }
    usize::try_from(n).map_err(|_| invalid("size does not fit usize"))
}

impl State {
    fn alive(&self) -> io::Result<()> {
        if self.crashed {
            Err(interrupted())
        } else {
            Ok(())
        }
    }

    /// Vor jeder mutierenden Operation: Crashplan prüfen.
    fn begin(&mut self) -> io::Result<()> {
        self.alive()?;
        let next = self.op_count.saturating_add(1);
        if self.crash_before.is_some_and(|n| next >= n) {
            self.crashed = true;
            return Err(interrupted());
        }
        Ok(())
    }

    /// Nach erfolgreicher Ausführung: zählen, protokollieren, ggf. abstürzen.
    fn finish(&mut self, kind: OpKind, path: &Path) {
        self.op_count = self.op_count.saturating_add(1);
        self.log.push(OpRecord {
            kind,
            path_hash: path_hash(path),
        });
        if self.crash_after.is_some_and(|n| self.op_count >= n) {
            self.crashed = true;
        }
    }

    fn fault(faults: &mut [Fault], path: &Path) -> io::Result<()> {
        for f in faults.iter_mut().filter(|f| under(path, &f.prefix)) {
            if f.remaining == 0 {
                return Err(f.kind.to_error());
            }
            f.remaining -= 1;
        }
        Ok(())
    }

    fn inode(&mut self, ino: u64) -> io::Result<&mut Inode> {
        self.inodes
            .get_mut(&ino)
            .ok_or_else(|| io::Error::other("dangling inode"))
    }

    fn file(&self, path: &Path) -> io::Result<u64> {
        self.volatile
            .files
            .get(path)
            .copied()
            .ok_or_else(|| not_found(path))
    }

    fn dir_exists(&self, p: &Path) -> bool {
        is_root(p) || self.volatile.dirs.contains(p)
    }

    fn require_parent(&self, p: &Path) -> io::Result<()> {
        if parent_exists(&self.volatile.dirs, p) {
            Ok(())
        } else {
            Err(not_found(p.parent().unwrap_or(p)))
        }
    }

    fn require_free_file_name(&self, p: &Path) -> io::Result<()> {
        if self.dir_exists(p) {
            return Err(io::Error::new(
                ErrorKind::IsADirectory,
                "path is a directory",
            ));
        }
        self.require_parent(p)
    }

    fn alloc(&mut self, inode: Inode) -> u64 {
        let ino = self.next_ino;
        self.next_ino += 1;
        self.inodes.insert(ino, inode);
        ino
    }

    fn namespace(&mut self, op: NsOp) {
        let uncovered = op.names();
        self.volatile.apply(&op);
        self.pending.push(Pending { op, uncovered });
    }

    fn sync_names(&mut self, dir: &Path) {
        self.commit_names(|name| name != dir && under(name, dir));
    }

    /// Macht abgedeckte Namensänderungen durabel. Ältere Änderungen, die denselben Namen
    /// betreffen wie eine jetzt durable, werden mit übernommen: sonst überschriebe ein
    /// späterer Sync den neueren Stand mit dem älteren.
    fn commit_names(&mut self, covers: impl Fn(&Path) -> bool) {
        let mut ready = Vec::new();
        let mut rest = Vec::new();
        let mut claimed: BTreeSet<PathBuf> = BTreeSet::new();
        for mut p in std::mem::take(&mut self.pending).into_iter().rev() {
            p.uncovered.retain(|name| !covers(name));
            let names = p.op.names();
            if p.uncovered.is_empty() || names.iter().any(|n| claimed.contains(n)) {
                claimed.extend(names);
                ready.push(p.op);
            } else {
                rest.push(p);
            }
        }
        for op in ready.iter().rev() {
            self.durable.apply(op);
        }
        rest.reverse();
        self.pending = rest;
    }

    fn survivors(&self, policy: &CrashPolicy) -> (Names, BTreeMap<u64, Vec<u8>>) {
        let mut rng = XorShift::new(self.seed.0 ^ self.op_count.rotate_left(32));
        let mut names = match policy {
            CrashPolicy::KeepAll => self.volatile.clone(),
            CrashPolicy::PartialNamespace => {
                let mut n = self.durable.clone();
                for p in &self.pending {
                    if rng.below(2) == 1 {
                        n.apply(&p.op);
                    }
                }
                n
            }
            _ => self.durable.clone(),
        };
        names.prune_orphans();

        let mut data = BTreeMap::new();
        for ino in names.files.values() {
            if data.contains_key(ino) {
                continue;
            }
            let Some(inode) = self.inodes.get(ino) else {
                continue;
            };
            let content = match policy {
                CrashPolicy::KeepAll => inode.volatile.clone(),
                CrashPolicy::TornTail { sector } => {
                    torn(inode, u64::from((*sector).max(1)), &mut rng)
                }
                _ => inode.durable.clone(),
            };
            data.insert(*ino, content);
        }

        if let CrashPolicy::BitFlip { path_prefix } = policy {
            let candidates: Vec<u64> = names
                .files
                .iter()
                .filter(|(p, i)| {
                    under(p, path_prefix) && data.get(i).is_some_and(|d| !d.is_empty())
                })
                .map(|(_, i)| *i)
                .collect();
            let pick = usize::try_from(rng.below(candidates.len() as u64)).unwrap_or(0);
            if let Some(bytes) = candidates.get(pick).and_then(|i| data.get_mut(i)) {
                let bit = rng.below(bytes.len() as u64 * 8);
                if let Some(b) = usize::try_from(bit / 8).ok().and_then(|i| bytes.get_mut(i)) {
                    *b ^= 1 << (bit % 8);
                }
            }
        }
        (names, data)
    }
}

/// Wendet ein seed-gewähltes Byte-Präfix der ungesyncten Schreibfolge an.
fn torn(inode: &Inode, sector: u64, rng: &mut XorShift) -> Vec<u8> {
    let total: u64 = inode
        .unsynced
        .iter()
        .map(|op| match op {
            DataOp::Write { data, .. } => data.len() as u64,
            DataOp::SetLen(_) => 1,
        })
        .sum();
    let mut budget = rng.below(total + 1);
    let mut out = inode.durable.clone();
    for op in &inode.unsynced {
        if budget == 0 {
            break;
        }
        match op {
            DataOp::SetLen(n) => {
                out.resize(usize::try_from(*n).unwrap_or(out.len()), 0);
                budget -= 1;
            }
            DataOp::Write { offset, data } => {
                let len = data.len() as u64;
                let mut take = len.min(budget);
                if take < len {
                    // Abriss nur an Sektorgrenzen der Datei.
                    take = ((offset + take) / sector * sector).saturating_sub(*offset);
                    budget = 0;
                } else {
                    budget -= len;
                }
                let prefix = usize::try_from(take).ok().and_then(|t| data.get(..t));
                if let (Ok(off), Some(prefix)) = (usize::try_from(*offset), prefix) {
                    // Offsets wurden beim Schreiben geprüft; ein leerer Abriss ändert nichts.
                    if !prefix.is_empty() && splice(&mut out, off, prefix).is_err() {
                        break;
                    }
                }
            }
        }
    }
    out
}

/// Deterministisches In-Memory-Dateisystem mit Absturzsimulation. `Clone` teilt den Zustand.
#[derive(Debug, Clone)]
pub struct CrashVfs {
    state: Arc<Mutex<State>>,
}

fn lift<T>(r: io::Result<T>) -> Result<T> {
    r.map_err(ContextraError::Io)
}

impl CrashVfs {
    /// Leeres, nicht abgestürztes Dateisystem.
    pub fn new(seed: CrashSeed) -> Self {
        Self::from_parts(seed, Names::default(), BTreeMap::new(), 1)
    }

    fn from_parts(seed: CrashSeed, names: Names, data: BTreeMap<u64, Vec<u8>>, next: u64) -> Self {
        let inodes = data
            .into_iter()
            .map(|(i, d)| (i, Inode::synced(d)))
            .collect();
        Self {
            state: Arc::new(Mutex::new(State {
                seed,
                inodes,
                next_ino: next,
                volatile: names.clone(),
                durable: names,
                pending: Vec::new(),
                op_count: 0,
                crash_after: None,
                crash_before: None,
                crashed: false,
                sync_faults: Vec::new(),
                write_faults: Vec::new(),
                log: Vec::new(),
                marks: Vec::new(),
            })),
        }
    }

    /// Seed dieser Instanz.
    pub fn seed(&self) -> CrashSeed {
        self.state.lock().seed
    }

    /// Anzahl ausgeführter mutierender Operationen.
    pub fn op_count(&self) -> u64 {
        self.state.lock().op_count
    }

    /// `true`, sobald der Crashpunkt erreicht wurde.
    pub fn is_crashed(&self) -> bool {
        self.state.lock().crashed
    }

    /// Absturz **nach** der n-ten Operation (sie wird noch ausgeführt). `0` = sofort.
    pub fn crash_after(&self, n: u64) {
        let mut s = self.state.lock();
        s.crash_after = Some(n);
        if s.op_count >= n {
            s.crashed = true;
        }
    }

    /// Absturz **vor** der n-ten Operation (1-basiert; sie wird nicht ausgeführt).
    pub fn crash_before(&self, n: u64) {
        self.state.lock().crash_before = Some(n);
    }

    /// Zustand nach Neustart gemäß `policy` als frisches Dateisystem. `self` bleibt unverändert.
    pub fn restart(&self, policy: CrashPolicy) -> CrashVfs {
        let s = self.state.lock();
        let (names, data) = s.survivors(&policy);
        Self::from_parts(s.seed, names, data, s.next_ino)
    }

    /// Syncs unter `path_prefix` schlagen nach `after` erfolgreichen dauerhaft fehl.
    pub fn fail_sync(&self, path_prefix: &Path, after: u64, errno: ErrKind) {
        self.state.lock().sync_faults.push(Fault {
            prefix: path_prefix.to_path_buf(),
            remaining: after,
            kind: errno,
        });
    }

    /// Schreibende Operationen (`create`, `write_at`, `set_len`, `write_atomic`) unter
    /// `path_prefix` schlagen nach `after` erfolgreichen dauerhaft fehl.
    pub fn fail_write(&self, path_prefix: &Path, after: u64, errno: ErrKind) {
        self.state.lock().write_faults.push(Fault {
            prefix: path_prefix.to_path_buf(),
            remaining: after,
            kind: errno,
        });
    }

    /// Testhilfe: Hardlink (zwei Namen, ein Inode). Nicht Teil des `Vfs`-Ports.
    pub fn link(&self, from: &Path, to: &Path) -> io::Result<()> {
        let mut s = self.state.lock();
        s.alive()?;
        let ino = s.file(from)?;
        if s.volatile.files.contains_key(to) {
            return Err(io::Error::new(
                ErrorKind::AlreadyExists,
                "link target exists",
            ));
        }
        s.require_free_file_name(to)?;
        s.begin()?;
        s.namespace(NsOp::Link {
            path: to.to_path_buf(),
            ino,
        });
        s.finish(OpKind::Link, to);
        Ok(())
    }

    /// Alle Dateien mit durablem und flüchtigem Inhalt (auch nach einem Crash lesbar).
    pub fn snapshot_tree(&self) -> TreeSnapshot {
        let s = self.state.lock();
        let mut files: BTreeMap<PathBuf, FileState> = BTreeMap::new();
        for (p, i) in &s.durable.files {
            files.entry(p.clone()).or_default().durable =
                s.inodes.get(i).map(|n| n.durable.clone());
        }
        for (p, i) in &s.volatile.files {
            files.entry(p.clone()).or_default().volatile =
                s.inodes.get(i).map(|n| n.volatile.clone());
        }
        TreeSnapshot { files }
    }

    /// Vollständiges Operationsprotokoll in Ausführungsreihenfolge.
    pub fn last_ops(&self) -> Vec<OpRecord> {
        self.state.lock().log.clone()
    }

    /// Benannter Marker am aktuellen Op-Zähler (nur für Szenario-Code, nie für Code unter Test).
    pub fn mark(&self, name: &str) {
        let mut s = self.state.lock();
        let at = s.op_count;
        s.marks.push((name.to_string(), at));
    }

    /// Gesetzte Marker als `(Name, Op-Zähler)`.
    pub fn marks(&self) -> Vec<(String, u64)> {
        self.state.lock().marks.clone()
    }
}

struct CrashFile {
    state: Arc<Mutex<State>>,
    ino: u64,
    path: PathBuf,
    opts: VfsOpenOptions,
}

impl CrashFile {
    fn sync(&self, kind: OpKind) -> io::Result<()> {
        let mut s = self.state.lock();
        s.begin()?;
        State::fault(&mut s.sync_faults, &self.path)?;
        s.inode(self.ino)?.sync();
        s.finish(kind, &self.path);
        Ok(())
    }

    fn require_write(&self) -> io::Result<()> {
        if self.opts.write || self.opts.append {
            Ok(())
        } else {
            Err(io::Error::new(
                ErrorKind::PermissionDenied,
                "file not opened for writing",
            ))
        }
    }
}

impl VfsFile for CrashFile {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        let mut s = self.state.lock();
        s.alive()?;
        if !self.opts.read {
            return Err(io::Error::new(
                ErrorKind::PermissionDenied,
                "file not opened for reading",
            ));
        }
        let data = &s.inode(self.ino)?.volatile;
        let tail = usize::try_from(offset)
            .ok()
            .and_then(|o| data.get(o..))
            .unwrap_or(&[]);
        let n = tail.len().min(buf.len());
        if let (Some(dst), Some(src)) = (buf.get_mut(..n), tail.get(..n)) {
            dst.copy_from_slice(src);
        }
        Ok(n)
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize> {
        self.require_write()?;
        let mut s = self.state.lock();
        s.begin()?;
        State::fault(&mut s.write_faults, &self.path)?;
        let append = self.opts.append;
        let inode = s.inode(self.ino)?;
        let offset = if append {
            inode.volatile.len() as u64
        } else {
            offset
        };
        // POSIX: ein Schreibvorgang der Länge 0 verlängert die Datei nicht.
        if !buf.is_empty() {
            checked_size(offset.saturating_add(buf.len() as u64))?;
            splice(&mut inode.volatile, checked_size(offset)?, buf)?;
            inode.unsynced.push(DataOp::Write {
                offset,
                data: buf.to_vec(),
            });
        }
        s.finish(OpKind::Write, &self.path);
        Ok(buf.len())
    }

    fn set_len(&self, len: u64) -> io::Result<()> {
        self.require_write()?;
        let mut s = self.state.lock();
        s.begin()?;
        State::fault(&mut s.write_faults, &self.path)?;
        let size = checked_size(len)?;
        let inode = s.inode(self.ino)?;
        inode.volatile.resize(size, 0);
        inode.unsynced.push(DataOp::SetLen(len));
        s.finish(OpKind::SetLen, &self.path);
        Ok(())
    }

    fn len(&self) -> io::Result<u64> {
        let mut s = self.state.lock();
        s.alive()?;
        Ok(s.inode(self.ino)?.volatile.len() as u64)
    }

    fn sync_data(&self) -> io::Result<()> {
        self.sync(OpKind::SyncData)
    }

    fn sync_all(&self) -> io::Result<()> {
        self.sync(OpKind::SyncAll)
    }
}

impl State {
    fn open(&mut self, path: &Path, opts: &VfsOpenOptions) -> io::Result<u64> {
        self.alive()?;
        let writable = opts.write || opts.append;
        if (opts.truncate || opts.create || opts.create_new) && !writable {
            return Err(invalid("create/truncate require write access"));
        }
        let ino = match self.volatile.files.get(path).copied() {
            Some(_) if opts.create_new => {
                return Err(io::Error::new(ErrorKind::AlreadyExists, "file exists"))
            }
            Some(ino) => {
                if opts.truncate && !self.inode(ino)?.volatile.is_empty() {
                    self.begin()?;
                    Self::fault(&mut self.write_faults, path)?;
                    let inode = self.inode(ino)?;
                    inode.volatile.clear();
                    inode.unsynced.push(DataOp::SetLen(0));
                    self.finish(OpKind::SetLen, path);
                    // Dieser Open ist die Crash-Operation selbst: Handle bleibt nutzlos.
                }
                ino
            }
            None if opts.create || opts.create_new => {
                self.require_free_file_name(path)?;
                self.begin()?;
                Self::fault(&mut self.write_faults, path)?;
                let ino = self.alloc(Inode::default());
                self.namespace(NsOp::Link {
                    path: path.to_path_buf(),
                    ino,
                });
                self.finish(OpKind::Create, path);
                ino
            }
            None => return Err(not_found(path)),
        };
        Ok(ino)
    }

    fn write_atomic(&mut self, path: &Path, data: &[u8]) -> io::Result<()> {
        self.alive()?;
        self.require_free_file_name(path)?;
        self.begin()?;
        Self::fault(&mut self.write_faults, path)?;
        Self::fault(&mut self.sync_faults, path)?;
        checked_size(data.len() as u64)?;
        let ino = self.alloc(Inode::synced(data.to_vec()));
        self.namespace(NsOp::Link {
            path: path.to_path_buf(),
            ino,
        });
        // `sync_dir(parent)`: nur die direkten Einträge des Elternverzeichnisses.
        let parent = path.parent();
        self.commit_names(|name| name.parent() == parent);
        self.finish(OpKind::WriteAtomic, path);
        Ok(())
    }

    fn create_dir_all(&mut self, path: &Path) -> io::Result<()> {
        self.alive()?;
        let mut missing: Vec<PathBuf> = path
            .ancestors()
            .filter(|a| !self.dir_exists(a))
            .map(Path::to_path_buf)
            .collect();
        missing.reverse();
        for dir in missing {
            if self.volatile.files.contains_key(&dir) {
                return Err(io::Error::new(ErrorKind::AlreadyExists, "file in the way"));
            }
            self.begin()?;
            self.namespace(NsOp::Mkdir(dir.clone()));
            self.finish(OpKind::Create, &dir);
        }
        Ok(())
    }

    fn remove(&mut self, path: &Path) -> io::Result<()> {
        self.alive()?;
        self.file(path)?;
        self.begin()?;
        self.namespace(NsOp::Unlink(path.to_path_buf()));
        self.finish(OpKind::Remove, path);
        Ok(())
    }

    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        self.alive()?;
        let op = if let Some(ino) = self.volatile.files.get(from).copied() {
            self.require_free_file_name(to)?;
            // POSIX: Rename auf einen Hardlink desselben Inodes ist ein No-op.
            if self.volatile.files.get(to) == Some(&ino) {
                return Ok(());
            }
            NsOp::Rename {
                from: from.to_path_buf(),
                to: to.to_path_buf(),
                ino,
            }
        } else if self.volatile.dirs.contains(from) {
            if to.starts_with(from) {
                return Err(invalid("cannot move a directory into itself"));
            }
            if self.dir_exists(to) || self.volatile.files.contains_key(to) {
                return Err(io::Error::new(
                    ErrorKind::AlreadyExists,
                    "rename target exists",
                ));
            }
            self.require_parent(to)?;
            NsOp::RenameDir {
                from: from.to_path_buf(),
                to: to.to_path_buf(),
            }
        } else {
            return Err(not_found(from));
        };
        self.begin()?;
        self.namespace(op);
        self.finish(OpKind::Rename, to);
        Ok(())
    }

    fn sync_dir(&mut self, path: &Path) -> io::Result<()> {
        self.alive()?;
        if !self.dir_exists(path) {
            return Err(not_found(path));
        }
        self.begin()?;
        Self::fault(&mut self.sync_faults, path)?;
        self.sync_names(path);
        self.finish(OpKind::SyncDir, path);
        Ok(())
    }

    fn list_dir(&self, path: &Path) -> io::Result<Vec<String>> {
        self.alive()?;
        if !self.dir_exists(path) {
            return Err(not_found(path));
        }
        let dir = if path == Path::new(".") {
            Path::new("")
        } else {
            path
        };
        let names: BTreeSet<String> = self
            .volatile
            .files
            .keys()
            .chain(self.volatile.dirs.iter())
            .filter(|p| p.parent() == Some(dir))
            .filter_map(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .collect();
        Ok(names.into_iter().collect())
    }
}

impl Vfs for CrashVfs {
    fn open(&self, path: &Path, opts: &VfsOpenOptions) -> Result<Box<dyn VfsFile>> {
        let ino = lift(self.state.lock().open(path, opts))?;
        Ok(Box::new(CrashFile {
            state: Arc::clone(&self.state),
            ino,
            path: path.to_path_buf(),
            opts: opts.clone(),
        }))
    }

    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        let mut s = self.state.lock();
        lift(s.alive())?;
        let ino = lift(s.file(path))?;
        Ok(lift(s.inode(ino))?.volatile.clone())
    }

    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<()> {
        lift(self.state.lock().write_atomic(path, data))
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        lift(self.state.lock().create_dir_all(path))
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        lift(self.state.lock().remove(path))
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<()> {
        lift(self.state.lock().rename(from, to))
    }

    fn exists(&self, path: &Path) -> Result<bool> {
        let s = self.state.lock();
        lift(s.alive())?;
        Ok(s.volatile.files.contains_key(path) || s.dir_exists(path))
    }

    fn len(&self, path: &Path) -> Result<u64> {
        let mut s = self.state.lock();
        lift(s.alive())?;
        let ino = lift(s.file(path))?;
        Ok(lift(s.inode(ino))?.volatile.len() as u64)
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<String>> {
        lift(self.state.lock().list_dir(path))
    }

    fn sync_dir(&self, path: &Path) -> Result<()> {
        lift(self.state.lock().sync_dir(path))
    }
}

// ───────────────────────────── Systematische Crash-Schleife ─────────────────────────────

/// Ausführbares Szenario. Fehler sind nach dem Crashpunkt erwartet, Panics nie.
pub struct Scenario {
    run: Box<dyn FnOnce() -> std::result::Result<(), String>>,
}

impl Scenario {
    /// Verpackt den Szenariokörper.
    pub fn new(run: impl FnOnce() -> std::result::Result<(), String> + 'static) -> Self {
        Self { run: Box::new(run) }
    }
}

/// Beschreibung eines geprüften Crashpunkts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrashPoint {
    /// Anzahl der vor dem Absturz ausgeführten Operationen.
    pub n: u64,
    /// Angewandte Neustart-Politik.
    pub policy: CrashPolicy,
    /// Operationsprotokoll bis zum Absturz.
    pub ops: Vec<OpRecord>,
}

/// Konfiguration der Crash-Schleife.
#[derive(Debug, Clone)]
pub struct CrashLoopConfig {
    /// Seed jedes Laufs.
    pub seed: CrashSeed,
    /// Maximale Anzahl Läufe; darüber seed-gezogene Stichprobe.
    pub max_points: usize,
}

impl Default for CrashLoopConfig {
    fn default() -> Self {
        Self {
            seed: CrashSeed(0xC0DE),
            max_points: 4_000,
        }
    }
}

/// Ergebnis einer vollständig bestandenen Crash-Schleife.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrashReport {
    /// Operationen des ungestörten Laufs.
    pub total_ops: u64,
    /// Geprüfte `(n, policy)`-Paare.
    pub runs: usize,
    /// `true`, wenn nur eine Stichprobe lief.
    pub sampled: bool,
}

/// Verstoß in der Crash-Schleife.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrashLoopFailure {
    /// Crashpunkt; `None` für den Referenzlauf ohne Crash.
    pub point: Option<(u64, CrashPolicy)>,
    /// Ursache.
    pub message: String,
}

impl std::fmt::Display for CrashLoopFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "crash point {:?}: {}", self.point, self.message)
    }
}

impl std::error::Error for CrashLoopFailure {}

fn run_guarded(
    setup: &impl Fn(&CrashVfs) -> Scenario,
    vfs: &CrashVfs,
) -> std::result::Result<std::result::Result<(), String>, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (setup(vfs).run)())).map_err(|p| {
        let msg = p
            .downcast_ref::<&str>()
            .map(|s| (*s).to_string())
            .or_else(|| p.downcast_ref::<String>().cloned())
            .unwrap_or_default();
        format!("scenario panicked: {msg}")
    })
}

/// [`for_each_crash_point`] mit expliziter Konfiguration.
pub fn for_each_crash_point_with<F>(
    config: &CrashLoopConfig,
    setup: impl Fn(&CrashVfs) -> Scenario,
    policies: &[CrashPolicy],
    check: F,
) -> std::result::Result<CrashReport, CrashLoopFailure>
where
    F: Fn(&CrashVfs, CrashPoint) -> std::result::Result<(), String>,
{
    let reference_failure = |message| CrashLoopFailure {
        point: None,
        message,
    };
    if policies.is_empty() {
        return Err(reference_failure("no crash policy given".to_string()));
    }
    let reference = CrashVfs::new(config.seed);
    run_guarded(&setup, &reference)
        .and_then(|r| r)
        .map_err(reference_failure)?;
    let total_ops = reference.op_count();

    let mut points: Vec<(u64, &CrashPolicy)> = (0..=total_ops)
        .flat_map(|n| policies.iter().map(move |p| (n, p)))
        .collect();
    let sampled = points.len() > config.max_points;
    if sampled {
        let all = points.len();
        let mut rng = XorShift::new(config.seed.0);
        // Partielles Fisher-Yates: die ersten `max_points` Elemente sind die Stichprobe.
        for i in 0..config.max_points {
            let j = i + usize::try_from(rng.below((all - i) as u64)).unwrap_or(0);
            points.swap(i, j);
        }
        points.truncate(config.max_points);
        points.sort_by_key(|(n, _)| *n);
        tracing::warn!(all, run = config.max_points, "crash loop sampled");
        // Sichtbar in der Testausgabe, auch ohne tracing-Subscriber.
        #[allow(clippy::print_stderr)]
        {
            eprintln!(
                "crash loop: sampling {} of {all} points (seed {})",
                config.max_points, config.seed.0
            );
        }
    }

    for (n, policy) in &points {
        let fail = |message| CrashLoopFailure {
            point: Some((*n, (*policy).clone())),
            message,
        };
        let vfs = CrashVfs::new(config.seed);
        vfs.crash_after(*n);
        let outcome = run_guarded(&setup, &vfs).map_err(fail)?;
        if !vfs.is_crashed() {
            return Err(fail(format!(
                "crash point not reached ({} ops): scenario is not deterministic",
                vfs.op_count()
            )));
        }
        // Fehler des Szenarios sind nach dem Crash erwartet und kein Befund.
        drop(outcome);
        let point = CrashPoint {
            n: *n,
            policy: (*policy).clone(),
            ops: vfs.last_ops(),
        };
        let restarted = vfs.restart((*policy).clone());
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| check(&restarted, point)))
            .unwrap_or_else(|_| Err("check panicked".to_string()))
            .map_err(fail)?;
    }
    Ok(CrashReport {
        total_ops,
        runs: points.len(),
        sampled,
    })
}

/// Bricht das Szenario nach jeder Operation `n ∈ 0..=N` unter jeder Politik ab,
/// startet neu und ruft `check` (Wiederaufnahme und Endzustandsprüfung).
pub fn for_each_crash_point<F>(
    setup: impl Fn(&CrashVfs) -> Scenario,
    policies: &[CrashPolicy],
    check: F,
) -> std::result::Result<CrashReport, CrashLoopFailure>
where
    F: Fn(&CrashVfs, CrashPoint) -> std::result::Result<(), String>,
{
    for_each_crash_point_with(&CrashLoopConfig::default(), setup, policies, check)
}

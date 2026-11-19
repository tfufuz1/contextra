//! Gemeinsame Typen der Löschkette (Messung, Tresor-VFS, Herkunft).
//!
//! Nach Entfernung der alten Löschmethode enthält dieses Modul nur Typen, die `chain` und
//! `vault` tatsächlich benötigen. Altlasten (EraseTarget/KeyRange, alter ErasureLayer, Observer,
//! Purge-/Fence-Barrieren, Signer, Ledger-/WAL-Sinks, Sealer, Keyring) sind entfernt.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

/// Gepinntes, geboxtes Future für objektsichere Trait-Methoden.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Opake Betreiberkennung für Attestierungen (SPEC §7.3).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct OperatorRef(Vec<u8>);

impl OperatorRef {
    /// Erzeugt eine geprüfte Instanz.
    pub fn try_new(bytes: Vec<u8>) -> Result<Self, String> {
        if bytes.is_empty() {
            return Err("Empty identifier".to_string());
        }
        if bytes.len() > 256 {
            return Err("Identifier too long".to_string());
        }
        Ok(Self(bytes))
    }

    /// Liefert die Bytes als Slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for OperatorRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OperatorRef([REDACTED {} bytes])", self.0.len())
    }
}

/// Schicht-Kennung.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LayerId(pub [u8; 16]);

impl LayerId {
    /// Erzeugt eine neue `LayerId`.
    pub const fn new(raw: [u8; 16]) -> Self {
        Self(raw)
    }
    /// Liefert die Bytes als Slice.
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Debug for LayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LayerId({:02x?})", self.0)
    }
}

impl fmt::Display for LayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.0 {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

/// Reichweite der Schlüsselabdeckung einer Schicht (SPEC §8.2, D-69).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyCoverage {
    /// Vollständige Schlüsselabdeckung.
    Full,
    /// Unverknüpfte Schlüssel.
    Unlinked,
    /// Keine eigene Schlüsselvernichtung.
    None,
}

/// Messklasse für Kontrollscans (SPEC §12.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum MeasureClass {
    /// MK1: Logische Strukturen.
    Mk1,
    /// MK2: Rohdaten-Bytescan.
    Mk2,
    /// MK3: Physisches Nulling / Medienprüfung.
    Mk3,
}

/// Typzustand: Scanner ist noch nicht scharfgeschaltet.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Unarmed;

/// Typzustand: Scanner ist scharfgeschaltet nach erfolgreicher Positivkontrolle.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Armed(pub ());

/// Beleg für eine beobachtete Positivkontrolle (D-72).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ControlEvidence(());

impl ControlEvidence {
    /// Bezeugt das Auffinden des Kontrollmusters.
    /// Nur crate-intern; externe Aufrufer nutzen `measure::prove_positive_control` (E-7, INV-WITNESS-MINT-1).
    #[inline]
    pub(crate) const fn observed() -> Self {
        Self(())
    }
}

/// Rückstandsbefund bei fehlgeschlagener Verifikation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Residue {
    /// Anzahl gefundener Rückstände.
    pub matches: u64,
}

impl fmt::Display for Residue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "erasure residue detected: {} matches found",
            self.matches
        )
    }
}

impl std::error::Error for Residue {}

/// Typisierter Scanner.
pub struct Scanner<State> {
    layer: LayerId,
    class: MeasureClass,
    method_id: u16,
    files: u64,
    bytes: u64,
    _state: State,
}

impl Scanner<Unarmed> {
    /// Erzeugt einen unbewaffneten Scanner.
    pub fn new(layer: LayerId, class: MeasureClass, method_id: u16) -> Self {
        Self {
            layer,
            class,
            method_id,
            files: 0,
            bytes: 0,
            _state: Unarmed,
        }
    }

    /// Schaltet Scanner über Positivkontrolle scharf.
    pub fn arm(self, _ev: ControlEvidence) -> Scanner<Armed> {
        Scanner {
            layer: self.layer,
            class: self.class,
            method_id: self.method_id,
            files: self.files,
            bytes: self.bytes,
            _state: Armed(()),
        }
    }
}

impl Scanner<Armed> {
    /// Erzeugt einen direkt scharfen Scanner (z.B. für Tests/Mocks).
    pub fn armed(layer: LayerId, class: MeasureClass, method_id: u16) -> Self {
        Self {
            layer,
            class,
            method_id,
            files: 0,
            bytes: 0,
            _state: Armed(()),
        }
    }

    /// Zeichnet geprüfte Dateien und Bytes auf.
    pub fn record(&mut self, files: u64, bytes: u64) {
        self.files = self.files.saturating_add(files);
        self.bytes = self.bytes.saturating_add(bytes);
    }

    /// Schließt Messung ab.
    pub fn finish(
        self,
        matches: u64,
        duration: Duration,
        hash: [u8; 32],
    ) -> Result<LayerWitness, Residue> {
        if matches != 0 {
            return Err(Residue { matches });
        }
        Ok(LayerWitness {
            layer: self.layer,
            class: self.class,
            method_id: self.method_id,
            files: self.files,
            bytes: self.bytes,
            duration,
            witness_hash: hash,
        })
    }
}

impl<State> fmt::Debug for Scanner<State> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Scanner {{ files: {}, bytes: {} }}",
            self.files, self.bytes
        )
    }
}

/// Zeuge einer überprüften physischen Löschung.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerWitness {
    layer: LayerId,
    class: MeasureClass,
    method_id: u16,
    files: u64,
    bytes: u64,
    duration: Duration,
    witness_hash: [u8; 32],
}

impl LayerWitness {
    /// Erzeugt einen leeren Zeugen.
    pub const fn empty() -> Self {
        Self {
            layer: LayerId([0u8; 16]),
            class: MeasureClass::Mk1,
            method_id: 0,
            files: 0,
            bytes: 0,
            duration: Duration::from_secs(0),
            witness_hash: [0u8; 32],
        }
    }

    /// Erzeugt einen Zeugen aus einem Hash.
    pub const fn from_hash(witness_hash: [u8; 32]) -> Self {
        Self {
            layer: LayerId([0u8; 16]),
            class: MeasureClass::Mk1,
            method_id: 0,
            files: 0,
            bytes: 0,
            duration: Duration::from_secs(0),
            witness_hash,
        }
    }

    /// Liefert die Schicht-ID.
    pub fn layer(&self) -> LayerId {
        self.layer
    }

    /// Liefert die Messklasse.
    pub fn class(&self) -> MeasureClass {
        self.class
    }

    /// Liefert die Methoden-ID.
    pub fn method_id(&self) -> u16 {
        self.method_id
    }

    /// Liefert geprüfte Dateien.
    pub fn files(&self) -> u64 {
        self.files
    }

    /// Liefert geprüfte Bytes.
    pub fn bytes(&self) -> u64 {
        self.bytes
    }

    /// Liefert Scandauer.
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Liefert den Hash.
    pub fn witness_hash(&self) -> &[u8; 32] {
        &self.witness_hash
    }

    /// Alias für witness_hash.
    pub fn hash(&self) -> &[u8; 32] {
        &self.witness_hash
    }

    /// matches() ist per Konstruktion immer 0.
    pub const fn matches(&self) -> u64 {
        0
    }
}

impl fmt::Debug for LayerWitness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LayerWitness {{ hash: {:02x?} }}", self.witness_hash)
    }
}

/// Verifikationsmodus beim Lesen von Medien (SPEC D-113).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifiedVia {
    /// Direktes I/O ohne Puffer (`O_DIRECT`).
    Direct,
    /// Nach Puffer-Verwerfung (`posix_fadvise`).
    CacheDropped,
    /// Gewöhnlicher Cache-Pfad.
    Cached,
}

/// Optionen zum Öffnen von Dateien im VFS.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VfsOpenOptions {
    /// Lesend öffnen.
    pub read: bool,
    /// Schreibend öffnen.
    pub write: bool,
    /// Datei anlegen falls nicht vorhanden.
    pub create: bool,
    /// Neue Datei erzwingen.
    pub create_new: bool,
    /// Dateilänge auf 0 kürzen.
    pub truncate: bool,
    /// Anhängend öffnen.
    pub append: bool,
}

impl VfsOpenOptions {
    /// Erzeugt neue leere Optionen.
    pub const fn new() -> Self {
        Self {
            read: false,
            write: false,
            create: false,
            create_new: false,
            truncate: false,
            append: false,
        }
    }
    /// Konfiguriert Optionen für reinen Lesezugriff.
    pub fn read_only() -> Self {
        Self {
            read: true,
            ..Default::default()
        }
    }
    /// Konfiguriert Optionen für Schreibzugriff mit Dateierzeugung.
    pub fn write_create() -> Self {
        Self {
            write: true,
            create: true,
            ..Default::default()
        }
    }
    /// Setzt Leseberechtigung.
    pub const fn read(mut self, val: bool) -> Self {
        self.read = val;
        self
    }
    /// Setzt Schreibberechtigung.
    pub const fn write(mut self, val: bool) -> Self {
        self.write = val;
        self
    }
    /// Setzt Erstellungsflag.
    pub const fn create(mut self, val: bool) -> Self {
        self.create = val;
        self
    }
    /// Setzt Create-New-Flag.
    pub const fn create_new(mut self, val: bool) -> Self {
        self.create_new = val;
        self
    }
}

/// In-Memory-VfsFile Adapter für In-Memory Vfs.
pub struct MemoryVfsFile {
    data: std::sync::Arc<parking_lot::RwLock<Vec<u8>>>,
    vfs_raw: std::sync::Arc<dyn crate::types_local::Vfs>,
    path: std::path::PathBuf,
}

impl MemoryVfsFile {
    /// Erzeugt ein MemoryVfsFile aus vorhandenen Bytes.
    pub fn new(
        data: Vec<u8>,
        vfs_raw: std::sync::Arc<dyn crate::types_local::Vfs>,
        path: std::path::PathBuf,
    ) -> Self {
        Self {
            data: std::sync::Arc::new(parking_lot::RwLock::new(data)),
            vfs_raw,
            path,
        }
    }
}

impl VfsFile for MemoryVfsFile {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
        let guard = self.data.read();
        let off = usize::try_from(offset).map_err(|e| std::io::Error::other(e.to_string()))?;
        if off >= guard.len() {
            return Ok(0);
        }
        let available = match guard.get(off..) {
            Some(a) => a,
            None => return Ok(0),
        };
        let to_copy = buf.len().min(available.len());
        if let (Some(dest), Some(src)) = (buf.get_mut(..to_copy), available.get(..to_copy)) {
            dest.copy_from_slice(src);
        }
        Ok(to_copy)
    }

    fn read_at_verified(
        &self,
        buf: &mut [u8],
        offset: u64,
    ) -> std::io::Result<(usize, VerifiedVia)> {
        let n = self.read_at(buf, offset)?;
        Ok((n, VerifiedVia::Direct))
    }

    fn read_verified(&self, offset: u64, len: usize) -> std::io::Result<(Vec<u8>, VerifiedVia)> {
        let mut buf = vec![0u8; len];
        let n = self.read_at(&mut buf, offset)?;
        buf.truncate(n);
        Ok((buf, VerifiedVia::Direct))
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize> {
        let mut guard = self.data.write();
        let off = usize::try_from(offset).map_err(|e| std::io::Error::other(e.to_string()))?;
        let required = off.saturating_add(buf.len());
        if guard.len() < required {
            guard.resize(required, 0);
        }
        if let Some(slice) = guard.get_mut(off..off + buf.len()) {
            slice.copy_from_slice(buf);
        }
        Ok(buf.len())
    }

    fn sync_data(&self) -> std::io::Result<()> {
        let guard = self.data.read();
        self.vfs_raw
            .write_atomic(&self.path, &guard)
            .map_err(|e| std::io::Error::other(e.to_string()))
    }

    fn sync_all(&self) -> std::io::Result<()> {
        self.sync_data()
    }

    fn set_len(&self, len: u64) -> std::io::Result<()> {
        let mut guard = self.data.write();
        let target = usize::try_from(len).map_err(|e| std::io::Error::other(e.to_string()))?;
        guard.resize(target, 0);
        Ok(())
    }

    fn len(&self) -> std::io::Result<u64> {
        let guard = self.data.read();
        Ok(guard.len() as u64)
    }
}

/// Port-Vertrag für geöffnete VFS-Dateihandles.
pub trait VfsFile: Send + Sync {
    /// Liest an einer bestimmten Position.
    fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize>;
    /// Verifiziertes Lesen unter Umgehung des Caches.
    fn read_at_verified(
        &self,
        buf: &mut [u8],
        offset: u64,
    ) -> std::io::Result<(usize, VerifiedVia)> {
        let n = self.read_at(buf, offset)?;
        Ok((n, VerifiedVia::Cached))
    }
    /// Schreibt an einer bestimmten Position.
    fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize>;
    /// Verifiziertes Lesen (SPEC D-113).
    fn read_verified(&self, offset: u64, len: usize) -> std::io::Result<(Vec<u8>, VerifiedVia)> {
        let mut buf = vec![0u8; len];
        let n = self.read_at(&mut buf, offset)?;
        buf.truncate(n);
        Ok((buf, VerifiedVia::Cached))
    }
    /// Schreibt Daten dauerhaft auf Festplatte.
    fn sync_data(&self) -> std::io::Result<()>;
    /// Schreibt alle Daten und Metadaten dauerhaft.
    fn sync_all(&self) -> std::io::Result<()> {
        self.sync_data()
    }
    /// Kürzt oder erweitert die Dateilänge.
    fn set_len(&self, _len: u64) -> std::io::Result<()> {
        Ok(())
    }
    /// Ermittelt die Dateigröße.
    fn len(&self) -> std::io::Result<u64>;
    /// Prüft ob die Datei leer ist.
    fn is_empty(&self) -> std::io::Result<bool> {
        self.len().map(|l| l == 0)
    }
}

/// Öffnet eine Datei im VFS als positionelles `VfsFile` Handle (In-Memory-Simulation).
pub fn open_vfs_file(
    vfs: std::sync::Arc<dyn crate::types_local::Vfs>,
    path: &std::path::Path,
    opts: &VfsOpenOptions,
) -> Result<Box<dyn VfsFile>, crate::error::ErasureError> {
    let exists = vfs
        .exists(path)
        .map_err(|e| crate::error::ErasureError::Io(std::io::Error::other(e.to_string())))?;
    if opts.create_new && exists {
        return Err(crate::error::ErasureError::Io(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "File already exists",
        )));
    }
    if !exists && !opts.create && !opts.create_new {
        return Err(crate::error::ErasureError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found",
        )));
    }
    let data = if exists {
        if opts.truncate {
            Vec::new()
        } else {
            vfs.read(path).unwrap_or_default()
        }
    } else {
        Vec::new()
    };
    Ok(Box::new(MemoryVfsFile::new(data, vfs, path.to_path_buf())))
}

/// Port trait for time operations.
pub trait Clock: Send + Sync + 'static {
    /// Returns the current wall-clock time as nanoseconds since the Unix epoch.
    fn now_unix_nanos(&self) -> u64;

    /// Returns nanoseconds elapsed since a monotonic baseline reference point.
    fn monotonic_nanos(&self) -> u64;
}

/// Abstract object-safe Virtual File System interface for `contextra-erasure`.
pub trait Vfs: Send + Sync + 'static {
    /// Opens a file according to the specified options.
    fn open(
        &self,
        _path: &std::path::Path,
        _opts: &VfsOpenOptions,
    ) -> Result<Box<dyn VfsFile>, crate::error::ErasureError> {
        Err(crate::error::ErasureError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "positional VfsFile open is not supported by default",
        )))
    }

    /// Reads data from byte offset `offset` into `buf` for file at `path`.
    fn read_at(
        &self,
        path: &std::path::Path,
        offset: u64,
        buf: &mut [u8],
    ) -> Result<usize, crate::error::ErasureError> {
        let file = self.open(path, &VfsOpenOptions::read_only())?;
        file.read_at(buf, offset)
            .map_err(crate::error::ErasureError::Io)
    }

    /// Reads the entire contents of a file at `path`.
    fn read(&self, path: &std::path::Path) -> Result<Vec<u8>, crate::error::ErasureError>;

    /// Atomically writes `data` to `path` with crash-consistency guarantees.
    fn write_atomic(
        &self,
        path: &std::path::Path,
        data: &[u8],
    ) -> Result<(), crate::error::ErasureError>;

    /// Creates a directory and all parent directories along `path` if missing.
    fn create_dir_all(&self, path: &std::path::Path) -> Result<(), crate::error::ErasureError>;

    /// Removes a file at `path`.
    fn remove_file(&self, path: &std::path::Path) -> Result<(), crate::error::ErasureError>;

    /// Returns `true` if a file or directory exists at `path`.
    fn exists(&self, path: &std::path::Path) -> Result<bool, crate::error::ErasureError>;

    /// Returns the length in bytes of the file at `path`.
    fn len(&self, path: &std::path::Path) -> Result<u64, crate::error::ErasureError>;

    /// Lists entry names inside directory `path`.
    fn list_dir(&self, path: &std::path::Path) -> Result<Vec<String>, crate::error::ErasureError>;

    /// Flushes directory metadata to persistent storage.
    fn sync_dir(&self, path: &std::path::Path) -> Result<(), crate::error::ErasureError>;
}

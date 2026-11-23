//! Persistenz-Ports für `contextra-storage` (SPEC.md §4, semver-stabil).
//!
//! Dieses Modul liegt im bestehenden Crate `contextra-ports`, damit der Kern
//! ausschließlich von `contextra-ports` abhängt (SPEC §3.3). Es enthält nur
//! Traits und kleine Wertetypen, keine Logik. Ausnahmen sind die Standard-Körper
//! von `File::read_batch` und `File::preallocate` (Spec-Vorgabe) sowie die
//! Standardmethoden `Cipher::seal_into` und `Cipher::open_into` (SA-4).
//!
//! Die Trait-Namen sind bewusst identisch mit der Spec. Der Modulpfad
//! `contextra_ports::persistence::*` ersetzt den früheren Platzhalter
//! `contextra_storage_ports`. Der Modul-Glob wird **nicht** in die Crate-Wurzel
//! re-exportiert, damit die bestehenden Namen `Clock` und `Metrics` des Crates
//! nicht kollidieren.

use std::io;
use std::path::Path;

/// Dateirechte beim Anlegen (SPEC §4, §15.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Perm {
    /// 0600 für Dateien, 0700 für Verzeichnisse bzw. restriktive ACL.
    /// MUSS unabhängig von der `umask` gelten.
    Private,
    /// Übernimmt die Rechte des Betriebssystems.
    Inherit,
}

/// Öffnungsart einer Datei.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenMode {
    /// Nur lesen.
    Read,
    /// Lesen und schreiben, Datei muss existieren.
    ReadWrite,
    /// Neu anlegen; schlägt fehl, wenn die Datei existiert.
    CreateNew(Perm),
}

/// Sperrart. Nicht blockierend (SPEC §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockMode {
    /// Exklusive Sperre.
    Exclusive,
    /// Geteilte Sperre.
    Shared,
}

/// Verzeichniseintrag aus `Fs::list_dir`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    /// Dateiname ohne Pfad.
    pub name: String,
    /// Länge in Bytes.
    pub len: u64,
}

/// Ein Lesewunsch für `File::read_batch`.
pub struct ReadReq<'a> {
    /// Ziel des Lesens.
    pub buf: &'a mut [u8],
    /// Byte-Offset in der Datei.
    pub offset: u64,
}

/// Zugesicherte Eigenschaften einer `Fs`-Implementierung (SPEC §4, SD-57).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FsCaps {
    /// `sync_data`/`sync_all` ist dauerhaft.
    pub durable_file_sync: bool,
    /// `sync_dir` macht Verzeichnisoperationen dauerhaft.
    pub durable_dir_sync: bool,
    /// `rename` ist atomar.
    pub atomic_rename: bool,
    /// `Perm::Private` wird eingehalten.
    pub private_perm: bool,
    /// Wurzel wird nicht über Symlinks betreten.
    pub no_follow_root: bool,
}

/// Verhalten bei nicht zugesicherten `FsCaps` (SPEC §4, INV-FS-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FsPolicy {
    /// Fail-closed: öffnet nur bei vollständigen Zusicherungen.
    #[default]
    Strict,
    /// Nur für Tests und Wegwerf-Instanzen.
    Relaxed,
}

/// Dateisystem-Port (SPEC §4).
pub trait Fs: Send + Sync + 'static {
    /// Legt Verzeichnisse rekursiv an.
    fn create_dir_all(&self, p: &Path, perm: Perm) -> io::Result<()>;
    /// Öffnet eine Datei.
    fn open(&self, p: &Path, mode: OpenMode) -> io::Result<Box<dyn File>>;
    /// Benennt um.
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
    /// Löscht eine Datei.
    fn remove_file(&self, p: &Path) -> io::Result<()>;
    /// Löscht ein leeres Verzeichnis.
    fn remove_dir(&self, p: &Path) -> io::Result<()>;
    /// Listet Einträge eines Verzeichnisses.
    fn list_dir(&self, p: &Path) -> io::Result<Vec<DirEntry>>;
    /// Macht Verzeichnisoperationen dauerhaft.
    fn sync_dir(&self, p: &Path) -> io::Result<()>;
    /// Setzt eine Sperre auf eine Datei.
    fn lock(&self, p: &Path, mode: LockMode) -> io::Result<Box<dyn LockGuard>>;
    /// Zugesicherte Eigenschaften.
    fn caps(&self) -> FsCaps;
    /// Harter Link. Nur für Checkpoints (Opt-in).
    fn hard_link(&self, from: &Path, to: &Path) -> io::Result<()>;
}

/// Geöffnete Datei (SPEC §4).
pub trait File: Send + Sync {
    /// Liest an Offset; liefert die Anzahl gelesener Bytes.
    fn read_at(&self, buf: &mut [u8], off: u64) -> io::Result<usize>;

    /// Liest mehrere Bereiche. Standard: Schleife über `read_at`.
    fn read_batch(&self, reqs: &mut [ReadReq<'_>]) -> io::Result<()> {
        for req in reqs.iter_mut() {
            let n = self.read_at(req.buf, req.offset)?;
            if let Some(tail) = req.buf.get_mut(n..) {
                tail.fill(0);
            }
        }
        Ok(())
    }

    /// Schreibt vollständig an Offset.
    fn write_all_at(&self, buf: &[u8], off: u64) -> io::Result<()>;

    /// Reserviert Platz. Standard: Nullen in 1-MiB-Schritten.
    fn preallocate(&self, len: u64) -> io::Result<()> {
        const STEP: u64 = 1 << 20;
        let zeros = vec![0u8; STEP as usize];
        let mut off = self.len()?;
        while off < len {
            let n = (len - off).min(STEP);
            let n_usize = usize::try_from(n).map_err(|_| io::Error::other("Größe"))?;
            let chunk = zeros.get(..n_usize).unwrap_or(&[]);
            self.write_all_at(chunk, off)?;
            off = off.saturating_add(n);
        }
        Ok(())
    }

    /// Aktuelle Länge in Bytes.
    fn len(&self) -> io::Result<u64>;
    /// Setzt die Länge.
    fn set_len(&self, len: u64) -> io::Result<()>;
    /// Synchronisiert Daten.
    fn sync_data(&self) -> io::Result<()>;
    /// Synchronisiert Daten und Metadaten.
    fn sync_all(&self) -> io::Result<()>;
}

/// Marker für eine gehaltene Sperre. Die Sperre endet mit dem `Drop`.
pub trait LockGuard: Send + Sync {}

/// Monotone und Wand-Uhr (SPEC §4).
pub trait Clock: Send + Sync + 'static {
    /// Wandzeit in Millisekunden seit Unix-Epoche.
    fn now_ms(&self) -> u64;
    /// Monotone Zeit in Nanosekunden.
    fn monotonic_ns(&self) -> u64;
}

/// Zufallsquelle (SPEC §4).
pub trait Entropy: Send + Sync {
    /// Füllt `dst` mit Zufallsbytes.
    fn fill(&self, dst: &mut [u8]) -> io::Result<()>;
}

/// Telemetrie-Port (SPEC §4). Namen sind `'static`, Inhalte nie.
pub trait Metrics: Send + Sync {
    /// Zähler erhöhen.
    fn counter(&self, name: &'static str, v: u64);
    /// Messwert setzen.
    fn gauge(&self, name: &'static str, v: i64);
    /// Beobachtung hinzufügen.
    fn observe(&self, name: &'static str, v: f64);
}

/// Prüfpunkte für Tests (SPEC §4, P-5). Standard: leer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookPoint {
    /// Nach dem Sync des Redo-Segments.
    AfterRedoSync,
    /// Nach dem Schreiben eines Redo-Frames.
    AfterRedoWrite,
    /// Vor einem Catalog-Edit.
    BeforeCatalogEdit,
    /// Vor der Sichtbarkeit eines Commits.
    BeforeVisible,
}

/// Prüfpunkt-Port (SPEC §4).
pub trait Hooks: Send + Sync {
    /// Wird an jedem Prüfpunkt aufgerufen. Standard: nichts.
    fn point(&self, _p: HookPoint) {}
}

/// Druckstufe des Speichers (SPEC §4, §17.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pressure {
    /// Normal.
    Normal,
    /// Weich: Caches verkleinern.
    Soft,
    /// Hart: Caches stark verkleinern.
    Hard,
}

/// Quelle der Druckstufe (SPEC §4, optional).
pub trait PressureSource: Send + Sync {
    /// Aktuelle Druckstufe.
    fn level(&self) -> Pressure;
}

/// Schlüssel für `Cipher`, `Mac` und `Kdf` (SPEC §4).
///
/// Typalias auf `[u8; 32]` (IMPL §2.2.1). Die Geheimnis-Eigenschaft trägt
/// `Secret32` bzw. `SecretCell`, nicht dieser Alias.
pub type Key32 = [u8; 32];

/// Fehler eines AEAD-Vorgangs (SPEC §4).
///
/// Trägt keine Inhalte: weder Schlüssel, Nonce, AAD noch Klartext oder
/// Chiffrat (INV-KEY-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoError {
    /// Die Tag-Prüfung schlägt fehl: falscher Schlüssel, Nonce oder AAD, oder
    /// das Chiffrat wurde verändert.
    AuthenticationFailed,
    /// Das Chiffrat ist kürzer als der 16-Byte-Tag und kann nicht geprüft werden.
    Malformed,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AuthenticationFailed => write!(f, "Authentisierung fehlgeschlagen"),
            Self::Malformed => write!(f, "Chiffrat zu kurz"),
        }
    }
}

impl std::error::Error for CryptoError {}

/// AEAD mit 32-Byte-Schlüssel, 24-Byte-Nonce und 16-Byte-Tag (SPEC §4, §10.1, UE-10).
///
/// Verträge:
/// - `seal` liefert `plain.len() + 16` Bytes. Der Tag steht am Ende.
/// - `open` liefert `CryptoError::Malformed`, wenn `sealed` kürzer als 16 Byte ist.
///   Jede andere Abweichung von Schlüssel, Nonce, AAD oder Chiffrat ergibt
///   `CryptoError::AuthenticationFailed`.
/// - `seal_into` und `open_into` haben Standardimplementierungen über `seal` und
///   `open` (SA-4, SPEC §20.5). Sie allokieren daher. Der Heißpfad (`submit`, `get`)
///   überschreibt beide Methoden, damit er aus einem vorab angelegten Puffer arbeitet.
/// - Der zählende Allokator-Test nach SPEC §20.5 läuft gegen die Implementierung in
///   `contextra-crypto` (WP-2.4). Er läuft nicht gegen die Standardmethoden, die
///   per Konstruktion allokieren.
pub trait Cipher: Send + Sync {
    /// Verschlüsselt und authentisiert. Hängt keine Länge an.
    fn seal(&self, key: &Key32, nonce: &[u8; 24], aad: &[u8], plain: &[u8]) -> Vec<u8>;

    /// Prüft und entschlüsselt.
    fn open(
        &self,
        key: &Key32,
        nonce: &[u8; 24],
        aad: &[u8],
        sealed: &[u8],
    ) -> Result<Vec<u8>, CryptoError>;

    /// Verschlüsselt und hängt das Ergebnis an `out` an. Der Aufrufer leert `out`
    /// vorher, wenn er den Puffer wiederverwendet. Standard: über `seal`.
    fn seal_into(
        &self,
        key: &Key32,
        nonce: &[u8; 24],
        aad: &[u8],
        plain: &[u8],
        out: &mut Vec<u8>,
    ) {
        out.extend_from_slice(&self.seal(key, nonce, aad, plain));
    }

    /// Prüft, entschlüsselt und hängt den Klartext an `out` an. Bei Fehler bleibt
    /// `out` unverändert. Standard: über `open`.
    fn open_into(
        &self,
        key: &Key32,
        nonce: &[u8; 24],
        aad: &[u8],
        sealed: &[u8],
        out: &mut Vec<u8>,
    ) -> Result<(), CryptoError> {
        let plain = self.open(key, nonce, aad, sealed)?;
        out.extend_from_slice(&plain);
        Ok(())
    }
}

/// MAC mit 16-Byte-Tag (SPEC §4).
pub trait Mac: Send + Sync {
    /// Berechnet den Tag über die Teile.
    fn mac16(&self, key: &[u8; 32], parts: &[&[u8]]) -> [u8; 16];
    /// Prüft den Tag in konstanter Zeit.
    fn verify16(&self, key: &[u8; 32], parts: &[&[u8]], tag: &[u8; 16]) -> bool;
}

/// Schlüsselableitung (SPEC §4).
pub trait Kdf: Send + Sync {
    /// Leitet einen 32-Byte-Schlüssel ab.
    fn derive(&self, key: &[u8; 32], context: &[&[u8]]) -> [u8; 32];
}

/// Verschlüsselungsdomäne eines Körpers (SPEC §6.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct KeyDomain(pub u32);

/// Fehler des Schlüsselports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyError {
    /// Schlüssel war nie angelegt.
    NotFound,
    /// Schlüsselquelle nicht erreichbar.
    Unavailable,
}

impl std::fmt::Display for KeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "Schlüssel nicht angelegt"),
            Self::Unavailable => write!(f, "Schlüsselquelle nicht erreichbar"),
        }
    }
}

impl std::error::Error for KeyError {}

/// 32-Byte-Geheimnis als undurchsichtiges Handle (SPEC §4, SD-56).
/// Kein `Debug`, kein `Clone`, kein `Serialize`.
pub struct Secret32 {
    bytes: [u8; 32],
}

impl Secret32 {
    /// Übernimmt 32 Bytes.
    pub fn new(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }

    /// Gibt den Schlüssel frei. Aufrufer dürfen ihn nicht loggen (P-12).
    pub fn expose(&self) -> &[u8; 32] {
        &self.bytes
    }
}

impl Drop for Secret32 {
    fn drop(&mut self) {
        // Best-effort-Nullung ohne `unsafe` (SD-56).
        self.bytes = [0u8; 32];
        std::hint::black_box(&self.bytes);
    }
}

/// Speicherzelle für ein Geheimnis (SPEC §4, optional).
pub trait SecretCell: Send + Sync {
    /// Gibt den Schlüssel frei.
    fn expose(&self) -> &[u8; 32];
}

/// Schlüsselport (SPEC §4, SD-55, INV-KEYP-1).
pub trait KeyProvider: Send + Sync {
    /// Schlüsselverschlüsselungsschlüssel.
    fn kek(&self) -> io::Result<Secret32>;
    /// Liefert den Domänenschlüssel. Legt nie an. `None` = dauerhaft vernichtet.
    fn domain_key(&self, d: KeyDomain) -> io::Result<Option<Secret32>>;
    /// Legt den Domänenschlüssel bei Bedarf an. `None` = vernichtet.
    fn domain_key_create(&self, d: KeyDomain) -> io::Result<Option<Secret32>>;
    /// Barriere: alle bisher angelegten Schlüssel sind dauerhaft.
    fn make_durable(&self) -> io::Result<()>;
}

/// Art einer Datei im Layout (SPEC §8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FileKind {
    /// Run-Datei (LSM-Tabelle).
    Run,
    /// Blob-Datei (große Körper).
    Blob,
    /// Segment.
    Segment,
}

/// Eindeutige Referenz auf eine Datei.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileRef {
    /// Art der Datei.
    pub kind: FileKind,
    /// Numerische Kennung innerhalb der Art.
    pub id: u64,
}

/// Commit-Stamp (SPEC §6.2): `u64`, lückenlos, erster Stamp ist 1,
/// Obergrenze 2^56 (INV-STAMP-1, INV-STAMP-2).
pub type Stamp = u64;

/// Entscheidung der Compaction für einen Eintrag (SPEC §12.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Eintrag behalten.
    Keep,
    /// Eintrag verwerfen.
    Drop,
    /// Nur den Körper verwerfen; Kopf ggf. ersetzen ("Zoom-Down").
    DropBody {
        /// Ersetzter Kopf, falls vorhanden.
        new_header: Option<Vec<u8>>,
    },
}

/// Aufrufer-Filter bei der Compaction (SPEC §4, §12.2).
pub trait CompactionFilter: Send + Sync {
    /// Entscheidet über einen Eintrag.
    fn decide(&self, key: &[u8], header: &[u8], stamp: Stamp, now_ms: u64) -> Decision;
}

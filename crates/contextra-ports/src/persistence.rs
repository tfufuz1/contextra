//! Persistenz-Ports für `contextra-storage` (SPEC.md §4, semver-stabil).
//!
//! Dieses Modul liegt im bestehenden Crate `contextra-ports`, damit der Kern
//! ausschließlich von `contextra-ports` abhängt (SPEC §3.3). Es enthält nur
//! Traits und kleine Wertetypen, keine Logik. Ausnahme: der Standard-Körper von
//! `File::read_batch` und `File::preallocate` (Spec-Vorgabe).
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
    /// Harter Link. Nur für Checkpoints (Opt-in). Standard: `Unsupported` (SA-2).
    fn hard_link(&self, _from: &Path, _to: &Path) -> io::Result<()> {
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }
    /// Referenzkopie (Copy-on-Write). Standard: `Unsupported` (SA-2).
    fn reflink(&self, _from: &Path, _to: &Path) -> io::Result<()> {
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }
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
    /// Setzt die Länge (kürzt oder verlängert; SA-3).
    fn truncate(&self, len: u64) -> io::Result<()>;
    /// Synchronisiert Daten.
    fn sync_data(&self) -> io::Result<()>;
    /// Synchronisiert Daten und Metadaten.
    fn sync_all(&self) -> io::Result<()>;
    /// Gibt den Bereich frei (Lochen). Standard: `Unsupported` (SPEC §4, optional).
    fn punch_hole(&self, _off: u64, _len: u64) -> io::Result<()> {
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }
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

/// AEAD mit 32-Byte-Schlüssel, 12-Byte-Nonce, 16-Byte-Tag (SPEC §4).
pub trait Cipher: Send + Sync {
    /// Verschlüsselt und authentisiert.
    fn seal(&self, key: &[u8; 32], nonce: &[u8; 12], aad: &[u8], plain: &[u8]) -> Vec<u8>;
    /// Prüft und entschlüsselt.
    fn open(&self, key: &[u8; 32], nonce: &[u8; 12], aad: &[u8], sealed: &[u8]) -> io::Result<Vec<u8>>;
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
    /// Segment mit Nummer innerhalb des Layouts (IF-1).
    Segment(u16),
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

/// Opake Cluster-Kennung (SPEC §6.1, IF-1). 16 Byte, kein Inhalt in `Debug`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ClusterId([u8; 16]);

impl ClusterId {
    /// Kennung für nicht geclusterte Spaces (SPEC §6.1).
    pub const NONE: Self = Self([0u8; 16]);

    /// Übernimmt 16 Bytes.
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Gibt die Bytes zurück.
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl std::fmt::Debug for ClusterId {
    /// Zeigt nur die ersten vier Bytes als Hex (P-12, Umsetzung UE-3).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ClusterId({:02x}{:02x}{:02x}{:02x}..)",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

/// 32-Byte-Schlüsselbytes (IF-1: Typalias; die Geheimnis-Eigenschaft liegt in
/// `Secret32` und `SecretCell`).
pub type Key32 = [u8; 32];

/// Zustand eines Clusterschlüssels beim `KeyProvider` (SPEC §4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    /// Schlüssel vorhanden.
    Present,
    /// Schlüssel dauerhaft vernichtet oder nie angelegt.
    Absent,
}

/// Fehler einer Versiegelung oder Entschlüsselung (SPEC §4). Ohne Inhalte (INV-KEY-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CryptoError {
    /// Authentisierung fehlgeschlagen: Tag, AAD oder Schlüssel passen nicht.
    AuthFailed,
    /// Eingabe hat eine unzulässige Länge.
    InvalidLength,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AuthFailed => write!(f, "Authentisierung fehlgeschlagen"),
            Self::InvalidLength => write!(f, "unzulässige Länge"),
        }
    }
}

impl std::error::Error for CryptoError {}

/// Prüfwert-Port über CRC-32C (SPEC §4, SD-64).
pub trait Checksum: Send + Sync {
    /// Setzt einen CRC-32C-Strom fort. `seed = 0` liefert den üblichen CRC-32C.
    fn crc32c(&self, seed: u32, data: &[u8]) -> u32;
}

/// Unkeyed Hash für die Ledger-Kette (SPEC §4, D-12).
pub trait Hasher: Send + Sync {
    /// 256-Bit-Hash über die Teile in Reihenfolge.
    fn hash256(&self, parts: &[&[u8]]) -> [u8; 32];
}

/// Rollback-Anker (SPEC §4, §16.7). Der Wert ist monoton.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnchorValue {
    /// Epoche der letzten abgeschlossenen Löschung (ER7).
    pub erase_epoch: u64,
    /// Kopf der Ledger-Kette.
    pub ledger_head: [u8; 32],
    /// Epoche des Catalogs.
    pub catalog_epoch: u64,
}

/// Fehler des Ankers. Ohne Inhalte (INV-KEY-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchorError {
    /// Neuer Wert wäre kleiner als der gespeicherte (SPEC §16.7).
    NotMonotonic,
    /// Anker nicht erreichbar oder nicht lesbar.
    Unavailable,
}

impl std::fmt::Display for AnchorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotMonotonic => write!(f, "Anker würde zurückgesetzt"),
            Self::Unavailable => write!(f, "Anker nicht erreichbar"),
        }
    }
}

impl std::error::Error for AnchorError {}

/// Rollback-Anker-Port (SPEC §4, §16.7).
pub trait Anchor: Send + Sync {
    /// Liest den gespeicherten Wert; `None`, wenn noch keiner gesetzt ist.
    fn read(&self) -> Result<Option<AnchorValue>, AnchorError>;
    /// Setzt einen neuen Wert. Monoton; durabel bei Rückkehr.
    fn advance(&self, to: AnchorValue) -> Result<(), AnchorError>;
}

/// Ereignisse für Beobachter. Keine Schlüssel- oder Körperinhalte (INV-KEY-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ObserverEvent {
    /// Zustand der Instanz hat sich geändert.
    HealthChanged,
    /// Ein Flush ist abgeschlossen.
    FlushDone,
    /// Eine Compaction ist abgeschlossen.
    CompactionDone,
    /// Ein Löschschritt ist abgeschlossen.
    EraseStepDone,
}

/// Beobachter-Port (SPEC §4). Nie unter einer Sperre aufrufen (§17.3).
pub trait Observer: Send + Sync {
    /// Nimmt ein Ereignis entgegen.
    fn event(&self, e: ObserverEvent);
}

/// Hintergrund-Laufzeit (SPEC §4, SA-9). Teilbarer Pool.
pub trait Runtime: Send + Sync {
    /// Reiht einen Auftrag ein.
    fn spawn(&self, job: Box<dyn FnOnce() + Send>);
}

/// Gewichtung eines Eintrags für Caches und Budgets (SPEC §4, §8.5, D-08).
pub trait Weigher: Send + Sync {
    /// Version des Zählverfahrens; ändert sich mit dem Verfahren.
    fn version(&self) -> u32;
    /// Schreibt bis zu vier Gewichtssummen (`u64`) nach `out`.
    fn weigh(&self, key: &[u8], header: &[u8], out: &mut [u64; 4]);
}

/// Aufrufer-Filter bei Lesevorgängen (SPEC §4).
pub trait Filter: Send + Sync {
    /// `true`, wenn der Eintrag im Ergebnis bleibt.
    fn keep(&self, key: &[u8], header: &[u8]) -> bool;
}

/// Eintrag, den Erweiterungen erhalten (SPEC §4, §15.9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Schlüssel des Eintrags.
    pub key: Vec<u8>,
    /// Kopf des Eintrags.
    pub header: Vec<u8>,
    /// Stamp des Eintrags.
    pub stamp: Stamp,
}

/// Menge von Clustern für Löschoperationen (SPEC §4, §15.9).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClusterSet {
    ids: Vec<ClusterId>,
}

impl ClusterSet {
    /// Leere Menge.
    pub fn new() -> Self {
        Self::default()
    }

    /// Fügt einen Cluster hinzu. Doppelte Einträge werden ignoriert.
    pub fn insert(&mut self, c: ClusterId) {
        if !self.contains(c) {
            self.ids.push(c);
        }
    }

    /// Prüft die Zugehörigkeit.
    pub fn contains(&self, c: ClusterId) -> bool {
        self.ids.contains(&c)
    }
}

/// Fehler einer Erweiterung (SPEC §4). Ohne Inhalte (INV-KEY-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExtError {
    /// Operation von der Erweiterung nicht unterstützt.
    Unsupported,
    /// Eingabe ist beschädigt.
    Corrupt,
    /// Erstellen oder Zusammenführen ist fehlgeschlagen.
    Failed,
}

/// Erweiterungs-Port für Indexstrukturen je Run (SPEC §4, §15.9).
/// `rewrite_without` ist Pflicht: ohne sie ist Löschung nicht zulässig.
pub trait ExtensionBuilder: Send + Sync {
    /// Eindeutige Kennung.
    fn id(&self) -> u16;
    /// Version des Formats.
    fn version(&self) -> u16;
    /// Baut eine Erweiterung aus Einträgen.
    fn build(&self, entries: &mut dyn Iterator<Item = Entry>) -> Result<Vec<u8>, ExtError>;
    /// Führt Erweiterungen zusammen.
    fn merge(
        &self,
        inputs: &[&[u8]],
        survivors: &mut dyn Iterator<Item = Entry>,
    ) -> Result<Vec<u8>, ExtError>;
    /// Schreibt einen Block ohne die Einträge der Cluster in `drop`.
    fn rewrite_without(&self, block: &[u8], drop: &ClusterSet) -> Result<Vec<u8>, ExtError>;
}

/// Kalter Speicher (SPEC §4, O-8, Phase P6, optional).
pub trait ColdTier: Send + Sync {
    /// Lädt eine Datei hoch.
    fn upload(&self, f: FileRef) -> io::Result<()>;
    /// Liest einen Bereich.
    fn read_range(&self, f: FileRef, off: u64, len: u32) -> io::Result<Vec<u8>>;
    /// Löscht eine Datei.
    fn delete(&self, f: FileRef) -> io::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cluster_id_debug_zeigt_nur_vier_byte() {
        let c = ClusterId::from_bytes([0xAB; 16]);
        let s = format!("{c:?}");
        assert_eq!(s, "ClusterId(abababab..)");
        assert!(!s.contains("abababababab"));
    }

    #[test]
    fn cluster_set_ignoriert_doppelte() {
        let mut s = ClusterSet::new();
        let c = ClusterId::from_bytes([1; 16]);
        s.insert(c);
        s.insert(c);
        assert!(s.contains(c));
        assert_eq!(s.ids.len(), 1);
    }

    #[test]
    fn neue_traits_sind_dyn_kompatibel() {
        fn _a(_: Option<&dyn Checksum>) {}
        fn _b(_: Option<&dyn Hasher>) {}
        fn _c(_: Option<&dyn Anchor>) {}
        fn _d(_: Option<&dyn Observer>) {}
        fn _e(_: Option<&dyn Runtime>) {}
        fn _f(_: Option<&dyn Weigher>) {}
        fn _g(_: Option<&dyn Filter>) {}
        fn _h(_: Option<&dyn ExtensionBuilder>) {}
        fn _i(_: Option<&dyn ColdTier>) {}
    }
}

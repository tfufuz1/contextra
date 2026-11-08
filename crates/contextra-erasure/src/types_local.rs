//! Lokale Domänentypen und Port-Abstraktionen für `contextra-erasure`.
//!
//! Kapselt Schnittstellendefinitionen für Gates, Barrieren, Ledger, VFS-Erweiterungen,
//! Schlüsselverwaltung und Messungen, sodass `contextra-erasure` vollkommen autark
//! und isoliert als Ring-3-Löschkern funktioniert.

use futures_util::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

// =========================================================================
// Identifikatoren & Sequenzen
// =========================================================================

/// 16-Byte Lösch-Identifikator (SPEC §7.4, TY-09).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ErasureId(pub [u8; 16]);

impl ErasureId {
    /// Erzeugt eine `ErasureId` aus rohen 16 Bytes.
    pub const fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Liefert die 16 Bytes als Slice.
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Debug for ErasureId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ErasureId({:02x}{:02x}{:02x}{:02x}..)",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

impl fmt::Display for ErasureId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.0 {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

/// Streng monoton steigende Sequenznummer (SPEC §8.7, I-12).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Seq(pub u64);

impl Seq {
    /// Erzeugt eine neue Sequenznummer.
    pub const fn new(val: u64) -> Self {
        Self(val)
    }

    /// Liefert den numerischen Wert als u64.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Diskreter Zeitstempel in Mikrosekunden oder Millisekunden seit Epoche.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(pub u64);

impl Timestamp {
    /// Erzeugt einen neuen Zeitstempel.
    pub const fn new(val: u64) -> Self {
        Self(val)
    }

    /// Erzeugt Zeitstempel aus Mikrosekunden.
    pub const fn from_micros(micros: u64) -> Self {
        Self(micros)
    }

    /// Liefert den numerischen Wert.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// Liefert Mikrosekunden.
    pub const fn as_micros(&self) -> u64 {
        self.0
    }
}

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

// =========================================================================
// ReadGate & Schlüsselpuffer
// =========================================================================

/// Fester 16-Byte Schlüsselpuffer für Speicher- und Gate-Operationen (SPEC §8.1).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct KeyBuf(pub [u8; 16]);

impl KeyBuf {
    /// Erzeugt einen neuen `KeyBuf` aus rohen 16 Bytes.
    pub const fn new(raw: [u8; 16]) -> Self {
        Self(raw)
    }

    /// Gibt eine Referenz auf die Bytes zurück.
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Debug for KeyBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KeyBuf([REDACTED])")
    }
}

/// Begrenzungs-Schlüssel für Scans und Kompaktion.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum KeyBound {
    /// Exakte Schlüsselbegrenzung.
    Exact(KeyBuf),
    /// Unbegrenzte untere Grenze.
    UnboundedLower,
    /// Unbegrenzte obere Grenze.
    UnboundedUpper,
}

impl KeyBound {
    /// Erzeugt eine exakte Grenze.
    pub const fn exact(key: KeyBuf) -> Self {
        Self::Exact(key)
    }
}

impl fmt::Debug for KeyBound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exact(_) => write!(f, "KeyBound::Exact([REDACTED])"),
            Self::UnboundedLower => write!(f, "KeyBound::UnboundedLower"),
            Self::UnboundedUpper => write!(f, "KeyBound::UnboundedUpper"),
        }
    }
}

/// Halboffenes Schlüsselintervall `[start, end)`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyRange {
    /// Untere Grenze (inklusiv).
    pub start: KeyBuf,
    /// Obere Grenze (exklusiv).
    pub end: KeyBuf,
}

impl KeyRange {
    /// Erzeugt einen neuen Schlüsselbereich.
    pub const fn new(start: KeyBuf, end: KeyBuf) -> Self {
        Self { start, end }
    }

    /// Prüft ob ein Schlüssel im Intervall liegt.
    pub fn contains(&self, key: &KeyBuf) -> bool {
        key >= &self.start && key < &self.end
    }
}

impl fmt::Debug for KeyRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KeyRange {{ [REDACTED] }}")
    }
}

/// Gate-Entscheidung für Punktabfragen oder Dateigrenzen (SPEC §4.2, §11.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateDecision {
    /// Schlüssel ist lesbar.
    Open,
    /// Schlüssel ist gelöscht.
    Erased,
    /// Zugriff vorübergehend eingeschränkt.
    Restricted,
}

/// Gate-Entscheidungsvariante für Bereichsoperationen (SPEC §4.2, I-18).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RangeGate {
    /// Bereich enthält gelöschte Daten.
    ContainsErased,
    /// Bereich ist gesperrt / darf nicht verworfen werden.
    RestrictedNeverDiscard,
    /// Vollständig lesbar.
    EntirelyOpen,
}

/// Synchroner ReadGate-Port (SPEC §4.2, §11.3, PO-07).
pub trait ReadGate: Send + Sync {
    /// Prüft den Status eines einzelnen Schlüssels.
    fn check(&self, k: &KeyBuf) -> GateDecision;
    /// Prüft den Status einer Bereichsgrenze.
    fn check_bound(&self, b: &KeyBound) -> GateDecision;
    /// Liefert die Bereichsentscheidung für Kompaktionen.
    fn variant_for_range(&self, r: &KeyRange) -> RangeGate;
    /// Liefert die Gating-Sequenznummer, falls vorhanden.
    fn gated_seq(&self, k: &KeyBuf) -> Option<Seq>;
    /// Liefert den monotonen Gate-Epochenzähler.
    fn gate_epoch(&self) -> u64 {
        0
    }
    /// Prüft ob ein Schlüssel durch ein Gate abgedeckt ist.
    fn covers(&self, k: &KeyBuf) -> bool {
        match self.check(k) {
            GateDecision::Erased | GateDecision::Restricted => true,
            GateDecision::Open => false,
        }
    }
    /// Prüft Status eines kanonischen 78-Byte Schlüssels.
    fn check_raw(&self, _raw: &[u8; 78]) -> GateDecision {
        GateDecision::Open
    }
    /// Prüft ob ein 78-Byte Schlüssel abgedeckt ist.
    fn covers_raw(&self, raw: &[u8; 78]) -> bool {
        match self.check_raw(raw) {
            GateDecision::Erased | GateDecision::Restricted => true,
            GateDecision::Open => false,
        }
    }
    /// Liefert Sequenznummer eines 78-Byte Schlüssels.
    fn gated_seq_raw(&self, _raw: &[u8; 78]) -> Option<Seq> {
        None
    }
}

// =========================================================================
// Synchronisations-Barrieren
// =========================================================================

/// RAII-Token für geteilte Commit-Barriere.
pub struct BarrierShared {
    _token: Option<Arc<()>>,
}

impl BarrierShared {
    /// Erzeugt ein neues Shared-Token.
    pub fn new(token: Option<Arc<()>>) -> Self {
        Self { _token: token }
    }
}

impl fmt::Debug for BarrierShared {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BarrierShared")
    }
}

/// RAII-Token für exklusive Gate-Barriere.
pub struct BarrierExclusive {
    _token: Option<Arc<()>>,
}

impl BarrierExclusive {
    /// Erzeugt ein neues Exclusive-Token.
    pub fn new(token: Option<Arc<()>>) -> Self {
        Self { _token: token }
    }
}

impl fmt::Debug for BarrierExclusive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BarrierExclusive")
    }
}

/// RAII-Token für Kompaktions-Erlaubnis.
pub struct CompactionPermit {
    _token: Option<Arc<()>>,
}

impl CompactionPermit {
    /// Erzeugt ein neues CompactionPermit-Token.
    pub fn new(token: Option<Arc<()>>) -> Self {
        Self { _token: token }
    }
}

impl fmt::Debug for CompactionPermit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CompactionPermit")
    }
}

/// RAII-Token für exklusive Purge-Erlaubnis.
pub struct PurgePermit {
    _token: Option<Arc<()>>,
}

impl PurgePermit {
    /// Erzeugt ein neues PurgePermit-Token.
    pub fn new(token: Option<Arc<()>>) -> Self {
        Self { _token: token }
    }
}

impl fmt::Debug for PurgePermit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PurgePermit")
    }
}

/// CommitBarrier-Port (SPEC §11.3, D-57, D-78).
pub trait CommitBarrier: Send + Sync {
    /// Bezieht geteilten Commit-Zugriff.
    fn shared(&self) -> BoxFuture<'_, BarrierShared>;
    /// Bezieht exklusiven Gate-Zugriff.
    fn exclusive(&self) -> BoxFuture<'_, BarrierExclusive>;
}

/// PurgeBarrier-Port (SPEC §11.3, §11.4).
pub trait PurgeBarrier: Send + Sync {
    /// Bezieht geteilte Kompaktions-Erlaubnis.
    fn compaction(&self) -> BoxFuture<'_, CompactionPermit>;
    /// Bezieht exklusive Purge-Erlaubnis.
    fn purge(&self) -> BoxFuture<'_, PurgePermit>;
    /// Prüft ob ein exklusiver Purge wartet.
    fn purge_waiting(&self) -> bool;
}

impl<T: CommitBarrier + ?Sized> CommitBarrier for Arc<T> {
    fn shared(&self) -> BoxFuture<'_, BarrierShared> {
        (**self).shared()
    }
    fn exclusive(&self) -> BoxFuture<'_, BarrierExclusive> {
        (**self).exclusive()
    }
}

impl<T: PurgeBarrier + ?Sized> PurgeBarrier for Arc<T> {
    fn compaction(&self) -> BoxFuture<'_, CompactionPermit> {
        (**self).compaction()
    }
    fn purge(&self) -> BoxFuture<'_, PurgePermit> {
        (**self).purge()
    }
    fn purge_waiting(&self) -> bool {
        (**self).purge_waiting()
    }
}

// =========================================================================
// Schlüsselverwaltung, Sealer & Keyring
// =========================================================================

/// Eindeutige Dateikennung.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FileId(pub u64);

impl FileId {
    /// Erzeugt eine neue `FileId`.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Liefert die numerische Kennung.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Dateityp für den FileKeyring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileKind {
    /// SSTable-Datendatei.
    SSTable,
    /// WAL-Datei.
    Wal,
    /// Textindex-Segment.
    TextSegment,
    /// Vektor-HNSW-Segment.
    VectorSegment,
    /// Allgemeine Hilfsdatei.
    Auxiliary,
}

/// Flüchtiger Dateischlüssel mit Zeroize-on-Drop.
pub struct FileKey {
    raw: Vec<u8>,
    file_id: FileId,
}

impl FileKey {
    /// Erzeugt einen neuen `FileKey`.
    pub fn new(raw: Vec<u8>, file_id: FileId) -> Self {
        Self { raw, file_id }
    }

    /// Liefert Referenz auf die geschützten Bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.raw
    }

    /// Liefert die Dateikennung.
    pub fn file_id(&self) -> FileId {
        self.file_id
    }
}

impl Drop for FileKey {
    fn drop(&mut self) {
        self.raw.fill(0);
    }
}

impl fmt::Debug for FileKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "FileKey {{ file_id: {:?}, key: [REDACTED] }}",
            self.file_id
        )
    }
}

/// Versiegelungskontext.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SealCtx {
    /// Epoche oder Slot-Identifikator.
    pub epoch: u64,
}

impl SealCtx {
    /// Erzeugt einen neuen Versiegelungskontext.
    pub const fn new(epoch: u64) -> Self {
        Self { epoch }
    }
}

impl fmt::Debug for SealCtx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SealCtx {{ epoch: {} }}", self.epoch)
    }
}

/// Versiegelter Datensatz-Umschlag.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    payload: Vec<u8>,
}

impl Envelope {
    /// Erzeugt einen neuen Envelope.
    pub fn new(payload: Vec<u8>) -> Self {
        Self { payload }
    }

    /// Gibt eine Referenz auf die Payload zurück.
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

impl fmt::Debug for Envelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Envelope([REDACTED, len={}])", self.payload.len())
    }
}

/// Port-Vertrag für RecordSealer.
pub trait RecordSealer: Send + Sync {
    /// Versiegelt Klartext.
    fn seal<'a>(
        &'a self,
        ctx: &'a SealCtx,
        key: &'a KeyBuf,
        plain: &'a [u8],
    ) -> BoxFuture<'a, Result<Envelope, String>>;

    /// Öffnet einen Envelope.
    fn open<'a>(&'a self, key: &'a KeyBuf, env: &'a [u8])
        -> BoxFuture<'a, Result<Vec<u8>, String>>;

    /// Invalidiert Schlüssel aus dem Cache.
    fn evict(&self, scope: &KeyRange);

    /// Erzwingt Slot-Dauerhaftigkeit.
    fn make_slots_durable(&self) -> BoxFuture<'_, Result<(), String>>;
}

/// Port-Vertrag für Dateischlüsselverwaltung.
pub trait FileKeyring: Send + Sync {
    /// Erzeugt einen neuen Dateischlüssel.
    fn create(&self, kind: FileKind, id: FileId) -> BoxFuture<'_, Result<FileKey, String>>;
    /// Öffnet einen existierenden Dateischlüssel.
    fn open(&self, id: FileId) -> BoxFuture<'_, Result<FileKey, String>>;
    /// Vernichtet den Dateischlüssel im Tresor dauerhaft.
    fn destroy(&self, id: FileId) -> BoxFuture<'_, Result<(), String>>;
}

// =========================================================================
// Ledger-Sink & Audit-Typen
// =========================================================================

/// Intervall-Menge für Ledger-Einträge.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RangeSet {
    ranges: Vec<(KeyBuf, KeyBuf)>,
}

impl RangeSet {
    /// Erzeugt eine leere Bereichsmenge.
    pub const fn empty() -> Self {
        Self { ranges: Vec::new() }
    }

    /// Erzeugt eine Bereichsmenge aus Intervallen.
    pub fn new(ranges: Vec<(KeyBuf, KeyBuf)>) -> Self {
        Self { ranges }
    }

    /// Liefert die Bereiche.
    pub fn ranges(&self) -> &[(KeyBuf, KeyBuf)] {
        &self.ranges
    }
}

impl fmt::Debug for RangeSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RangeSet(count={})", self.ranges.len())
    }
}

/// Revisionssicherer Ledger-Eintrag.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// Commitment-Hash der Löschdefinition.
    pub commitment: [u8; 32],
    /// Abgedeckte Schlüsselbereiche.
    pub range_set: RangeSet,
    /// Gate-Schließsequenz.
    pub gated_seq: Seq,
    /// Abschluss-Zeitstempel.
    pub completed_at: Timestamp,
    /// Verketteter Hash.
    pub chain_hash: [u8; 32],
    /// Integritäts-MAC.
    pub ledger_mac: [u8; 32],
}

impl LedgerEntry {
    /// Erzeugt einen neuen Ledger-Eintrag.
    pub const fn new(
        commitment: [u8; 32],
        range_set: RangeSet,
        gated_seq: Seq,
        completed_at: Timestamp,
        chain_hash: [u8; 32],
        ledger_mac: [u8; 32],
    ) -> Self {
        Self {
            commitment,
            range_set,
            gated_seq,
            completed_at,
            chain_hash,
            ledger_mac,
        }
    }

    /// Berechnet die verkettete Hash-Kette.
    pub fn compute_chained(
        commitment: [u8; 32],
        range_set: RangeSet,
        gated_seq: Seq,
        completed_at: Timestamp,
        prev_hash: [u8; 32],
        ledger_mac: [u8; 32],
    ) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&prev_hash);
        hasher.update(&commitment);
        hasher.update(&gated_seq.as_u64().to_le_bytes());
        hasher.update(&completed_at.as_u64().to_le_bytes());
        for (start, end) in range_set.ranges() {
            hasher.update(start.as_bytes());
            hasher.update(end.as_bytes());
        }
        let chain_hash = *hasher.finalize().as_bytes();

        Self {
            commitment,
            range_set,
            gated_seq,
            completed_at,
            chain_hash,
            ledger_mac,
        }
    }

    /// Verifiziert den Ketten-Hash.
    pub fn verify_chain_hash(&self, prev_hash: &[u8; 32]) -> bool {
        let mut hasher = blake3::Hasher::new();
        hasher.update(prev_hash);
        hasher.update(&self.commitment);
        hasher.update(&self.gated_seq.as_u64().to_le_bytes());
        hasher.update(&self.completed_at.as_u64().to_le_bytes());
        for (start, end) in self.range_set.ranges() {
            hasher.update(start.as_bytes());
            hasher.update(end.as_bytes());
        }
        hasher.finalize().as_bytes() == &self.chain_hash
    }
}

impl fmt::Debug for LedgerEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LedgerEntry {{ gated_seq: {:?}, completed_at: {:?}, chain_hash: {:02x?} }}",
            self.gated_seq, self.completed_at, &self.chain_hash
        )
    }
}

/// Port-Vertrag für das Lösch-Ledger.
pub trait LedgerSink: Send + Sync {
    /// Hängt einen Eintrag an das Ledger an.
    fn append(&self, e: &LedgerEntry) -> Result<(), String>;
    /// Liest alle Einträge aus.
    fn read_all(&self) -> Result<Vec<LedgerEntry>, String>;
    /// Bereinigt alte Einträge nach Aufbewahrungsfrist.
    fn prune(&self, older_than: Timestamp) -> Result<usize, String>;
}

// =========================================================================
// Schicht-Definitionen & Zeugen
// =========================================================================

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
        write!(f, "LayerId({:02x?})", &self.0)
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
pub struct ControlEvidence(pub ());

impl ControlEvidence {
    /// Bezeugt das Auffinden des Kontrollmusters.
    #[inline]
    pub const fn observed() -> Self {
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

/// Schicht-Geltungsbereich vor dem Gating.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerScope {
    scope_id: [u8; 16],
    affected_units: Vec<u64>,
}

impl LayerScope {
    /// Erzeugt einen leeren Geltungsbereich.
    pub const fn empty() -> Self {
        Self {
            scope_id: [0u8; 16],
            affected_units: Vec::new(),
        }
    }
    /// Erzeugt einen neuen Geltungsbereich.
    pub fn new(scope_id: [u8; 16], affected_units: Vec<u64>) -> Self {
        Self {
            scope_id,
            affected_units,
        }
    }
    /// Liefert die Scope-ID.
    pub fn scope_id(&self) -> &[u8; 16] {
        &self.scope_id
    }
    /// Liefert die betroffenen Einheiten.
    pub fn affected_units(&self) -> &[u64] {
        &self.affected_units
    }
}

impl fmt::Debug for LayerScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LayerScope {{ units_count: {} }}",
            self.affected_units.len()
        )
    }
}

/// Lösch-Zielangabe.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EraseTarget {
    /// Schlüsselintervall.
    KeyRange {
        /// Beginn inklusiv.
        start: Vec<u8>,
        /// Ende exklusiv.
        end: Vec<u8>,
    },
    /// Diskrete Menge von Schlüsseln.
    ExactKeys(Vec<Vec<u8>>),
}

impl fmt::Debug for EraseTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KeyRange { .. } => write!(f, "EraseTarget::KeyRange([REDACTED])"),
            Self::ExactKeys(k) => write!(f, "EraseTarget::ExactKeys(count={})", k.len()),
        }
    }
}

/// Bereinigungs-Ergebnis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PurgeOutcome {
    /// Bereinigte Elemente.
    pub purged_count: u64,
    /// Freigegebene Bytes.
    pub bytes_reclaimed: u64,
}

impl PurgeOutcome {
    /// Erzeugt ein neues PurgeOutcome.
    pub const fn new(purged_count: u64, bytes_reclaimed: u64) -> Self {
        Self {
            purged_count,
            bytes_reclaimed,
        }
    }
}

/// Bereinigungs-Kontext mit Deadline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PurgeCx {
    /// Deadline in Millisekunden.
    pub deadline_ms: u64,
}

impl PurgeCx {
    /// Erzeugt einen neuen Kontext.
    pub const fn new(deadline_ms: u64) -> Self {
        Self { deadline_ms }
    }
}

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
        write!(f, "LayerWitness {{ hash: {:02x?} }}", &self.witness_hash)
    }
}

/// Schichtvertrag für Speicher- und Index-Teilnehmer (SPEC §8.2).
pub trait ErasureLayer: Send + Sync {
    /// Schichtkennung.
    fn id(&self) -> LayerId;
    /// Löst Ziel vor dem Gating in den konkreten Bereich auf.
    fn resolve(&self, t: &EraseTarget) -> BoxFuture<'_, Result<LayerScope, String>>;
    /// Vorbereitungsmaßnahmen vor dem Gating.
    fn on_gate(&self, s: &LayerScope) -> BoxFuture<'_, Result<(), String>>;
    /// Führt physische Bereinigung durch.
    fn purge(&self, s: &LayerScope, cx: &PurgeCx) -> BoxFuture<'_, Result<PurgeOutcome, String>>;
    /// Prüft Rückstände und erzeugt Zeugen.
    fn witness(
        &self,
        s: &LayerScope,
        sc: &Scanner<Armed>,
    ) -> BoxFuture<'_, Result<LayerWitness, String>>;
    /// Meldet die Schlüsselabdeckung.
    fn key_coverage(&self) -> KeyCoverage;
}

// =========================================================================
// Berichtssignatur & Observer
// =========================================================================

/// Signer-ID-Typalias.
pub type SignerId = u8;

/// Signatur eines DeletionReports (SPEC F.10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportSignature {
    /// Signer ID (0 = abgeleitet, 1 = extern/Betreiber).
    pub signer_id: SignerId,
    /// Base64url-kodierte Signatur.
    pub signature_base64: String,
    /// SHA-256 Fingerabdruck des öffentlichen Schlüssels.
    pub pub_fingerprint: String,
}

/// Signierer-Fehlerklasse.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SignerError {
    /// Schlüssel nicht verfügbar.
    #[error("signing key unavailable: {0}")]
    KeyUnavailable(String),
    /// Externer Signierer fehlgeschlagen.
    #[error("external signer failed: {0}")]
    ExternalSignerFailed(String),
    /// Ungültige Nutzlast.
    #[error("invalid report payload: {0}")]
    InvalidPayload(String),
}

/// Port-Vertrag für Berichtssignierung.
pub trait ReportSigner: Send + Sync {
    /// Liefert die Signer-ID.
    fn signer_id(&self) -> SignerId;
    /// Signiert kanonische Berichts-Bytes.
    fn sign_report<'a>(
        &'a self,
        canonical_report_bytes: &'a [u8],
    ) -> BoxFuture<'a, Result<ReportSignature, SignerError>>;
}

impl<T: ReportSigner + ?Sized> ReportSigner for Arc<T> {
    fn signer_id(&self) -> SignerId {
        (**self).signer_id()
    }
    fn sign_report<'a>(
        &'a self,
        canonical_report_bytes: &'a [u8],
    ) -> BoxFuture<'a, Result<ReportSignature, SignerError>> {
        (**self).sign_report(canonical_report_bytes)
    }
}

/// Ziel-Ebene für Observer-Ereignisse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetKind {
    /// Dokument.
    Document,
    /// Betroffener.
    Subject,
    /// Collection.
    Collection,
    /// Mandant.
    Tenant,
    /// Ersetzung (`replace_erasing`).
    DocumentReplace,
    /// Überschreibung (`update_with(..., Purge)`).
    Superseded,
}

/// Beobachtete Lebenszyklus-Phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservedPhase {
    /// Read-Gate eingerichtet.
    Gated,
    /// Bereinigung und Verifikation abgeschlossen.
    Completed,
    /// Einschränkung aktiv.
    Restricted,
    /// Einschränkung aufgehoben.
    Lifted,
}

/// Ereignis für den ErasureObserver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErasureEvent {
    /// ID der Löschoperation.
    pub id: ErasureId,
    /// Art des Ziels.
    pub target_kind: TargetKind,
    /// Erreichte Phase.
    pub phase: ObservedPhase,
    /// Zeitstempel.
    pub at: u64,
}

/// Fehlerklasse für den Observer.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ObserverError {
    /// Vorübergehender Fehler.
    #[error("transient delivery failure: {0}")]
    Transient(String),
    /// Dauerhafter Fehler.
    #[error("permanent delivery failure: {0}")]
    Permanent(String),
    /// Zeitüberschreitung.
    #[error("observer delivery timed out")]
    Timeout,
}

/// Port-Vertrag für Lösch-Ereignis-Benachrichtigungen.
pub trait ErasureObserver: Send + Sync + 'static {
    /// Benachrichtigt den Observer über ein Ereignis.
    fn on_erased(&self, ev: ErasureEvent) -> BoxFuture<'static, Result<(), ObserverError>>;
}

impl<T: ErasureObserver + ?Sized> ErasureObserver for Arc<T> {
    fn on_erased(&self, ev: ErasureEvent) -> BoxFuture<'static, Result<(), ObserverError>> {
        (**self).on_erased(ev)
    }
}

// =========================================================================
// Saga-Lebenszyklus, Status & Erase-Typen
// =========================================================================

/// Löschmodus.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum EraseMode {
    /// Vollständige physische Löschung und Schlüsselvernichtung.
    Erase,
    /// Einschränkung der Verarbeitung.
    Restrict,
}

/// Timing der Löschung.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Timing {
    /// Sofort.
    Now,
    /// Gebündelt.
    Batched,
}

/// Aufbewahrungscode.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum RetentionCode {
    /// Keine Aufbewahrungspflicht.
    NoObligation,
    /// Pflicht abgelaufen.
    ObligationExpired,
    /// Pflicht aktiv.
    ObligationActive,
    /// Rechtsansprüche anhängig.
    LegalClaimsPending,
    /// Betroffene Person hat Einschränkung verlangt.
    DataSubjectRequestedRestriction,
    /// Betreiber trägt Verantwortung per Direktive / Anordnung.
    OperatorOverride,
}

/// Attestierung zur Aufbewahrung.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct RetentionAttestation {
    /// Code.
    pub code: RetentionCode,
    /// Betreiberkennung.
    pub attested_by: OperatorRef,
    /// Zeitstempel.
    pub attested_at: Timestamp,
}

/// Löschquittung.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct EraseReceipt {
    /// Eindeutige Kennung.
    pub id: ErasureId,
    /// Status.
    pub status: ErasureStatus,
    /// Zeitstempel der Sperrung.
    pub gated_at: Timestamp,
}

/// Grund für Schwebezustand.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum PendingReason {
    /// I/O-Fehler.
    Io,
    /// SLA überschritten.
    SlaExceeded,
    /// Schichtfehler.
    LayerFailed(LayerId),
    /// Zeuge fehlt.
    WitnessMissing,
    /// Plattenreserve unterschritten.
    DiskReserve,
    /// Unspezifischer Fehler.
    Other,
}

/// Ablehnungsgrund vor Gate.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum RefusalReason {
    /// Aufbewahrungspflicht.
    RetentionObligation,
    /// Ungültige Parameter.
    InvalidParameters,
}

/// Phase eines Löschablaufs (SPEC §7.4, D-124 Abs. 5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Phase {
    /// Intent-Registrierung.
    Intent,
    /// Gate-Setzung.
    Gate,
    /// Schlüsselvernichtung.
    KeyDestroy,
    /// Physisches Bereinigen.
    Purge,
    /// Unabhängige Nachweismessung.
    Verify,
    /// Ersetzung.
    Replace,
    /// Überschreibung.
    Supersede,
    /// Einschränkung.
    Restrict,
}

/// Status einer Lösch-Saga (SPEC §11.2, D-124).
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum ErasureStatus {
    /// Auftrag empfangen.
    Requested,
    /// Im Gate gesperrt.
    Gated,
    /// Schlüssel vernichtet.
    KeyDestroyed,
    /// Bereinigung läuft.
    Purging,
    /// Messung läuft.
    Verifying,
    /// Abgeschlossen.
    Completed,
    /// Eingeschränkt.
    Restricted,
    /// Einschränkung aufgehoben.
    Lifted,
    /// Verzögert.
    Pending {
        /// Phase.
        phase: Phase,
        /// Grund.
        reason: PendingReason,
    },
    /// Abgelehnt.
    Refused(RefusalReason),
}

// =========================================================================
// VFS-Erweiterungen & Verifikation
// =========================================================================

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

/// Erweiterungstraits für `Vfs` zum Öffnen von positionellen `VfsFile` Handles.
pub trait VfsExt {
    /// Öffnet eine Datei als VfsFile.
    fn open_file(
        &self,
        path: &std::path::Path,
        opts: &VfsOpenOptions,
    ) -> Result<Box<dyn VfsFile>, crate::error::ErasureError>;
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

// =========================================================================
// Autarke Basis- & Porttypen (Klasse A Entkopplung)
// =========================================================================

/// Mandanten-Identifikator.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct TenantId(pub u64);

impl TenantId {
    /// Erzeugt eine neue `TenantId`.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Liefert den internen `u64`-Wert.
    pub const fn inner(self) -> u64 {
        self.0
    }
}

impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TenantId({})", self.0)
    }
}

/// Collection-Identifikator.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct CollectionId(pub u64);

impl CollectionId {
    /// Erzeugt eine neue `CollectionId`.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Liefert den internen `u64`-Wert.
    pub const fn inner(self) -> u64 {
        self.0
    }
}

impl fmt::Display for CollectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CollectionId({})", self.0)
    }
}

/// Dokument-Identifikator.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct DocId(pub u64);

impl DocId {
    /// Erzeugt eine neue `DocId`.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Liefert den internen `u64`-Wert.
    pub const fn inner(self) -> u64 {
        self.0
    }

    /// Erzeugt eine `DocId` aus einem Schlüssel mittels BLAKE3.
    pub fn from_key(key: &str) -> Result<Self, crate::error::ErasureError> {
        if key.is_empty() {
            return Err(crate::error::ErasureError::Internal(
                "Key cannot be empty".to_string(),
            ));
        }
        let hash = blake3::hash(key.as_bytes());
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&hash.as_bytes()[..8]);
        Ok(Self(u64::from_le_bytes(buf)))
    }
}

impl fmt::Display for DocId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DocId({})", self.0)
    }
}

/// Transaktions-Identifikator.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct TxId(pub u64);

impl TxId {
    /// Erzeugt eine neue `TxId`.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Liefert den internen `u64`-Wert.
    pub const fn inner(self) -> u64 {
        self.0
    }
}

impl fmt::Display for TxId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TxId({})", self.0)
    }
}

/// RequestContext für Löschoperationen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestContext {
    /// Tenant identifier for authorization and scope isolation.
    pub tenant_id: TenantId,
    /// Transaction ID allocated for the operation.
    pub tx_id: TxId,
}

impl RequestContext {
    /// Erzeugt einen neuen `RequestContext`.
    pub fn new(tenant_id: TenantId, tx_id: TxId) -> Self {
        Self { tenant_id, tx_id }
    }
}

/// Saga record containing deletion tracking state and digests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SagaRecord {
    /// BLAKE3 hash of deleted keys.
    pub deleted_keys_hash: [u8; 32],
}

impl SagaRecord {
    /// Erzeugt einen neuen `SagaRecord`.
    pub fn new(deleted_keys_hash: [u8; 32]) -> Self {
        Self { deleted_keys_hash }
    }
}

/// Identifikator für die physische Storage-Schicht.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WitnessLayer {
    /// LSM Memtable
    LsmMemtable,
    /// SsTable Dateien über alle Level
    SsTableAllLevels,
    /// HNSW Vektorgraph-Index
    HnswIndex,
    /// Write-Ahead-Log Segmente
    WalAllSegments,
    /// Compressed Sparse Row Graph
    CsrGraph,
    /// Key-Value Cache Segmente
    KvCacheSegments,
    /// In-Memory Embedding Cache
    EmbeddingCache,
    /// Benutzerdefinierte Schichtbezeichnung für Erweiterungen
    Custom(String),
}

impl WitnessLayer {
    /// Gibt den String-Namen der Schicht zurück.
    pub fn as_str(&self) -> &str {
        match self {
            Self::LsmMemtable => "LsmMemtable",
            Self::SsTableAllLevels => "SsTableAllLevels",
            Self::HnswIndex => "HnswIndex",
            Self::WalAllSegments => "WalAllSegments",
            Self::CsrGraph => "CsrGraph",
            Self::KvCacheSegments => "KvCacheSegments",
            Self::EmbeddingCache => "EmbeddingCache",
            Self::Custom(s) => s.as_str(),
        }
    }
}

impl From<&str> for WitnessLayer {
    fn from(s: &str) -> Self {
        match s {
            "LsmMemtable" => WitnessLayer::LsmMemtable,
            "SsTableAllLevels" => WitnessLayer::SsTableAllLevels,
            "HnswIndex" => WitnessLayer::HnswIndex,
            "WalAllSegments" => WitnessLayer::WalAllSegments,
            "CsrGraph" => WitnessLayer::CsrGraph,
            "KvCacheSegments" => WitnessLayer::KvCacheSegments,
            "EmbeddingCache" => WitnessLayer::EmbeddingCache,
            other => WitnessLayer::Custom(other.to_string()),
        }
    }
}

impl From<String> for WitnessLayer {
    fn from(s: String) -> Self {
        WitnessLayer::from(s.as_str())
    }
}

/// Angewandte Methode/Technik während der Verifikation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanTechnique {
    /// Direkter struktureller Scan.
    Structural,
    /// Scan über entschlüsselte Ansicht.
    DecryptedView,
    /// Index-Introspektion.
    IndexIntrospection,
    /// Nadel-Scan (Needle Scan).
    NeedleScan,
    /// Schicht wurde in der aktuellen Ausführung nicht gemessen mit Begründung.
    NotMeasured(String),
    /// Schichtmessung ist aufgrund der Feature-Konfiguration deaktiviert.
    FeatureDisabled,
}

/// Scan-Methode für Berichts- und Nachweiszwecke.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanMethod {
    /// Direct structural scan of storage files, indices, or tables.
    Structural,
    /// Layer was not measured during this execution with specified reason.
    NotMeasured(String),
    /// Layer measurement is disabled due to feature configuration.
    FeatureDisabled,
}

/// Zielbeschreibung des Scans inklusive Hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanTarget {
    /// Bezeichner des Scan-Ziels (z. B. Tenant, Collection, Path).
    pub target_id: String,
    /// BLAKE3 Hash des Scan-Ziels.
    pub target_hash: [u8; 32],
}

impl ScanTarget {
    /// Erzeugt ein neues ScanTarget.
    pub fn new(target_id: impl Into<String>, target_hash: [u8; 32]) -> Self {
        Self {
            target_id: target_id.into(),
            target_hash,
        }
    }
}

/// Statistiken über bereinigte Einheiten.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PurgeStats {
    /// Anzahl physisch bereinigter Einheiten.
    pub purged_units: u64,
    /// Anzahl freigegebener Bytes.
    pub bytes_freed: u64,
}

/// Abstrakter Bezeichner für den zu bereinigenden Schlüsselbereich (SPEC v4.0 §6.2, B3).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyScope {
    /// Mandanten-Löschung.
    Tenant(TenantId),
    /// Collection-weite Löschung.
    Collection {
        /// Mandant.
        tenant: TenantId,
        /// Collection.
        collection: CollectionId,
    },
    /// Einzelnes Dokument.
    Document {
        /// Mandant.
        tenant: TenantId,
        /// Collection.
        collection: CollectionId,
        /// Dokument-ID.
        doc_id: DocId,
    },
}

/// Trait für physische Bereinigungsaktionen (Purger) einer Schicht.
pub trait Purger: Send + Sync {
    /// Führt die physische Bereinigung für das gegebene Ziel aus.
    fn purge(
        &self,
        ctx: &RequestContext,
        target: &ScanTarget,
    ) -> Result<PurgeStats, crate::error::ErasureError>;
}

/// Laufende Scan-Session zur kontrollierten Erzeugung eines `ScanWitness`.
#[derive(Debug)]
pub struct ScanSession {
    layer: WitnessLayer,
    target: ScanTarget,
    technique: ScanTechnique,
    scanned: u64,
    remaining: u64,
    control_matches: u64,
    unit_hashes: Vec<[u8; 32]>,
}

impl ScanSession {
    /// Startet eine neue ScanSession.
    pub fn begin(layer: WitnessLayer, target: ScanTarget, technique: ScanTechnique) -> Self {
        Self {
            layer,
            target,
            technique,
            scanned: 0,
            remaining: 0,
            control_matches: 0,
            unit_hashes: Vec::new(),
        }
    }

    /// Registriert eine gescannte Einheit.
    pub fn record_unit(&mut self, unit_hash: [u8; 32], is_live_residual: bool) {
        self.scanned = self.scanned.saturating_add(1);
        if is_live_residual {
            self.remaining = self.remaining.saturating_add(1);
        }
        self.unit_hashes.push(unit_hash);
    }

    /// Registriert eine bestandene Positivkontrolle (Control Match).
    pub fn record_control_match(&mut self) {
        self.control_matches = self.control_matches.saturating_add(1);
    }

    /// Schließt die Session ab und erzeugt den `ScanWitness` (gesperrter Mint-Pfad).
    pub fn finish(mut self, clock: &dyn Clock) -> Result<ScanWitness, crate::error::ErasureError> {
        match &self.technique {
            ScanTechnique::FeatureDisabled => {}
            ScanTechnique::NotMeasured(reason) => {
                if reason.trim().is_empty() {
                    return Err(crate::error::ErasureError::Internal(
                        "INV-WITNESS-MINT-1 violation: NotMeasured technique requires non-empty reason".to_string(),
                    ));
                }
            }
            _ => {
                if self.scanned == 0 {
                    return Err(crate::error::ErasureError::Internal(
                        "INV-WITNESS-MINT-1 violation: scanned count cannot be 0 for active scan technique".to_string(),
                    ));
                }
                if self.control_matches == 0 {
                    return Err(crate::error::ErasureError::Internal(
                        "INV-WITNESS-MINT-1 violation: positive control check failed (control_matches == 0)".to_string(),
                    ));
                }
            }
        }

        self.unit_hashes.sort();

        let mut hasher = blake3::Hasher::new();
        hasher.update(self.layer.as_str().as_bytes());
        hasher.update(format!("{:?}", self.technique).as_bytes());
        hasher.update(&self.target.target_hash);
        for uh in &self.unit_hashes {
            hasher.update(uh);
        }
        let digest: [u8; 32] = *hasher.finalize().as_bytes();
        let measured_at = clock.now_unix_nanos();

        Ok(ScanWitness::new(
            self.layer,
            self.scanned,
            self.remaining,
            self.technique,
            digest,
            measured_at,
        ))
    }
}

/// Verifiziertes Messergebnis eines Storage-Scanners (SPEC §12.7.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanWitness {
    layer: WitnessLayer,
    scanned: u64,
    remaining: u64,
    technique: ScanTechnique,
    method: ScanMethod,
    digest: [u8; 32],
    measured_at: u64,
}

impl ScanWitness {
    /// Erzeugt einen neuen `ScanWitness`.
    pub fn new(
        layer: WitnessLayer,
        scanned: u64,
        remaining: u64,
        technique: ScanTechnique,
        digest: [u8; 32],
        measured_at: u64,
    ) -> Self {
        let method = match &technique {
            ScanTechnique::Structural
            | ScanTechnique::DecryptedView
            | ScanTechnique::IndexIntrospection
            | ScanTechnique::NeedleScan => ScanMethod::Structural,
            ScanTechnique::NotMeasured(reason) => ScanMethod::NotMeasured(reason.clone()),
            ScanTechnique::FeatureDisabled => ScanMethod::FeatureDisabled,
        };
        Self {
            layer,
            scanned,
            remaining,
            technique,
            method,
            digest,
            measured_at,
        }
    }

    /// Gibt die gescannte Storage-Schicht als &str zurück.
    pub fn layer(&self) -> &str {
        self.layer.as_str()
    }

    /// Gibt die gescannte Storage-Schicht als &WitnessLayer zurück.
    pub fn witness_layer(&self) -> &WitnessLayer {
        &self.layer
    }

    /// Gibt die Anzahl der während der Verifikation gescannten Einheiten zurück.
    pub fn scanned(&self) -> u64 {
        self.scanned
    }

    /// Gibt die Anzahl der verbleibenden Lebendeinträge zurück.
    pub fn remaining(&self) -> u64 {
        self.remaining
    }

    /// Gibt die angewandte Scantechnik zurück.
    pub fn technique(&self) -> &ScanTechnique {
        &self.technique
    }

    /// Gibt die ScanMethod-Entsprechung zurück.
    pub fn method(&self) -> &ScanMethod {
        &self.method
    }

    /// Gibt den BLAKE3-Digest der Messung zurück.
    pub fn digest(&self) -> &[u8; 32] {
        &self.digest
    }

    /// Gibt den Zeitstempel der Messung (Unix Nanos) zurück.
    pub fn measured_at(&self) -> u64 {
        self.measured_at
    }
}

/// Trait für Storage-Scanner zur Durchführung von Restmengen-Messungen.
pub trait LayerScanner: Send + Sync {
    /// Gibt die von diesem Scanner bediente Schicht zurück.
    fn layer(&self) -> WitnessLayer;

    /// Führt das Scannen von Einheiten auf der übergebenen `ScanSession` aus.
    fn scan_units(
        &self,
        session: &mut ScanSession,
        ctx: &RequestContext,
    ) -> Result<(), crate::error::ErasureError>;

    /// Default-Implementierung von `scan`.
    fn scan(
        &self,
        ctx: &RequestContext,
        target: ScanTarget,
        technique: ScanTechnique,
        clock: &dyn Clock,
    ) -> Result<ScanWitness, crate::error::ErasureError> {
        let mut session = ScanSession::begin(self.layer(), target, technique);
        self.scan_units(&mut session, ctx)?;
        session.finish(clock)
    }
}

/// Port trait for time operations.
pub trait Clock: Send + Sync + 'static {
    /// Returns the current wall-clock time as nanoseconds since the Unix epoch.
    fn now_unix_nanos(&self) -> u64;

    /// Returns nanoseconds elapsed since a monotonic baseline reference point.
    fn monotonic_nanos(&self) -> u64;
}

/// Standard implementation of [`Clock`].
#[derive(Debug, Clone, Copy)]
pub struct SystemClock {
    start_instant: std::time::Instant,
}

impl SystemClock {
    /// Creates a new [`SystemClock`].
    #[allow(clippy::disallowed_methods)]
    pub fn new() -> Self {
        Self {
            start_instant: std::time::Instant::now(),
        }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    #[allow(clippy::disallowed_methods)]
    fn now_unix_nanos(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX))
            .unwrap_or(0)
    }

    #[allow(clippy::disallowed_methods)]
    fn monotonic_nanos(&self) -> u64 {
        let nanos = std::time::Instant::now()
            .saturating_duration_since(self.start_instant)
            .as_nanos();
        u64::try_from(nanos).unwrap_or(u64::MAX)
    }
}

impl<T: Clock + ?Sized> Clock for Arc<T> {
    fn now_unix_nanos(&self) -> u64 {
        (**self).now_unix_nanos()
    }

    fn monotonic_nanos(&self) -> u64 {
        (**self).monotonic_nanos()
    }
}

/// Port trait for unique ID generation.
pub trait IdGen: Send + Sync + 'static {
    /// Generates and returns the next unique 64-bit unsigned identifier.
    fn next_id(&self) -> u64;
}

/// A thread-safe sequential ID generator.
#[derive(Debug)]
pub struct SequentialIdGen {
    next: std::sync::atomic::AtomicU64,
}

impl SequentialIdGen {
    /// Creates a new [`SequentialIdGen`].
    pub fn new(start: u64) -> Self {
        Self {
            next: std::sync::atomic::AtomicU64::new(start),
        }
    }
}

impl Default for SequentialIdGen {
    fn default() -> Self {
        Self::new(1)
    }
}

impl IdGen for SequentialIdGen {
    fn next_id(&self) -> u64 {
        self.next.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
}

impl<T: IdGen + ?Sized> IdGen for Arc<T> {
    fn next_id(&self) -> u64 {
        (**self).next_id()
    }
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

impl<T: Vfs + ?Sized> Vfs for Arc<T> {
    fn open(
        &self,
        path: &std::path::Path,
        opts: &VfsOpenOptions,
    ) -> Result<Box<dyn VfsFile>, crate::error::ErasureError> {
        (**self).open(path, opts)
    }

    fn read_at(
        &self,
        path: &std::path::Path,
        offset: u64,
        buf: &mut [u8],
    ) -> Result<usize, crate::error::ErasureError> {
        (**self).read_at(path, offset, buf)
    }

    fn read(&self, path: &std::path::Path) -> Result<Vec<u8>, crate::error::ErasureError> {
        (**self).read(path)
    }

    fn write_atomic(
        &self,
        path: &std::path::Path,
        data: &[u8],
    ) -> Result<(), crate::error::ErasureError> {
        (**self).write_atomic(path, data)
    }

    fn create_dir_all(&self, path: &std::path::Path) -> Result<(), crate::error::ErasureError> {
        (**self).create_dir_all(path)
    }

    fn remove_file(&self, path: &std::path::Path) -> Result<(), crate::error::ErasureError> {
        (**self).remove_file(path)
    }

    fn exists(&self, path: &std::path::Path) -> Result<bool, crate::error::ErasureError> {
        (**self).exists(path)
    }

    fn len(&self, path: &std::path::Path) -> Result<u64, crate::error::ErasureError> {
        (**self).len(path)
    }

    fn list_dir(&self, path: &std::path::Path) -> Result<Vec<String>, crate::error::ErasureError> {
        (**self).list_dir(path)
    }

    fn sync_dir(&self, path: &std::path::Path) -> Result<(), crate::error::ErasureError> {
        (**self).sync_dir(path)
    }
}

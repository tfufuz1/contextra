//! Kanonische Datentypen der Löschkette (Ring 0).
//!
//! Dieses Modul hält alle Datentypen, die von mehr als einem Crate der Löschkette
//! gebraucht werden: Auftragsidentität (`TargetKind`), Ziel (`EraseTarget`),
//! Politik (`ErasureReason`, `ErasureProfile`, `ErasurePolicy`, ...),
//! Garantiemodell (`Guarantee`, `ErasureException`), Zustandsautomat (`ErasureState`,
//! `ErasureStatus`, `LayerStatus`, `LayerState`), Fortschritt (`Cursor`, `LayerProgress`)
//! und Fehler (`ErasureError`, `LayerErrorKind`).
//!
//! Rein deterministisch ohne I/O, ohne Zeit und ohne Zufall.

use core::fmt;
use serde::{Deserialize, Serialize};

use crate::error::ContextraError;
use crate::ids::ErasureId;
use crate::types::TenantId;

/// Maximale UTF-8-Bytelänge eines Collection-Namens in [`EraseTarget`].
const MAX_COLLECTION_LEN: usize = 255;

/// Maximale UTF-8-Bytelänge eines Dokumentenschlüssels in [`EraseTarget`].
const MAX_KEY_LEN: usize = 4096;

/// Zieltyp eines Löschauftrags zur Klassifikation der Hierarchieebene.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TargetKind {
    /// Löschung eines einzelnen Dokuments.
    Document,
    /// Betroffenen-Löschung (alle Dokumente eines Subjekts, ADR-E).
    Subject,
    /// Löschung einer gesamten Collection.
    Collection,
    /// Löschung eines gesamten Mandanten.
    Tenant,
    /// Ersetzung eines Dokuments (`replace_erasing`, D-115).
    DocumentReplace,
    /// Verdrängte Dokumentversion (`update_with(..., Purge)`, D-117).
    Superseded,
}

impl fmt::Display for TargetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Document => write!(f, "Document"),
            Self::Subject => write!(f, "Subject"),
            Self::Collection => write!(f, "Collection"),
            Self::Tenant => write!(f, "Tenant"),
            Self::DocumentReplace => write!(f, "DocumentReplace"),
            Self::Superseded => write!(f, "Superseded"),
        }
    }
}

/// Zielspezifikation eines Löschauftrags.
///
/// Implementiert manuelles `Debug` ohne Namens- oder Schlüsselinhalt (`[REDACTED]`).
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EraseTarget {
    /// Einzelnes Dokument innerhalb einer Collection eines Mandanten.
    Document {
        /// Mandanten-ID.
        tenant: TenantId,
        /// Name der Collection.
        collection: String,
        /// Dokumentenschlüssel.
        key: String,
    },
    /// Gesamte Collection eines Mandanten.
    Collection {
        /// Mandanten-ID.
        tenant: TenantId,
        /// Name der Collection.
        collection: String,
    },
    /// Gesamter Mandant.
    Tenant {
        /// Mandanten-ID.
        tenant: TenantId,
    },
}

impl fmt::Debug for EraseTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Document { tenant, .. } => f
                .debug_struct("EraseTarget::Document")
                .field("tenant", &tenant.as_u64())
                .field("collection", &"[REDACTED]")
                .field("key", &"[REDACTED]")
                .finish(),
            Self::Collection { tenant, .. } => f
                .debug_struct("EraseTarget::Collection")
                .field("tenant", &tenant.as_u64())
                .field("collection", &"[REDACTED]")
                .finish(),
            Self::Tenant { tenant } => f
                .debug_struct("EraseTarget::Tenant")
                .field("tenant", &tenant.as_u64())
                .finish(),
        }
    }
}

fn validate_collection_name(collection: &str) -> Result<(), ErasureError> {
    if collection.is_empty() {
        return Err(ErasureError::InvalidTarget(
            "collection name must not be empty",
        ));
    }
    if collection.len() > MAX_COLLECTION_LEN {
        return Err(ErasureError::InvalidTarget(
            "collection name exceeds 255 bytes",
        ));
    }
    if collection.contains('\0') {
        return Err(ErasureError::InvalidTarget(
            "collection name must not contain NUL byte",
        ));
    }
    if collection.starts_with("__") {
        return Err(ErasureError::InvalidTarget(
            "collection name must not use reserved '__' prefix",
        ));
    }
    Ok(())
}

fn validate_doc_key(key: &str) -> Result<(), ErasureError> {
    if key.is_empty() {
        return Err(ErasureError::InvalidTarget(
            "document key must not be empty",
        ));
    }
    if key.len() > MAX_KEY_LEN {
        return Err(ErasureError::InvalidTarget(
            "document key exceeds 4096 bytes",
        ));
    }
    if key.contains('\0') {
        return Err(ErasureError::InvalidTarget(
            "document key must not contain NUL byte",
        ));
    }
    Ok(())
}

fn push_len_prefixed_str(buf: &mut Vec<u8>, s: &str) {
    let len = u32::try_from(s.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(s.as_bytes());
}

impl EraseTarget {
    /// Liefert die zugehörige [`TargetKind`]-Variante.
    pub fn kind(&self) -> TargetKind {
        match self {
            Self::Document { .. } => TargetKind::Document,
            Self::Collection { .. } => TargetKind::Collection,
            Self::Tenant { .. } => TargetKind::Tenant,
        }
    }

    /// Liefert die Mandanten-ID des Ziels.
    pub fn tenant(&self) -> TenantId {
        match self {
            Self::Document { tenant, .. }
            | Self::Collection { tenant, .. }
            | Self::Tenant { tenant } => *tenant,
        }
    }

    /// Führt eine reine Syntaxprüfung des Ziels ohne I/O durch.
    ///
    /// # Fehler
    /// Liefert [`ErasureError::InvalidTarget`], wenn Collection-Name oder Schlüssel leer,
    /// zu lang, mit NUL-Byte behaftet, reserviert (`__`-Präfix) oder die `default`-Collection
    /// als Ganzes gelöscht werden soll.
    ///
    /// # Crash
    /// Reine Validierung im Arbeitsspeicher; panikfrei, keine Seiteneffekte.
    pub fn validate(&self) -> Result<(), ErasureError> {
        match self {
            Self::Document {
                collection, key, ..
            } => {
                validate_collection_name(collection)?;
                validate_doc_key(key)?;
                Ok(())
            }
            Self::Collection { collection, .. } => {
                validate_collection_name(collection)?;
                if collection == "default" {
                    return Err(ErasureError::InvalidTarget(
                        "default collection cannot be erased as collection",
                    ));
                }
                Ok(())
            }
            Self::Tenant { .. } => Ok(()),
        }
    }

    /// Erzeugt eine kanonische, injektive Byte-Repräsentation für Journal und Ledger.
    ///
    /// Format: `[tag: u8][tenant: u64 LE][...]` mit längenpräfigierten UTF-8-Strings
    /// `[len: u32 LE][utf8 bytes]`.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        match self {
            Self::Document {
                tenant,
                collection,
                key,
            } => {
                buf.reserve(1 + 8 + 4 + collection.len() + 4 + key.len());
                buf.push(1);
                buf.extend_from_slice(&tenant.as_u64().to_le_bytes());
                push_len_prefixed_str(&mut buf, collection);
                push_len_prefixed_str(&mut buf, key);
            }
            Self::Collection { tenant, collection } => {
                buf.reserve(1 + 8 + 4 + collection.len());
                buf.push(2);
                buf.extend_from_slice(&tenant.as_u64().to_le_bytes());
                push_len_prefixed_str(&mut buf, collection);
            }
            Self::Tenant { tenant } => {
                buf.reserve(1 + 8);
                buf.push(3);
                buf.extend_from_slice(&tenant.as_u64().to_le_bytes());
            }
        }
        buf
    }

    /// Berechnet den keyed BLAKE3-Hash über [`Self::canonical_bytes`].
    ///
    /// `key` ist der 32-Byte-Log-Schlüssel.
    pub fn target_hash(&self, key: &[u8; 32]) -> [u8; 32] {
        let canonical = self.canonical_bytes();
        *blake3::keyed_hash(key, &canonical).as_bytes()
    }
}

/// Strukturierter Löschgrund ohne Freitext (Vermeidung personenbezogener Daten im Ledger).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
#[non_exhaustive]
pub enum ErasureReason {
    /// Löschung auf Nutzeranfrage.
    UserRequest,
    /// Löschung aufgrund gesetzlicher Anordnung.
    LegalRequest,
    /// Ablauf der Aufbewahrungsfrist.
    RetentionExpiry,
    /// Manuelle Betreiberaktion.
    OperatorAction,
    /// Offboarding eines gesamten Mandanten.
    TenantOffboarding,
}

/// Konfigurationsprofil für das Garantie- und Betriebsverhalten einer Löschung.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Default)]
pub enum ErasureProfile {
    /// Standardprofil.
    #[default]
    Standard,
    /// Reguliertes Profil mit verschärften Nachweis- und Zeitvorgaben.
    Regulated,
}

/// Richtlinie zur Behandlung erneuter Einfügungen unter demselben Schlüssel.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Default)]
pub enum ReinsertPolicy {
    /// Erneutes Einfügen wird abgelehnt.
    #[default]
    Deny,
    /// Erneutes Einfügen ist erst nach Erreichen von `Sealed` zulässig.
    AllowAfterSealed,
}

/// Richtlinie für Checkpoints, die von einer Löschung betroffen sind.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Default)]
pub enum CheckpointTaintPolicy {
    /// Betroffene Checkpoints verwerfen.
    #[default]
    Drop,
    /// Betroffene Checkpoints ohne die gelöschten Daten neu schreiben.
    Rewrite,
    /// Checkpoint unter Legal-Hold belassen.
    LegalHold,
}

/// Erwartete Synchronisationsstufe beim Warten auf einen Löschauftrag.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Default)]
pub enum AwaitLevel {
    /// Auftrag wurde akzeptiert und geplant.
    Accepted,
    /// Read-Gate ist aktiv (G1).
    #[default]
    Gated,
    /// Physische Bereinigung und Verifikation abgeschlossen (G3).
    Purged,
}

/// Richtlinienparameter für einen Löschauftrag.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Default)]
pub struct ErasurePolicy {
    /// Umgang mit betroffenen Checkpoints.
    pub checkpoint: CheckpointTaintPolicy,
    /// Umgang mit erneuten Schreibzugriffen.
    pub reinsert: ReinsertPolicy,
    /// SLA-Frist in Sekunden (`0` = Defaultwert des Workers).
    pub sla_secs: u64,
    /// Erlaubt Fortsetzung bei nicht verfügbarem G2 (nur Stufe 2).
    pub allow_g2_unavailable: bool,
}

/// Erreichte Garantien eines Löschauftrags.
///
/// Invariante: `purged ⇒ gated` und `crypto_erased ⇒ gated`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, Default)]
pub struct Guarantee {
    /// G1: Lesezugriff durch aktives Gate blockiert.
    pub gated: bool,
    /// G3: Physische Bereinigung und Verifikation abgeschlossen.
    pub purged: bool,
    /// G2: Kryptografische Schlüsselvernichtung abgeschlossen.
    pub crypto_erased: bool,
}

impl Guarantee {
    /// Keine Garantie erreicht (`G0`).
    pub const NONE: Self = Self {
        gated: false,
        purged: false,
        crypto_erased: false,
    };

    /// Erzeugt eine geprüfte [`Guarantee`]-Instanz unter Wahrung der Invariante
    /// `purged ⇒ gated` und `crypto_erased ⇒ gated`.
    ///
    /// # Fehler
    /// Liefert [`ErasureError::InvalidTransition`], wenn `purged` oder `crypto_erased`
    /// ohne `gated == true` gesetzt ist.
    ///
    /// # Crash
    /// Reine Konstruktion ohne Seiteneffekte.
    pub const fn new(
        gated: bool,
        purged: bool,
        crypto_erased: bool,
    ) -> Result<Self, ErasureError> {
        if (purged || crypto_erased) && !gated {
            return Err(ErasureError::InvalidTransition);
        }
        Ok(Self {
            gated,
            purged,
            crypto_erased,
        })
    }

    /// Schnitt zweier Garantien: eine Garantie gilt nur, wenn sie in beiden gilt.
    pub const fn meet(self, o: Self) -> Self {
        Self {
            gated: self.gated && o.gated,
            purged: self.purged && o.purged,
            crypto_erased: self.crypto_erased && o.crypto_erased,
        }
    }

    /// Liefert das kanonische Garantie-Label (`"G0"`, `"G1"`, `"G1+G3"`, `"G1+G2"`, `"G1+G2+G3"`).
    pub fn label(&self) -> &'static str {
        if !self.gated {
            return "G0";
        }
        match (self.purged, self.crypto_erased) {
            (false, false) => "G1",
            (true, false) => "G1+G3",
            (false, true) => "G1+G2",
            (true, true) => "G1+G2+G3",
        }
    }
}

/// Dokumentierte Ausnahme oder Einschränkung einer Löschgarantie.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[non_exhaustive]
pub enum ErasureException {
    /// Externe Kopien (Backups, Exporte, Replikate, Client-Caches, LLM-Anbieter) außerhalb des Scopes.
    OutOfScopeExternalCopies,
    /// Checkpoint steht unter Legal-Hold.
    LegalHold {
        /// Bezeichner des betroffenen Checkpoints.
        checkpoint: String,
    },
    /// Layer wurde nicht vermessen.
    NotMeasured {
        /// Name des Layers.
        layer: String,
    },
    /// Feature in einem Layer deaktiviert.
    FeatureDisabled {
        /// Name des Layers.
        layer: String,
    },
    /// Altbestand ohne Herkunftsnachweis (`derived_from`).
    Unprovenanced {
        /// Anzahl betroffener Objekte.
        objects: u64,
    },
    /// Überlappender Snapshot pinnt noch Sequenzen (G3 verzögert, I-11).
    OverlappingSnapshotPinned {
        /// Minimale gepinnte Sequenznummer.
        min_seq: u64,
    },
}

/// Zustandsautomat eines Löschauftrags.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
#[non_exhaustive]
pub enum ErasureState {
    /// Auftrag eingegangen.
    Requested,
    /// Löschplan durabel erstellt.
    Planned,
    /// Gate wird aufgebaut.
    Gating,
    /// Gate ist aktiv und durabel markiert.
    Gated,
    /// Kryptografische Schlüssel werden vernichtet (Stufe 2).
    KeysDestroying,
    /// Physische Bereinigung der Layer läuft.
    Purging,
    /// Verifikation der Bereinigung läuft.
    Verifying,
    /// Alle Layer bereinigt und verifiziert, Gate wartet auf Ablauf der Lebensdauer.
    Sealed,
    /// Auftrag vollständig abgeschlossen (Endzustand).
    Completed,
    /// Fehlerzustand mit optionalem Layer und Versuchszähler.
    Failed {
        /// Betroffener Layer (falls bekannt).
        layer: Option<String>,
        /// Aktuelle Versuchsnummer.
        attempt: u32,
    },
    /// Layer wurde in Quarantäne verschoben und wartet auf Wiederholung oder Eingriff.
    Quarantined {
        /// Name des betroffenen Layers.
        layer: String,
    },
}

impl ErasureState {
    /// Prüft gemäß Übergangsmatrix (§5), ob der Übergang nach `next` erlaubt ist.
    ///
    /// Reine Funktion ohne I/O.
    pub fn can_transition_to(&self, next: &ErasureState) -> bool {
        matches!(
            (self, next),
            (Self::Requested, Self::Planned | Self::Failed { .. })
                | (Self::Planned, Self::Gating | Self::Failed { .. })
                | (Self::Gating, Self::Gated | Self::Failed { .. })
                | (Self::Gated, Self::KeysDestroying | Self::Purging)
                | (Self::KeysDestroying, Self::Purging)
                | (Self::Purging, Self::Verifying | Self::Failed { .. })
                | (
                    Self::Verifying,
                    Self::Purging | Self::Sealed | Self::Failed { .. }
                )
                | (Self::Sealed, Self::Completed)
                | (
                    Self::Failed { .. },
                    Self::Purging | Self::Verifying | Self::Quarantined { .. }
                )
                | (Self::Quarantined { .. }, Self::Purging)
        )
    }

    /// Liefert die durch diesen Zustand mindestens erreichte [`Guarantee`].
    pub fn reached(&self) -> Guarantee {
        match self {
            Self::Requested | Self::Planned | Self::Gating | Self::Failed { .. } => Guarantee::NONE,
            Self::Gated
            | Self::KeysDestroying
            | Self::Purging
            | Self::Verifying
            | Self::Quarantined { .. } => Guarantee {
                gated: true,
                purged: false,
                crypto_erased: false,
            },
            Self::Sealed | Self::Completed => Guarantee {
                gated: true,
                purged: true,
                crypto_erased: false,
            },
        }
    }

    /// Liefert `true`, wenn der Zustand ein Endzustand ist (nur `Completed`).
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed)
    }

    /// Liefert `true`, wenn das Read-Gate in diesem Zustand aktiv ist.
    pub fn gate_active(&self) -> bool {
        matches!(
            self,
            Self::Gating
                | Self::Gated
                | Self::KeysDestroying
                | Self::Purging
                | Self::Verifying
                | Self::Sealed
                | Self::Quarantined { .. }
        )
    }
}

/// Zustand eines einzelnen Layers innerhalb eines Löschauftrags.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum LayerState {
    /// Ausstehend.
    Pending,
    /// Gate im Layer aktiv.
    Gated,
    /// Physische Bereinigung läuft.
    Purging,
    /// Physische Bereinigung abgeschlossen.
    Purged,
    /// Verifikation abgeschlossen.
    Verified,
    /// Fehlgeschlagen.
    Failed,
    /// In Quarantäne.
    Quarantined,
}

/// Status eines einzelnen Layers in [`ErasureStatus`].
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct LayerStatus {
    /// Logischer Name des Layers.
    pub layer: String,
    /// Aktueller Zustand des Layers.
    pub state: LayerState,
    /// Aktuelle Versuchsnummer.
    pub attempt: u32,
    /// Optional verbleibende Einheiten.
    pub remaining: Option<u64>,
}

/// Öffentlicher Statusbericht eines Löschauftrags (ohne Schlüssel, Namen oder Terme).
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct ErasureStatus {
    /// Auftrags-ID.
    pub id: ErasureId,
    /// Aktueller Zustand im Zustandsautomaten.
    pub state: ErasureState,
    /// Erreichte Garantie.
    pub reached: Guarantee,
    /// Status der beteiligten Layer.
    pub layers: Vec<LayerStatus>,
    /// Dokumentierte Ausnahmen.
    pub exceptions: Vec<ErasureException>,
    /// Erstellungszeitpunkt in Unix-Nanosekunden.
    pub requested_at: u64,
    /// Zeitpunkt der Gate-Aktivierung in Unix-Nanosekunden.
    pub gated_at: Option<u64>,
    /// Zeitpunkt der Versiegelung in Unix-Nanosekunden.
    pub sealed_at: Option<u64>,
    /// Optionaler Index im Audit-Ledger.
    pub ledger_index: Option<u64>,
}

/// Undurchsichtiger Wiederaufnahmepunkt eines Layers. Maximal 256 Byte (Journal-Limit).
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Default)]
pub struct Cursor(Vec<u8>);

impl Cursor {
    /// Maximale Bytelänge eines Cursors im Journal.
    pub const MAX_LEN: usize = 256;

    /// Erzeugt einen geprüften [`Cursor`].
    ///
    /// # Fehler
    /// Liefert [`ErasureError::InvalidCursor`], wenn `b.len() > Self::MAX_LEN`.
    ///
    /// # Crash
    /// Reine Längenprüfung im Arbeitsspeicher; panikfrei.
    pub fn new(b: Vec<u8>) -> Result<Self, ErasureError> {
        if b.len() > Self::MAX_LEN {
            return Err(ErasureError::InvalidCursor);
        }
        Ok(Self(b))
    }

    /// Liefert die inneren Bytes als Slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Fortschrittsbericht eines Layers während der Bereinigung.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct LayerProgress {
    /// Wiederaufnahmepunkt (`None` = Layer abgeschlossen).
    pub cursor: Option<Cursor>,
    /// Anzahl bearbeiteter Einheiten.
    pub units_done: u64,
    /// Anzahl freigegebener Bytes.
    pub bytes_freed: u64,
}

/// Fehlerklassifikation für die Retry-Politik von Layern.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum LayerErrorKind {
    /// Vorübergehender Fehler; erneuter Versuch sinnvoll.
    Retryable,
    /// Dauerhafter Fehler.
    Permanent,
    /// Journal oder Zustand vergiftet.
    Poisoned,
    /// Mandanten- oder Scope-Verletzung.
    ScopeViolation,
    /// Operation abgebrochen.
    Cancelled,
}

/// Fehlertyp der Löschkette.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ErasureError {
    /// Ungültiges Löschziel.
    #[error("invalid erase target: {0}")]
    InvalidTarget(&'static str),
    /// Ungültiger Cursor (z. B. > 256 Byte).
    #[error("invalid cursor")]
    InvalidCursor,
    /// Idempotenzkonflikt durch Wiederverwendung derselben ID mit anderem Plan.
    #[error("erasure id reused with different plan")]
    IdempotencyConflict,
    /// Mehrdeutiger Scope durch Schlüsselkollision.
    #[error("scope ambiguous (key collision)")]
    ScopeAmbiguous,
    /// Verletzung der Mandantenisolation.
    #[error("tenant isolation violated")]
    TenantViolation,
    /// Unbekannte Lösch-ID.
    #[error("unknown erasure id")]
    NotFound,
    /// Ungültiger Zustandsübergang.
    #[error("invalid state transition")]
    InvalidTransition,
    /// Löschjournal ist vergiftet.
    #[error("erasure journal poisoned")]
    JournalPoisoned,
    /// Löschledger ist korrupt.
    #[error("erasure ledger corrupt")]
    LedgerCorrupt,
    /// Zu viele aktive Löschaufträge.
    #[error("too many active erasure jobs ({active})")]
    Busy {
        /// Anzahl aktiver Aufträge.
        active: u32,
    },
    /// Unzureichender Speicherplatz.
    #[error("insufficient space: need {needed}, have {available}")]
    InsufficientSpace {
        /// Benötigte Bytes.
        needed: u64,
        /// Verfügbare Bytes.
        available: u64,
    },
    /// Fehler in einem spezifischen Layer.
    #[error("layer {layer} failed ({kind:?})")]
    Layer {
        /// Name des Layers.
        layer: String,
        /// Fehlerklassifikation.
        kind: LayerErrorKind,
    },
    /// Auftrag wurde abgebrochen.
    #[error("cancelled")]
    Cancelled,
    /// I/O-Fehler ohne Pfad- oder Schlüsselbytes.
    #[error("io: {0}")]
    Io(String),
}

impl ErasureError {
    /// Ordnet den Fehler einer [`LayerErrorKind`]-Kategorie für die Retry-Politik zu.
    pub fn kind(&self) -> LayerErrorKind {
        match self {
            Self::Io(_) | Self::Busy { .. } | Self::InsufficientSpace { .. } => {
                LayerErrorKind::Retryable
            }
            Self::JournalPoisoned => LayerErrorKind::Poisoned,
            Self::TenantViolation => LayerErrorKind::ScopeViolation,
            Self::Cancelled => LayerErrorKind::Cancelled,
            Self::InvalidTarget(_)
            | Self::InvalidCursor
            | Self::IdempotencyConflict
            | Self::ScopeAmbiguous
            | Self::NotFound
            | Self::InvalidTransition
            | Self::LedgerCorrupt
            | Self::Layer { .. } => LayerErrorKind::Permanent,
        }
    }
}

impl From<ErasureError> for ContextraError {
    fn from(err: ErasureError) -> Self {
        match err {
            ErasureError::Busy { .. } => {
                ContextraError::Storage("erasure worker busy".to_string())
            }
            ErasureError::Io(_) => ContextraError::Storage("erasure io error".to_string()),
            ErasureError::InsufficientSpace { .. } => {
                ContextraError::Storage("erasure insufficient space".to_string())
            }
            ErasureError::InvalidTarget(reason) => ContextraError::InvalidInput(reason.to_string()),
            ErasureError::InvalidCursor => {
                ContextraError::InvalidInput("invalid erasure cursor".to_string())
            }
            ErasureError::JournalPoisoned => {
                ContextraError::Storage("erasure journal poisoned".to_string())
            }
            ErasureError::LedgerCorrupt => {
                ContextraError::Storage("erasure ledger corrupt".to_string())
            }
            ErasureError::IdempotencyConflict => {
                ContextraError::Internal("erasure idempotency conflict".to_string())
            }
            ErasureError::ScopeAmbiguous => {
                ContextraError::Internal("erasure scope ambiguous".to_string())
            }
            ErasureError::TenantViolation => {
                ContextraError::Internal("erasure tenant violation".to_string())
            }
            ErasureError::NotFound => ContextraError::Internal("erasure not found".to_string()),
            ErasureError::InvalidTransition => {
                ContextraError::Internal("erasure invalid state transition".to_string())
            }
            ErasureError::Layer { .. } => {
                ContextraError::Internal("erasure layer failed".to_string())
            }
            ErasureError::Cancelled => ContextraError::Internal("erasure cancelled".to_string()),
        }
    }
}

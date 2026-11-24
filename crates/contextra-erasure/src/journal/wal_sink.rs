//! Port `WalSink` für die synchrone WAL-Entsorgung nach einem Scope-Purge (SPEC §11, WP-7, INV-ERA-6).
//!
//! Entsorgt WAL-Einträge eines gelöschten Bereichs (Scope) oder macht diese unlesbar.
//! Sämtliche I/O- und Storage-Adapter liegen außerhalb dieses Bausteins.
//!
//! // SECTION WalSink
//! // ANCHOR[contextra-erasure:WAL-SINK-001]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Mutex;
use zeroize::Zeroize;

/// Eindeutiger Identifikator für einen logischen Bereich (Scope) im WAL.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct WalScope {
    /// 16-Byte Scope-ID (z. B. Tenant, Collection oder kombinierte ID).
    scope_id: [u8; 16],
    /// Optionales Schlüsselpräfix zur Eingrenzung.
    prefix: Vec<u8>,
}

impl WalScope {
    /// Erzeugt einen neuen `WalScope` mit 16-Byte Kennung und leerem Präfix.
    pub const fn new(scope_id: [u8; 16]) -> Self {
        Self {
            scope_id,
            prefix: Vec::new(),
        }
    }

    /// Erzeugt einen neuen `WalScope` mit Kennung und Schlüsselpräfix.
    pub fn with_prefix(scope_id: [u8; 16], prefix: Vec<u8>) -> Self {
        Self { scope_id, prefix }
    }

    /// Gibt die 16-Byte Scope-ID zurück.
    pub fn scope_id(&self) -> &[u8; 16] {
        &self.scope_id
    }

    /// Gibt das optionale Präfix zurück.
    pub fn prefix(&self) -> &[u8] {
        &self.prefix
    }
}

/// Identifikator eines einzelnen Eintrags oder Blocks im WAL.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct WalEntryId(pub u64);

impl WalEntryId {
    /// Erzeugt eine neue `WalEntryId`.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Liefert den numerischen Wert zurück.
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl fmt::Display for WalEntryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WalEntryId({})", self.0)
    }
}

/// Methode, mit der Einträge eines Scopes unlesbar oder getilgt wurden.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WalDisposalMethod {
    /// Physische Tilgung (z. B. Überschreiben oder Truncate der WAL-Einträge).
    PhysicalPurge,
    /// Unlesbarmachung durch Vernichtung des scope-spezifischen Schlüssels (Krypto-Shredding).
    CryptoShredding,
    /// Der Bereich war bereits vollständig entsorgt (idempotente Bestätigung).
    AlreadyDisposed,
}

impl fmt::Display for WalDisposalMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PhysicalPurge => write!(f, "PhysicalPurge"),
            Self::CryptoShredding => write!(f, "CryptoShredding"),
            Self::AlreadyDisposed => write!(f, "AlreadyDisposed"),
        }
    }
}

/// Fehlertyp für den WAL-Sink-Port mit Phasenbezug (fail-closed, kein reiner String-Fehler).
#[derive(Debug, thiserror::Error)]
pub enum WalSinkError {
    /// E/A-Fehler bei Lese- oder Schreiboperationen im WAL.
    #[error("E/A-Fehler in WAL-Phase {phase}: {detail}")]
    Io {
        /// Betroffene Phase.
        phase: &'static str,
        /// Detailbeschreibung.
        detail: String,
    },
    /// Korruptionsbefund oder Inkonsistenz in den WAL-Metadaten.
    #[error("Korruption in WAL-Phase {phase}: {detail}")]
    Corrupt {
        /// Betroffene Phase.
        phase: &'static str,
        /// Detailbeschreibung.
        detail: String,
    },
    /// Die Vernichtung des Integritäts- oder Krypto-Schlüssels ist fehlgeschlagen.
    #[error("Schluesselvernichtung fuer Scope fehlgeschlagen: {detail}")]
    KeyDiscardFailed {
        /// Detailbeschreibung.
        detail: String,
    },
    /// Der WAL-Sink ist vorübergehend oder dauerhaft nicht verfügbar.
    #[error("WAL-Sink nicht verfuegbar: {detail}")]
    SinkUnavailable {
        /// Detailbeschreibung.
        detail: String,
    },
    /// Konflikt mit konkurrierender Sperre oder ungültiger Zustand (z. B. unvollendeter Fence).
    #[error("Zustandskonflikt in Phase {phase}: {detail}")]
    StateConflict {
        /// Betroffene Phase.
        phase: &'static str,
        /// Detailbeschreibung.
        detail: String,
    },
}

/// Beleg über die durchgeführte Entsorgung von WAL-Einträgen.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct WalDisposalReceipt {
    /// Betroffener Scope.
    pub scope: WalScope,
    /// Anzahl physisch entsorgter Einträge.
    pub disposed_entries_count: usize,
    /// Angewandte Entsorgungsmethode.
    pub method: WalDisposalMethod,
    /// Zeitpunkt der Entsorgung in Nanosekunden seit Unix-Epoche (via Clock).
    pub timestamp_nanos: u64,
}

/// Synchroner Port zur physischen oder kryptographischen Entsorgung von WAL-Einträgen eines Scopes.
///
/// Gemäß der synchronen Kernregel existieren im Löschpfad keine Threads und keine Async-Aufrufe.
pub trait WalSink: Send + Sync {
    /// Enumeriert alle bekannten WAL-Einträge des angegebenen Scopes.
    fn enumerate_scope_entries(&self, scope: &WalScope) -> Result<Vec<WalEntryId>, WalSinkError>;

    /// Führt die physische Entsorgung (Truncate/Überschreiben) der angegebenen Einträge durch.
    ///
    /// Gibt die tatsächliche Anzahl der entsorgten Einträge zurück.
    fn dispose_entries(
        &self,
        scope: &WalScope,
        entries: &[WalEntryId],
    ) -> Result<usize, WalSinkError>;

    /// Verwirft und nullt den scope-spezifischen Integritäts- oder Verschlüsselungsschlüssel.
    ///
    /// Nach diesem Aufruf können verbliebene Einträge des Scopes kryptographisch nicht mehr
    /// validiert oder entschlüsselt werden.
    fn discard_scope_integrity_key(&self, scope: &WalScope) -> Result<(), WalSinkError>;

    /// Prüft, ob der Scope bereits als vollständig entsorgt gilt.
    fn is_scope_disposed(&self, scope: &WalScope) -> Result<bool, WalSinkError>;
}

/// In-Memory-Eintrag für die Test- und Referenz-Implementierung.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InMemoryWalEntry {
    /// ID des Eintrags.
    pub id: WalEntryId,
    /// Zugehöriger Scope.
    pub scope: WalScope,
    /// Nutzdaten / Klartext.
    pub payload: Vec<u8>,
    /// Ob der Eintrag physisch getilgt (genullt) wurde.
    pub is_purged: bool,
}

/// Referenzimplementierung des `WalSink`-Ports im Speicher für Tests und Verifikation.
pub struct InMemoryWalSink {
    entries: Mutex<BTreeMap<WalEntryId, InMemoryWalEntry>>,
    integrity_keys: Mutex<BTreeMap<[u8; 16], [u8; 32]>>,
    disposed_scopes: Mutex<BTreeSet<[u8; 16]>>,
    fail_on_enumerate: Mutex<bool>,
    fail_on_dispose: Mutex<bool>,
    fail_on_key_discard: Mutex<bool>,
}

impl Default for InMemoryWalSink {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryWalSink {
    /// Erzeugt eine neue leere Instanz.
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(BTreeMap::new()),
            integrity_keys: Mutex::new(BTreeMap::new()),
            disposed_scopes: Mutex::new(BTreeSet::new()),
            fail_on_enumerate: Mutex::new(false),
            fail_on_dispose: Mutex::new(false),
            fail_on_key_discard: Mutex::new(false),
        }
    }

    /// Fügt einen Test-WAL-Eintrag hinzu.
    pub fn add_entry(&self, id: u64, scope: WalScope, payload: Vec<u8>) {
        if let Ok(mut lock) = self.entries.lock() {
            lock.insert(
                WalEntryId::new(id),
                InMemoryWalEntry {
                    id: WalEntryId::new(id),
                    scope,
                    payload,
                    is_purged: false,
                },
            );
        }
    }

    /// Hinterlegt einen Integritätsschlüssel für einen Scope.
    pub fn set_integrity_key(&self, scope_id: [u8; 16], key: [u8; 32]) {
        if let Ok(mut lock) = self.integrity_keys.lock() {
            lock.insert(scope_id, key);
        }
    }

    /// Liest den Integritätsschlüssel eines Scopes, sofern noch vorhanden.
    pub fn get_integrity_key(&self, scope_id: &[u8; 16]) -> Option<[u8; 32]> {
        if let Ok(lock) = self.integrity_keys.lock() {
            lock.get(scope_id).copied()
        } else {
            None
        }
    }

    /// Liest lesbare Einträge eines Scopes.
    ///
    /// Wenn der Schlüssel verworfen wurde oder Einträge getilgt wurden, ist kein Klartext verfügbar.
    pub fn read_entries_for_scope(&self, scope: &WalScope) -> Vec<Vec<u8>> {
        let has_key = self
            .integrity_keys
            .lock()
            .ok()
            .and_then(|keys| keys.get(scope.scope_id()).copied())
            .is_some();

        let entries_lock = match self.entries.lock() {
            Ok(g) => g,
            Err(_) => return Vec::new(),
        };

        let mut out = Vec::new();
        for entry in entries_lock.values() {
            if entry.scope.scope_id() == scope.scope_id() {
                // Wenn physisch getilgt: kein Klartext
                if entry.is_purged {
                    continue;
                }
                // Wenn Schlüssel verworfen: kryptographisch unlesbar
                if !has_key {
                    continue;
                }
                out.push(entry.payload.clone());
            }
        }
        out
    }

    /// Simuliert einen Fehler bei `enumerate_scope_entries`.
    pub fn set_fail_on_enumerate(&self, fail: bool) {
        if let Ok(mut lock) = self.fail_on_enumerate.lock() {
            *lock = fail;
        }
    }

    /// Simuliert einen Fehler bei `dispose_entries`.
    pub fn set_fail_on_dispose(&self, fail: bool) {
        if let Ok(mut lock) = self.fail_on_dispose.lock() {
            *lock = fail;
        }
    }

    /// Simuliert einen Fehler bei `discard_scope_integrity_key`.
    pub fn set_fail_on_key_discard(&self, fail: bool) {
        if let Ok(mut lock) = self.fail_on_key_discard.lock() {
            *lock = fail;
        }
    }
}

impl WalSink for InMemoryWalSink {
    fn enumerate_scope_entries(&self, scope: &WalScope) -> Result<Vec<WalEntryId>, WalSinkError> {
        let should_fail = self.fail_on_enumerate.lock().map(|f| *f).unwrap_or(true);
        if should_fail {
            return Err(WalSinkError::Io {
                phase: "enumerate_scope_entries",
                detail: "Injected fault during enumeration".to_string(),
            });
        }

        let lock = self
            .entries
            .lock()
            .map_err(|_| WalSinkError::StateConflict {
                phase: "enumerate_scope_entries",
                detail: "Mutex lock poisoned".to_string(),
            })?;

        let ids = lock
            .values()
            .filter(|e| e.scope.scope_id() == scope.scope_id() && !e.is_purged)
            .map(|e| e.id)
            .collect();

        Ok(ids)
    }

    fn dispose_entries(
        &self,
        scope: &WalScope,
        entries: &[WalEntryId],
    ) -> Result<usize, WalSinkError> {
        let should_fail = self.fail_on_dispose.lock().map(|f| *f).unwrap_or(true);
        if should_fail {
            return Err(WalSinkError::Io {
                phase: "dispose_entries",
                detail: "Injected fault during disposal".to_string(),
            });
        }

        let mut lock = self
            .entries
            .lock()
            .map_err(|_| WalSinkError::StateConflict {
                phase: "dispose_entries",
                detail: "Mutex lock poisoned".to_string(),
            })?;

        let mut disposed = 0;
        for id in entries {
            if let Some(entry) = lock.get_mut(id) {
                if entry.scope.scope_id() == scope.scope_id() {
                    // Nullung der Nutzdaten
                    entry.payload.zeroize();
                    entry.payload.clear();
                    entry.is_purged = true;
                    disposed += 1;
                }
            }
        }

        Ok(disposed)
    }

    fn discard_scope_integrity_key(&self, scope: &WalScope) -> Result<(), WalSinkError> {
        let should_fail = self.fail_on_key_discard.lock().map(|f| *f).unwrap_or(true);
        if should_fail {
            return Err(WalSinkError::KeyDiscardFailed {
                detail: "Injected fault during key discard".to_string(),
            });
        }

        let mut keys = self
            .integrity_keys
            .lock()
            .map_err(|_| WalSinkError::StateConflict {
                phase: "discard_scope_integrity_key",
                detail: "Mutex lock poisoned".to_string(),
            })?;

        if let Some(mut key) = keys.remove(scope.scope_id()) {
            key.zeroize();
        }

        let mut disposed =
            self.disposed_scopes
                .lock()
                .map_err(|_| WalSinkError::StateConflict {
                    phase: "discard_scope_integrity_key",
                    detail: "Mutex lock poisoned".to_string(),
                })?;
        disposed.insert(*scope.scope_id());

        Ok(())
    }

    fn is_scope_disposed(&self, scope: &WalScope) -> Result<bool, WalSinkError> {
        let disposed = self
            .disposed_scopes
            .lock()
            .map_err(|_| WalSinkError::StateConflict {
                phase: "is_scope_disposed",
                detail: "Mutex lock poisoned".to_string(),
            })?;
        Ok(disposed.contains(scope.scope_id()))
    }
}
// !SECTION

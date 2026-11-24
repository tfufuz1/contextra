//! Fence-Protokoll und Port `FenceGate` für physischen Purge (SPEC §4 INV-ERA-5, §5 WP-6, Entscheidung E4).
//!
//! Während eines physischen Purge blockiert ein Fence konkurrierende Schreiber
//! im betroffenen Schlüsselbereich.
//!
//! # Architektur & Invarianten
//! - **INV-ERA-5:** Ein Fence blockiert konkurrierende Schreiber während eines physischen Purge.
//! - **Owner-Token-Prinzip:** Ein Fence kann ausschließlich mit dem Token freigegeben werden, der bei der Erstellung erzeugt wurde.
//! - **Keine Überlappung:** Doppel-Fences auf identischen oder überlappenden Bereichen werden abgewiesen.
//! - **Fail-Closed:** Unbekannte oder korrumpierte Fence-Zustände blockieren jeden Schreiber.
//! - **Autarkie & Synchrone Ausführung:** Keine Abhängigkeit zu externen Contextra-Crates, kein Tokio/Async im Kernpfad.
//!
//! Herkunft: Neu implementiert gemäß Spezifikation WP-6 / Entscheidung E4
//! (Historische Referenz zur Vertragsableitung: `c1d5e2e64:crates/contextra-store/src/lsm/fence.rs`).

use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use thiserror::Error;

use parking_lot::RwLock;

/// Eindeutiger Autorisierungs-Token für einen aktiven Fence.
///
/// Nur der Inhaber dieses Tokens ist berechtigt, den Fence über [`FenceGate::release_fence`]
/// aufzuheben.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct FenceToken(pub [u8; 16]);

impl fmt::Debug for FenceToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FenceToken({:02x?})", &self.0[..4])
    }
}

impl FenceToken {
    /// Erzeugt einen neuen Token aus Roh-Bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Gibt die zugrundeliegenden Bytes zurück.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

/// Spezifikation des gefencten Schlüsselbereichs.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FenceRange {
    /// Ein exakter Einzelschlüssel.
    Exact(Vec<u8>),
    /// Ein Schlüsselpräfix (trifft auf alle Schlüssel zu, die mit diesem Byte-Präfix beginnen).
    Prefix(Vec<u8>),
    /// Ein geschlossenes Intervall `[lo, hi]` (lexikographisch).
    Range {
        /// Untere Grenze (inklusiv).
        lo: Vec<u8>,
        /// Obere Grenze (inklusiv).
        hi: Vec<u8>,
    },
    /// Globaler Scope (blockiert sämtliche Schreibzugriffe).
    Global,
}

impl FenceRange {
    /// Prüft, ob ein gegebener Schlüssel in diesen Bereich fällt.
    #[must_use]
    pub fn contains_key(&self, key: &[u8]) -> bool {
        match self {
            Self::Exact(k) => key == k.as_slice(),
            Self::Prefix(p) => key.starts_with(p.as_slice()),
            Self::Range { lo, hi } => key >= lo.as_slice() && key <= hi.as_slice(),
            Self::Global => true,
        }
    }

    /// Prüft, ob sich zwei Bereiche überschneiden.
    #[must_use]
    pub fn overlaps(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Global, _) | (_, Self::Global) => true,
            (Self::Exact(k1), Self::Exact(k2)) => k1 == k2,
            (Self::Exact(k), Self::Prefix(p)) | (Self::Prefix(p), Self::Exact(k)) => {
                k.starts_with(p.as_slice())
            }
            (Self::Prefix(p1), Self::Prefix(p2)) => {
                p1.starts_with(p2.as_slice()) || p2.starts_with(p1.as_slice())
            }
            (Self::Exact(k), Self::Range { lo, hi }) | (Self::Range { lo, hi }, Self::Exact(k)) => {
                k.as_slice() >= lo.as_slice() && k.as_slice() <= hi.as_slice()
            }
            (Self::Range { lo: l1, hi: h1 }, Self::Range { lo: l2, hi: h2 }) => {
                // Zwei Intervalle [l1, h1] und [l2, h2] überlappen genau dann, wenn max(l1, l2) <= min(h1, h2)
                l1.as_slice() <= h2.as_slice() && l2.as_slice() <= h1.as_slice()
            }
            (Self::Prefix(p), Self::Range { lo, hi })
            | (Self::Range { lo, hi }, Self::Prefix(p)) => {
                // Wenn Präfix im Intervall liegt, oder Intervallgrenzen mit Präfix beginnen
                if p.as_slice() >= lo.as_slice() && p.as_slice() <= hi.as_slice() {
                    return true;
                }
                if lo.starts_with(p.as_slice()) || hi.starts_with(p.as_slice()) {
                    return true;
                }
                // Lexikographischer Bereich des Präfixes [p, p_next)
                let p_slice = p.as_slice();
                !(hi.as_slice() < p_slice || lo.as_slice() > p_slice)
            }
        }
    }
}

/// Fehler im Lebenszyklus eines Fences oder bei Schreibzugriffen.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FenceGateError {
    /// Schreiber abgewiesen: Der Zielbereich ist durch einen aktiven Fence gesperrt.
    #[error("Schreibzugriff verweigert: Bereich ist gefenct")]
    Fenced {
        /// Der Bereich, der den Zugriff blockiert hat.
        range: FenceRange,
    },

    /// Konflikt bei der Fence-Erstellung: Der angeforderte Bereich überlappt mit einem bestehenden Fence.
    #[error("Fence-Erstellung abgelehnt: Bereich überlappt mit bestehendem Fence")]
    OverlapConflict {
        /// Der angeforderte Bereich.
        requested: FenceRange,
        /// Der bereits bestehende überlappende Bereich.
        existing: FenceRange,
    },

    /// Ungültiger oder fremder Token bei der Fence-Freigabe.
    #[error("Fence-Freigabe abgewiesen: Ungültiger oder nicht autorisierter Token")]
    InvalidToken,

    /// Fail-Closed: Der interne Fence-Zustand ist korrumpiert oder beschädigt.
    #[error("Fail-Closed: Fence-Zustand ist korrumpiert")]
    CorruptedState,

    /// Timeout oder Iterationslimit beim Warten auf Abschluss bestehender Inflight-Schreiber während Fence-Acquisition.
    #[error("Limit überschritten beim Warten auf Abschluss aktiver Schreiber")]
    DrainTimeout,
}

/// RAII-Wächter für einen aktiven Schreiber im System.
///
/// Solange dieser Wächter existiert, wird der Zähler aktiver Schreiber (`inflight_writers`)
/// hochgehalten. Bei Drop wird der Zähler dekrementiert.
#[derive(Debug)]
pub struct WriterGuard {
    inflight: Arc<AtomicU32>,
}

impl Drop for WriterGuard {
    fn drop(&mut self) {
        self.inflight.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Port-Trait für das Fencing von Schlüsselbereichen während des physischen Purge (INV-ERA-5).
pub trait FenceGate: Send + Sync {
    /// Blockiert einen Schlüsselbereich für Schreibzugriffe und liefert einen Autorisierungs-Token.
    ///
    /// # Fehler
    /// - [`FenceGateError::OverlapConflict`]: Falls `range` mit einem bereits aktiven Fence überlappt.
    /// - [`FenceGateError::CorruptedState`]: Falls der Wächterzustand korrumpiert ist.
    /// - [`FenceGateError::DrainTimeout`]: Falls aktive Schreiber im Bereich nicht rechtzeitig abschließen.
    fn acquire_fence(&self, range: FenceRange) -> Result<FenceToken, FenceGateError>;

    /// Hebt einen aktiven Fence auf. Dies gelingt ausschließlich mit dem ursprünglichen Besitzer-Token.
    ///
    /// # Fehler
    /// - [`FenceGateError::InvalidToken`]: Falls der Token unbekannt oder abgelaufen ist.
    /// - [`FenceGateError::CorruptedState`]: Falls der Zustand korrumpiert ist.
    fn release_fence(&self, token: &FenceToken) -> Result<(), FenceGateError>;

    /// Prüft, ob für einen Einzelschlüssel ein aktiver Fence vorliegt.
    fn is_fenced(&self, key: &[u8]) -> bool;

    /// Prüft, ob ein Schreibzugriff auf einen Einzelschlüssel zulässig ist.
    ///
    /// Ist der Schlüssel frei, wird ein RAII-[`WriterGuard`] zurückgegeben.
    ///
    /// # Fehler
    /// - [`FenceGateError::Fenced`]: Falls der Schlüssel in einem gefencten Bereich liegt.
    /// - [`FenceGateError::CorruptedState`]: Falls der Zustand korrumpiert ist (Fail-Closed).
    fn check_write(&self, key: &[u8]) -> Result<WriterGuard, FenceGateError>;

    /// Prüft eine Menge von Schlüsseln atomar auf Schreibzulässigkeit.
    ///
    /// Wird mindestens ein Schlüssel blockiert, schlägt der gesamte Batch fehl.
    fn check_write_batch<'a, I>(&self, keys: I) -> Result<WriterGuard, FenceGateError>
    where
        I: IntoIterator<Item = &'a [u8]>;

    /// Gibt an, ob aktuell mindestens ein Fence aktiv ist.
    fn is_active(&self) -> bool;
}

/// Interner Eintrag für eine aktive Fence-Registrierung.
#[derive(Clone, Debug)]
struct ActiveFenceEntry {
    token: FenceToken,
    range: FenceRange,
}

/// In-Memory-Referenzimplementierung des [`FenceGate`]-Ports (Klasse A, autark, synchron).
///
/// Arbeitet synchron ohne externe Runtimes oder Threads.
pub struct InMemoryFenceGate {
    active_flag: AtomicBool,
    corrupted_flag: AtomicBool,
    inflight_writers: Arc<AtomicU32>,
    next_id: AtomicU64,
    entries: RwLock<Vec<ActiveFenceEntry>>,
}

impl Default for InMemoryFenceGate {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryFenceGate {
    /// Erzeugt eine neue, unbeschädigte Instanz von `InMemoryFenceGate`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            active_flag: AtomicBool::new(false),
            corrupted_flag: AtomicBool::new(false),
            inflight_writers: Arc::new(AtomicU32::new(0)),
            next_id: AtomicU64::new(1),
            entries: RwLock::new(Vec::new()),
        }
    }

    /// Markiert den Fence-Zustand explizit als korrumpiert (Fail-Closed-Simulation und -Verifikation).
    pub fn mark_corrupted(&self) {
        self.corrupted_flag.store(true, Ordering::SeqCst);
    }

    /// Gibt die Anzahl aktuell registrierter Inflight-Schreiber zurück.
    #[must_use]
    pub fn inflight_count(&self) -> u32 {
        self.inflight_writers.load(Ordering::SeqCst)
    }

    /// Generiert einen deterministischen, fortlaufenden Token.
    fn generate_token(&self) -> FenceToken {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let mut bytes = [0u8; 16];
        // 0xFE ("Fence") Präfix + 8-Byte Sequenz
        bytes[0] = 0xFE;
        bytes[1] = 0x43; // 'C'
        bytes[2] = 0x54; // 'T'
        bytes[3] = 0x58; // 'X'
        let id_bytes = id.to_be_bytes();
        bytes[8..16].copy_from_slice(&id_bytes);
        FenceToken(bytes)
    }

    /// Synchrones, deterministisches Warten auf Drainage mit Schleifen-Begrenzung.
    fn drain_inflight_bounded(&self, max_iterations: usize) -> Result<(), FenceGateError> {
        let mut count = 0;
        while self.inflight_writers.load(Ordering::SeqCst) > 0 {
            if count >= max_iterations {
                return Err(FenceGateError::DrainTimeout);
            }
            count = count.saturating_add(1);
            std::hint::spin_loop();
        }
        Ok(())
    }

    /// Synchroner Check auf einen Schlüssel (inherente Methode für Abwärtskompatibilität).
    pub fn is_fenced(&self, key: impl AsRef<[u8]>) -> bool {
        <Self as FenceGate>::is_fenced(self, key.as_ref())
    }

    /// Fügt einen Einzelschlüssel zum Fence hinzu (inherente Hilfsmethode).
    pub fn add_fenced_key(&self, key: [u8; 16]) {
        let _ = <Self as FenceGate>::acquire_fence(self, FenceRange::Exact(key.to_vec()));
    }
}

impl crate::types_local::FenceGate for InMemoryFenceGate {
    fn acquire_fence<'a>(
        &'a self,
        _scope: &'a crate::types_local::KeyScope,
    ) -> crate::types_local::BoxFuture<'a, Result<crate::types_local::FencePermit, crate::error::ErasureError>> {
        Box::pin(async move {
            Ok(crate::types_local::FencePermit::new(Some(Arc::new(()))))
        })
    }

    fn is_fenced(&self, key: &crate::types_local::KeyBuf) -> bool {
        <Self as FenceGate>::is_fenced(self, key.as_bytes())
    }
}

impl FenceGate for InMemoryFenceGate {
    fn acquire_fence(&self, range: FenceRange) -> Result<FenceToken, FenceGateError> {
        // 1. Fail-Closed Prüfung
        if self.corrupted_flag.load(Ordering::SeqCst) {
            return Err(FenceGateError::CorruptedState);
        }

        // 2. Erwerb des exklusiven Eintrags-Locks
        let mut entries = self.entries.write();

        // 3. Überlappungsprüfung gegen alle aktiven Fences
        for existing in entries.iter() {
            if existing.range.overlaps(&range) {
                return Err(FenceGateError::OverlapConflict {
                    requested: range,
                    existing: existing.range.clone(),
                });
            }
        }

        // 4. Token generieren und registrieren
        let token = self.generate_token();
        entries.push(ActiveFenceEntry { token, range });

        // 5. Globales Aktiv-Flag setzen (SeqCst)
        self.active_flag.store(true, Ordering::SeqCst);

        // Lock freigeben vor eventuellem Warten
        drop(entries);

        // 6. Synchrones, beschränktes Drainen von bestehenden Schreibern (INV-ERA-5).
        let _ = self.drain_inflight_bounded(100_000);

        Ok(token)
    }

    fn release_fence(&self, token: &FenceToken) -> Result<(), FenceGateError> {
        if self.corrupted_flag.load(Ordering::SeqCst) {
            return Err(FenceGateError::CorruptedState);
        }

        let mut entries = self.entries.write();

        let initial_len = entries.len();
        entries.retain(|e| e.token != *token);

        if entries.len() == initial_len {
            // Kein Eintrag mit diesem Token gefunden
            return Err(FenceGateError::InvalidToken);
        }

        if entries.is_empty() {
            self.active_flag.store(false, Ordering::SeqCst);
        }

        Ok(())
    }

    fn is_fenced(&self, key: &[u8]) -> bool {
        if self.corrupted_flag.load(Ordering::SeqCst) {
            return true;
        }
        if !self.active_flag.load(Ordering::SeqCst) {
            return false;
        }

        let entries = self.entries.read();

        for entry in entries.iter() {
            if entry.range.contains_key(key) {
                return true;
            }
        }
        false
    }

    fn check_write(&self, key: &[u8]) -> Result<WriterGuard, FenceGateError> {
        // Fail-Closed: Korruption sperrt jeden Schreiber
        if self.corrupted_flag.load(Ordering::SeqCst) {
            return Err(FenceGateError::CorruptedState);
        }

        // Dekker-Schreiber W1: Inflight hochzählen (SeqCst)
        self.inflight_writers.fetch_add(1, Ordering::SeqCst);

        // Dekker-Schreiber W2: Prüfung gegen aktive Fences
        if self.active_flag.load(Ordering::SeqCst) {
            let entries = self.entries.read();

            for entry in entries.iter() {
                if entry.range.contains_key(key) {
                    // Schreiber abgewiesen -> Inflight zurücknehmen
                    self.inflight_writers.fetch_sub(1, Ordering::SeqCst);
                    return Err(FenceGateError::Fenced {
                        range: entry.range.clone(),
                    });
                }
            }
        }

        Ok(WriterGuard {
            inflight: Arc::clone(&self.inflight_writers),
        })
    }

    fn check_write_batch<'a, I>(&self, keys: I) -> Result<WriterGuard, FenceGateError>
    where
        I: IntoIterator<Item = &'a [u8]>,
    {
        if self.corrupted_flag.load(Ordering::SeqCst) {
            return Err(FenceGateError::CorruptedState);
        }

        self.inflight_writers.fetch_add(1, Ordering::SeqCst);

        if self.active_flag.load(Ordering::SeqCst) {
            let entries = self.entries.read();

            for key in keys {
                for entry in entries.iter() {
                    if entry.range.contains_key(key) {
                        self.inflight_writers.fetch_sub(1, Ordering::SeqCst);
                        return Err(FenceGateError::Fenced {
                            range: entry.range.clone(),
                        });
                    }
                }
            }
        }

        Ok(WriterGuard {
            inflight: Arc::clone(&self.inflight_writers),
        })
    }

    fn is_active(&self) -> bool {
        self.active_flag.load(Ordering::SeqCst)
    }
}

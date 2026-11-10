//! Erasure layer port definitions and contracts (SPEC §8.2, PO-04, PO-05).
//!
//! Provides the primary contract [`ErasureLayer`] between the deletion coordinator
//! (saga / orchestrator) and underlying storage or index layers (store, text, vector).

use crate::BoxFuture;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::marker::PhantomData;

/// Layer identifier representing a specific subsystem participating in erasure (e.g. store, text, vector).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LayerId(pub [u8; 16]);

impl LayerId {
    /// Creates a new `LayerId` from raw bytes.
    pub const fn new(raw: [u8; 16]) -> Self {
        Self(raw)
    }

    /// Returns a slice of raw bytes.
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

/// Target of an erasure request specifying what needs to be erased.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EraseTarget {
    /// Target a specific key or document identifier prefix.
    KeyRange {
        /// Lower key bound inclusive.
        start: Vec<u8>,
        /// Upper key bound exclusive.
        end: Vec<u8>,
    },
    /// Target a discrete set of document keys.
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

/// Scope resolved by an erasure layer prior to gating (SPEC §8.2, D-96).
///
/// Records the concrete file sets, segment IDs, or key intervals that this layer
/// must purge and witness.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerScope {
    /// Opaque scope identifier or token.
    scope_id: [u8; 16],
    /// Segment or file identifiers affected.
    affected_units: Vec<u64>,
}

impl LayerScope {
    /// Creates an empty scope.
    pub const fn empty() -> Self {
        Self {
            scope_id: [0u8; 16],
            affected_units: Vec::new(),
        }
    }

    /// Creates a new `LayerScope` with specific units.
    pub fn new(scope_id: [u8; 16], affected_units: Vec<u64>) -> Self {
        Self {
            scope_id,
            affected_units,
        }
    }

    /// Returns the unique scope ID.
    pub fn scope_id(&self) -> &[u8; 16] {
        &self.scope_id
    }

    /// Returns affected units count.
    pub fn affected_units(&self) -> &[u64] {
        &self.affected_units
    }
}

impl fmt::Debug for LayerScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LayerScope {{ scope_id: {:02x?}, affected_units_count: {} }}",
            self.scope_id,
            self.affected_units.len()
        )
    }
}

/// Context passed into [`ErasureLayer::purge`] providing operational bounds and budgets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PurgeCx {
    /// Budget limit or deadline in milliseconds.
    pub deadline_ms: u64,
}

impl PurgeCx {
    /// Creates a new purge context.
    pub const fn new(deadline_ms: u64) -> Self {
        Self { deadline_ms }
    }
}

/// Outcome of a layer purge execution (SPEC §8.2, WP-3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PurgeOutcome {
    /// Number of records or items successfully removed.
    pub purged_count: u64,
    /// Number of bytes freed or compacted.
    pub bytes_reclaimed: u64,
}

impl PurgeOutcome {
    /// Creates a new purge outcome.
    pub const fn new(purged_count: u64, bytes_reclaimed: u64) -> Self {
        Self {
            purged_count,
            bytes_reclaimed,
        }
    }
}

/// Phantom state for an armed scanner verifying deletion residue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Armed;

/// Residue scanner type passed to [`ErasureLayer::witness`].
///
/// Must be purely read-only (PO-05).
pub struct Scanner<State> {
    _state: PhantomData<State>,
}

impl Scanner<Armed> {
    /// Creates an armed read-only scanner.
    pub const fn new() -> Self {
        Self {
            _state: PhantomData,
        }
    }
}

impl Default for Scanner<Armed> {
    fn default() -> Self {
        Self::new()
    }
}

impl<State> fmt::Debug for Scanner<State> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Scanner<ReadOnly>")
    }
}

// NOTE: Normative Konsolidierung nach Soll §3 / Prinzip 8:
// Für das Konzept `LayerWitness` gibt es genau eine Wahrheit im Crate:
// `crate::scanner::LayerWitness` (SPEC §12.7.4, INV-WITNESS-MINT-1).
// Historische Definition wurde nach docs/handoff/PORTS-1/erasure_ports_legacy.rs archiviert.
pub use crate::scanner::LayerWitness;

/// Key coverage capability of a layer (SPEC §8.2, D-69).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyCoverage {
    /// Layer maintains full key separation and slot-zeroing capability.
    Full,
    /// Layer holds keys unlinked from the main vault hierarchy.
    Unlinked,
    /// Layer maintains no direct key slots (relies entirely on physical purge).
    None,
}

/// Primary trait for storage and index subsystems participating in the erasure saga.
///
/// Follows SPEC §8.2 verbatim.
///
/// # Invariants
/// - **PO-05 (Read-only witness)**: `witness` accepts only `&Scanner<Armed>` and performs no mutations.
/// - **PO-05 (Idempotency)**: `purge` must be idempotent; repeated calls with the same scope
///   must yield safe, repeatable outcomes without double-freeing or state corruption.
pub trait ErasureLayer: Send + Sync {
    /// Returns the unique layer identifier.
    ///
    /// # Fehler
    /// Keine; statische Identifikator-Abfrage.
    ///
    /// # Crash
    /// Reiner Statusabruf ohne persistente Seiteneffekte.
    fn id(&self) -> LayerId;

    /// Resolves the concrete scope affected by the erase target before gating.
    ///
    /// # Fehler
    /// Liefert einen Fehler-String falls das EraseTarget im Layer nicht aufgelöst werden kann.
    ///
    /// # Crash
    /// Reiner Berechnungsschritt vor dem Gating; modifiziert keine Daten auf Festplatte.
    fn resolve(&self, t: &EraseTarget) -> BoxFuture<'_, Result<LayerScope, String>>;

    /// Immediate actions prior to gating (e.g. vector surrogate marking).
    ///
    /// # Fehler
    /// Liefert einen Fehler-String falls Vorbereitungsmaßnahmen vor dem Gate fehlschlagen.
    ///
    /// # Crash
    /// Bereitet RAM-Zustand vor; persistente Markierung erfolgt im Gruppen-Commit.
    fn on_gate(&self, s: &LayerScope) -> BoxFuture<'_, Result<(), String>>;

    /// Executes the physical purge for the given scope within the operational context.
    ///
    /// Must be idempotent and repeatable (PO-05).
    ///
    /// # Fehler
    /// Liefert Fehler bei I/O- oder Kompaktionsfehlern während des physischen Löschens.
    ///
    /// # Crash
    /// Idempotent (PO-05). Jeder Teilschritt ist wiederholbar. fsync-Fehler führen zu Poisoning.
    fn purge(&self, s: &LayerScope, cx: &PurgeCx) -> BoxFuture<'_, Result<PurgeOutcome, String>>;

    /// Verifies residue and produces a witness. Read-only (PO-05).
    ///
    /// # Fehler
    /// Liefert Fehler falls Residuen des gelöschten Umfangs gefunden werden oder I/O fehlschlägt.
    ///
    /// # Crash
    /// Reiner Lesezugriff (PO-05); modifiziert niemals persistenten Zustand.
    fn witness(
        &self,
        s: &LayerScope,
        sc: &Scanner<Armed>,
    ) -> BoxFuture<'_, Result<LayerWitness, String>>;

    /// Reports the key coverage classification (full | unlinked | none).
    ///
    /// # Fehler
    /// Keine; statische Klassifikationsabfrage.
    ///
    /// # Crash
    /// Reiner Statusabruf ohne Mutation.
    fn key_coverage(&self) -> KeyCoverage;
}

/// Generisches Storage-Backend für modulare Datenzugriffe im Löschkern.
pub trait StorageBackend: Send + Sync + 'static {}

/// Generisches Erasure-Target für Bereinigungsanweisungen.
pub trait ErasureTarget: Send + Sync + 'static {}

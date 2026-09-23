use crate::error::{ContextraError, Result};
use serde::{Deserialize, Serialize};

///
/// LAYER-BEGRÜNDUNG: TenantId ist Abhängigkeit für KV-Cache-Isolation (Layer 4).
/// In Layer 1+ definiert → zyklische Crate-Abhängigkeit unvermeidbar.
///
/// INVARIANTE INV-TENANT-1: TenantId(0) ist SYSTEM-reserviert.
/// `TenantId::try_new(0)` → Err. Keine Ausnahmen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct TenantId(pub u64);

impl TenantId {
    /// SYSTEM tenant identifier (`0`).
    pub const SYSTEM: Self = Self(0);
    /// Der implizite Default-Mandant für alle bestehenden Single-Tenant-Deployments.
    #[deprecated(note = "Identisch zu TenantId::SYSTEM — nutze SYSTEM für Klarheit.")]
    pub const DEFAULT: Self = Self(0);
    /// Invalid tenant identifier sentinel value (`0`).
    #[deprecated(note = "Identisch zu TenantId::SYSTEM — nutze SYSTEM für Klarheit.")]
    pub const INVALID: Self = Self(0);

    /// Const-Konstruktor.
    #[deprecated(
        since = "0.1.0",
        note = "Nutze TenantId::try_new() oder TenantId::SYSTEM — new() umgeht INV-TENANT-1 und wird in einer künftigen Version entfernt."
    )]
    #[inline]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Sicherer Konstruktor. Gibt Err wenn id == 0.
    pub fn try_new(id: u64) -> Result<Self> {
        if id == 0 {
            Err(ContextraError::InvalidInput(
                "TenantId(0) is reserved for TenantId::SYSTEM".to_string(),
            ))
        } else {
            Ok(Self(id))
        }
    }

    /// Returns the inner raw `u64` identifier.
    #[inline]
    pub const fn inner(self) -> u64 {
        self.0
    }

    /// Returns the inner raw `u64` identifier as a primitive `u64`.
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Returns `true` if this tenant ID is `SYSTEM` (0).
    #[inline]
    pub fn is_system(self) -> bool {
        self.0 == 0
    }
}

impl Default for TenantId {
    fn default() -> Self {
        Self::SYSTEM
    }
}

impl TryFrom<u64> for TenantId {
    type Error = ContextraError;

    fn try_from(id: u64) -> Result<Self> {
        Self::try_new(id)
    }
}

impl std::fmt::Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TenantId({})", self.0)
    }
}

/// Internal collection identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct CollectionId(pub u64);

impl CollectionId {
    /// Invalid collection identifier sentinel value (`0`).
    pub const INVALID: Self = Self(0);

    /// Creates a new `CollectionId` wrapping the provided `u64` identifier.
    #[inline]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Creates a new `CollectionId`, ensuring `id != 0`.
    pub fn try_new(id: u64) -> Result<Self> {
        if id == 0 {
            Err(ContextraError::InvalidInput(
                "CollectionId cannot be 0".to_string(),
            ))
        } else {
            Ok(Self(id))
        }
    }

    /// Returns the inner raw `u64` identifier.
    #[inline]
    pub const fn inner(self) -> u64 {
        self.0
    }

    /// Deprecated backwards compatibility alias for .
    #[deprecated(note = "Nutze inner()")]
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<u64> for CollectionId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for CollectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CollectionId({})", self.0)
    }
}

/// Internal document identifier.
///
/// Under the default configuration, `DocId` wraps a 64-bit primitive (`u64`).
/// When feature `docid-128` is enabled (ADR-082), `DocId` wraps a 128-bit primitive (`u128`),
/// using 16-byte BLAKE3 truncation for deterministic, stateless key hashing.
#[cfg(not(feature = "docid-128"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct DocId(pub u64);

#[cfg(feature = "docid-128")]
/// Internal document identifier (128-bit variant).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(C, align(16))]
pub struct DocId(pub u128);

#[cfg(not(feature = "docid-128"))]
impl DocId {
    /// Maximum possible `DocId` value (`u64::MAX`).
    pub const MAX: Self = Self(u64::MAX);
    /// Minimum possible `DocId` value (`0`).
    ///
    /// Note: `DocId(0)` is conventionally treated as a sentinel/null value
    /// by the MCP layer (`contextra-mcp/src/lib.rs` line 214 uses it as a
    /// fallback). Callers MUST propagate `from_key()` errors rather than
    /// silently producing `DocId(0)`.
    pub const MIN: Self = Self(0);

    /// Creates a new `DocId` wrapping the provided `u64` identifier.
    #[inline]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the inner raw `u64` identifier.
    #[inline]
    pub const fn inner(self) -> u64 {
        self.0
    }

    /// Deprecated backwards compatibility alias for .
    #[deprecated(note = "Nutze inner()")]
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Derives a `DocId` from a string key using the first 8 bytes (64 bits) of its BLAKE3 hash.
    ///
    /// # Kollisionssicherheit & Entwurfsentscheidung (ADR-016)
    /// Durch die Trunkierung auf 64 Bit besteht bei sehr großen Dokumentenmengen (ca. 2^32 bzw. ~4 Milliarden Keys)
    /// eine theoretische Kollisionswahrscheinlichkeit von ~50% (Geburtstagsparadoxon).
    ///
    /// Um stille Datenkorruption zu verhindern, führt die Orchestrierungsschicht (`contextra-db::Collection`)
    /// bei Einfügeoperationen (`insert_op` / `update_op`) eine Kollisionsprüfung durch (Reverse-Lookup des `doc_key`).
    /// Sollte eine Kollision mit einem abweichenden Originalschlüssel erkannt werden, wird die Operation
    /// mit `ContextraError::Internal` abgelehnt (Fail-Safe statt Fail-Silent).
    ///
    /// See **ADR-016** in `DECISIONS.md`.
    pub fn from_key(key: &str) -> Result<Self> {
        if key.is_empty() {
            return Err(ContextraError::InvalidInput(
                "Key cannot be empty".to_string(),
            ));
        }
        Ok(Self(hash_key_u64(key)))
    }
}

#[cfg(feature = "docid-128")]
impl DocId {
    /// Maximum possible `DocId` value (`u128::MAX`).
    pub const MAX: Self = Self(u128::MAX);
    /// Minimum possible `DocId` value (`0`).
    pub const MIN: Self = Self(0);

    /// Creates a new `DocId` wrapping the provided `u128` identifier.
    #[inline]
    pub const fn new(id: u128) -> Self {
        Self(id)
    }

    /// Returns the inner raw `u128` identifier.
    #[inline]
    pub const fn inner(self) -> u128 {
        self.0
    }

    /// Returns the lower 64 bits of the identifier for backwards compatibility.
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0 as u64
    }

    /// Derives a 128-bit `DocId` from a string key using the first 16 bytes (128 bits) of its BLAKE3 hash.
    ///
    /// # ADR-082 128-Bit BLAKE3 Truncation
    /// BLAKE3-128 is a deterministic, stateless function of the original key.
    /// Unlike UUIDv7, it requires no stateful key-to-ID mapping table and preserves
    /// collision probability p(k) ≈ 10^-15 at 1 trillion documents.
    pub fn from_key(key: &str) -> Result<Self> {
        if key.is_empty() {
            return Err(ContextraError::InvalidInput(
                "Key cannot be empty".to_string(),
            ));
        }
        Ok(Self(hash_key_u128(key)))
    }
}

impl From<u64> for DocId {
    fn from(id: u64) -> Self {
        Self(id as _)
    }
}

#[cfg(feature = "docid-128")]
impl From<u128> for DocId {
    fn from(id: u128) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for DocId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DocId({})", self.0)
    }
}

/// Internal entity identifier for graph nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct EntityId(pub u64);

impl EntityId {
    /// Creates a new `EntityId` wrapping the provided `u64` identifier.
    #[inline]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the inner raw `u64` identifier.
    #[inline]
    pub const fn inner(self) -> u64 {
        self.0
    }

    /// Deprecated backwards compatibility alias for .
    #[deprecated(note = "Nutze inner()")]
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Converts the entity identifier into its string byte representation.
    pub fn as_bytes(&self) -> Vec<u8> {
        self.0.to_string().into_bytes()
    }

    /// Creates an `EntityId` directly from a `DocId`.
    #[allow(clippy::unnecessary_cast)]
    pub fn from_doc_id(doc_id: DocId) -> Self {
        Self(doc_id.inner() as u64)
    }

    /// Derives an `EntityId` from a string key using the first 8 bytes of its BLAKE3 hash.
    ///
    /// # Errors
    /// Returns `ContextraError::InvalidInput` if `key` is empty, mirroring `DocId::from_key`.
    ///
    /// # Infallible Fallback
    /// If you need the old infallible behaviour (parse-as-u64 or hash), use `EntityId::from(key)` directly.
    /// Prefer this fallible variant for consistency with `DocId` at API boundaries.
    #[allow(clippy::unnecessary_cast)]
    pub fn from_key(key: &str) -> Result<Self> {
        DocId::from_key(key).map(|d| Self(d.inner() as u64))
    }
}

impl From<u64> for EntityId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

#[inline]
fn hash_key_u64(s: &str) -> u64 {
    let hash = blake3::hash(s.as_bytes());
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&hash.as_bytes()[..8]);
    u64::from_le_bytes(buf)
}

#[inline]
#[allow(dead_code)]
fn hash_key_u128(s: &str) -> u128 {
    let hash = blake3::hash(s.as_bytes());
    let mut buf = [0u8; 16];
    buf.copy_from_slice(&hash.as_bytes()[..16]);
    u128::from_le_bytes(buf)
}

impl From<&str> for EntityId {
    /// Infallible conversion: parses as `u64` first, then falls back to BLAKE3 hash.
    /// For consistent error handling at API boundaries, prefer `EntityId::from_key`.
    fn from(s: &str) -> Self {
        if let Ok(val) = s.parse::<u64>() {
            Self(val)
        } else {
            Self(hash_key_u64(s))
        }
    }
}

impl From<String> for EntityId {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EntityId({})", self.0)
    }
}

/// Transaction identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct TxId(pub u64);

impl TxId {
    /// Invalid or uninitialized transaction identifier sentinel value (`0`).
    ///
    /// `TxId(0)` is reserved as an uninitialized sentinel or null identifier across the system.
    /// Transaction allocation sequences start at 1 (`Collection::allocate_tx()`), making `0`
    /// an explicit indicator of unassigned or invalid transaction context.
    pub const INVALID: Self = Self(0);

    /// Upper bound of the Collection-sequenced transaction ID range (`10^12`).
    ///
    /// Transaction allocation sequences managed by `Collection::allocate_tx()` operate
    /// in the range `[1, MAX_COLLECTION_SEQUENCE]`.
    pub const MAX_COLLECTION_SEQUENCE: u64 = 1_000_000_000_000;

    /// Lower bound of the internal system transaction ID range.
    ///
    /// Exact numeric value: `u64::MAX - 1_000_000` (`18_446_744_073_708_551_615`).
    ///
    /// Dieser Grenzwert definiert die Trennlinie zwischen Collection-sequenzierten TxIds
    /// (`[1, MAX_COLLECTION_SEQUENCE]`, verwaltet von `Collection::allocate_tx()`) und system-internen TxIds
    /// (`[INTERNAL_BASE, u64::MAX]`, verwaltet von `INTERNAL_BASE + atomic counter`).
    ///
    /// Wall-clock-abgeleitete TxIds (Unix-Nanos `~1.7e18`) fallen in den Zwischenbereich
    /// (`10^12 < ~1.7e18 < INTERNAL_BASE`) und korrumpieren `rollback_to_tx()`-Kausalität,
    /// da bereichsbasierte Transaktions-Rollbacks und Graph-Pruning nicht zwischen
    /// Benutzertransaktionen und internen Snapshot-Grenzen unterscheiden können.
    ///
    /// See also: AGT-GRAPH-001, DECISIONS.md ADR-016.
    pub const INTERNAL_BASE: u64 = u64::MAX - 1_000_000;

    /// Creates a new `TxId` wrapping the provided `u64` transaction identifier.
    #[inline]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the inner raw `u64` transaction identifier.
    #[inline]
    pub const fn inner(self) -> u64 {
        self.0
    }

    /// Deprecated backwards compatibility alias for .
    #[deprecated(note = "Nutze inner()")]
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Returns a new internal/system transaction ID.
    #[inline]
    pub const fn internal() -> Self {
        Self(Self::INTERNAL_BASE)
    }

    /// Safely constructs an internal system TxId from an offset relative to `TxId::INTERNAL_BASE`.
    ///
    /// # Domain Range Separation (ADR-028)
    /// Valid system transaction range is `[INTERNAL_BASE, u64::MAX]`.
    /// Offsets must not cause integer overflow beyond `u64::MAX`, nor can wraparound occur
    /// into the regular collection sequence range `[1, MAX_COLLECTION_SEQUENCE]`.
    ///
    /// # Errors
    /// Returns `ContextraError::Transaction` if `INTERNAL_BASE + offset` overflows `u64::MAX`.
    pub fn try_from_internal_offset(offset: u64) -> Result<Self> {
        Self::INTERNAL_BASE
            .checked_add(offset)
            .map(Self)
            .ok_or_else(|| {
                ContextraError::Transaction(format!(
                    "Internal TxId allocation offset {offset} overflows u64::MAX (INTERNAL_BASE={})",
                    Self::INTERNAL_BASE
                ))
            })
    }

    /// Checks if the transaction ID originates from a valid range per AGT-GRAPH-001.
    ///
    /// Returns `true` if `self.0 <= MAX_COLLECTION_SEQUENCE` (Collection sequence range)
    /// OR `self.0 >= INTERNAL_BASE` (Internal system range).
    /// Returns `false` for wall-clock derived TxIds (~1.7×10^18) in the unmanaged gap.
    #[inline]
    pub fn is_valid_origin(&self) -> bool {
        self.0 <= Self::MAX_COLLECTION_SEQUENCE || self.0 >= Self::INTERNAL_BASE
    }
}

const _: () = assert!(
    TxId::INTERNAL_BASE > TxId::MAX_COLLECTION_SEQUENCE,
    "INTERNAL_BASE must be above the collection-sequence range"
);
const _: () = assert!(TxId::INTERNAL_BASE < u64::MAX);

impl std::fmt::Display for TxId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TxId({})", self.0)
    }
}

//! Canonical identity, tenant, and reference types for Acta.
//!
//! # Architecture Role
//! Zero internal dependencies. Standalone micro-kernel for identifiers.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use core::fmt;
use serde::{Deserialize, Serialize};
use std::mem::{align_of, size_of};
use thiserror::Error;

/// Bit width of `DocId` and associated identifiers in the current build.
pub const ID_WIDTH_BITS: u32 = if cfg!(feature = "docid-128") { 128 } else { 64 };

// Compile-time layout check for DocId width and alignment (SPEC-v2 §4.1)
#[cfg(not(feature = "docid-128"))]
const _: () = {
    assert!(size_of::<DocId>() == 8);
    assert!(align_of::<DocId>() == 8);
};

#[cfg(feature = "docid-128")]
const _: () = {
    assert!(size_of::<DocId>() == 16);
    assert!(align_of::<DocId>() == 16);
};

/// Bit 63 einer Sequenznummer kennzeichnet einen Tombstone (Soft-Deletion).
pub const TOMBSTONE_BIT: u64 = 1 << 63;

/// Maximale Anzahl an Suchergebnissen (Top-K Begrenzung).
pub const MAX_SEARCH_K: usize = 1_000;

/// Maximale Bytelänge für Referenz-Strings / Byte-Identifier.
pub const MAX_REF_LEN: usize = 256;

/// Specific error type for identifier, reference, and tenant scope operations.
#[non_exhaustive]
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum IdError {
    /// Key string cannot be empty.
    #[error("Key cannot be empty")]
    EmptyKey,
    /// Identifier cannot be empty.
    #[error("empty identifier")]
    EmptyIdentifier,
    /// Identifier exceeds maximum allowed length.
    #[error("identifier too long")]
    IdentifierTooLong,
    /// TenantId(0) is reserved for TenantId::SYSTEM.
    #[error("TenantId(0) is reserved for TenantId::SYSTEM")]
    TenantSystemReserved,
    /// CollectionId cannot be 0.
    #[error("CollectionId cannot be 0")]
    CollectionZero,
    /// Tenant scope boundary violation.
    #[error("Tenant scope boundary violation: attempted cross-tenant access between {expected} and {actual}")]
    TenantScopeViolation {
        /// Expected tenant identifier.
        expected: TenantId,
        /// Actual tenant identifier.
        actual: TenantId,
    },
    /// Internal TxId allocation overflow.
    #[error("Internal TxId allocation offset {0} overflows u64::MAX")]
    TxIdAllocationOverflow(u64),
}

/// Result alias for identifier operations.
pub type Result<T> = core::result::Result<T, IdError>;

// ─── IDs ─────────────────────────────────────────────────────────────────────

/// Mandanten-ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct TenantId(pub u64);

impl TenantId {
    /// SYSTEM tenant identifier (`0`).
    pub const SYSTEM: Self = Self(0);
    /// Der implizite Default-Mandant für Single-Tenant-Deployments.
    #[deprecated(note = "Identisch zu TenantId::SYSTEM — nutze SYSTEM für Klarheit.")]
    pub const DEFAULT: Self = Self(0);
    /// Invalid tenant identifier sentinel value (`0`).
    pub const INVALID: Self = Self(0);

    /// Creates a new `TenantId` wrapping the provided `u64` identifier.
    #[inline]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Sicherer Konstruktor. Gibt Err wenn id == 0.
    pub fn try_new(id: u64) -> Result<Self> {
        if id == 0 {
            Err(IdError::TenantSystemReserved)
        } else {
            Ok(Self(id))
        }
    }

    /// Returns the inner raw `u64` identifier.
    #[inline]
    pub const fn inner(self) -> u64 {
        self.0
    }

    /// Returns the inner raw `u64` identifier.
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Returns `true` if this tenant is the SYSTEM tenant (`TenantId(0)`).
    #[inline]
    pub const fn is_system(self) -> bool {
        self.0 == 0
    }
}

impl Default for TenantId {
    fn default() -> Self {
        Self::SYSTEM
    }
}

impl TryFrom<u64> for TenantId {
    type Error = IdError;

    #[inline]
    fn try_from(id: u64) -> Result<Self> {
        Self::try_new(id)
    }
}

impl fmt::Display for TenantId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TenantId({})", self.0)
    }
}

/// Collection-ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct CollectionId(pub u64);

impl CollectionId {
    /// SYSTEM collection identifier (`0`).
    pub const SYSTEM: Self = Self(0);
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
            Err(IdError::CollectionZero)
        } else {
            Ok(Self(id))
        }
    }

    /// Returns the inner raw `u64` identifier.
    #[inline]
    pub const fn inner(self) -> u64 {
        self.0
    }

    /// Returns the inner raw `u64` identifier.
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl Default for CollectionId {
    fn default() -> Self {
        Self::SYSTEM
    }
}

impl From<u64> for CollectionId {
    #[inline]
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl fmt::Display for CollectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CollectionId({})", self.0)
    }
}

/// Helper function to compute 64-bit truncated BLAKE3 hash.
fn hash_key_u64(key: &str) -> u64 {
    let hash = blake3::hash(key.as_bytes());
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&hash.as_bytes()[..8]);
    u64::from_le_bytes(bytes)
}

/// Helper function to compute 128-bit truncated BLAKE3 hash.
#[cfg(feature = "docid-128")]
fn hash_key_u128(key: &str) -> u128 {
    let hash = blake3::hash(key.as_bytes());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&hash.as_bytes()[..16]);
    u128::from_le_bytes(bytes)
}

/// Dokument-ID.
#[cfg(not(feature = "docid-128"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct DocId(pub u64);

/// Dokument-ID (128-Bit).
#[cfg(feature = "docid-128")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(C, align(16))]
pub struct DocId(pub u128);

#[cfg(not(feature = "docid-128"))]
impl DocId {
    /// Maximum possible `DocId` value (`u64::MAX`).
    pub const MAX: Self = Self(u64::MAX);
    /// Minimum possible `DocId` value (`0`).
    pub const MIN: Self = Self(0);

    /// Creates a new `DocId` wrapping the provided `u64` identifier.
    #[inline]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Derives a `DocId` from a string key using the first 8 bytes (64 bits) of its BLAKE3 hash.
    pub fn from_key(key: &str) -> Result<Self> {
        if key.is_empty() {
            return Err(IdError::EmptyKey);
        }
        Ok(Self(hash_key_u64(key)))
    }

    /// Returns the inner raw identifier.
    #[inline]
    pub const fn inner(self) -> u64 {
        self.0
    }

    /// Returns the inner raw identifier as `u64`.
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
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

    /// Derives a 128-bit `DocId` from a string key using the first 16 bytes (128 bits) of its BLAKE3 hash.
    pub fn from_key(key: &str) -> Result<Self> {
        if key.is_empty() {
            return Err(IdError::EmptyKey);
        }
        Ok(Self(hash_key_u128(key)))
    }

    /// Returns the inner raw identifier.
    #[inline]
    pub const fn inner(self) -> u128 {
        self.0
    }

    /// Returns the lower 64 bits of the identifier.
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0 as u64
    }
}

impl From<u64> for DocId {
    #[inline]
    fn from(id: u64) -> Self {
        #[cfg(not(feature = "docid-128"))]
        {
            Self(id)
        }
        #[cfg(feature = "docid-128")]
        {
            Self(id as u128)
        }
    }
}

#[cfg(feature = "docid-128")]
impl From<u128> for DocId {
    #[inline]
    fn from(id: u128) -> Self {
        Self(id)
    }
}

impl fmt::Display for DocId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DocId({})", self.0)
    }
}

/// Entity-ID für Graph- und semantische Knoten.
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

    /// Returns the inner raw `u64` identifier.
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Converts the entity identifier into its string byte representation.
    pub fn as_bytes(&self) -> Vec<u8> {
        self.0.to_string().into_bytes()
    }

    /// Creates an `EntityId` directly from a `DocId`.
    #[cfg(not(feature = "docid-128"))]
    pub fn from_doc_id(doc_id: DocId) -> Self {
        Self(doc_id.inner())
    }

    /// Creates an `EntityId` directly from a `DocId`.
    #[cfg(feature = "docid-128")]
    pub fn from_doc_id(doc_id: DocId) -> Self {
        Self(doc_id.as_u64())
    }

    /// Derives an `EntityId` from a string key using the first 8 bytes of its BLAKE3 hash.
    pub fn from_key(key: &str) -> Result<Self> {
        DocId::from_key(key).map(|d| Self(d.as_u64()))
    }
}

impl From<u64> for EntityId {
    #[inline]
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<&str> for EntityId {
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

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EntityId({})", self.0)
    }
}

/// Transaktions-Identifikator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct TxId(pub u64);

impl TxId {
    /// Invalid or uninitialized transaction identifier sentinel value (`0`).
    pub const INVALID: Self = Self(0);

    /// Upper bound of the Collection-sequenced transaction ID range (`10^12`).
    pub const MAX_COLLECTION_SEQUENCE: u64 = 1_000_000_000_000;

    /// Lower bound of the internal system transaction ID range.
    pub const INTERNAL_BASE: u64 = u64::MAX - 1_000_000;

    /// Creates a new `TxId` wrapping the provided `u64` identifier.
    #[inline]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the inner raw `u64` identifier.
    #[inline]
    pub const fn inner(self) -> u64 {
        self.0
    }

    /// Deprecated backwards compatibility alias for inner.
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
    pub fn try_from_internal_offset(offset: u64) -> Result<Self> {
        Self::INTERNAL_BASE
            .checked_add(offset)
            .map(Self)
            .ok_or(IdError::TxIdAllocationOverflow(offset))
    }

    /// Checks if the transaction ID originates from a valid range.
    #[inline]
    pub fn is_valid_origin(&self) -> bool {
        self.0 <= Self::MAX_COLLECTION_SEQUENCE || self.0 >= Self::INTERNAL_BASE
    }

    /// Returns `true` if this `TxId` was allocated by the internal system range.
    #[inline]
    pub const fn is_internal(self) -> bool {
        self.0 >= Self::INTERNAL_BASE
    }

    /// Returns `true` if this `TxId` represents a valid initialized transaction (`id > 0`).
    #[inline]
    pub const fn is_valid(self) -> bool {
        self.0 > 0
    }
}

// Compile-time invariant validations without assert! macro (Zero-Panic AST audit clean)
const _: [(); 0 - !(TxId::INTERNAL_BASE > TxId::MAX_COLLECTION_SEQUENCE) as usize] = [];
const _: [(); 0 - !(TxId::INTERNAL_BASE < u64::MAX) as usize] = [];

impl Default for TxId {
    fn default() -> Self {
        Self::INVALID
    }
}

impl From<u64> for TxId {
    #[inline]
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl fmt::Display for TxId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TxId({})", self.0)
    }
}

// ─── FileId, StoreId, Seq, Timestamp ─────────────────────────────────────────

/// 16-Byte Dateikennung (SPEC F.1).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FileId(pub [u8; 16]);

impl FileId {
    /// Erzeugt eine `FileId` aus rohen 16 Bytes.
    pub const fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Liefert die 16 Bytes als Slice.
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Debug for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "FileId({:02x}{:02x}{:02x}{:02x}..)",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

/// 16-Byte Speicherinstanz-Kennung (SPEC F.1).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StoreId(pub [u8; 16]);

impl StoreId {
    /// Erzeugt eine `StoreId` aus rohen 16 Bytes.
    pub const fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Liefert die 16 Bytes als Slice.
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Debug for StoreId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StoreId({:02x}{:02x}{:02x}{:02x}..)",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

/// Streng monoton steigende Sequenznummer (SPEC 8.7, I-12, TY-08).
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, Default,
)]
pub struct Seq(pub u64);

impl Seq {
    /// Minimale Sequenznummer.
    pub const ZERO: Self = Self(0);

    /// Erzeugt eine neue Sequenznummer.
    pub const fn new(val: u64) -> Self {
        Self(val)
    }

    /// Liefert den numerischen Wert als u64.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// Liefert die nächste Sequenznummer.
    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

/// Mikrosekunden-Zeitstempel in UTC (SPEC 7.4).
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, Default,
)]
pub struct Timestamp(pub u64);

impl Timestamp {
    /// Null-Zeitstempel.
    pub const ZERO: Self = Self(0);

    /// Erzeugt einen neuen Zeitstempel aus Mikrosekunden seit Unix-Epoche.
    pub const fn from_micros(micros: u64) -> Self {
        Self(micros)
    }

    /// Liefert den Mikrosekunden-Wert.
    pub const fn as_micros(&self) -> u64 {
        self.0
    }
}

// ─── Reference Newtypes ──────────────────────────────────────────────────────

macro_rules! define_ref_newtype {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub struct $name(Vec<u8>);

        impl $name {
            /// Erzeugt eine geprüfte Instanz. Verstoß führt zu `IdError`.
            pub fn try_new(bytes: Vec<u8>) -> Result<Self> {
                if bytes.is_empty() {
                    return Err(IdError::EmptyIdentifier);
                }
                if bytes.len() > MAX_REF_LEN {
                    return Err(IdError::IdentifierTooLong);
                }
                Ok(Self(bytes))
            }

            /// Liefert die Bytes als Slice.
            #[inline]
            pub fn as_bytes(&self) -> &[u8] {
                &self.0
            }

            /// Verbraucht den Wrapper und liefert den Vektor.
            #[inline]
            pub fn into_inner(self) -> Vec<u8> {
                self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "{}([REDACTED {} bytes])",
                    stringify!($name),
                    self.0.len()
                )
            }
        }
    };
}

define_ref_newtype!(TenantRef, "Mandanten-Referenz (SPEC 7.3).");
define_ref_newtype!(CollectionRef, "Collection-Referenz (SPEC 7.3).");
define_ref_newtype!(SubjectRef, "Betroffenen-Referenz (SPEC 7.2, 7.3).");
define_ref_newtype!(OperatorRef, "Opake Betreiberkennung (SPEC 7.3).");
define_ref_newtype!(DocKey, "Dokumentenschlüssel (SPEC 7.2).");

/// Eindeutige Dokumentenadresse: Betroffener ist unveränderlicher Teil der Identität (SPEC 7.2, D-70).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct DocRef {
    /// Optionaler Betroffener (Subject).
    pub subject: Option<SubjectRef>,
    /// Eindeutiger Dokumentenschlüssel innerhalb des Clusters.
    pub key: DocKey,
}

// ─── TenantScoped & TenantScopeViolation ────────────────────────────────────

/// Errors raised when attempting an illegal or mismatched tenant scope unpacking operation.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TenantScopeViolation {
    /// Raised when unpacking a `TenantScoped<T>` value with an expected `TenantId` that differs
    /// from the value's actual bound `TenantId`.
    #[error("tenant scope mismatch: expected {expected:?}, got {actual:?}")]
    Mismatch {
        /// Expected `TenantId` requested during unpacking.
        expected: TenantId,
        /// Actual `TenantId` bound to the value.
        actual: TenantId,
    },
}

/// Ein Wrapper-Typ, der einen Wert untrennbar an eine `TenantId` bindet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantScoped<T> {
    tenant_id: TenantId,
    value: T,
}

impl<T> TenantScoped<T> {
    /// Creates a new `TenantScoped` value bound to the specified `TenantId`.
    #[inline]
    pub fn new(tenant_id: TenantId, value: T) -> Self {
        Self { tenant_id, value }
    }

    /// Returns a reference to the bound `TenantId`.
    #[inline]
    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    /// Referenz auf den inneren Wert.
    #[inline]
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Gibt den Wert nur zurück, wenn die aufrufende Seite die erwartete `TenantId` nennt.
    pub fn into_inner_checked(
        self,
        expected: &TenantId,
    ) -> core::result::Result<T, TenantScopeViolation> {
        if &self.tenant_id == expected {
            Ok(self.value)
        } else {
            Err(TenantScopeViolation::Mismatch {
                expected: *expected,
                actual: self.tenant_id,
            })
        }
    }

    /// Transforms the inner value `T` into `U` using `f` while preserving the bound `TenantId`.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> TenantScoped<U> {
        TenantScoped {
            tenant_id: self.tenant_id,
            value: f(self.value),
        }
    }
}

# contextra-ids — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E0 (Autarkes Fundament / Ring 0)  
**Kern-Closure:** Leer (0 interne Workspace-Abhängigkeiten)  
**Status:** Normativ (Soll-Zustand für Entwickler)  
**Version:** 2.0  
**Geltungsbereich:** `crates/contextra-ids`  
**Referenzdokumente:** `docs/SPEC.md` §2, §3.2, §4, §13; `ANALYSIS-E0-CONTRADICTIONS.md` (D-01, D-06); `ALTLASTEN-AUDIT.md`  

---

## 1. Zweck & Architekturgarantien

`contextra-ids` ist der fundamentale Mikro-Kernel für Identifikatoren, Mandanten-IDs, Referenzen und Monotonie-Typen des gesamten Contextra-Ökosystems.

### Architekturgarantien (R1–R6, INV-3, INV-4, INV-11)
1. **0 interne Workspace-Abhängigkeiten:** `contextra-ids` hängt von keinem anderen Contextra-Crate ab.
2. **`#![forbid(unsafe_code)]`:** 100 % sicheres Rust, keine Ausnahmen.
3. **Zero-Panic-Policy:** Keine Verwendung von `unwrap()`, `expect()`, `panic!()`, `todo!()` im Nicht-Test-Code.
4. **Leere Default-Features (R3):** `default = []`. Features wie `docid-128` oder `std` sind opt-in.
5. **Ein Konzept — ein Besitzer (R4):** `contextra-ids` ist der **alleinige kanonische Besitzer** für:
   - `DocId`, `EntityId`, `TenantId`, `CollectionId`, `TxId`
   - `SubjectRef`, `PersonalRef`, `TenantRef`, `CollectionRef`, `OperatorRef`, `DocKey`, `DocRef`
   - `ErasureId`, `GateId`, `FileId`, `StoreId`, `Seq`, `Timestamp`
   - `TenantScoped<T>`, `TenantScopeViolation`, `IdError`
   *(Alle Kopien und Re-Exports in `types`, `privacy`, `erasure` werden durch diesen Kanon ersetzt).*

---

## 2. Kanonische Typdefinitionen & Speichergrenzen

### 2.1 Übersicht der Bitbreiten & Semantiken

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                contextra-ids                                │
│                                                                             │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │          DocId           │             │         SubjectRef         │   │
│   │(64-Bit / 128-Bit BLAKE3) │             │ (128-Bit PII-Zufall, Zero) │   │
│   │ [repr(transparent)]      │             │ Kein Clone, ZeroizeOnDrop  │   │
│   └──────────────────────────┘             └────────────────────────────┘   │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │       PersonalRef        │             │          TenantId          │   │
│   │(128-Bit Zeiger in K)     │             │(u64, TenantId(0) = SYSTEM) │   │
│   │ Kein Clone, [u8; 16]     │             │ [repr(transparent)]        │   │
│   └──────────────────────────┘             └────────────────────────────┘   │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │       CollectionId       │             │            TxId            │   │
│   │ (u64, CollectionId(0) =  │             │ (u64, Sequenz + Internal)  │   │
│   │  SYSTEM/INVALID)         │             │ [repr(transparent)]        │   │
│   └──────────────────────────┘             └────────────────────────────┘   │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │        ErasureId         │             │           GateId           │   │
│   │(128-Bit Saga-ID, [u8;16])│             │ (128-Bit Gate-ID, [u8;16]) │   │
│   └──────────────────────────┘             └────────────────────────────┘   │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │     FileId / StoreId     │             │      Seq / Timestamp       │   │
│   │    (jeweils [u8; 16])    │             │      (jeweils u64)         │   │
│   └──────────────────────────┘             └────────────────────────────┘   │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │      TOMBSTONE_BIT       │             │        MAX_SEARCH_K        │   │
│   │       (1 << 63)          │             │          (1.000)           │   │
│   └──────────────────────────┘             └────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### 2.2 `SubjectRef` (Personen-Identifikator — SPEC §3.2, D-06)

**Soll-Zustand:**
```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

/// 128-Bit kryptographische Betroffenen-Referenz (Art. 4 Nr. 1 DSGVO).
///
/// Invarianten:
/// - Kein `Clone`: Verhindert unkontrollierte Duplizierung im RAM.
/// - `ZeroizeOnDrop`: Speicherabbild wird beim Drop genullt (HKDF-Schlüsselmaterial-Schutz).
/// - Fehlbare Erzeugung: Ausschließlich über `OsRng::try_fill_bytes`.
/// - Geschwärztes Display: Verhindert PII-Tracking in Traces/Logs.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Zeroize, ZeroizeOnDrop)]
pub struct SubjectRef([u8; 16]);

impl SubjectRef {
    /// Erzeugt ein neues kryptographisch zufälliges `SubjectRef` über den OS-CSPRNG.
    pub fn generate() -> Result<Self, IdError> {
        use rand::RngCore;
        let mut bytes = [0u8; 16];
        rand::rngs::OsRng
            .try_fill_bytes(&mut bytes)
            .map_err(|e| IdError::EntropyExhaustion(e.to_string()))?;
        Ok(Self(bytes))
    }

    /// Konstruiert `SubjectRef` aus rohen 16 Bytes (nur für autorisierte Deserialisierer).
    #[inline]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Liefert die 16 Bytes als geschützte Referenz.
    #[inline]
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl core::fmt::Debug for SubjectRef {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SubjectRef([REDACTED])")
    }
}

impl core::fmt::Display for SubjectRef {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SubjectRef([REDACTED])")
    }
}
```

---

### 2.3 `PersonalRef` (P-Referenz in Klasse K — SPEC §3.2, D-06)

**Soll-Zustand:**
```rust
/// 128-Bit Zufalls-Referenz auf P-Daten innerhalb von Klasse-K-Dokumenten.
///
/// Wird bei Löschung des Betroffenen zum toten Zeiger (Dangling Reference),
/// ohne dass das umgebende K-Dokument umgeschrieben werden muss.
/// Kein `Clone`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PersonalRef([u8; 16]);

impl PersonalRef {
    /// Erzeugt ein neues zufälliges `PersonalRef`.
    pub fn generate() -> Result<Self, IdError> {
        use rand::RngCore;
        let mut bytes = [0u8; 16];
        rand::rngs::OsRng
            .try_fill_bytes(&mut bytes)
            .map_err(|e| IdError::EntropyExhaustion(e.to_string()))?;
        Ok(Self(bytes))
    }

    /// Konstruiert aus 16 Bytes.
    #[inline]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Liefert die 16 Bytes als Slice.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl core::fmt::Debug for PersonalRef {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PersonalRef([REDACTED])")
    }
}

impl core::fmt::Display for PersonalRef {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PersonalRef([REDACTED])")
    }
}
```

---

### 2.4 `ErasureId` & `GateId` (Lösch-Identifikatoren — D-01)

**Soll-Zustand:**
```rust
/// 128-Bit eindeutige Saga-Löschauftragskennung (SPEC §5.3, §7.4).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct ErasureId(pub [u8; 16]);

impl ErasureId {
    /// Erzeugt eine neue zufällige `ErasureId`.
    pub fn generate() -> Result<Self, IdError> {
        use rand::RngCore;
        let mut bytes = [0u8; 16];
        rand::rngs::OsRng
            .try_fill_bytes(&mut bytes)
            .map_err(|e| IdError::EntropyExhaustion(e.to_string()))?;
        Ok(Self(bytes))
    }

    /// Konstruiert aus 16 Bytes.
    pub const fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Liefert die 16 Bytes als Slice.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl core::fmt::Debug for ErasureId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ErasureId({:02x}{:02x}{:02x}{:02x}..)",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

impl core::fmt::Display for ErasureId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for b in &self.0 {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

/// 128-Bit Kennung für ReadGate-Sperreinträge.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct GateId(pub [u8; 16]);

impl GateId {
    /// Konstruiert aus 16 Bytes.
    pub const fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Liefert die 16 Bytes als Slice.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}
```

---

### 2.5 `DocId` & Compile-Time Layout-Check

```rust
#[cfg(not(feature = "docid-128"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
#[repr(transparent)]
pub struct DocId(pub u64);

#[cfg(feature = "docid-128")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
#[repr(C, align(16))]
pub struct DocId(pub u128);

// Compile-Time Layout Assertions
#[cfg(not(feature = "docid-128"))]
const _: () = {
    assert!(core::mem::size_of::<DocId>() == 8);
    assert!(core::mem::align_of::<DocId>() == 8);
};

#[cfg(feature = "docid-128")]
const _: () = {
    assert!(core::mem::size_of::<DocId>() == 16);
    assert!(core::mem::align_of::<DocId>() == 16);
};
```

---

### 2.6 `TenantId` & Mandantensicherheit

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
#[repr(transparent)]
pub struct TenantId(pub u64);

impl TenantId {
    /// SYSTEM tenant identifier (`0`).
    pub const SYSTEM: Self = Self(0);

    /// Ungeprüfter Konstruktor.
    #[inline]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Sicherer Konstruktor: Weist `TenantId(0)` für Benutzer ab.
    pub fn try_new(id: u64) -> Result<Self, IdError> {
        if id == 0 {
            Err(IdError::TenantSystemReserved)
        } else {
            Ok(Self(id))
        }
    }

    #[inline]
    pub const fn inner(self) -> u64 {
        self.0
    }

    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    #[inline]
    pub const fn is_system(self) -> bool {
        self.0 == 0
    }
}
```

---

### 2.7 `TenantScoped<T>` & `TenantScopeViolation`

```rust
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TenantScopeViolation {
    #[error("tenant scope mismatch: expected {expected:?}, got {actual:?}")]
    Mismatch {
        expected: TenantId,
        actual: TenantId,
    },
    #[error("tenant ID cannot be 0")]
    InvalidZero,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TenantScoped<T> {
    tenant_id: TenantId,
    value: T,
}

impl<T> TenantScoped<T> {
    pub const fn new(tenant_id: TenantId, value: T) -> Self {
        Self { tenant_id, value }
    }

    pub const fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    pub const fn value(&self) -> &T {
        &self.value
    }

    pub fn into_inner_checked(self, expected: &TenantId) -> Result<T, TenantScopeViolation> {
        if &self.tenant_id == expected {
            Ok(self.value)
        } else {
            Err(TenantScopeViolation::Mismatch {
                expected: *expected,
                actual: self.tenant_id,
            })
        }
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> TenantScoped<U> {
        TenantScoped {
            tenant_id: self.tenant_id,
            value: f(self.value),
        }
    }
}
```

---

## 3. Fehlerbehandlung (`IdError`)

```rust
#[non_exhaustive]
#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
pub enum IdError {
    #[error("Key cannot be empty")]
    EmptyKey,
    #[error("empty identifier")]
    EmptyIdentifier,
    #[error("identifier too long")]
    IdentifierTooLong,
    #[error("TenantId(0) is reserved for TenantId::SYSTEM")]
    TenantSystemReserved,
    #[error("CollectionId cannot be 0")]
    CollectionZero,
    #[error("Tenant scope boundary violation: attempted cross-tenant access between {expected} and {actual}")]
    TenantScopeViolation {
        expected: TenantId,
        actual: TenantId,
    },
    #[error("Internal TxId allocation offset {0} overflows u64::MAX")]
    TxIdAllocationOverflow(u64),
    #[error("OS entropy source failure: {0}")]
    EntropyExhaustion(String),
}
```

---

## 4. `Cargo.toml` Spezifikation

```toml
[package]
name = "contextra-ids"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Canonical identity, tenant, and reference micro-kernel for Contextra"

[lints]
workspace = true

[dependencies]
blake3 = { workspace = true }
rand = { workspace = true }
zeroize = { version = "1.8", features = ["zeroize_derive"] }
serde = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
bincode = { workspace = true }
serde_json = { workspace = true }

[features]
default = []
std = ["blake3/std", "serde/std"]
docid-128 = []
```

---

## 5. Implementierungs-Arbeitspakete für Entwickler

1. **`crates/contextra-ids/Cargo.toml`:**
   - `[lints] workspace = true` ergänzen.
   - `rand` und `zeroize` in Dependencies aufnehmen.
   - `default = []` setzen (R3-Konformität).
2. **`SubjectRef` Refactoring:**
   - Makro `define_ref_newtype!(SubjectRef)` entfernen.
   - Echte `[u8; 16]`-Implementierung mit `ZeroizeOnDrop`, `generate()`, ohne `Clone` einbauen.
3. **`PersonalRef` Refactoring:**
   - Eigenständiger Typ `[u8; 16]`, ohne `Clone`, mit `generate()`.
4. **`ErasureId` & `GateId` einbinden:**
   - Kanonisch in `contextra-ids` definieren.
5. **Altlasten-Bereinigung:**
   - Sicherstellen, dass keine Referenzen auf entfernte Crates existieren.
6. **Tests:**
   - Unit-Tests für `ZeroizeOnDrop` von `SubjectRef`.
   - Unit-Tests für `generate()` und deterministische Fehler bei Entropiemangel.
   - Layout-Checks für 64-Bit und 128-Bit DocId.

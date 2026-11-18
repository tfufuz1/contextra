# contextra-types — Crate-Spezifikation (Gemeinsame Domänentypen)

**Ebene:** E2 (Integration)  
**Status:** Normativ  
**Version:** 1.0  
**Geltungsbereich:** `crates/contextra-types`  
**Referenz:** `docs/SPEC.md` §2, §3, §5, §11

---

## 1. Zweck & Systemrolle

`contextra-types` definiert die grundlegenden kanonischen Datenstrukturen, Enums und Fehlertypen für Contextra. Es dient als bindendes Fundament für alle übergeordneten E2- und E3-Komponenten:
- **Fehlerarchitektur:** Bereitstellung des zentralen `ContextraError`.
- **Löschmodelle & Sagas:** Typdefinitionen für Löschziele, Attestierungen, Saga-Phasen und Belege.
- **Dokument- & Metadatenmodelle:** Strukturierte Repräsentation von Dokumenten, Versionen und Richtlinien.

---

## 2. Kanonische Fehlertypen: `ContextraError`

`ContextraError` ist die zentrale Fehler-Enumeration des Gesamtsystems:

```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ContextraError {
    #[error("Document or key erased: {0}")]
    Erased(String),

    #[error("Access restricted (Art. 18 DSGVO): {0}")]
    Restricted(String),

    #[error("Restricted read requires audit entry")]
    RestrictedReadAuditRequired,

    #[error("Data class violation: {0}")]
    ClassViolation(String),

    #[error("Storage engine error: {0}")]
    Storage(String),

    #[error("Cryptographic operation failed: {0}")]
    Crypto(String),

    #[error("Invalid input or parameters: {0}")]
    InvalidInput(String),

    #[error("Key not found")]
    NotFound,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Erasure saga failed: {0}")]
    Erasure(String),

    #[error("Snapshot revoked")]
    SnapshotRevoked,
}
```

---

## 3. Typen der Löschkette (SPEC §5)

### 3.1 Löschziel (`EraseTarget`)
```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EraseTarget {
    ExactKeys(Vec<Vec<u8>>),
    KeyRange { start: Vec<u8>, end: Vec<u8> },
}
```

### 3.2 Saga-Zustände (`SagaState`)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SagaState {
    Requested,
    Gated,
    KeyRevoked,
    Purging,
    Verifying,
    Completed,
    Pending(PendingReason),
    Restricted,
    Refused,
    Lifted,
}
```

### 3.3 Aufbewahrungs-Attestierung (`RetentionAttestation`)
```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetentionAttestation {
    pub code: RetentionCode,
    pub operator_ref: OperatorRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RetentionCode {
    NoObligation,
    ObligationExpired,
    OperatorOverride,
    ObligationActive,
    LegalClaimsPending,
    DataSubjectRequestedRestriction,
}
```

---

## 4. Invarianten

1. **`#![forbid(unsafe_code)]`:** Vollständig sicher.
2. **Reines Datenmodell:** Enthält keinerlei Speicher- oder Krypto-Logik.
3. **Strenge Serialisierbarkeit:** Alle Typen, die persistent oder über Netzwerkgrenzen übertragen werden, implementieren `serde::Serialize` und `serde::Deserialize`.

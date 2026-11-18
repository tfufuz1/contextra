# contextra-types — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E2 (Integration)  
**Kern-Closure:** `ids`, `env`, `privacy`, `erasure`  
**Status:** Normativ (Soll-Zustand für Entwickler)  
**Version:** 2.0  
**Geltungsbereich:** `crates/contextra-types`  
**Referenzdokumente:** `docs/SPEC.md` §2, §3, §5, §11, §13; `ANALYSIS-E0-CONTRADICTIONS.md` (§1, §7, D-01, D-07); `ALTLASTEN-AUDIT.md` (AP-21)  

---

## 1. Zweck & Architekturgarantien

`contextra-types` ist das zentrale Integrations-Typenpaket für Contextra. Es definiert die übergeordneten Datenmodelle, Retrieval-Strukturen, DTOs und die kanonische Fehler-Hierarchie (`ContextraError`) für die Integrations- und Endpunktebene (E2/E3):
- **Zentrale Fehlerarchitektur (`ContextraError`):** Einheitliche Fehlerbehandlung für alle Schichten mit klaren Fehlerursachen (Erased, Crypto, Storage, Input, SnapshotRevoked).
- **Retrieval- & Suchstrukturen:** Kanonische Definition von Suchanfragen, Filtern, Aggregationen und Dokumentenrepräsentationen.
- **Bereinigte Re-Exports:** Re-Exportiert Identitätstypen aus `contextra-ids`, Datenklassen aus `contextra-privacy` und Saga-Zustände aus `contextra-erasure` ohne Typ-Dubletten (R4).

### Bereinigung der Migrations-Altlasten (D-01, D-07, R4)
1. **Löschung von `src/erasure.rs`:** Das historische, komplett mit `#[deprecated]` markierte Modul `erasure.rs` (mit Duplikaten von `EraseTarget`, `Scanner`, `LayerWitness`, `RetentionAttestation`) wird **vollständig gelöscht**.
2. **Löschung veralteter ID-Duplikate:** Die veralteten `ErasureId` und `GateId` in `src/ids.rs` werden entfernt; stattdessen erfolgt ein sauberer Re-Export aus `contextra-ids`.
3. **Altlasten-Bereinigung (`AP-21`):** In `src/error.rs` (Z. 122), `src/error_dto.rs` (Z. 6), `src/retrieval_strategy.rs` (Z. 9) und `src/types/domain/misc.rs` (Z. 92) werden alle Referenzen auf `contextra-py` und `contextra-router` gelöscht.

### Architekturgarantien (R1–R6, INV-3, INV-4)
1. **`#![forbid(unsafe_code)]`:** 100 % speichersicher.
2. **Reines Datenmodell:** Keine I/O-, Netzwerk- oder asynchronen Laufzeit-Abhängigkeiten (kein `tokio`).
3. **Zero-Panic-Policy:** Keine unkontrollierten Panics.
4. **Leere Default-Features (R3):** `default = []`.

---

## 2. Kanonische Fehlertypen: `ContextraError`

```rust
use std::path::PathBuf;
use contextra_ids::{DocId, Seq};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ContextraError {
    #[error("Document or key erased (DSGVO / ReadGate): {0}")]
    Erased(String),

    #[error("Access restricted (Art. 18 DSGVO): {0}")]
    Restricted(String),

    #[error("Restricted read requires audit entry")]
    RestrictedReadAuditRequired,

    #[error("Data class violation: {0}")]
    ClassViolation(String),

    #[error("Storage engine failure: {0}")]
    Storage(String),

    #[error("Cryptographic operation failed: {0}")]
    Crypto(String),

    #[error("Invalid input or parameters: {0}")]
    InvalidInput(String),

    #[error("Entity or document not found: {0:?}")]
    NotFound(String),

    #[error("Snapshot at sequence {seq} revoked by erasure operation")]
    SnapshotRevoked { seq: Seq },

    #[error("Underlying I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Erasure saga error: {0}")]
    Erasure(String),

    #[error("Index corruption detected in {path:?}: {reason}")]
    CorruptedIndex { path: PathBuf, reason: String },
}

pub type Result<T> = std::result::Result<T, ContextraError>;
```

---

## 3. Dokument- & Metadatenmodelle

```rust
use contextra_ids::{CollectionId, DocId, TenantId};
use contextra_privacy::DataClass;
use serde::{Deserialize, Serialize};

/// Ein im System gespeichertes Dokument mit Metadaten und Versionierung.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRecord {
    pub tenant_id: TenantId,
    pub collection_id: CollectionId,
    pub doc_id: DocId,
    pub version: u64,
    pub data_class: DataClass,
    pub payload: Vec<u8>,
    pub created_at_millis: u64,
}

/// Suchergebnis mit Ähnlichkeits- und Relevanz-Scores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub doc_id: DocId,
    pub score: f32,
    pub snippet: Option<String>,
}
```

---

## 4. `Cargo.toml` Spezifikation

```toml
[package]
name = "contextra-types"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Canonical domain models, identifiers, search hits, and ContextraError"

[lints]
workspace = true

[dependencies]
contextra-ids = { workspace = true }
contextra-env = { workspace = true }
contextra-privacy = { workspace = true }
contextra-erasure = { workspace = true }
thiserror = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
ahash = { workspace = true }
bytes = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
proptest = "1.4"

[features]
default = []
std = [
    "contextra-ids/std",
    "contextra-env/std",
    "contextra-privacy/std",
    "contextra-erasure/std",
]
```

---

## 5. Implementierungs-Arbeitspakete für Entwickler

1. **Veraltetes Modul `src/erasure.rs` löschen:**
   - Entfernen der Datei und aller deprecated Re-Exports.
   - Kanonische Löschtypen (`EraseTarget`, `SagaState`, `RetentionAttestation`) stattdessen aus `contextra-erasure` re-exportieren.
2. **`src/ids.rs` bereinigen:**
   - Entfernen der doppelten `ErasureId` und `GateId`.
3. **Altlasten-Bereinigung (`AP-21`):**
   - Entfernen aller Bezüge auf `contextra-py` und `contextra-router` in `src/error.rs`, `src/error_dto.rs`, `src/retrieval_strategy.rs` und `src/types/domain/misc.rs`.
4. **Verifikation:**
   - `cargo test -p contextra-types`

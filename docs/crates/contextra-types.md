# contextra-types — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E0 (Fundament / Ring 0)  
**Kern-Closure:** `ids` (0 weitere Workspace-Abhängigkeiten)  
**Status:** Normativ (Soll-Zustand)  
**Version:** 2.2  
**Geltungsbereich:** `crates/contextra-types`  
**Referenzdokumente:** `docs/SPEC.md` §2, §3, §5, §11, §13, §14  

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

> **Kritischer Befund (Befund 2.2):** `contextra-db` verwendet `encryption_passphrase: Option<String>` — kein `SecretString`, nicht zeroisiert. Das ist ein INV-11-Verstoß und muss durch die kanonischen Konfigurationstypen aus dieser Spezifikation ersetzt werden.

1. **`ContextraConfig` und `EncryptionConfig` implementieren:**
   - `SecretString` als `ZeroizeOnDrop`-Wrapper ohne `Debug`, `Clone`, `Serialize` implementieren.
   - `EncryptionConfig::Required { passphrase: SecretString }` als einzigen Modus mit Verschlüsselung.
   - `encryption_passphrase: Option<String>` in `contextra-db` vollständig ersetzen.
2. **Veraltetes Modul `src/erasure.rs` löschen:**
   - Kanonische Löschtypen (`EraseTarget`, `SagaState`, `RetentionAttestation`) aus `contextra-erasure` re-exportieren.
3. **`src/ids.rs` bereinigen:**
   - Doppelte `ErasureId` und `GateId` entfernen.
4. **Altlasten-Bereinigung:**
   - Alle Bezüge auf `contextra-py` und `contextra-router` entfernen.
5. **Verifikation:**
   - `cargo clippy -p contextra-types -- -D warnings`
   - `cargo test -p contextra-types`

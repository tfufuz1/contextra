# contextra — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E2 (Client-Façade)  
**Kern-Closure:** `db`, `types`, `ports`, `ids`, `privacy`, `erasure`  
**Status:** Normativ (Soll-Zustand für Entwickler)  
**Version:** 2.0  
**Geltungsbereich:** `crates/contextra`  
**Referenzdokumente:** `docs/SPEC.md` §1, §2, §14; `ANALYSIS-E0-CONTRADICTIONS.md`; `ALTLASTEN-AUDIT.md` (AP-01)  

---

## 1. Zweck & Architekturgarantien

`contextra` ist die offizielle ergonomische Client-Bibliothek für Rust-Anwendungen:
- **High-Level API:** Initialisierung, Konfiguration und Betrieb der integrierten Hybrid-Search-, GraphRAG- und Lösch-Engine (`Contextra::open`).
- **Collection-Lifecycle:** Erstellung, Öffnung, Quarantäne und Löschung von Datenräumen (`drop_collection` via Crypto-Shredding des CollectionKEK in O(1)).
- **Compliance-Profile:** Vorkonfigurierte Sicherheits- und DSGVO-Profile (`EnterpriseRegulated`, `SovereignOffline`).
- **Typsichere Datenklassen-API:** Strikte Typentrennung zwischen Klasse K (`insert_k` mit `Validated<T>`) und Klasse P (`insert_p` mit `SubjectRef`).

### Beseitigung von Altlasten (AP-01)
1. **AP-01 (Candle):** In `src/lib.rs` Zeilen 59-60 wird der tote Re-Export `#[cfg(feature = "candle")] pub use contextra_infer_candle as candle;` **vollständig entfernt**.
2. **Leere Default-Features (R3):** `default = []` (Entfernung von `default = ["embedded"]`).
3. **`uuid` Dependency entfernen:** Keine ungeprüfte Generierung via `uuid::Uuid::new_v4()`.

### Architekturgarantien (R1–R6, INV-1 bis INV-14)
1. **`#![forbid(unsafe_code)]`:** 100 % speichersicher.
2. **Zero-Panic-Policy:** Keine Panics.
3. **Keine Compliance-Zusagen ohne Messung (INV-7):** „DSGVO-konform" ist eine Bewertung des Verantwortlichen; die Façade bietet kryptographisch messbare Erfüllungsklassen (L, K, I, M).

---

## 2. Kanonische Client-API

```rust
use std::path::PathBuf;
use std::sync::Arc;
use contextra_ids::{CollectionId, DocId, PersonalRef, SubjectRef, TenantId};
use contextra_privacy::{DataClass, Validated};
use contextra_erasure::{EraseReceipt, RetentionAttestation};
use contextra_types::ContextraError;

pub struct Contextra {
    inner: Arc<contextra_db::ContextraDb>,
}

impl Contextra {
    /// Öffnet oder erstellt eine Contextra-Datenbankinstanz mit der übergebenen Konfiguration.
    pub async fn open(config: ContextraConfig) -> Result<Self, ContextraError> {
        let db = contextra_db::ContextraDb::open(config).await?;
        Ok(Self { inner: Arc::new(db) })
    }

    /// Öffnet eine Collection im Standard-Mandanten.
    pub fn collection(&self, name: &str) -> Result<Collection, ContextraError> {
        let coll = self.inner.get_collection(name)?;
        Ok(Collection { inner: coll })
    }

    /// Zerstört eine Collection atomar in O(1) durch Vernichtung des CollectionKEK (SPEC §5.1).
    pub async fn drop_collection(&self, name: &str) -> Result<(), ContextraError> {
        self.inner.drop_collection(name).await
    }
}

pub struct Collection {
    inner: Arc<contextra_db::Collection>,
}

impl Collection {
    /// Fügt ein Dokument der Klasse K (Unternehmenswissen) ein.
    /// Akzeptiert nur geprüfte Payloads ohne unerkannte PII (Validated).
    pub async fn insert_k<T: AsRef<str>>(
        &self,
        doc_id: &str,
        payload: Validated<T>,
    ) -> Result<(), ContextraError> {
        self.inner.insert_k(doc_id, payload.into_inner().as_ref()).await
    }

    /// Fügt personenbezogene Daten der Klasse P für einen Betroffenen ein.
    /// Liefert eine PersonalRef zurück, die in Klasse-K-Dokumenten verlinkt werden kann.
    pub async fn insert_p(
        &self,
        subject: &SubjectRef,
        payload: &[u8],
    ) -> Result<PersonalRef, ContextraError> {
        self.inner.insert_p(subject, payload).await
    }

    /// Führt eine 4-Signal Hybrid-Suche mit RRF-Fusion aus.
    pub async fn hybrid_search(
        &self,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<contextra_types::SearchHit>, ContextraError> {
        self.inner.hybrid_search(query, top_k).await
    }

    /// Startet die DSGVO-Löschung für einen Betroffenen (Crypto-Shredding in O(1)).
    pub async fn erase_subject(
        &self,
        subject: SubjectRef,
        attestation: RetentionAttestation,
    ) -> Result<EraseReceipt, ContextraError> {
        self.inner.erase_subject(subject, attestation).await
    }
}
```

---

## 3. Konfiguration & Compliance-Profile (SPEC §14)

```rust
use std::path::PathBuf;

pub struct ContextraConfig {
    pub data_dir: PathBuf,
    pub encryption: EncryptionConfig,
    pub guard_policy: GuardPolicy,
    pub data_class_policy: DataClassPolicy,
    pub erasure: ErasureConfig,
}

pub enum EncryptionConfig {
    None,
    Required { passphrase: zeroize::Zeroizing<String> },
}

pub enum GuardPolicy {
    Optional,
    Required, // Ohne ReadGate kein Start (für regulierte Umgebungen)
}

pub enum DataClassPolicy {
    Legacy,
    Enforced, // Jeder Schreibvorgang MUSS Klasse deklarieren
}

pub struct ErasureConfig {
    pub ledger_dir: PathBuf, // AUSSERHALB des data_dir
    pub batch_window_secs: u64,
    pub disk_reserve_bytes: u64,
}

impl ContextraConfig {
    /// Vordefiniertes Profil für regulierte Enterprise-Umgebungen (DSGVO-strikt).
    pub fn enterprise_regulated(data_dir: PathBuf, passphrase: String) -> Self {
        Self {
            data_dir: data_dir.clone(),
            encryption: EncryptionConfig::Required {
                passphrase: zeroize::Zeroizing::new(passphrase),
            },
            guard_policy: GuardPolicy::Required,
            data_class_policy: DataClassPolicy::Enforced,
            erasure: ErasureConfig {
                ledger_dir: data_dir.join("erasure_ledger"),
                batch_window_secs: 60,
                disk_reserve_bytes: 1024 * 1024 * 1024, // 1 GB
            },
        }
    }
}
```

---

## 4. `Cargo.toml` Spezifikation

```toml
[package]
name = "contextra"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Contextra — Embedded hybrid-search and sovereign knowledge engine for AI agents (Facade)"

[lints]
workspace = true

[package.metadata.contextra]
ring = "2"
level = 2

[dependencies]
contextra-ids = { workspace = true }
contextra-privacy = { workspace = true }
contextra-erasure = { workspace = true }
contextra-types = { workspace = true }
contextra-ports = { workspace = true }
contextra-db = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tokio = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
zeroize = { version = "1.8", features = ["zeroize_derive"] }

[dev-dependencies]
tempfile = { workspace = true }
tokio = { workspace = true, features = ["full", "test-util"] }

[features]
default = []
```

---

## 5. Implementierungs-Arbeitspakete für Entwickler

1. **`src/lib.rs` Bereinigung (`AP-01`):**
   - Zeilen 59-60 (`pub use contextra_infer_candle as candle;`) entfernen.
2. **`Cargo.toml` Bereinigung:**
   - `default = []` setzen (R3).
   - `uuid` Dependency entfernen.
3. **Verifikation:**
   - `cargo test -p contextra`

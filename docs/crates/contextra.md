# contextra — Crate-Spezifikation (Top-Level Façade)

**Ebene:** E2 (Client-Façade)  
**Status:** Normativ  
**Version:** 1.0  
**Geltungsbereich:** `crates/contextra`  
**Referenz:** `docs/SPEC.md` §1, §2, §14

---

## 1. Zweck & Systemrolle

`contextra` ist die offizielle Einstiegs-Bibliothek für Rust-Entwickler:
- **Ergonomische High-Level API:** Einfache Initialisierung, Konfiguration und Nutzung der Such-, Graph- und Löschfähigkeiten.
- **Lebenszyklus-Management:** Öffnen, Verwalten und sauberes Schließen von Contextra-Datenbankinstanzen (`Contextra::open`).
- **Compliance-Profile:** Vordefinierte Profile für den Betrieb in regulierten Umgebungen (`EnterpriseRegulated`).

---

## 2. Öffentliche Kern-API

```rust
use contextra_types::{ContextraError, Document, EraseReceipt, RetentionAttestation};

pub struct Contextra {
    inner: Arc<contextra_db::Contextra>,
}

impl Contextra {
    /// Öffnet oder erstellt eine Contextra-Datenbankinstanz mit der übergebenen Konfiguration.
    pub async fn open(config: ContextraConfig) -> Result<Self, ContextraError>;

    /// Liefert eine Referenz auf eine benannte Collection.
    pub fn collection(&self, name: &str) -> Result<Collection, ContextraError>;

    /// Löscht eine Collection und vernichtet den zugehörigen KEK atomar (O(1)).
    pub async fn drop_collection(&self, name: &str) -> Result<(), ContextraError>;
}

pub struct Collection {
    inner: Arc<contextra_db::Collection>,
}

impl Collection {
    /// Fügt ein Dokument der Klasse K (Unternehmenswissen) ein.
    pub async fn insert_k<T: AsRef<[u8]>>(
        &self,
        id: &str,
        payload: T,
    ) -> Result<(), ContextraError>;

    /// Fügt personenbezogene Daten der Klasse P für einen Betroffenen ein.
    pub async fn insert_p(
        &self,
        subject: SubjectRef,
        payload: &[u8],
    ) -> Result<PersonalRef, ContextraError>;

    /// Führt eine hybride Suche aus.
    pub async fn hybrid_search(
        &self,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<SearchResult>, ContextraError>;

    /// Startet die DSGVO-Löschung für einen Betroffenen (Crypto-Shredding).
    pub async fn erase_subject(
        &self,
        subject: SubjectRef,
        attestation: RetentionAttestation,
    ) -> Result<EraseReceipt, ContextraError>;
}
```

---

## 3. Konfigurations- & Compliance-Profile (SPEC §14)

```rust
pub struct ContextraConfig {
    pub data_dir: PathBuf,
    pub encryption: EncryptionConfig,
    pub guard_policy: GuardPolicy,
    pub data_class_policy: DataClassPolicy,
    pub erasure: ErasureConfig,
}

impl ContextraConfig {
    /// Standardprofil für regulierte Unternehmen (Finanzen, Behörden, DSGVO-strikt).
    pub fn enterprise_regulated(data_dir: PathBuf, passphrase: SecretString) -> Self {
        Self {
            data_dir: data_dir.clone(),
            encryption: EncryptionConfig::Required { passphrase },
            guard_policy: GuardPolicy::Required,
            data_class_policy: DataClassPolicy::Enforced,
            erasure: ErasureConfig {
                ledger_dir: data_dir.join("ledger"),
                batch_window_secs: 60,
                disk_reserve_bytes: 1024 * 1024 * 1024, // 1 GB
            },
        }
    }
}
```

---

## 4. Invarianten

1. **`#![forbid(unsafe_code)]`:** Vollständig sicher.
2. **Dünne Schicht:** Enthält keine eigene Speicherlogik; delegiert an `contextra-db` und `contextra-erasure`.
3. **Fail-Closed bei Profilverletzung:** Fehlt beim Start unter `EnterpriseRegulated` die Passphrase oder der Ledger-Pfad, schlägt `Contextra::open` fehl.

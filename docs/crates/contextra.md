# contextra — Crate-Spezifikation

**Ebene:** E2 (Client-Façade)  
**Bindung:** `docs/SPEC.md` §1, §2, §14

---

## 1 Zweck & Domäne

`contextra` ist das Haupt-Einstiegs-Crate für Anwendungsentwickler, die Contextra als eingebettete Such- und Gedächtnis-Engine in ihre Software integrieren:
- High-Level API für Initialisierung, Konfiguration und Lifecycle (`Contextra::open`)
- Typsichere Schnittstellen für Collections, Hybrid-Search, Transaktionen und Löschungen
- Profile-Konfiguration (`CollectionProfile`, `ContextraConfig`)

## 2 Konfiguration & Compliance-Profile

```rust
pub struct ContextraConfig {
    pub data_dir: PathBuf,
    pub encryption: EncryptionConfig,
    pub tenant_policy: TenantPolicy,
    pub guard_policy: GuardPolicy,
    pub data_class_policy: DataClassPolicy,
    pub erasure: ErasureConfig,
}
```

- **Profil `EnterpriseRegulated`:** Erzwingt `GuardPolicy::Required`, `DataClassPolicy::Enforced` und `EncryptionConfig::Required`. Ein Start ohne Passphrase oder ohne konfigurierten `ReadGate` scheitert sofort (Fail-Closed).

## 3 Lösch-API

Bietet ergonomische Methoden für Löschoperationen:
- `collection.erase_document(doc_id, attestation).await -> EraseReceipt`
- `collection.erase_subject(subject_ref, attestation).await -> EraseReceipt`
- `contextra.drop_collection(collection_id).await -> Result<()>`

## 4 Invarianten

- Dünne Façade über `contextra-db` und `contextra-erasure`.
- `#![forbid(unsafe_code)]`.

# Contextra — Mikrofeingranulare Implementierungsspezifikation EP2: Contextra Proof

**Dokumenten-ID:** `DOC-SPEC-EP2-PROOF-2026-10-14`  
**Version:** 1.0 · **Datum:** 14. Oktober 2026  
**Autor:** Principal Senior Rust Architect für Contextra  
**Status:** Verbindliche, normative Implementierungsanweisung für das Löschnachweis-Subsystem  
**Geltungsbereich (Disjunktive Crates):**  
- **Ring 0 (Krypto-Kern):** [`contextra-crypto`](file:///home/freddy/Projekte/contextra/crates/contextra-crypto)  
- **Ring 1 (Assured Mode Plugin):** `contextra-erasure` (Crate-Split aus Store/Engine)  
- **Ring 3 (Attested Mode Plugin):** `contextra-evidence` (Crate-Split aus Crypto/Engine)  
- **Ring 0 (Autonomer Verifier):** `contextra-verify` (I/O-freier Standalone-Prüfer)  
- **Ring 0 (Compliance):** [`contextra-audit-export`](file:///home/freddy/Projekte/contextra/crates/contextra-audit-export), [`contextra-avv-generator`](file:///home/freddy/Projekte/contextra/crates/contextra-avv-generator)

---

## 1. Architektonische Grenzziehung & Disjunktion

Contextra Proof (EP2) implementiert die mathematisch und kryptografisch beweisbare Löschung (DSGVO Art. 17, ZPO § 371a) über die **Zero-Cost-Plugin-Doktrin**.

**Disjunktionsregeln für EP2:**
1. **Kein Basisspeicher:** EP2 enthält **keinen** eigenen LSM-Tree oder Vektorindex. Es bindet sich rein über die in `contextra-ports::erasure` deklarierten Traits an EP1 an.
2. **Autonomie des Verifiers:** `contextra-verify` darf **weder** `contextra-store`, `contextra-engine`, noch `tokio` als Abhängigkeit führen. Die Verifikation eines Beweises muss offline und in $< 500\,\mu\text{s}$ rein deterministisch erfolgen.
3. **Klare Trennung der Modi:**
   - `ErasureMode::Off`: 0 ns Overhead im Schreibpfad, keine Beweiscookies.
   - `ErasureMode::Assured`: Physischer Purge, Anti-Resurrection-Ledger, Schreib-Fence Option B.
   - `ErasureMode::Attested`: Echte 7-Schicht-Scans, Ed25519-Signatur, RFC 3161 Zeitanker.

---

## 2. Modul- und Datei-Feinspezifikationen

### 2.1 Komponente 1: KMS Provider Integration & Lock-Free Gruppen-Revokation (`contextra-crypto`)

#### Problem
1. **FINDING-CRYPTO-01:** `KmsProvider` (`src/kms.rs:18`) und `LocalKmsProvider` (`src/kms.rs:58`) sind in keinem Composition-Root-Crate verdrahtet (`orphan-symbols`).
2. **FEHLER-A1:** `kv_shredding.rs:247` enthält ein offenes TODO bezüglich Lock-freier `is_group_revoked`-Prüfung.
3. **FEHLER-D1:** Ungenutzte `pub fn` `default_signature_version` und `mandatory_ids`.

#### Datei: `crates/contextra-crypto/src/kv_shredding.rs`
Ersetzung des TODO durch atomare Bitmaske für lock-freie Gruppen-Revokationsprüfungen im Hot-Path:
```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct LockFreeGroupRevocationRegistry {
    /// 64 Gruppen pro AtomicU64 Wort (Bitmap)
    bitmap: Arc<[AtomicU64; 16]>, // Bis zu 1024 Schlüsselgruppen
}

impl LockFreeGroupRevocationRegistry {
    pub fn new() -> Self {
        const INIT: AtomicU64 = AtomicU64::new(0);
        Self {
            bitmap: Arc::new([INIT; 16]),
        }
    }

    /// O(1) Lock-freie Prüfung im Lese-/Schreibpfad (Zero-Lock Invariante)
    #[inline(always)]
    pub fn is_group_revoked(&self, group_id: u16) -> bool {
        let idx = (group_id / 64) as usize;
        let bit = group_id % 64;
        if idx >= self.bitmap.len() {
            return true; // Fail-Closed bei unbekannter Gruppe
        }
        let word = self.bitmap[idx].load(Ordering::Acquire);
        (word & (1u64 << bit)) != 0
    }

    /// Markiert eine Gruppe atomar und unwiderruflich als vernichtet (Crypto-Shredding)
    pub fn revoke_group(&self, group_id: u16) {
        let idx = (group_id / 64) as usize;
        let bit = group_id % 64;
        if idx < self.bitmap.len() {
            self.bitmap[idx].fetch_or(1u64 << bit, Ordering::SeqCst);
        }
    }
}
```

#### Datei: `crates/contextra-crypto/src/deletion_proof/proof.rs` & `exclusions.rs`
Bereinigung der Orphan-Funktionen:
```rust
// In crates/contextra-crypto/src/deletion_proof/proof.rs:
// Ändere von `pub fn default_signature_version` auf `pub(crate)`:
#[inline(always)]
pub(crate) fn default_signature_version() -> u8 {
    4
}

// In crates/contextra-crypto/src/deletion_proof/exclusions.rs:
// Ändere von `pub fn mandatory_ids` auf `pub(crate)`:
pub(crate) fn mandatory_ids() -> &'static [LayerId] {
    &[LayerId::LSM_MANIFEST, LayerId::WAL_HMAC, LayerId::HNSW_VECTOR]
}
```

---

### 2.2 Komponente 2: Anti-Resurrection Ledger & Streaming `RestoreGuard` (`contextra-erasure`)

#### Architektur
Der `ErasureLedger` speichert physische Lösch-Intents persistent. Ein Backup oder Crash-Recovery-Vorgang muss vor dem Wiederherstellen gelöschter Schlüssel warnen oder diese abfangen.

#### Blueprint: `crates/contextra-erasure/src/ledger.rs`
```rust
use std::path::{Path, PathBuf};
use contextra_ports::erasure::{PhysicalScope, ErasureMode};
use contextra_types::{ContextraError, Result, TxId};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LedgerEntry {
    pub intent_id: [u8; 16],
    pub as_of_tx: TxId,
    pub scope: PhysicalScope,
    pub timestamp_nanos: u64,
}

pub struct AntiResurrectionLedger {
    ledger_path: PathBuf,
}

impl AntiResurrectionLedger {
    pub fn open(store_dir: &Path) -> Result<Self> {
        let ledger_path = store_dir.join("erasure_ledger.v1");
        Ok(Self { ledger_path })
    }

    /// Schreibt den Lösch-Intent persistent VOR dem Tombstone-Commit (INV-DELETION-1)
    pub fn append_intent(&self, entry: &LedgerEntry) -> Result<()> {
        let bytes = bincode::serialize(entry)
            .map_err(|e| ContextraError::Serialization(e.to_string()))?;
        
        let frame_len = (bytes.len() as u32).to_le_bytes();
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.ledger_path)?;
            
        use std::io::Write;
        file.write_all(&frame_len)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        Ok(())
    }

    /// Verhindert die Re-Surrection gelöschter Daten beim Restore älterer SSTables
    pub fn create_restore_guard(&self) -> Result<RestoreGuard> {
        let mut tombstones = Vec::new();
        if self.ledger_path.exists() {
            let data = std::fs::read(&self.ledger_path)?;
            let mut cursor = 0;
            while cursor + 4 <= data.len() {
                let len = u32::from_le_bytes(data[cursor..cursor+4].try_into().unwrap()) as usize;
                cursor += 4;
                if cursor + len <= data.len() {
                    let entry: LedgerEntry = bincode::deserialize(&data[cursor..cursor+len])
                        .map_err(|e| ContextraError::Serialization(e.to_string()))?;
                    tombstones.push(entry);
                    cursor += len;
                }
            }
        }
        Ok(RestoreGuard { tombstones })
    }
}

pub struct RestoreGuard {
    tombstones: Vec<LedgerEntry>,
}

impl RestoreGuard {
    pub fn should_reject_key(&self, key: &[u8], tx_id: TxId) -> bool {
        for t in &self.tombstones {
            if t.as_of_tx >= tx_id && t.scope.matches_key(key) {
                return true;
            }
        }
        false
    }
}
```

---

### 2.3 Komponente 3: Echte 7-Schicht-Scans (P-E4 Fail-Closed, `contextra-evidence`)

#### Architektur
Gemäß P-E4 werden keine fabrizierten Zeugen akzeptiert. Jede Komponente stellt einen echten `LayerScan` bereit:

| Schicht-ID | Schicht | Scan-Gegenstand | Realer Prüfpfad |
|---|---|---|---|
| **`01`** | `LSM_MANIFEST` | Aktive SSTables | Manifest-Scan: Kein aktives SSTable referenziert Schlüssel |
| **`02`** | `WAL_HMAC` | Aktive WAL-Frames | Scan durch aktives WAL: Keine unkompaktierte Mutation |
| **`03`** | `HNSW_VECTOR` | Vektor-Arena | Arena-Scan: Slot genullt, keine Kanten im Graph |
| **`04`** | `BM25_TEXT` | Invertierter Index | Posting-List Scan: Keine Postings für Doc-ID |
| **`05`** | `CSR_GRAPH` | CSR-Adjazenz | Kanten-Scan: Keine Kanten zu Knoten-ID |
| **`06`** | `KMS_KEY` | KMS-Schlüsselstatus | KeyManager: Key-Revokation verifiziert (`RevocationLog`) |
| **`40`** | `EGRESS_AUDIT` | Egress-Kette | AuditChain-Scan: Kein Klartext im Egress-Protokoll |

#### Blueprint: `crates/contextra-evidence/src/issuer.rs`
```rust
pub struct ProofIssuer {
    signer: Arc<dyn ProofSigner>,
    scanners: Vec<Box<dyn LayerScan>>,
}

impl ProofIssuer {
    pub async fn issue_v4_proof(
        &self,
        scope: &PhysicalScope,
        as_of_tx: TxId,
    ) -> Result<ProofV4, ContextraError> {
        let mut layer_observations = Vec::new();

        // Alle 7 Layer zwingend scannen (Fail-Closed)
        for scanner in &self.scanners {
            let obs = scanner.scan(scope, as_of_tx).await?;
            if !obs.clean {
                return Err(ContextraError::erasure_verification_failed(format!(
                    "Layer {:?} contains residual data for scope", obs.layer
                )));
            }
            layer_observations.push(obs);
        }

        // Deterministische v4-TLV Kodierung und Ed25519-Signierung
        let tlv_payload = canon::encode_v4_tlv(scope, as_of_tx, &layer_observations)?;
        let signature = self.signer.sign(&tlv_payload)?;

        Ok(ProofV4 {
            version: 4,
            as_of_tx,
            scope: scope.clone(),
            observations: layer_observations,
            signature,
        })
    }
}
```

---

### 2.4 Komponente 4: Autonomer Offline-Verifier (`contextra-verify`)

#### Anforderung
Stand-alone CLI- und Library-Crate ohne I/O, ohne Datenbanklaufzeit, mit minimalen Abhängigkeiten (`ed25519-dalek`, `blake3`).

#### Datei: `crates/contextra-verify/src/lib.rs`
```rust
//! Standalone Verifier für Contextra v4 Löschnachweise.
use ed25519_dalek::{Signature, VerifyingKey};

pub struct StandaloneVerifier;

impl StandaloneVerifier {
    /// Offline-Verifikation ohne Systemzugriff
    pub fn verify_v4_proof(
        raw_proof_bytes: &[u8],
        trusted_public_key: &[u8; 32],
    ) -> Result<VerificationOutcome, VerifyError> {
        let verifying_key = VerifyingKey::from_bytes(trusted_public_key)
            .map_err(|_| VerifyError::InvalidPublicKey)?;

        // 1. TLV Header & Domain Separation Tag prüfen
        if raw_proof_bytes.len() < 72 || &raw_proof_bytes[0..4] != b"CXPR" {
            return Err(VerifyError::InvalidMagic);
        }

        let signed_len = raw_proof_bytes.len() - 64;
        let signed_payload = &raw_proof_bytes[..signed_len];
        let sig_bytes: [u8; 64] = raw_proof_bytes[signed_len..].try_into().unwrap();
        let signature = Signature::from_bytes(&sig_bytes);

        // 2. Mathematische Ed25519-Signaturprüfung
        verifying_key.verify(signed_payload, &signature)
            .map_err(|_| VerifyError::SignatureInvalid)?;

        // 3. Schicht-Vollständigkeit validieren (Pflicht-Layer 1, 2, 3, 6)
        let manifest_info = parse_and_validate_layers(signed_payload)?;

        Ok(VerificationOutcome {
            valid: true,
            as_of_tx: manifest_info.as_of_tx,
            scanned_layers: manifest_info.layer_count,
        })
    }
}
```

---

### 2.5 Komponente 5: MCP `contextra_forget` Durchreichung (Behebung B4)

#### Problem
In `crates/contextra-mcp/src/server_tools.rs` wurde `erase_document` aufgerufen, das Ergebnis jedoch verworfen und `{ "proof": null }` zurückgegeben.

#### Datei: `crates/contextra-mcp/src/tools_crud.rs`
```rust
pub async fn handle_forget(
    db: &ContextraDb,
    tenant_id: TenantId,
    doc_id: DocId,
) -> Result<serde_json::Value, McpError> {
    // Führt Löschung aus und empfängt den echten Beweis
    let erasure_outcome = db.forget_document(tenant_id, doc_id).await
        .map_err(|e| McpError::internal(e.to_string()))?;

    // Beweis wird unverändert an den Aufrufer als Base64/Hex exponiert
    let proof_hex = erasure_outcome.proof.map(|p| hex::encode(p.to_bytes()));

    Ok(serde_json::json!({
        "status": "forgotten",
        "doc_id": doc_id.to_string(),
        "mode": format!("{:?}", erasure_outcome.mode),
        "proof": proof_hex, // Nicht mehr null!
        "reclaimed_bytes": erasure_outcome.bytes_reclaimed,
    }))
}
```

---

### 2.6 Komponente 6: DSGVO Art. 30 & Art. 28 Builder-Verdrahtung

#### Datei: `crates/contextra/src/builder.rs`
```rust
impl ContextraBuilder {
    pub fn with_audit_export(mut self, enabled: bool) -> Self {
        self.audit_export_enabled = enabled;
        self
    }

    pub fn with_avv_generator(mut self, enabled: bool) -> Self {
        self.avv_generator_enabled = enabled;
        self
    }

    // In build():
    // Initialisiert contextra_audit_export::AuditRegisterExporter und
    // contextra_avv_generator::AvvContext, falls aktiviert.
}
```

---

## 3. Akzeptanzkriterien & Qualitätsgates (EP2)

1. **Löschnachweis-Gate:**
   ```bash
   cargo xtask deletion-guarantee-gate
   ```
   Muss mit Exit-Code 0 alle Schichtscans bestätigen.
2. **Offline-Verifikation:**
   Ein generierter v4-Beweis muss via `contextra-verify` in einem isolierten Prozess ohne Netzwerk und ohne Disk-I/O erfolgreich verifiziert werden.
3. **Tampering-Resistenz:**
   Ein manipulierte Byte im Beweis muss sofort zum Verifikationsfehler `SignatureInvalid` führen.

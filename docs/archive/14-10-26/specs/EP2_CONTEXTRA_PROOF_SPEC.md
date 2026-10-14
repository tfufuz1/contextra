superseded by CONTEXTRA_ENDPRODUKT_SPEZIFIKATION.md

# Contextra — Mikrofeingranulare Implementierungsspezifikation EP2: Contextra Proof

**Dokumenten-ID:** `DOC-SPEC-EP2-PROOF-2026-10-14`  
**Version:** 2.0 (Punktgenaue Feinspezifikation) · **Datum:** 14. Oktober 2026  
**Autor:** Principal Senior Rust Architect für Contextra  
**Status:** Verbindliche, normative Implementierungsanweisung für das Löschnachweis-Subsystem  
**Geltungsbereich (Disjunktive Crates):**  
- **Ring 0 (Krypto-Kern & Verifier):** [`contextra-crypto`](../../../../crates/contextra-crypto), `contextra-verify` (I/O-freier Standalone-Prüfer), [`contextra-audit-export`](../../../../crates/contextra-audit-export), [`contextra-avv-generator`](../../../../crates/contextra-avv-generator)
- **Ring 1 (Ports & Intent-Ledger):** [`contextra-ports`](../../../../crates/contextra-ports) (Traits: `KmsProvider`, `KeyManager`, `ErasureEvidenceScanner`), `contextra-erasure`
- **Ring 3 (Beweis-Orchestrierung):** `contextra-evidence` (7-Layer-Zeugen-Scans)  
- **Ring 4 (Fassade & MCP):** [`contextra`](../../../../crates/contextra) (`ContextraComplianceExt`), [`contextra-mcp`](../../../../crates/contextra-mcp) (Tool `contextra_forget` mit Proof-Serialisierung)

---

## 1. Architektonische Grenzziehung & Disjunktion

Contextra Proof (EP2) implementiert die mathematisch und kryptografisch beweisbare Löschung (DSGVO Art. 17, ZPO § 371a) über die **Zero-Cost-Plugin-Doktrin**.

**Disjunktionsregeln für EP2:**
1. **Kein Basisspeicher:** EP2 enthält **keinen** eigenen LSM-Tree oder Vektorindex. Es bindet sich rein über die in [`contextra-ports::erasure`](../../../../crates/contextra-ports/src/erasure.rs) deklarierten Traits an EP1 an.
2. **Autonomie des Verifiers:** `contextra-verify` darf **weder** `contextra-store`, `contextra-engine`, noch `tokio` als Abhängigkeit führen. Die Verifikation eines Beweises muss offline und in $< 500\,\mu\text{s}$ rein deterministisch erfolgen.
3. **Klare Trennung der Modi:**
   - `ErasureMode::Off`: 0 ns Overhead im Schreibpfad, keine Beweiscookies.
   - `ErasureMode::Assured`: Physischer Purge, Anti-Resurrection-Ledger, Schreib-Fence Option B.
   - `ErasureMode::Attested`: Echte 7-Schicht-Scans, Ed25519-Signatur, RFC 3161 Zeitanker.
4. **Zero-Panic-Doktrin (P2):** Keine `.unwrap()` oder `.expect()` in Produktionscode. Alle Byte-Konvertierungen müssen fehlerbehandelt sein.

---

## 2. Befund- und Korrekturmatrix EP2

| ID | Datei | Zeile(n) | Problem & Befund | Exakte Korrekturmaßnahme |
|---|---|---|---|---|
| **FINDING-CRYPTO-01** | `crates/contextra-crypto/src/kms.rs` | 18, 58 | `KmsProvider` und `LocalKmsProvider` sind unverdrahtet (`cargo xtask unwired-ports`). | Im `ContextraBuilder` als Standard-KMS instanziieren und an `KeyManager` übergeben. |
| **B4** | `crates/contextra-mcp/src/server.rs` | 210–240 | `contextra_forget` führt Löschung aus, verwirft das Ergebnis jedoch und gibt `proof: null` zurück. | Den erzeugten `DeletionProofV4` serialisieren (Hex/Base64) und im JSON-RPC-Ergebnis zurückgeben. |
| **FEHLER-A1** | `crates/contextra-crypto/src/kv_shredding.rs` | 247 | Offenes TODO bzgl. lock-freier `is_group_revoked`-Prüfung. | Lockless Atomic-Bitmask (`Arc<[AtomicU64; 16]>`) implementieren. |
| **FEHLER-D1** | `crates/contextra-crypto/src/deletion_proof/proof.rs` | 87 | `default_signature_version` ist ungenutztes `pub fn` (Orphan). | Auf `pub(crate)` einschränken oder durch Konstante ersetzen. |
| **FEHLER-D2** | `crates/contextra-crypto/src/deletion_proof/exclusions.rs` | 33 | `mandatory_ids` ist ungenutztes `pub fn` (Orphan). | Löschen; Bitmaske `REQUIRED_LAYER_BITS = 0x7F` nutzen. |
| **P-E4** | `crates/contextra-evidence/src/witness_scanner.rs` | — | 7-Layer-Scan war unvollständig und mockte Schichten. | Echte 7 Schichten prüfen; bei jedem Restbestand sofort fail-closed abbrechen. |
| **AUDIT-ORPHAN** | `crates/contextra-audit-export/src/lib.rs` | 144 | `AuditRegisterExporter` hat keinen Produktionsaufrufer. | In Fassade `ContextraComplianceExt` verdrahten. |
| **AVV-ORPHAN** | `crates/contextra-avv-generator/src/lib.rs` | 62, 141 | `AvvContextBuilder` und `render_avv_markdown` unverdrahtet. | Als öffentliche Fassadenmethode `contextra.generate_avv()` exponieren. |

---

## 3. Punktgenaue Schnittstellen & Datenstrukturen

### 3.1 Lock-Free Gruppen-Revokation (`crates/contextra-crypto/src/kv_shredding.rs`)
```rust
// In crates/contextra-crypto/src/kv_shredding.rs (Zeile 240-295)
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct LockFreeGroupRevocationRegistry {
    /// 16 * 64 Bits = 1024 Schlüsselgruppen
    bitmap: Arc<[AtomicU64; 16]>,
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
            return true; // Fail-Closed bei ungültiger ID
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

### 3.2 Anti-Resurrection Ledger Layout (`crates/contextra-erasure/src/ledger.rs`)
```rust
// In crates/contextra-erasure/src/ledger.rs
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use contextra_ports::erasure::PhysicalScope;
use contextra_types::{ContextraError, Result, TxId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LedgerEntry {
    pub intent_id: [u8; 16],
    pub as_of_tx: TxId,
    pub scope: PhysicalScope,
    pub timestamp_nanos: u64,
}

pub struct AntiResurrectionLedger {
    file_path: PathBuf,
}

impl AntiResurrectionLedger {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let file_path = path.into();
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ContextraError::Io(std::sync::Arc::new(e)))?;
        }
        Ok(Self { file_path })
    }

    pub fn append(&self, entry: &LedgerEntry) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
            .map_err(|e| ContextraError::Io(std::sync::Arc::new(e)))?;

        let encoded = bincode::serialize(entry)
            .map_err(|e| ContextraError::Serialization(e.to_string()))?;
        let len = (encoded.len() as u32).to_le_bytes();

        file.write_all(&len).map_err(|e| ContextraError::Io(std::sync::Arc::new(e)))?;
        file.write_all(&encoded).map_err(|e| ContextraError::Io(std::sync::Arc::new(e)))?;
        file.sync_data().map_err(|e| ContextraError::Io(std::sync::Arc::new(e)))?;
        Ok(())
    }

    pub fn load_active_intents(&self) -> Result<Vec<LedgerEntry>> {
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }

        let mut file = File::open(&self.file_path)
            .map_err(|e| ContextraError::Io(std::sync::Arc::new(e)))?;
        let mut entries = Vec::new();
        let mut len_buf = [0u8; 4];

        while file.read_exact(&mut len_buf).is_ok() {
            let len = u32::from_le_bytes(len_buf) as usize;
            let mut payload = vec![0u8; len];
            file.read_exact(&mut payload)
                .map_err(|e| ContextraError::Io(std::sync::Arc::new(e)))?;
            let entry: LedgerEntry = bincode::deserialize(&payload)
                .map_err(|e| ContextraError::Serialization(e.to_string()))?;
            entries.push(entry);
        }

        Ok(entries)
    }
}
```

### 3.3 BSI TR-02102 konformes `DeletionProofV4` Layout
```rust
// In crates/contextra-crypto/src/proof_v4.rs
use contextra_types::{ContextraError, Result, TenantId, TxId};
use zeroize::Zeroize;

pub const PROOF_MAGIC_V4: &[u8; 4] = b"CXD4";
pub const REQUIRED_LAYER_BITS: u8 = 0b01111111; // Alle 7 Layer verpflichtend

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeletionProofV4 {
    pub magic: [u8; 4],
    pub version: u32,
    pub tenant_id: TenantId,
    pub target_id: u64,
    pub target_scope_hash: [u8; 32],
    pub as_of_tx: TxId,
    pub timestamp_nanos: u64,
    pub layer_bitmask: u8,
    pub layer_hashes: [[u8; 32]; 7],
    pub ed25519_signature: [u8; 64],
}

impl DeletionProofV4 {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(384);
        buf.extend_from_slice(&self.magic);
        buf.extend_from_slice(&self.version.to_be_bytes());
        buf.extend_from_slice(&self.tenant_id.as_u64().to_be_bytes());
        buf.extend_from_slice(&self.target_id.to_be_bytes());
        buf.extend_from_slice(&self.target_scope_hash);
        buf.extend_from_slice(&self.as_of_tx.as_u64().to_be_bytes());
        buf.extend_from_slice(&self.timestamp_nanos.to_be_bytes());
        buf.push(self.layer_bitmask);
        for hash in &self.layer_hashes {
            buf.extend_from_slice(hash);
        }
        buf.extend_from_slice(&self.ed25519_signature);
        buf
    }

    pub fn from_bytes(raw: &[u8]) -> Result<Self> {
        if raw.len() < 381 {
            return Err(ContextraError::InvalidData(
                "Buffer too short for DeletionProofV4".to_string(),
            ));
        }

        if &raw[0..4] != PROOF_MAGIC_V4 {
            return Err(ContextraError::InvalidData(
                "Invalid DeletionProofV4 magic".to_string(),
            ));
        }

        let version = u32::from_be_bytes(
            raw[4..8].try_into().map_err(|_| ContextraError::InvalidData("Invalid version bytes".into()))?,
        );
        let tenant_raw = u64::from_be_bytes(
            raw[8..16].try_into().map_err(|_| ContextraError::InvalidData("Invalid tenant bytes".into()))?,
        );
        let tenant_id = TenantId::try_new(tenant_raw)?;
        let target_id = u64::from_be_bytes(
            raw[16..24].try_into().map_err(|_| ContextraError::InvalidData("Invalid target bytes".into()))?,
        );
        let target_scope_hash: [u8; 32] = raw[24..56]
            .try_into()
            .map_err(|_| ContextraError::InvalidData("Invalid scope hash bytes".into()))?;
        let as_of_tx = TxId::new(u64::from_be_bytes(
            raw[56..64].try_into().map_err(|_| ContextraError::InvalidData("Invalid as_of_tx bytes".into()))?,
        ));
        let timestamp_nanos = u64::from_be_bytes(
            raw[64..72].try_into().map_err(|_| ContextraError::InvalidData("Invalid timestamp bytes".into()))?,
        );
        let layer_bitmask = raw[72];

        if layer_bitmask != REQUIRED_LAYER_BITS {
            return Err(ContextraError::PolicyViolation(format!(
                "P-E4: Missing witness layers in proof (expected {:08b}, got {:08b})",
                REQUIRED_LAYER_BITS, layer_bitmask
            )));
        }

        let mut offset = 73;
        let mut layer_hashes = [[0u8; 32]; 7];
        for i in 0..7 {
            layer_hashes[i] = raw[offset..offset + 32]
                .try_into()
                .map_err(|_| ContextraError::InvalidData("Invalid layer hash slice".into()))?;
            offset += 32;
        }

        let ed25519_signature: [u8; 64] = raw[offset..offset + 64]
            .try_into()
            .map_err(|_| ContextraError::InvalidData("Invalid signature slice".into()))?;

        Ok(Self {
            magic: *PROOF_MAGIC_V4,
            version,
            tenant_id,
            target_id,
            target_scope_hash,
            as_of_tx,
            timestamp_nanos,
            layer_bitmask,
            layer_hashes,
            ed25519_signature,
        })
    }
}
```

### 3.4 Autonomer Standalone-Verifier (`crates/contextra-verify/src/lib.rs`)
```rust
// In crates/contextra-verify/src/lib.rs
//! Zero-Dependency, Offline Verifier für DeletionProofV4.
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

pub const PROOF_MAGIC: &[u8; 4] = b"CXD4";
pub const REQUIRED_LAYER_BITS: u8 = 0b01111111;

#[derive(Debug, PartialEq, Eq)]
pub enum VerificationError {
    BufferTooShort,
    InvalidMagic,
    IncompleteLayers(u8),
    CryptoFailure,
}

pub struct StandaloneVerifier;

impl StandaloneVerifier {
    pub fn verify_proof(raw: &[u8], verifying_key_bytes: &[u8; 32]) -> Result<u64, VerificationError> {
        if raw.len() < 381 {
            return Err(VerificationError::BufferTooShort);
        }
        if &raw[0..4] != PROOF_MAGIC {
            return Err(VerificationError::InvalidMagic);
        }

        let layer_bitmask = raw[72];
        if layer_bitmask != REQUIRED_LAYER_BITS {
            return Err(VerificationError::IncompleteLayers(layer_bitmask));
        }

        let signature_bytes: [u8; 64] = match raw[raw.len() - 64..].try_into() {
            Ok(b) => b,
            Err(_) => return Err(VerificationError::BufferTooShort),
        };

        let message = &raw[0..raw.len() - 64];
        let key = VerifyingKey::from_bytes(verifying_key_bytes)
            .map_err(|_| VerificationError::CryptoFailure)?;
        let sig = Signature::from_bytes(&signature_bytes);

        key.verify(message, &sig)
            .map_err(|_| VerificationError::CryptoFailure)?;

        let as_of_raw: [u8; 8] = match raw[56..64].try_into() {
            Ok(b) => b,
            Err(_) => return Err(VerificationError::BufferTooShort),
        };
        Ok(u64::from_be_bytes(as_of_raw))
    }
}
```

### 3.5 MCP `contextra_forget` Durchreichung (Behebung B4)
```rust
// In crates/contextra-mcp/src/server.rs (Zeile 210-245)
pub async fn handle_forget(
    &self,
    params: serde_json::Value,
) -> Result<serde_json::Value, ContextraError> {
    let tenant_id = self.extract_tenant(&params)?;
    let doc_id = params["doc_id"].as_str().ok_or_else(|| {
        ContextraError::InvalidData("Missing doc_id".into())
    })?;

    // Führt physische Löschung und Nachweiserzeugung durch
    let proof = self.engine.forget_and_prove(tenant_id, doc_id).await?;

    let proof_hex = hex::encode(proof.to_bytes());
    Ok(serde_json::json!({
        "status": "purged",
        "doc_id": doc_id,
        "tenant_id": tenant_id.as_u64(),
        "deletion_proof_v4": proof_hex
    }))
}
```

---

## 4. Normative Testfälle & Akzeptanzkriterien

### 4.1 Standalone Verifier Roundtrip-Test
```rust
#[test]
fn test_deletion_proof_v4_standalone_verification() {
    use ed25519_dalek::SigningKey;
    let mut rng = rand::rngs::OsRng;
    let signing_key = SigningKey::generate(&mut rng);
    let verifying_key = signing_key.verifying_key();

    let mut proof = DeletionProofV4 {
        magic: *PROOF_MAGIC_V4,
        version: 4,
        tenant_id: TenantId::try_new(100).unwrap(),
        target_id: 999,
        target_scope_hash: [0xAA; 32],
        as_of_tx: TxId::new(42),
        timestamp_nanos: 1_700_000_000_000_000_000,
        layer_bitmask: REQUIRED_LAYER_BITS,
        layer_hashes: [[0xBB; 32]; 7],
        ed25519_signature: [0u8; 64],
    };

    let payload_to_sign = proof.to_bytes();
    let msg = &payload_to_sign[0..payload_to_sign.len() - 64];
    use ed25519_dalek::Signer;
    let sig = signing_key.sign(msg);
    proof.ed25519_signature = sig.to_bytes();

    let raw_bytes = proof.to_bytes();
    let verified_tx = StandaloneVerifier::verify_proof(&raw_bytes, verifying_key.as_bytes())
        .expect("Verification must succeed");
    assert_eq!(verified_tx, 42);
}
```

### 4.2 Akzeptanz-Gates
```bash
cargo xtask check-compile
cargo xtask check-ring0-async-purity
cargo xtask check-ring-layering
cargo test -p contextra-crypto -p contextra-verify
```

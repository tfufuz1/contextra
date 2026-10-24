// FILE-CONTEXT
// ZWECK: Cryptographic Blake3 audit hash chain with periodic Ed25519 signatures and encrypted commitments.
// INVARIANTEN: Audit chain entries strictly exclude personal data. Chain links are linked via Blake3 hashes.
// NICHT-OFFENSICHTLICH: Commitments are Blake3(salt || attr) where salt is encrypted via key shredding.
// HOTSPOTS: [AuditChain, AuditChainEntry, compute_record_commitment]

#![forbid(unsafe_code)]

//! Audit-Log als Blake3-Hash-Kette mit periodischer Ed25519-Signatur.
//!
//! DSGVO Art. 17 / Art. 30 Compliance:
//! - Jeder Schreibvorgang erzeugt einen AuditChainEntry ohne Personenbezug.
//! - Optionale Commitments Blake3(salt || attr), wobei salt unter einem Shredding-Schlüssel
//!   verschlüsselt gespeichert ist. Nach dem Schreddern des Schlüssels ist das salt unöffenbar,
//!   womit das Commitment nicht mehr auf Originalattribute zurückführbar ist (Wörterbuchangriff
//!   unmöglich, da salt unbekannt).

use crate::crypto::KeyManager;
use crate::error::{CryptoError, Result};
use crate::kv_shredding::KeyRegistry;
use contextra_types::{DocId, TxId};
use serde::{Deserialize, Serialize};

/// Datenklassifikation für Audit-Einträge (keine Freitext-Strings).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataClass {
    /// Öffentliche Daten.
    Public,
    /// Interne Betriebsdaten.
    Internal,
    /// Vertrauliche Geschäftsdaten.
    Confidential,
    /// Streng vertrauliche / reglementierte Daten.
    Restricted,
    /// Personenbezogene Daten (im Audit-Log nur pseudonymisiert/entkoppelt).
    Personal,
    /// Besondere Kategorien personenbezogener Daten (Art. 9 DSGVO).
    SpecialCategoryPersonal,
}

/// Einzelner Eintrag in der Audit-Hash-Kette.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditChainEntry {
    /// Fortlaufender Index in der Audit-Kette (0-basiert).
    pub index: u64,
    /// Identifikator des Datenschemas.
    pub schema_id: String,
    /// Blake3-Hash des zum Zeitpunkt des Schreibvorgangs geltenden Regelsatzes.
    pub rules_hash: [u8; 32],
    /// Datenklasse des geschriebenen Datensatzes.
    pub data_class: DataClass,
    /// Zugehörige Dokument-ID.
    pub doc_id: DocId,
    /// Zugehörige Transaktions-ID.
    pub tx_id: TxId,
    /// Blake3-Hash des Vorgängereintrags in der Kette (oder [0; 32] für den Genesis-Eintrag).
    pub prev_hash: [u8; 32],
    /// Optionales Blake3(salt || attr) Commitment.
    pub commitment: Option<[u8; 32]>,
    /// Blake3-Hash dieses Eintrags über alle vorgenannten Felder.
    pub entry_hash: [u8; 32],
}

impl AuditChainEntry {
    /// Berechnet den Blake3-Hash für die Felder eines Audit-Ketten-Eintrags.
    #[allow(clippy::too_many_arguments)]
    pub fn compute_hash(
        index: u64,
        schema_id: &str,
        rules_hash: &[u8; 32],
        data_class: DataClass,
        doc_id: DocId,
        tx_id: TxId,
        prev_hash: &[u8; 32],
        commitment: Option<&[u8; 32]>,
    ) -> Result<[u8; 32]> {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&index.to_le_bytes());
        hasher.update(&(schema_id.len() as u32).to_le_bytes());
        hasher.update(schema_id.as_bytes());
        hasher.update(rules_hash);

        let class_bytes = bincode::serialize(&data_class)
            .map_err(|e| CryptoError::Crypto(format!("DataClass serialization failed: {e}")))?;
        hasher.update(&(class_bytes.len() as u32).to_le_bytes());
        hasher.update(&class_bytes);

        hasher.update(&doc_id.0.to_le_bytes());
        hasher.update(&tx_id.0.to_le_bytes());
        hasher.update(prev_hash);

        if let Some(comm) = commitment {
            hasher.update(&[1u8]);
            hasher.update(comm);
        } else {
            hasher.update(&[0u8]);
        }

        Ok(*hasher.finalize().as_bytes())
    }
}

/// Periodisch erzeugte Ed25519-Signatur über den aktuellen Kettenkopf.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditChainHeadSignature {
    /// Index des signierten Kettenkopfs.
    pub chain_index: u64,
    /// Blake3-Hash des Kettenkopfs.
    pub head_hash: [u8; 32],
    /// Ed25519-Signatur (64 Bytes).
    pub signature: Vec<u8>,
}

/// Verschlüsselter Salt-Wert für ein Record-Commitment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedCommitmentSalt {
    /// Shredding Group ID zur Verwaltung des Sub-Keys.
    pub group_id: u64,
    /// Verschlüsselter Salt.
    pub ciphertext: Vec<u8>,
    /// AES-GCM-SIV Nonce (12 Bytes).
    pub nonce: [u8; 12],
}

impl EncryptedCommitmentSalt {
    /// Verschlüsselt ein Salt-Byte-Array über den `KeyRegistry`-Shredding-Mechanismus.
    pub fn encrypt(
        registry: &KeyRegistry,
        master_key: &KeyManager,
        group_id: u64,
        salt: &[u8],
    ) -> Result<Self> {
        let (ciphertext, nonce) = registry.encrypt_with_group(master_key, group_id, salt)?;
        Ok(Self {
            group_id,
            ciphertext,
            nonce,
        })
    }

    /// Entschlüsselt das Salt. Schlägt fehl, wenn der Sub-Key für `group_id` geschreddert/widerrufen wurde.
    pub fn decrypt(&self, registry: &KeyRegistry) -> Result<Vec<u8>> {
        registry.decrypt_with_group(self.group_id, &self.ciphertext, &self.nonce)
    }
}

/// Berechnet das Commitment `Blake3(salt || attr)`.
pub fn compute_record_commitment(salt: &[u8], attr: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(salt);
    hasher.update(attr);
    *hasher.finalize().as_bytes()
}

/// Manipulationssichere Blake3-Hash-Kette für Audit-Einträge.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditChain {
    /// Öffentlicher Zugriff auf die Liste der Ketten-Einträge für Prüfungen/Tests.
    pub entries: Vec<AuditChainEntry>,
    head_hash: [u8; 32],
}

impl AuditChain {
    /// Erzeugt eine neue, leere Audit-Kette.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            head_hash: [0u8; 32],
        }
    }

    /// Fügt einen neuen Eintrag zur Audit-Kette hinzu und aktualisiert den Kettenkopf-Hash.
    #[allow(clippy::too_many_arguments)]
    pub fn append(
        &mut self,
        schema_id: impl Into<String>,
        rules_hash: [u8; 32],
        data_class: DataClass,
        doc_id: DocId,
        tx_id: TxId,
        commitment: Option<[u8; 32]>,
    ) -> Result<&AuditChainEntry> {
        let index = self.entries.len() as u64;
        let schema_id = schema_id.into();
        let prev_hash = self.head_hash;

        let entry_hash = AuditChainEntry::compute_hash(
            index,
            &schema_id,
            &rules_hash,
            data_class,
            doc_id,
            tx_id,
            &prev_hash,
            commitment.as_ref(),
        )?;

        let entry = AuditChainEntry {
            index,
            schema_id,
            rules_hash,
            data_class,
            doc_id,
            tx_id,
            prev_hash,
            commitment,
            entry_hash,
        };

        self.head_hash = entry_hash;
        self.entries.push(entry);

        let last_idx = self.entries.len() - 1;
        Ok(&self.entries[last_idx])
    }

    /// Liefert die Anzahl der Einträge in der Kette.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Prüft, ob die Kette leer ist.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Gibt eine Referenz auf alle Einträge der Kette zurück.
    pub fn entries(&self) -> &[AuditChainEntry] {
        &self.entries
    }

    /// Liefert den aktuellen Kettenkopf-Hash.
    pub fn head_hash(&self) -> [u8; 32] {
        self.head_hash
    }

    /// Periodischer Aufruf: Signiert den aktuellen Kettenkopf mit Ed25519.
    pub fn sign_head(
        &self,
        signing_key: &ed25519_dalek::SigningKey,
    ) -> Result<AuditChainHeadSignature> {
        let last_entry = match self.entries.last() {
            Some(e) => e,
            None => {
                return Err(CryptoError::Crypto(
                    "Cannot sign head of an empty AuditChain".to_string(),
                ))
            }
        };

        let chain_index = last_entry.index;
        let head_hash = self.head_hash;

        let mut payload = Vec::with_capacity(8 + 32);
        payload.extend_from_slice(&chain_index.to_le_bytes());
        payload.extend_from_slice(&head_hash);

        use ed25519_dalek::Signer;
        let sig = signing_key.sign(&payload);

        Ok(AuditChainHeadSignature {
            chain_index,
            head_hash,
            signature: sig.to_bytes().to_vec(),
        })
    }

    /// Verifiziert eine periodische Kettenkopf-Signatur gegen den angegebenen VerifyingKey.
    pub fn verify_head_signature(
        head_sig: &AuditChainHeadSignature,
        verifying_key: &ed25519_dalek::VerifyingKey,
    ) -> Result<bool> {
        let mut payload = Vec::with_capacity(8 + 32);
        payload.extend_from_slice(&head_sig.chain_index.to_le_bytes());
        payload.extend_from_slice(&head_sig.head_hash);

        use ed25519_dalek::Verifier;
        let sig = match ed25519_dalek::Signature::from_slice(&head_sig.signature) {
            Ok(s) => s,
            Err(_) => return Ok(false),
        };

        Ok(verifying_key.verify(&payload, &sig).is_ok())
    }

    /// Verifiziert die vollständige Kette auf Integrität.
    /// Prüft für jeden Eintrag, ob `prev_hash` dem `entry_hash` des Vorgängers entspricht
    /// und ob `entry_hash` korrekt berechnet wurde.
    pub fn verify_chain(&self) -> Result<bool> {
        let mut expected_prev = [0u8; 32];

        for (expected_idx, entry) in self.entries.iter().enumerate() {
            if entry.index != expected_idx as u64 {
                return Ok(false);
            }

            if entry.prev_hash != expected_prev {
                return Ok(false);
            }

            let computed = AuditChainEntry::compute_hash(
                entry.index,
                &entry.schema_id,
                &entry.rules_hash,
                entry.data_class,
                entry.doc_id,
                entry.tx_id,
                &entry.prev_hash,
                entry.commitment.as_ref(),
            )?;

            if computed != entry.entry_hash {
                return Ok(false);
            }

            expected_prev = entry.entry_hash;
        }

        if let Some(last) = self.entries.last() {
            if self.head_hash != last.entry_hash {
                return Ok(false);
            }
        } else if self.head_hash != [0u8; 32] {
            return Ok(false);
        }

        Ok(true)
    }
}

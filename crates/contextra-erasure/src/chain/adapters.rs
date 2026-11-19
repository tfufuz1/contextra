//! Produktionsadapter für die Ports der Löschkette.
//!
//! - [`SagaGateSink`]: bildet Ziele auf das In-Memory-[`SagaGate`] ab.
//! - [`FileVaultKeys`]: bildet Ziele auf Slots des [`FileVault`] ab.
//!
//! Subjekt-Ziele benötigen einen Subjekt→Schlüssel-Index, den dieses Crate nicht kennt.
//! Der Gate-Adapter lehnt sie deshalb ab (fail-closed), statt sie stillschweigend zu übergehen.

use std::sync::Arc;

use contextra_ids::{CollectionId, DocId, TenantId};
use parking_lot::Mutex;
use subtle::ConstantTimeEq;

use crate::error::ErasureError;
use crate::gate::saga_gate::{GateDecision, ReadGate, SagaGate};
use crate::vault::FileVault;

use super::ports::{GateMode, GateSink, KeyVault};
use super::target::EraseTarget;

/// Textschlüssel-Layout `t:{tenant}:{collection}:chunk:{doc}` (wie in den Gate-Tests).
pub struct TextKeyLayout;

impl TextKeyLayout {
    /// Präfix eines Mandanten (delimiter-terminiert).
    pub fn tenant_prefix(tenant: TenantId) -> Vec<u8> {
        format!("t:{}:", tenant.0).into_bytes()
    }

    /// Präfix einer Collection (delimiter-terminiert).
    pub fn collection_prefix(tenant: TenantId, collection: CollectionId) -> Vec<u8> {
        format!("t:{}:{}:", tenant.0, collection.0).into_bytes()
    }

    /// Exakter Schlüssel eines Dokuments.
    pub fn document_key(
        tenant: TenantId,
        collection: CollectionId,
        doc: DocId,
    ) -> Result<Vec<u8>, ErasureError> {
        let doc_text =
            serde_json::to_string(&doc).map_err(|e| ErasureError::Json(e.to_string()))?;
        Ok(format!("t:{}:{}:chunk:{}", tenant.0, collection.0, doc_text).into_bytes())
    }
}

/// Gate-Adapter auf Basis von [`SagaGate`].
pub struct SagaGateSink {
    gate: Arc<SagaGate>,
}

impl SagaGateSink {
    /// Erstellt den Adapter.
    pub fn new(gate: Arc<SagaGate>) -> Self {
        Self { gate }
    }
}

fn subject_unsupported() -> ErasureError {
    ErasureError::NotImplemented("subject erasure requires a subject-to-key index port".into())
}

impl GateSink for SagaGateSink {
    fn activate(&self, target: &EraseTarget, mode: GateMode) -> Result<(), ErasureError> {
        match (target, mode) {
            (
                EraseTarget::Document {
                    tenant,
                    collection,
                    doc,
                },
                GateMode::Erased,
            ) => self
                .gate
                .activate_key(&TextKeyLayout::document_key(*tenant, *collection, *doc)?),
            (
                EraseTarget::Document {
                    tenant,
                    collection,
                    doc,
                },
                GateMode::Restricted,
            ) => self
                .gate
                .activate_restricted_key(&TextKeyLayout::document_key(*tenant, *collection, *doc)?),
            (EraseTarget::Collection { tenant, collection }, GateMode::Erased) => self
                .gate
                .activate_prefix(&TextKeyLayout::collection_prefix(*tenant, *collection)),
            (EraseTarget::Collection { tenant, collection }, GateMode::Restricted) => self
                .gate
                .activate_restricted_prefix(&TextKeyLayout::collection_prefix(
                    *tenant,
                    *collection,
                )),
            (EraseTarget::Tenant { tenant }, GateMode::Erased) => self
                .gate
                .activate_prefix(&TextKeyLayout::tenant_prefix(*tenant)),
            (EraseTarget::Tenant { tenant }, GateMode::Restricted) => self
                .gate
                .activate_restricted_prefix(&TextKeyLayout::tenant_prefix(*tenant)),
            (EraseTarget::Subject { .. }, _) => Err(subject_unsupported()),
        }
    }

    fn lift_restricted(&self, target: &EraseTarget) -> Result<(), ErasureError> {
        match target {
            EraseTarget::Document {
                tenant,
                collection,
                doc,
            } => {
                self.gate.lift_restricted_key(&TextKeyLayout::document_key(
                    *tenant,
                    *collection,
                    *doc,
                )?);
                Ok(())
            }
            EraseTarget::Collection { tenant, collection } => {
                self.gate
                    .lift_restricted_prefix(&TextKeyLayout::collection_prefix(
                        *tenant,
                        *collection,
                    ));
                Ok(())
            }
            EraseTarget::Tenant { tenant } => {
                self.gate
                    .lift_restricted_prefix(&TextKeyLayout::tenant_prefix(*tenant));
                Ok(())
            }
            EraseTarget::Subject { .. } => Err(subject_unsupported()),
        }
    }

    fn decision(&self, target: &EraseTarget) -> GateDecision {
        let key = match target {
            EraseTarget::Document {
                tenant,
                collection,
                doc,
            } => match TextKeyLayout::document_key(*tenant, *collection, *doc) {
                Ok(k) => k,
                Err(_) => return GateDecision::Open,
            },
            EraseTarget::Collection { tenant, collection } => {
                TextKeyLayout::collection_prefix(*tenant, *collection)
            }
            EraseTarget::Tenant { tenant } => TextKeyLayout::tenant_prefix(*tenant),
            // Ohne Subjekt-Index gibt es keinen Nachweis; `Open` lässt die Messung scheitern (fail-closed).
            EraseTarget::Subject { .. } => return GateDecision::Open,
        };
        self.gate.check(&key)
    }
}

/// Schlüsseltresor-Adapter auf Basis von [`FileVault`].
///
/// Vernichtung nutzt die Tombstone-Semantik des Tresors (D-111/D-113), danach `fsync` auf alle Shards.
pub struct FileVaultKeys {
    vault: Mutex<FileVault>,
}

impl FileVaultKeys {
    /// Erstellt den Adapter über einem bestehenden Tresor.
    pub fn new(vault: FileVault) -> Self {
        Self {
            vault: Mutex::new(vault),
        }
    }

    /// Legt den Schlüssel eines Ziels an (Provisionierung).
    pub fn provision(&self, target: &EraseTarget, payload: [u8; 32]) -> Result<(), ErasureError> {
        let tag = target.vault_tag()?;
        self.vault.lock().put_slot(target.slot_type(), tag, payload)
    }
}

impl KeyVault for FileVaultKeys {
    fn destroy(&self, target: &EraseTarget) -> Result<(), ErasureError> {
        let slot = target.slot_type();
        let tag = target.vault_tag()?;
        let mut vault = self.vault.lock();
        // Idempotent: bereits fehlender Schlüssel ist kein Fehler.
        if vault.get_slot(slot, tag)?.is_none() {
            return Ok(());
        }
        vault.destroy_slot(slot, tag)?;
        vault.sync_all_shards()
    }

    fn verify_absent(&self, target: &EraseTarget) -> Result<bool, ErasureError> {
        let slot = target.slot_type();
        let tag = target.vault_tag()?;
        let vault = self.vault.lock();
        match vault.get_slot(slot, tag)? {
            None => Ok(true),
            // Konstantzeit-Vergleich: nur ein vollständig genullter Slot gilt als vernichtet.
            Some(sd) => Ok(bool::from(sd.payload.as_slice().ct_eq(&[0u8; 32][..]))),
        }
    }
}

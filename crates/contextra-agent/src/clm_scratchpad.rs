// FILE-CONTEXT
// ZWECK: Temporärer, aufgabengebundener Kontext-Scratchpad für sehr lange Agentenschleifen (Context-Language-Model).
// INVARIANTEN: Zero-Panic, pure RAM-Belegung via VolatileContextVault (mlock, Zeroize-on-Drop), kein Staging auf Disk.
// STAND: TS:2026-10-01T00:00:00Z

#![forbid(unsafe_code)]

//! Temporärer, aufgabengebundener Kontext-Scratchpad für sehr lange Agentenschleifen.
//!
//! Folgt dem Context-Language-Model-Prinzip (CLM): Der laufende Arbeitskontext ist editierbar
//! und wird bei Erfüllung eines Teilziels geleert (zurückgesetzt), anstatt append-only
//! ins Unermessliche zu wachsen.
//!
//! # Port-Entscheidung & Lokale Traits
//! `ContextEditAuditSink` und `ScratchpadCacheInvalidator` sind als schlanke, lokale Traits
//! in diesem Modul definiert, da die endgültige Entscheidung über eine Auslagerung in `contextra-ports`
//! noch aussteht (siehe ADR-106). Sie verhindern direkte Ring-Verstöße gegen `contextra-privacy`
//! und `contextra-kvcache`.

use contextra_db::volatile_vault::{
    PurgeReceipt, SignalModality, VaultChunk, VaultConfig, VolatileContextVault,
};
use contextra_ports::{Clock, SystemClock};
use contextra_types::{ContextraError, DocId, Result, TenantId, TxId};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Audit-Datensatz für Scratchpad-Kontextänderungen und Resets.
#[derive(Debug, Clone)]
pub struct ContextEditAuditRecord {
    pub tenant_id: TenantId,
    pub task_id: String,
    pub operation: String,
    pub subgoal_index: u64,
    pub timestamp_nanos: u64,
}

/// Lokaler Trait für Audit-Benachrichtigungen bei Scratchpad-Änderungen.
pub trait ContextEditAuditSink: Send + Sync {
    fn record_edit(&self, record: ContextEditAuditRecord);
}

/// No-op-Implementierung von [`ContextEditAuditSink`].
#[derive(Debug, Default)]
pub struct NoopContextEditAuditSink;

impl ContextEditAuditSink for NoopContextEditAuditSink {
    fn record_edit(&self, _record: ContextEditAuditRecord) {}
}

/// Test-Implementierung von [`ContextEditAuditSink`], die Aufrufe zählt.
#[derive(Debug, Default)]
pub struct CountingContextEditAuditSink {
    count: AtomicU64,
}

impl CountingContextEditAuditSink {
    pub fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
        }
    }

    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }
}

impl ContextEditAuditSink for CountingContextEditAuditSink {
    fn record_edit(&self, _record: ContextEditAuditRecord) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }
}

/// Lokaler Trait zur Cache-Invalidierung des Scratchpad-Scopes bei Resets.
pub trait ScratchpadCacheInvalidator: Send + Sync {
    fn invalidate_scratchpad_scope(&self, tenant_id: TenantId, task_id: &str, subgoal_index: u64);
}

/// No-op-Implementierung von [`ScratchpadCacheInvalidator`].
#[derive(Debug, Default)]
pub struct NoopScratchpadCacheInvalidator;

impl ScratchpadCacheInvalidator for NoopScratchpadCacheInvalidator {
    fn invalidate_scratchpad_scope(
        &self,
        _tenant_id: TenantId,
        _task_id: &str,
        _subgoal_index: u64,
    ) {
    }
}

/// Test-Implementierung von [`ScratchpadCacheInvalidator`], die Aufrufe zählt.
#[derive(Debug, Default)]
pub struct CountingScratchpadCacheInvalidator {
    count: AtomicU64,
}

impl CountingScratchpadCacheInvalidator {
    pub fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
        }
    }

    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }
}

impl ScratchpadCacheInvalidator for CountingScratchpadCacheInvalidator {
    fn invalidate_scratchpad_scope(
        &self,
        _tenant_id: TenantId,
        _task_id: &str,
        _subgoal_index: u64,
    ) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }
}

/// Marker für unveränderliche Kontextteile (Systemprompt, Gesamtziel, Safety-Regeln).
///
/// Wird ausschließlich gelesen und NIEMALS durch diesen Typ selbst verändert.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PinnedRegionId(pub String);

impl PinnedRegionId {
    /// Erzeugt eine neue `PinnedRegionId`.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Gibt den inneren String-Bezeichner zurück.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Operationen zur Bearbeitung des Scratchpads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScratchpadEditOp {
    /// Hängt einen neuen Kontext-Chunk an das Scratchpad an.
    Append {
        /// Byte-Inhalt des neuen Chunks.
        content: Vec<u8>,
        /// Optionaler Label-Bezeichner für spätere Referenzierung (Replace/Remove).
        label: Option<String>,
    },
    /// Ersetzt einen bestehenden Chunk identifiziert durch seinen Label-Bezeichner.
    Replace {
        /// Label-Bezeichner des zu ersetzenden Chunks.
        chunk_label: String,
        /// Neuer Byte-Inhalt.
        new_content: Vec<u8>,
    },
    /// Entfernt einen bestehenden Chunk identifiziert durch seinen Label-Bezeichner.
    Remove {
        /// Label-Bezeichner des zu entfernenden Chunks.
        chunk_label: String,
    },
}

/// Zusammenfassung eines Subgoal-Checkpoints vor dem Zurücksetzen des Scratchpads.
///
/// Dieser Typ dient als Übergabeobjekt für nachfolgende Stufen (z. B. Speicher-Konsolidierung).
#[derive(Debug)]
pub struct ScratchpadCheckpoint {
    /// Eindeutige Aufgaben-ID.
    pub task_id: String,
    /// Index des erfolgreich abgeschlossenen Teilziels.
    pub completed_subgoal_index: u64,
    /// Löschquittung nach dem Leeren des VolatileContextVault.
    pub purge_receipt: PurgeReceipt,
    /// Anzahl der Chunks zum Zeitpunkt des Checkpoints.
    pub chunk_count_at_checkpoint: usize,
}

/// Temporärer, aufgabengebundener Arbeitskontext-Scratchpad für Agentenschleifen.
///
/// Kapselt eine [`VolatileContextVault`]-Instanz für die sichere RAM-native Speicherung
/// (mlock, Zeroize bei Drop) ohne persistenten Disk-Schreibpfad.
pub struct ClmScratchpad {
    task_id: String,
    tenant_id: TenantId,
    current_subgoal_index: u64,
    created_at_tx: u64,
    config: VaultConfig,
    vault: VolatileContextVault,
    chunks: Vec<VaultChunk>,
    next_chunk_id: u64,
    pinned_regions: Vec<PinnedRegionId>,
    clock: Arc<dyn Clock>,
    audit_sink: Arc<dyn ContextEditAuditSink>,
    cache_invalidator: Arc<dyn ScratchpadCacheInvalidator>,
}

/// Hilfsfunktion zum Duplizieren eines [`VaultChunk`] ohne `Clone`-Derivierung.
fn clone_chunk(chunk: &VaultChunk) -> VaultChunk {
    let mut cloned = VaultChunk::new(
        chunk.id,
        chunk.content.clone(),
        chunk.modality.clone(),
        TxId(chunk.captured_tx),
    );
    if let Some(ref label) = chunk.label {
        cloned = cloned.with_label(label.as_str());
    }
    cloned
}

impl ClmScratchpad {
    /// Erzeugt einen neuen `ClmScratchpad` für eine Task mit gegebener Vault-Konfiguration.
    pub fn new(task_id: String, config: VaultConfig) -> Self {
        let vault = VolatileContextVault::open(config.clone());
        Self {
            task_id,
            tenant_id: TenantId::SYSTEM,
            current_subgoal_index: 0,
            created_at_tx: 0,
            config,
            vault,
            chunks: Vec::new(),
            next_chunk_id: 1,
            pinned_regions: Vec::new(),
            clock: Arc::new(SystemClock::new()),
            audit_sink: Arc::new(NoopContextEditAuditSink),
            cache_invalidator: Arc::new(NoopScratchpadCacheInvalidator),
        }
    }

    /// Setzt die Mandanten-ID (Builder-Muster).
    pub fn with_tenant_id(mut self, tenant_id: TenantId) -> Self {
        self.tenant_id = tenant_id;
        self
    }

    /// Setzt die Clock für deterministisches Zeitmanagement (Builder-Muster).
    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    /// Setzt die Audit-Sink (Builder-Muster).
    pub fn with_audit_sink(mut self, audit_sink: Arc<dyn ContextEditAuditSink>) -> Self {
        self.audit_sink = audit_sink;
        self
    }

    /// Setzt den Cache-Invalidator (Builder-Muster).
    pub fn with_cache_invalidator(
        mut self,
        cache_invalidator: Arc<dyn ScratchpadCacheInvalidator>,
    ) -> Self {
        self.cache_invalidator = cache_invalidator;
        self
    }

    /// Registriert eine gepinnte Region (unveränderlicher Kontextteil).
    pub fn with_pinned_region(mut self, region_id: PinnedRegionId) -> Self {
        if !self.pinned_regions.contains(&region_id) {
            self.pinned_regions.push(region_id);
        }
        self
    }

    /// Setzt die Transaktions-ID der Erstellung (Builder-Muster).
    pub fn with_created_at_tx(mut self, created_at_tx: TxId) -> Self {
        self.created_at_tx = created_at_tx.inner();
        self
    }

    /// Gibt die Task-ID zurück.
    pub fn task_id(&self) -> &str {
        &self.task_id
    }

    /// Gibt die Mandanten-ID zurück.
    pub fn tenant_id(&self) -> TenantId {
        self.tenant_id
    }

    /// Gibt den aktuellen Subgoal-Index zurück.
    pub fn current_subgoal_index(&self) -> u64 {
        self.current_subgoal_index
    }

    /// Gibt die Transaktions-ID der Erzeugung zurück.
    pub fn created_at_tx(&self) -> u64 {
        self.created_at_tx
    }

    /// Gibt eine immutable Referenz auf die registrierten gepinnten Regionen zurück.
    pub fn pinned_regions(&self) -> &[PinnedRegionId] {
        &self.pinned_regions
    }

    /// Wendet eine Bearbeitungs-Operation ([`ScratchpadEditOp`]) auf das Scratchpad an.
    ///
    /// # Fehler
    /// Liefert `ContextraError::InvalidInput` falls:
    /// - Versucht wird, eine gepinnte Region zu verändern oder mit einem Chunk zu überschreiben,
    /// - Ein anzusprechendes Label nicht existiert,
    /// - Die Kapazität des Vaults überschritten wird.
    pub fn apply_edit(&mut self, op: ScratchpadEditOp, captured_tx: TxId) -> Result<()> {
        // Governance check: Edit darf keine gepinnte Region adressieren
        match &op {
            ScratchpadEditOp::Append {
                label: Some(lbl), ..
            } => {
                if self.pinned_regions.iter().any(|p| p.as_str() == lbl) {
                    return Err(ContextraError::InvalidInput(format!(
                        "Anhängen mit Label '{lbl}' nicht erlaubt: Label adressiert eine gepinnte Region"
                    )));
                }
            }
            ScratchpadEditOp::Replace { chunk_label, .. } => {
                if self
                    .pinned_regions
                    .iter()
                    .any(|p| p.as_str() == chunk_label)
                {
                    return Err(ContextraError::InvalidInput(format!(
                        "Gepinnte Region '{chunk_label}' darf nicht ersetzt werden"
                    )));
                }
            }
            ScratchpadEditOp::Remove { chunk_label } => {
                if self
                    .pinned_regions
                    .iter()
                    .any(|p| p.as_str() == chunk_label)
                {
                    return Err(ContextraError::InvalidInput(format!(
                        "Gepinnte Region '{chunk_label}' darf nicht entfernt werden"
                    )));
                }
            }
            _ => {}
        }

        match op {
            ScratchpadEditOp::Append { content, label } => {
                let doc_id = DocId::new(self.next_chunk_id);
                self.next_chunk_id += 1;

                let mut chunk =
                    VaultChunk::new(doc_id, content, SignalModality::TextInput, captured_tx);
                if let Some(ref lbl) = label {
                    chunk = chunk.with_label(lbl.as_str());
                }

                // Ingest in den bestehenden Vault
                let chunk_to_ingest = clone_chunk(&chunk);
                self.vault.ingest(chunk_to_ingest).map_err(|e| {
                    ContextraError::InvalidInput(format!(
                        "Scratchpad-Kapazität beim Anhängen überschritten: {e}"
                    ))
                })?;

                self.chunks.push(chunk);
            }
            ScratchpadEditOp::Replace {
                chunk_label,
                new_content,
            } => {
                let idx = self
                    .chunks
                    .iter()
                    .position(|c| c.label.as_deref() == Some(&chunk_label))
                    .ok_or_else(|| {
                        ContextraError::InvalidInput(format!(
                            "Chunk mit Label '{chunk_label}' für Replace nicht gefunden"
                        ))
                    })?;

                let mut candidate_chunks: Vec<VaultChunk> =
                    self.chunks.iter().map(clone_chunk).collect();
                candidate_chunks[idx].content = new_content;
                candidate_chunks[idx].captured_tx = captured_tx.inner();

                let mut test_vault = VolatileContextVault::open(self.config.clone());
                for chunk in &candidate_chunks {
                    test_vault.ingest(clone_chunk(chunk)).map_err(|e| {
                        ContextraError::InvalidInput(format!(
                            "Scratchpad-Kapazität bei Replace überschritten: {e}"
                        ))
                    })?;
                }

                // Transaktionssicher austauschen: alter Vault wird purged
                self.chunks = candidate_chunks;
                let old_vault = std::mem::replace(&mut self.vault, test_vault);
                let _ = old_vault.purge();
            }
            ScratchpadEditOp::Remove { chunk_label } => {
                let idx = self
                    .chunks
                    .iter()
                    .position(|c| c.label.as_deref() == Some(&chunk_label))
                    .ok_or_else(|| {
                        ContextraError::InvalidInput(format!(
                            "Chunk mit Label '{chunk_label}' für Remove nicht gefunden"
                        ))
                    })?;

                let mut candidate_chunks: Vec<VaultChunk> =
                    self.chunks.iter().map(clone_chunk).collect();
                candidate_chunks.remove(idx);

                let mut test_vault = VolatileContextVault::open(self.config.clone());
                for chunk in &candidate_chunks {
                    test_vault.ingest(clone_chunk(chunk)).map_err(|e| {
                        ContextraError::InvalidInput(format!(
                            "Scratchpad-Kapazität bei Remove überschritten: {e}"
                        ))
                    })?;
                }

                self.chunks = candidate_chunks;
                let old_vault = std::mem::replace(&mut self.vault, test_vault);
                let _ = old_vault.purge();
            }
        }

        // Audit-Meldung für die Bearbeitung
        self.audit_sink.record_edit(ContextEditAuditRecord {
            tenant_id: self.tenant_id,
            task_id: self.task_id.clone(),
            operation: "apply_edit".to_string(),
            subgoal_index: self.current_subgoal_index,
            timestamp_nanos: self.clock.now_unix_nanos(),
        });

        Ok(())
    }

    /// Erzeugt eine [`ScratchpadCheckpoint`]-Zusammenfassung, leert den Vault intern via `purge()`
    /// (inkl. Zeroize aller RAM-Inhalte), meldet Audit und Cache-Invalidierung genau einmal,
    /// und inkrementiert den `current_subgoal_index`.
    pub fn checkpoint_and_reset(&mut self) -> Result<ScratchpadCheckpoint> {
        let chunk_count = self.chunks.len();
        self.chunks.clear();

        let new_vault = VolatileContextVault::open(self.config.clone());
        let old_vault = std::mem::replace(&mut self.vault, new_vault);
        let purge_receipt = old_vault.purge();

        let completed_subgoal = self.current_subgoal_index;

        // INV-CLM-SCRATCHPAD-1: Audit-Meldung für den Reset
        self.audit_sink.record_edit(ContextEditAuditRecord {
            tenant_id: self.tenant_id,
            task_id: self.task_id.clone(),
            operation: "checkpoint_and_reset".to_string(),
            subgoal_index: completed_subgoal,
            timestamp_nanos: self.clock.now_unix_nanos(),
        });

        // INV-CLM-SCRATCHPAD-2: Cache-Invalidierung nur für den eigenen Scratchpad-Scope
        self.cache_invalidator.invalidate_scratchpad_scope(
            self.tenant_id,
            &self.task_id,
            completed_subgoal,
        );

        self.current_subgoal_index += 1;

        Ok(ScratchpadCheckpoint {
            task_id: self.task_id.clone(),
            completed_subgoal_index: completed_subgoal,
            purge_receipt,
            chunk_count_at_checkpoint: chunk_count,
        })
    }

    /// Gibt die Gesamtzahl der belegten Bytes im aktuellen Scratchpad-Vault zurück.
    pub fn total_bytes(&self) -> usize {
        self.vault.current_size_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scratchpad_apply_edit_all_variants() -> Result<()> {
        let config = VaultConfig {
            max_capacity_bytes: 1024 * 1024,
            attempt_mlock: false,
        };
        let mut pad =
            ClmScratchpad::new("task-42".to_string(), config).with_created_at_tx(TxId(100));

        assert_eq!(pad.task_id(), "task-42");
        assert_eq!(pad.created_at_tx(), 100);
        assert_eq!(pad.current_subgoal_index(), 0);
        assert_eq!(pad.total_bytes(), 0);

        // 1. Append
        pad.apply_edit(
            ScratchpadEditOp::Append {
                content: b"Initial step output".to_vec(),
                label: Some("step1".to_string()),
            },
            TxId(101),
        )?;
        assert_eq!(pad.total_bytes(), b"Initial step output".len());

        // 2. Append zweiter Chunk
        pad.apply_edit(
            ScratchpadEditOp::Append {
                content: b"Second step payload".to_vec(),
                label: Some("step2".to_string()),
            },
            TxId(102),
        )?;
        let bytes_after_two = b"Initial step output".len() + b"Second step payload".len();
        assert_eq!(pad.total_bytes(), bytes_after_two);

        // 3. Replace
        pad.apply_edit(
            ScratchpadEditOp::Replace {
                chunk_label: "step1".to_string(),
                new_content: b"Updated initial output".to_vec(),
            },
            TxId(103),
        )?;
        let bytes_after_replace = b"Updated initial output".len() + b"Second step payload".len();
        assert_eq!(pad.total_bytes(), bytes_after_replace);

        // 4. Remove
        pad.apply_edit(
            ScratchpadEditOp::Remove {
                chunk_label: "step2".to_string(),
            },
            TxId(104),
        )?;
        assert_eq!(pad.total_bytes(), b"Updated initial output".len());

        Ok(())
    }

    #[test]
    fn test_scratchpad_checkpoint_and_reset() -> Result<()> {
        let config = VaultConfig {
            max_capacity_bytes: 1024 * 1024,
            attempt_mlock: false,
        };
        let mut pad = ClmScratchpad::new("task-reset".to_string(), config);

        pad.apply_edit(
            ScratchpadEditOp::Append {
                content: b"Subgoal 0 chunk A".to_vec(),
                label: Some("a".to_string()),
            },
            TxId(10),
        )?;
        pad.apply_edit(
            ScratchpadEditOp::Append {
                content: b"Subgoal 0 chunk B".to_vec(),
                label: Some("b".to_string()),
            },
            TxId(11),
        )?;

        assert_eq!(
            pad.total_bytes(),
            b"Subgoal 0 chunk A".len() + b"Subgoal 0 chunk B".len()
        );

        let checkpoint = pad.checkpoint_and_reset()?;
        assert_eq!(checkpoint.task_id, "task-reset");
        assert_eq!(checkpoint.completed_subgoal_index, 0);
        assert_eq!(checkpoint.chunk_count_at_checkpoint, 2);
        assert_eq!(checkpoint.purge_receipt.chunks_purged, 2);

        // total_bytes ist nun 0, current_subgoal_index inkrementiert
        assert_eq!(pad.total_bytes(), 0);
        assert_eq!(pad.current_subgoal_index(), 1);

        // Weiteres Arbeiten im Subgoal 1
        pad.apply_edit(
            ScratchpadEditOp::Append {
                content: b"Subgoal 1 chunk C".to_vec(),
                label: Some("c".to_string()),
            },
            TxId(12),
        )?;
        assert_eq!(pad.total_bytes(), b"Subgoal 1 chunk C".len());

        Ok(())
    }

    #[test]
    fn test_scratchpad_capacity_exceeded_returns_error() {
        let config = VaultConfig {
            max_capacity_bytes: 30,
            attempt_mlock: false,
        };
        let mut pad = ClmScratchpad::new("task-cap".to_string(), config);

        let res1 = pad.apply_edit(
            ScratchpadEditOp::Append {
                content: vec![0u8; 20],
                label: Some("chunk1".to_string()),
            },
            TxId(1),
        );
        assert!(res1.is_ok());

        // Zweiter Chunk würde 20 + 20 = 40 Bytes belegen (> 30 Max)
        let res2 = pad.apply_edit(
            ScratchpadEditOp::Append {
                content: vec![0u8; 20],
                label: Some("chunk2".to_string()),
            },
            TxId(2),
        );
        assert!(res2.is_err());
        assert_eq!(pad.total_bytes(), 20); // Unverändert nach Fehler
    }

    #[test]
    fn test_scratchpad_missing_label_returns_error() {
        let config = VaultConfig {
            max_capacity_bytes: 1024,
            attempt_mlock: false,
        };
        let mut pad = ClmScratchpad::new("task-label".to_string(), config);

        let res_replace = pad.apply_edit(
            ScratchpadEditOp::Replace {
                chunk_label: "non-existent".to_string(),
                new_content: b"abc".to_vec(),
            },
            TxId(1),
        );
        assert!(res_replace.is_err());

        let res_remove = pad.apply_edit(
            ScratchpadEditOp::Remove {
                chunk_label: "non-existent".to_string(),
            },
            TxId(2),
        );
        assert!(res_remove.is_err());
    }

    #[test]
    fn test_pinned_region_immutability_guards() {
        let config = VaultConfig {
            max_capacity_bytes: 1024,
            attempt_mlock: false,
        };
        let pinned = PinnedRegionId::new("sys_prompt");
        let mut pad = ClmScratchpad::new("task-pinned".to_string(), config)
            .with_pinned_region(pinned.clone());

        assert_eq!(pad.pinned_regions().len(), 1);
        assert_eq!(pad.pinned_regions()[0], pinned);

        // 1. Attempt to append with pinned region label -> Err
        let err_append = pad.apply_edit(
            ScratchpadEditOp::Append {
                content: b"malicious append".to_vec(),
                label: Some("sys_prompt".to_string()),
            },
            TxId(1),
        );
        assert!(err_append.is_err());

        // 2. Attempt to replace pinned region -> Err
        let err_replace = pad.apply_edit(
            ScratchpadEditOp::Replace {
                chunk_label: "sys_prompt".to_string(),
                new_content: b"overwrite".to_vec(),
            },
            TxId(2),
        );
        assert!(err_replace.is_err());

        // 3. Attempt to remove pinned region -> Err
        let err_remove = pad.apply_edit(
            ScratchpadEditOp::Remove {
                chunk_label: "sys_prompt".to_string(),
            },
            TxId(3),
        );
        assert!(err_remove.is_err());
    }
}

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

use contextra_db::volatile_vault::{
    PurgeReceipt, SignalModality, VaultChunk, VaultConfig, VolatileContextVault,
};
use contextra_types::{ContextraError, DocId, Result, TxId};
use serde::{Deserialize, Serialize};

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
    current_subgoal_index: u64,
    created_at_tx: u64,
    config: VaultConfig,
    vault: VolatileContextVault,
    chunks: Vec<VaultChunk>,
    next_chunk_id: u64,
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
        cloned = cloned.with_label(label.clone());
    }
    cloned
}

impl ClmScratchpad {
    /// Erzeugt einen neuen `ClmScratchpad` für eine Task mit gegebener Vault-Konfiguration.
    pub fn new(task_id: String, config: VaultConfig) -> Self {
        let vault = VolatileContextVault::open(config.clone());
        Self {
            task_id,
            current_subgoal_index: 0,
            created_at_tx: 0,
            config,
            vault,
            chunks: Vec::new(),
            next_chunk_id: 1,
        }
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

    /// Gibt den aktuellen Subgoal-Index zurück.
    pub fn current_subgoal_index(&self) -> u64 {
        self.current_subgoal_index
    }

    /// Gibt die Transaktions-ID der Erzeugung zurück.
    pub fn created_at_tx(&self) -> u64 {
        self.created_at_tx
    }

    /// Wendet eine Bearbeitungs-Operation ([`ScratchpadEditOp`]) auf das Scratchpad an.
    ///
    /// # Fehler
    /// Liefert `ContextraError::InvalidInput` falls ein anzusprechendes Label nicht
    /// existiert oder die Kapazität des Vaults überschritten wird.
    pub fn apply_edit(&mut self, op: ScratchpadEditOp, captured_tx: TxId) -> Result<()> {
        match op {
            ScratchpadEditOp::Append { content, label } => {
                let doc_id = DocId::new(self.next_chunk_id);
                self.next_chunk_id += 1;

                let mut chunk =
                    VaultChunk::new(doc_id, content, SignalModality::TextInput, captured_tx);
                if let Some(lbl) = label {
                    chunk = chunk.with_label(lbl);
                }

                // Ingest in den bestehenden Vault
                let chunk_to_ingest = clone_chunk(&chunk);
                self.vault.ingest(chunk_to_ingest).map_err(|e| {
                    ContextraError::InvalidInput(format!(
                        "Scratchpad-Kapazität beim Anhängen überschritten: {e}"
                    ))
                })?;

                self.chunks.push(chunk);
                Ok(())
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
                Ok(())
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
                Ok(())
            }
        }
    }

    /// Erzeugt eine [`ScratchpadCheckpoint`]-Zusammenfassung, leert den Vault intern via `purge()`
    /// und inkrementiert den `current_subgoal_index`.
    pub fn checkpoint_and_reset(&mut self) -> Result<ScratchpadCheckpoint> {
        let chunk_count = self.chunks.len();
        self.chunks.clear();

        let new_vault = VolatileContextVault::open(self.config.clone());
        let old_vault = std::mem::replace(&mut self.vault, new_vault);
        let purge_receipt = old_vault.purge();

        let completed_subgoal = self.current_subgoal_index;
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
}

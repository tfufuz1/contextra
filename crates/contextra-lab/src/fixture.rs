//! Lab Fixturen zur Generierung definierter Testdaten mit bekannten Mustern und gesicherten DEKs.
//!
//! Gemäß SPEC §12.3:
//! - Unabhängiges Erasure-Lab ohne Abhängigkeit zu Produktions-Decodern (AR-4).
//! - Fixturen mit bekannten Mustern und vor der Löschung gesicherten DEKs (MK3).
//! - Das Lab löst die Löschung ausschließlich über eine vom Aufrufer übergebene Funktion aus
//!   (`F: FnOnce(&Path) -> Result<(), LabError>`), um jede zyklische oder Produktionsabhängigkeit
//!   zu vermeiden.

use std::path::{Path, PathBuf};

use crate::error::LabError;
use crate::format::file_header::{FileHeader, FileKind};
use crate::format::record_envelope::RecordEnvelopeV2;

/// Eine Test-Dokumenten-Fixtur mit bekanntem Klartextmuster und gesichertem DEK.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabDocumentFixture {
    /// Eindeutiger Dokumenten-Tag.
    pub doc_tag: [u8; 16],
    /// Klartext-Muster (bekanntes Needle für MK2).
    pub plaintext_needle: Vec<u8>,
    /// Vor der Löschung gesicherter Dokumenten-DEK (für MK3).
    pub retained_dek: [u8; 32],
    /// Schlüsselbytes für AAD.
    pub key_bytes: Vec<u8>,
    /// Erzeugtes RecordEnvelope v2.
    pub envelope_bytes: Vec<u8>,
}

impl LabDocumentFixture {
    /// Erzeugt eine neue Dokumenten-Fixtur.
    pub fn new(
        doc_tag: [u8; 16],
        plaintext_needle: &[u8],
        retained_dek: [u8; 32],
        key_bytes: &[u8],
    ) -> Result<Self, LabError> {
        let nonce = [0x5au8; 12];
        let envelope_bytes = RecordEnvelopeV2::create_bytes(
            doc_tag,
            nonce,
            plaintext_needle,
            &retained_dek,
            key_bytes,
        )?;

        Ok(Self {
            doc_tag,
            plaintext_needle: plaintext_needle.to_vec(),
            retained_dek,
            key_bytes: key_bytes.to_vec(),
            envelope_bytes,
        })
    }
}

/// Ein Test-Bestand für Laborversuche, Differenzprüfung und Mutationstests.
#[derive(Debug, Clone)]
pub struct LabStoreFixture {
    /// Pfad zum simulierten Datenverzeichnis.
    pub base_path: PathBuf,
    /// Ziel-Dokument (soll gelöscht werden).
    pub target_doc: LabDocumentFixture,
    /// Nicht-Ziel Dokumente (müssen bitgleich erhalten bleiben, T-DIFF-1).
    pub non_target_docs: Vec<LabDocumentFixture>,
    /// Bekannte Positivkontroll-Needle (für HR-10).
    pub control_needle: Vec<u8>,
}

impl LabStoreFixture {
    /// Initialisiert eine Store-Fixtur mit Ziel- und Nicht-Ziel-Dokumenten.
    pub fn new(
        base_path: PathBuf,
        target_doc: LabDocumentFixture,
        non_target_docs: Vec<LabDocumentFixture>,
        control_needle: &[u8],
    ) -> Self {
        Self {
            base_path,
            target_doc,
            non_target_docs,
            control_needle: control_needle.to_vec(),
        }
    }

    /// Löst die Löschung über einen vom Aufrufer bereitgestellten Callback aus (AR-4).
    ///
    /// Das Lab hat keine Abhängigkeit zu `engine` oder `erasure`.
    pub fn execute_deletion<F>(&self, deletion_fn: F) -> Result<(), LabError>
    where
        F: FnOnce(&Path) -> Result<(), LabError>,
    {
        deletion_fn(&self.base_path)
    }

    /// Hilfsmethode: Erzeugt eine einfache SSTable-Datei mit den Fixtur-Dokumenten.
    pub fn create_mock_sstable_bytes(
        file_id: [u8; 16],
        file_key: &[u8; 32],
        records: &[&LabDocumentFixture],
    ) -> Result<Vec<u8>, LabError> {
        let header = FileHeader::create_bytes(FileKind::SsTable, file_id, 1, file_key);
        let mut out = Vec::new();
        out.extend_from_slice(&header);

        for doc in records {
            let klen = u16::try_from(doc.key_bytes.len())
                .map_err(|_| LabError::Corrupted("Key zu lang"))?;
            let vlen = u32::try_from(doc.envelope_bytes.len())
                .map_err(|_| LabError::Corrupted("Value zu lang"))?;

            out.extend_from_slice(&klen.to_be_bytes());
            out.extend_from_slice(&doc.key_bytes);
            out.extend_from_slice(&vlen.to_be_bytes());
            out.extend_from_slice(&doc.envelope_bytes);
        }

        // Dummy Footer
        let mut footer = [0u8; 64];
        if let Some(m) = footer.get_mut(56..64) {
            m.copy_from_slice(crate::format::sstable::SSTABLE_FOOTER_MAGIC);
        }
        out.extend_from_slice(&footer);

        Ok(out)
    }
}

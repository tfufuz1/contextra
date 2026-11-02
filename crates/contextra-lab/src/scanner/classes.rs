//! Implementierung der unabhängigen Messklassen MK1, MK2, MK3 laut SPEC §12.1.
//!
//! - MK1: Strukturscan (Manifeste, SSTable-Indexblöcke, Segmentwörterbücher, Slot-Tabellen)
//! - MK2: Rohscan auf Klartext-Needles des Betreibers und bekannte Muster
//! - MK3: Schlüsselprobe (DEK-Abwesenheit / Chiffrat-Unöffenbarkeit im Lab mit Alt-DEK)

use crate::error::LabError;
use crate::format::record_envelope::RecordEnvelopeV2;
use crate::format::vault::{VaultFile, VaultSlotStatus, VaultSlotType};

/// Matcher-Kriterien für MK1 (Struktur).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureTarget {
    /// Betroffener doc_tag.
    pub doc_tag: [u8; 16],
}

impl StructureTarget {
    /// Prüft, ob ein Tresor-Slot diesem Ziel entspricht und noch aktiv ist.
    pub fn matches_vault_slot(
        &self,
        slot_type: VaultSlotType,
        tag: &[u8; 16],
        status: VaultSlotStatus,
    ) -> bool {
        if status != VaultSlotStatus::Active {
            return false;
        }
        if slot_type == VaultSlotType::Document && tag == &self.doc_tag {
            return true;
        }
        false
    }
}

/// Scanner für MK1: Strukturscan.
pub struct Mk1StructureScanner;

impl Mk1StructureScanner {
    /// Führt den Strukturscan über eine geparste Tresordatei aus.
    pub fn scan_vault(vault: &VaultFile, target: &StructureTarget) -> usize {
        let mut matches = 0usize;
        for slot in &vault.slots {
            if target.matches_vault_slot(slot.slot_type, &slot.tag, slot.status) {
                matches = matches.saturating_add(1);
            }
        }
        matches
    }
}

/// Scanner für MK2: Rohscan.
pub struct Mk2RawScanner;

impl Mk2RawScanner {
    /// Durchsucht ein Byte-Array (z.B. Dateirohbytes) nach einer Klartext-Needle oder einem Muster.
    pub fn scan_bytes(raw: &[u8], needle: &[u8]) -> usize {
        if needle.is_empty() || raw.len() < needle.len() {
            return 0;
        }

        let mut count = 0usize;
        let limit = raw.len() - needle.len();
        let mut i = 0;

        while i <= limit {
            if let Some(window) = raw.get(i..i + needle.len()) {
                if window == needle {
                    count = count.saturating_add(1);
                    i = i.saturating_add(needle.len());
                    continue;
                }
            }
            i = i.saturating_add(1);
        }

        count
    }
}

/// Scanner für MK3: Schlüsselprobe (Lab).
pub struct Mk3KeyProbeScanner;

impl Mk3KeyProbeScanner {
    /// Versucht, einen vorgefundenen RecordEnvelope mit dem vor der Löschung gesicherten DEK zu öffnen.
    /// Im Erfolgsfall (Chiffrat öffenbar) liegt ein Verstoß/Match vor.
    pub fn probe_envelope_decryptable(
        envelope_bytes: &[u8],
        retained_old_dek: &[u8; 32],
        key_bytes: &[u8],
    ) -> Result<bool, LabError> {
        let envelope = RecordEnvelopeV2::parse(envelope_bytes)?;
        match envelope.decrypt(retained_old_dek, key_bytes) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

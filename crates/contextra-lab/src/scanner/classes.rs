//! Implementierung der unabhängigen Messklassen MK1, MK2, MK3 laut SPEC §12.1.
//!
//! - MK1: Strukturscan (Manifeste, SSTable-Indexblöcke, Segmentwörterbücher, Slot-Tabellen, T-RAW-1)
//! - MK2: Rohscan auf Klartext-Needles des Betreibers und bekannte Muster (T-RAW-2, T-RAW-4, T-RAW-6)
//! - MK3: Schlüsselprobe (DEK-Abwesenheit / Chiffrat-Unöffenbarkeit im Lab mit Alt-DEK, T-RAW-3)

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
    /// Führt den Strukturscan über eine geparste Tresordatei aus (T-RAW-1).
    pub fn scan_vault(vault: &VaultFile, target: &StructureTarget) -> usize {
        let mut matches = 0usize;
        for slot in &vault.slots {
            if target.matches_vault_slot(slot.slot_type, &slot.tag, slot.status) {
                matches = matches.saturating_add(1);
            }
        }
        matches
    }

    /// Prüft, ob ein Eintrag im Segment- oder SSTable-Index das Ziel referenziert (T-RAW-1).
    pub fn scan_index_keys(keys: &[[u8; 16]], target_tag: &[u8; 16]) -> usize {
        let mut count = 0usize;
        for k in keys {
            if k == target_tag {
                count = count.saturating_add(1);
            }
        }
        count
    }
}

/// Scanner für MK2: Rohscan (T-RAW-2, T-RAW-4, T-RAW-6).
pub struct Mk2RawScanner;

impl Mk2RawScanner {
    /// Durchsucht ein Byte-Array (z.B. Dateirohbytes) nach einer Klartext-Needle oder einem Muster (T-RAW-2).
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

    /// Prüft Index-Wörterbuch-Tags auf unzulässigen Klartext-Einschluss (T-RAW-4).
    /// Tags dürfen keine Klartextfragmente enthalten; sie sind keyed BLAKE3-Hashes (D-21).
    pub fn scan_index_terms_for_plain_leak(terms: &[&[u8]], plaintext_needle: &[u8]) -> usize {
        let mut leaks = 0usize;
        for term in terms {
            if Self::scan_bytes(term, plaintext_needle) > 0 {
                leaks = leaks.saturating_add(1);
            }
        }
        leaks
    }

    /// Prüft Datei- und Verzeichnisnamen auf unzulässige Klartext-Bestandteile (T-RAW-6, SPEC D-09, D-75).
    /// Erlaubt sind ausschließlich hexadezimale Tags, file_id, erasure_id oder feste Systemnamen (z.B. "text", "vector").
    pub fn validate_path_elements_blind_tags_only(
        path_str: &str,
        forbidden_user_names: &[&str],
    ) -> Result<usize, LabError> {
        let mut matches = 0usize;
        for forbidden in forbidden_user_names {
            if forbidden.is_empty() {
                continue;
            }
            if path_str.contains(forbidden) {
                matches = matches.saturating_add(1);
            }
        }
        Ok(matches)
    }
}

/// Scanner für MK3: Schlüsselprobe (Lab, T-RAW-3).
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

    /// Prüft Vektor-Snapshot-Slot-Envelope auf Unöffenbarkeit mit dem gesicherten DEK (T-RAW-3, D-19).
    /// Nach Schlüsselvernichtung muss das Vektor-Envelope unlesbar sein (key_coverage = full).
    pub fn probe_vector_slot_envelope_decryptable(
        envelope_bytes: &[u8],
        retained_old_dek: &[u8; 32],
        slot_aad: &[u8],
    ) -> Result<bool, LabError> {
        let envelope = RecordEnvelopeV2::parse(envelope_bytes)?;
        match envelope.decrypt(retained_old_dek, slot_aad) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

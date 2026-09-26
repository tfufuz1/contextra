//! BSI Grundschutz / TR-02102 Cryptographic Mapping (Spec §2.4 Punkt 2, §16.3).
//!
//! # Maintenance Notice / Single Source of Truth
//! This module serves as the Single Source of Truth (SSOT) for the BSI Grundschutz / TR-02102
//! cryptographic mapping table included in the generated GDPR Art. 30 processing register export.
//!
//! **IMPORTANT FOR MAINTAINERS:**
//! Whenever a cryptographic primitive in `contextra-crypto` or related storage security modules
//! is added, removed, or updated, this mapping table (`bsi_mapping_table()`) MUST be updated
//! synchronously. The corresponding reference documentation at `docs/bsi-grundschutz-mapping.md`
//! must also be updated accordingly.
//!
//! **COMPLIANCE & LEGAL NOTICE:**
//! This mapping document provides reference alignment against BSI Technical Guidelines (TR-02102).
//! It constitutes a technical mapping documentation and NOT a formal certification, audit approval,
//! or warranty of BSI compliance.

use serde::{Deserialize, Serialize};

/// A mapping entry linking a cryptographic primitive in the codebase to a BSI Technical Guideline (Spec §2.4 Punkt 2, §16.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BsiMappingEntry {
    /// Name of the cryptographic primitive (e.g. "Ed25519").
    pub primitive_name: String,
    /// Code location path within the workspace (e.g. "crates/contextra-crypto/src/deletion_proof.rs").
    pub code_location: String,
    /// BSI TR-02102 reference identifier (e.g. "BSI TR-02102-1").
    pub bsi_reference: String,
    /// Additional context, date of assessment, or operational constraints.
    pub note: String,
}

/// Returns the static, code-maintained cryptographic mapping table for Contextra.
///
/// Serves as the Single Source of Truth for audit exports and compliance documentation.
pub fn bsi_mapping_table() -> Vec<BsiMappingEntry> {
    vec![
        BsiMappingEntry {
            primitive_name: "Ed25519".to_string(),
            code_location: "crates/contextra-crypto/src/deletion_proof.rs".to_string(),
            bsi_reference: "BSI TR-02102-1".to_string(),
            note: "Digitale Signatur für Löschnachweise (DeletionProof v3, §14.1). BSI TR-02102-1 §1.5/§3.2 bewertet Ed25519 (Curve25519) als empfohlenes Asymmetrisches Signaturverfahren mit 128-Bit Sicherheit. Stand: März 2024, zu prüfen gegen aktuelle BSI-Veröffentlichung (keine Konformitätsbehauptung).".to_string(),
        },
        BsiMappingEntry {
            primitive_name: "HMAC-SHA256".to_string(),
            code_location: "crates/contextra-crypto/src/wal_crypto.rs".to_string(),
            bsi_reference: "BSI TR-02102-1".to_string(),
            note: "WAL-Integritätskette und Tamper-Detection (§6.5). BSI TR-02102-1 §1.6 empfiehlt HMAC in Kombination mit SHA-256 (Mindestschlüssellänge 128 Bit, Contextra nutzt 256 Bit IntegrityKey) für Keyed Hash Message Authentication. Stand: März 2024, zu prüfen gegen aktuelle BSI-Veröffentlichung (keine Konformitätsbehauptung).".to_string(),
        },
        BsiMappingEntry {
            primitive_name: "AES-256-GCM-SIV".to_string(),
            code_location: "crates/contextra-crypto/src/crypto.rs".to_string(),
            bsi_reference: "BSI TR-02102-1".to_string(),
            note: "Symmetrische Verschlüsselung (Envelope Encryption, KeyManager, §10). BSI TR-02102-1 §1.3 empfiehlt AES mit ≥128-Bit Schlüssellänge; AES-256-GCM-SIV stellt missbrauchsresistente authentifizierte Authenticated Encryption (AEAD) nach RFC 8452 bereit. Stand: März 2024, zu prüfen gegen aktuelle BSI-Veröffentlichung (keine Konformitätsbehauptung).".to_string(),
        },
        BsiMappingEntry {
            primitive_name: "Argon2id".to_string(),
            code_location: "crates/contextra-crypto/src/kdf.rs".to_string(),
            bsi_reference: "BSI TR-02102-1 / BSI TR-02102-4".to_string(),
            note: "Passwortbasierte Schlüsselableitung (Tenant-DEK / Passphrase-DEK, KdfHeader, §10). Argon2id erfüllt OWASP/BSI-Anforderungen für speicherharte Schlüsselableitungsfunktionen (KDF) mit m_cost >= 19456 KiB, t_cost >= 2, p_cost >= 1. Stand: März 2024, zu prüfen gegen aktuelle BSI-Veröffentlichung (keine Konformitätsbehauptung).".to_string(),
        },
    ]
}

/// Sanitizes special characters (pipes and newlines) within a markdown table cell.
fn sanitize_markdown_cell(input: &str) -> String {
    input
        .replace('|', "\\|")
        .replace("\r\n", "<br/>")
        .replace(['\n', '\r'], "<br/>")
}

/// Renders the BSI Grundschutz / TR-02102 cryptographic mapping table as Markdown.
///
/// Consistent with the rendering style in [`crate::markdown_template::render_register_markdown`].
pub fn render_bsi_mapping_markdown(entries: &[BsiMappingEntry]) -> String {
    let mut out = String::new();
    out.push_str("# BSI Grundschutz / TR-02102 Kryptographische Zuordnung\n\n");
    out.push_str("> **Hinweis:** Dieses Dokument / dieser Export dient der technischen Referenzzuordnung nach BSI TR-02102 und stellt KEINE Zertifizierungs- oder Konformitätsbehauptung dar. Die BSI-Empfehlungen sind regelmäßig gegen die neuesten Veröffentlichungen des BSI zu prüfen.\n\n");
    out.push_str("| Krypto-Primitive | Code-Fundstelle | BSI-Referenz | Anmerkung / Kontext |\n");
    out.push_str("| --- | --- | --- | --- |\n");

    for entry in entries {
        let name = sanitize_markdown_cell(&entry.primitive_name);
        let location = sanitize_markdown_cell(&entry.code_location);
        let bsi_ref = sanitize_markdown_cell(&entry.bsi_reference);
        let note = sanitize_markdown_cell(&entry.note);

        out.push_str(&format!("| {name} | `{location}` | {bsi_ref} | {note} |\n"));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bsi_mapping_table_exact_entries() {
        let table = bsi_mapping_table();
        assert_eq!(table.len(), 4, "Mapping table must contain exactly 4 entries");

        let primitive_names: Vec<&str> = table.iter().map(|e| e.primitive_name.as_str()).collect();
        assert!(primitive_names.contains(&"Ed25519"));
        assert!(primitive_names.contains(&"HMAC-SHA256"));
        assert!(primitive_names.contains(&"AES-256-GCM-SIV"));
        assert!(primitive_names.contains(&"Argon2id"));

        for entry in &table {
            assert!(!entry.primitive_name.is_empty(), "primitive_name must not be empty");
            assert!(!entry.code_location.is_empty(), "code_location must not be empty");
            assert!(!entry.bsi_reference.is_empty(), "bsi_reference must not be empty");
            assert!(!entry.note.is_empty(), "note must not be empty");
        }
    }

    #[test]
    fn test_render_bsi_mapping_markdown() {
        let table = bsi_mapping_table();
        let markdown = render_bsi_mapping_markdown(&table);

        assert!(markdown.contains("# BSI Grundschutz / TR-02102 Kryptographische Zuordnung"));
        assert!(markdown.contains("Ed25519"));
        assert!(markdown.contains("HMAC-SHA256"));
        assert!(markdown.contains("AES-256-GCM-SIV"));
        assert!(markdown.contains("Argon2id"));
        assert!(markdown.contains("BSI TR-02102-1"));
    }
}

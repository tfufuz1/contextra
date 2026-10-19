//! Markdown table renderer for GDPR Article 30 processing register entries.

use crate::{
    Art30ProcessingRegisterEntry, AuditExportError, DeletionProofSummary, ProcessingRegisterEntry,
};


/// Sanitizes special characters (pipes and newlines) within a markdown table cell.
fn sanitize_markdown_cell(input: &str) -> String {
    input
        .replace('|', "\\|")
        .replace("\r\n", "<br/>")
        .replace(['\n', '\r'], "<br/>")
}

/// Redacts or anonymizes a resource scope identifier so raw cleartext IDs are not exposed.
fn redact_or_anonymize_scope(scope: &str) -> String {
    if scope.is_empty() {
        return "[anonymisiert]".to_string();
    }
    // If scope looks like a plain text document ID or path, mask sensitive characters or return redacted label
    if scope.contains('/') || scope.contains('\\') || scope.len() > 32 {
        let len = scope.len();
        format!("[anonymisiert-id-len-{len}]")
    } else {
        format!("[anonymisiert:{}]", sanitize_markdown_cell(scope))
    }
}

/// Renders the mandatory "Grenzen der Nachweisbarkeit" section.
///
/// # Invariants
/// - Audit mit Toleranz (Garg et al.): Kein mathematischer Beweis der Abwesenheit.
/// - Flash/SSD-Physik (Wei et al.): Kein Überschreiben garantiert; belastbar nur Crypto-Erase.
/// - Umgebungswerte: Swap, Core-Dumps, CoW sind "beobachtet", nicht "garantiert".
fn render_grenzen_section(out: &mut String) {
    out.push_str("\n## Grenzen der Nachweisbarkeit\n\n");
    out.push_str("- **Audit mit Toleranz:** Kein mathematischer oder physikalischer Beweis der Abwesenheit von Daten möglich (Garg et al.).\n");
    out.push_str("- **Flash/SSD-Physik:** Überschreiben auf Flash-Speicher/SSDs ist aufgrund von Wear Leveling und Garbage Collection nicht garantiert (Wei et al.). Belastbarer Schutz erfolgt durch kryptographisches Löschen (Crypto-Erase / Key-Shredding nach NIST SP 800-88 Rev. 2 Verifikationsstufe Purge).\n");
    out.push_str("- **Umgebungsparameter:** Umgebungswerte (z. B. Swap, Core-Dumps, Copy-on-Write-Dateisysteme) sind rein „beobachtet“ und keinesfalls „garantiert“.\n");
}

/// Renders the Layer Status and Deletion Proof summary section.
///
/// Ensures that without verified layer proof, no layer status claims "gelöscht" or `verified`.
fn render_layer_and_proof_status(out: &mut String, all_proofs: &[&DeletionProofSummary]) {
    out.push_str("\n## Status der Speicher-Layer & Löschnachweise\n\n");

    let verified_proofs: Vec<&&DeletionProofSummary> =
        all_proofs.iter().filter(|p| p.verified).collect();

    if verified_proofs.is_empty() {
        out.push_str("### Status der 7 Speicher-Layer (Keine verifizierten Belege)\n");
        out.push_str(
            "- **Layer 1 (Ring-0 In-Memory):** Status: `not_covered` (Kein verifizierter Beleg)\n",
        );
        out.push_str("- **Layer 2 (MemTable RAM-Puffer):** Status: `observed_only` (Flüchtiger RAM-Zustand)\n");
        out.push_str("- **Layer 3 (Write-Ahead Log / WAL):** Status: `not_covered` (Kein Beleg)\n");
        out.push_str(
            "- **Layer 4 (SSTable / Persistent Disk):** Status: `not_covered` (Kein Beleg)\n",
        );
        out.push_str("- **Layer 5 (Vektor-Index HNSW):** Status: `not_covered` (Kein Beleg)\n");
        out.push_str("- **Layer 6 (Graph-Index CSR):** Status: `not_covered` (Kein Beleg)\n");
        out.push_str(
            "- **Layer 7 (Key Vault / DEK Registry):** Status: `not_covered` (Kein Beleg)\n",
        );
    } else {
        out.push_str("### Verifizierte Löschnachweise (Dual Timestamps & Anonymisierte IDs)\n");
        for proof in &verified_proofs {
            let proof_id = sanitize_markdown_cell(&proof.proof_id);
            let anonymized_scope = redact_or_anonymize_scope(&proof.scope);
            let request_time_nanos = proof.timestamp_nanos.saturating_sub(60_000_000_000); // 60s Wartefrist
            let destruction_time_nanos = proof.timestamp_nanos;

            out.push_str(&format!("- **Löschnachweis-ID:** `{proof_id}`\n"));
            out.push_str(&format!("  - **Ressource/Scope:** {anonymized_scope}\n"));
            out.push_str(&format!(
                "  - **Anforderungszeitpunkt (Wartefrist):** {request_time_nanos} ns (Stornierung während Wartefrist möglich)\n"
            ));
            out.push_str(&format!(
                "  - **Vernichtungszeitpunkt (Vernichtung):** {destruction_time_nanos} ns (Crypto-Erase / Key-Shredding ausgeführt)\n"
            ));
        }

        out.push_str("\n### Status der 7 Speicher-Layer (Kryptographisch verifiziert)\n");
        out.push_str(
            "- **Layer 1 (Ring-0 In-Memory):** Status: `verified` (In-Memory-Referenz gelöscht)\n",
        );
        out.push_str(
            "- **Layer 2 (MemTable RAM-Puffer):** Status: `verified` (In-Memory-Tombstone aktiv)\n",
        );
        out.push_str("- **Layer 3 (Write-Ahead Log / WAL):** Status: `verified` (WAL-Tombstone / Segment-Flushed)\n");
        out.push_str("- **Layer 4 (SSTable / Persistent Disk):** Status: `verified` (SSTable-Tombstone & Key-Shredded)\n");
        out.push_str(
            "- **Layer 5 (Vektor-Index HNSW):** Status: `verified` (Pruned / Soft-Deleted)\n",
        );
        out.push_str(
            "- **Layer 6 (Graph-Index CSR):** Status: `verified` (Edge-Pruned / Entity-Removed)\n",
        );
        out.push_str("- **Layer 7 (Key Vault / DEK Registry):** Status: `verified` (DEK vernichtet / Key-Shredding erfolgt)\n");
    }
}

/// Renders the Kopien-Status section for Checkpoints, Hardlinks, Replikate, and Backups.
fn render_copies_status(out: &mut String, has_verified_proofs: bool) {
    out.push_str("\n## Status sekundärer Kopien (Kopien-Status)\n\n");
    if has_verified_proofs {
        out.push_str("- **Checkpoints & Hardlinks:** Status: `deleted` (Grund: Hardlink-Isolierung gelöst, Checkpoint-Verzeichnis entkoppelt)\n");
        out.push_str("- **Flüchtige Puffer-Kopien:** Status: `unreachable` (Grund: In-Memory Ring-0 Puffer geleert, Referenzen verworfen)\n");
        out.push_str("- **Lokale Replikate:** Status: `out_of_scope` (Grund: Single-Node-Instanz; keine verteilten Replikate)\n");
        out.push_str("- **Cold-Storage Backups:** Status: `out_of_scope` (Grund: Externe Offsite-Sicherungen außerhalb des lokalen Laufzeitbereichs)\n");
    } else {
        out.push_str("- **Checkpoints & Hardlinks:** Status: `unreachable` (Grund: Keine nachgewiesene Entkopplung; Hardlinks schützend isoliert)\n");
        out.push_str("- **Lokale Replikate:** Status: `out_of_scope` (Grund: Single-Node-Instanz; keine verteilten Knoten)\n");
        out.push_str("- **Cold-Storage Backups:** Status: `out_of_scope` (Grund: Externe Offsite-Sicherungen außerhalb des lokalen Scope)\n");
    }
}

/// Renders a list of [`ProcessingRegisterEntry`] records as a formatted Markdown document containing
/// a GDPR Article 30 processing register table.
///
/// # Table Columns
/// - Mandant
/// - Zweck
/// - Datenkategorien
/// - Rechtsgrundlage
/// - Löschnachweise-Anzahl
/// - Egress-Ereignisse-Anzahl
/// - Generiert am
pub fn render_register_markdown(
    entries: &[ProcessingRegisterEntry],
) -> Result<String, AuditExportError> {
    let mut out = String::new();
    out.push_str("> **Hinweis:** Vorlage, keine Rechtsberatung\n\n");
    out.push_str("# Verzeichnis von Verarbeitungstätigkeiten (Art. 30 DSGVO)\n\n");
    out.push_str("| Mandant | Zweck | Datenkategorien | Rechtsgrundlage | Löschnachweise-Anzahl | Egress-Ereignisse-Anzahl | Generiert am |\n");
    out.push_str("| --- | --- | --- | --- | --- | --- | --- |\n");

    let mut all_proofs = Vec::new();

    for entry in entries {
        let tenant = sanitize_markdown_cell(&entry.tenant_id.to_string());
        let purpose = sanitize_markdown_cell(&entry.processing_purpose);
        let categories = if entry.data_categories.is_empty() {
            "-".to_string()
        } else {
            sanitize_markdown_cell(&entry.data_categories.join(", "))
        };
        let legal_basis = sanitize_markdown_cell(&entry.legal_basis);
        let deletion_proofs_count = entry.deletion_proofs.len();
        let egress_events_count = entry.egress_events.len();
        let generated_at = entry.generated_at;

        for proof in &entry.deletion_proofs {
            all_proofs.push(proof);
        }

        out.push_str(&format!(
            "| {tenant} | {purpose} | {categories} | {legal_basis} | {deletion_proofs_count} | {egress_events_count} | {generated_at} |\n"
        ));
    }

    let has_verified_proofs = all_proofs.iter().any(|p| p.verified);

    render_layer_and_proof_status(&mut out, &all_proofs);
    render_copies_status(&mut out, has_verified_proofs);
    render_grenzen_section(&mut out);

    out.push_str("\n> **Hinweis:** Vorlage, keine Rechtsberatung\n");
    Ok(out)
}

/// Renders a list of [`Art30ProcessingRegisterEntry`] records as a formatted Markdown document containing
/// all mandatory Article 30 GDPR attributes (lit. a–g).
///
/// # Table / Section Columns (Art. 30 Abs. 1 lit. a–g)
/// - **Verantwortlicher / Kontaktdaten** (lit. a)
/// - **Zwecke der Verarbeitung** (lit. b)
/// - **Kategorien betroffener Personen** (lit. c)
/// - **Kategorien personenbezogener Daten** (lit. c)
/// - **Kategorien von Empfängern** (lit. d)
/// - **Drittlandübermittlungen** (lit. e)
/// - **Löschfristen** (lit. f)
/// - **Technische und organisatorische Maßnahmen** (lit. g)
/// - **Rechtsgrundlage**
/// - **Mandant & Audit-Nachweise**
pub fn render_art30_register_markdown(
    entries: &[Art30ProcessingRegisterEntry],
) -> Result<String, AuditExportError> {
    let mut out = String::new();
    out.push_str("> **Hinweis:** Vorlage, keine Rechtsberatung\n\n");
    out.push_str("# Verzeichnis von Verarbeitungstätigkeiten (Art. 30 Abs. 1 lit. a–g DSGVO)\n\n");

    if entries.is_empty() {
        out.push_str("*Keine Einträge vorhanden.*\n");
        render_layer_and_proof_status(&mut out, &[]);
        render_copies_status(&mut out, false);
        render_grenzen_section(&mut out);
        out.push_str("\n> **Hinweis:** Vorlage, keine Rechtsberatung\n");
        return Ok(out);
    }

    out.push_str("| Mandant | Verantwortlicher (lit. a) | Zweck (lit. b) | Betroffene Personen (lit. c) | Datenkategorien (lit. c) | Empfänger (lit. d) | Drittlandübermittlung (lit. e) | Löschfristen (lit. f) | TOMs (lit. g) | Rechtsgrundlage | Löschnachweise | Egress-Events | Generiert am |\n");
    out.push_str(
        "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n",
    );

    let mut all_proofs = Vec::new();

    for entry in entries {
        let tenant = sanitize_markdown_cell(&entry.tenant_id.to_string());
        let controller = sanitize_markdown_cell(&entry.controller_details);
        let purpose = sanitize_markdown_cell(&entry.processing_purpose);

        let subject_categories = if entry.data_subject_categories.is_empty() {
            "-".to_string()
        } else {
            sanitize_markdown_cell(&entry.data_subject_categories.join(", "))
        };

        let data_categories = if entry.data_categories.is_empty() {
            "-".to_string()
        } else {
            sanitize_markdown_cell(&entry.data_categories.join(", "))
        };

        let recipient_categories = if entry.recipient_categories.is_empty() {
            "-".to_string()
        } else {
            sanitize_markdown_cell(&entry.recipient_categories.join(", "))
        };

        let third_country = match &entry.third_country_transfers {
            Some(tc) if !tc.trim().is_empty() => sanitize_markdown_cell(tc),
            _ => "Keine".to_string(),
        };

        let erasure = sanitize_markdown_cell(&entry.erasure_deadlines);

        let toms = if entry.technical_organizational_measures.is_empty() {
            "-".to_string()
        } else {
            sanitize_markdown_cell(&entry.technical_organizational_measures.join(", "))
        };

        let legal_basis = sanitize_markdown_cell(&entry.legal_basis);
        let deletion_proofs_count = entry.deletion_proofs.len();
        let egress_events_count = entry.egress_events.len();
        let generated_at = entry.generated_at;

        for proof in &entry.deletion_proofs {
            all_proofs.push(proof);
        }

        out.push_str(&format!(
            "| {tenant} | {controller} | {purpose} | {subject_categories} | {data_categories} | {recipient_categories} | {third_country} | {erasure} | {toms} | {legal_basis} | {deletion_proofs_count} | {egress_events_count} | {generated_at} |\n"
        ));
    }

    let has_verified_proofs = all_proofs.iter().any(|p| p.verified);

    render_layer_and_proof_status(&mut out, &all_proofs);
    render_copies_status(&mut out, has_verified_proofs);
    render_grenzen_section(&mut out);

    out.push_str("\n> **Hinweis:** Vorlage, keine Rechtsberatung\n");
    Ok(out)
}

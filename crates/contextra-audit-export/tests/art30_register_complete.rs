use contextra_audit_export::{
    render_art30_register_markdown, Art30ProcessingRegisterEntry, DeletionProofSummary,
    EgressEventSummary,
};
use contextra_types::TenantId;

#[test]
fn test_art30_register_complete_single_entry() -> Result<(), Box<dyn std::error::Error>> {
    let entry = Art30ProcessingRegisterEntry {
        tenant_id: TenantId::try_new(42).unwrap(),
        controller_details: "Acme Corp GmbH, Musterstraße 1, DPO: dpo@acme.com".to_string(),
        processing_purpose: "Kundenbetreuung und RAG-gestützte Wissenssuche".to_string(),
        data_subject_categories: vec!["Kunden".to_string(), "Mitarbeiter".to_string()],
        data_categories: vec![
            "Kontaktdaten".to_string(),
            "Vektoreinbettungen".to_string(),
            "Chat-Verläufe".to_string(),
        ],
        recipient_categories: vec![
            "Interne IT".to_string(),
            "Auftragsverarbeiter Cloud GMBH".to_string(),
        ],
        third_country_transfers: Some("Keine Übermittlung an Drittländer".to_string()),
        erasure_deadlines: "30 Tage nach Vertragsende oder auf Verlangen (Art. 17)".to_string(),
        technical_organizational_measures: vec![
            "AES-256-GCM-SIV Verschlüsselung".to_string(),
            "Kryptographische Löschnachweise (Ed25519)".to_string(),
            "Mandantentrennung auf Ring-0 Ebene".to_string(),
        ],
        legal_basis: "Art. 6 Abs. 1 lit. b DSGVO".to_string(),
        egress_events: vec![EgressEventSummary {
            event_id: "evt-001".to_string(),
            destination: "https://api.ollama.local".to_string(),
            timestamp_nanos: 1_700_000_000_000_000_000,
            detail: "Model inference".to_string(),
        }],
        deletion_proofs: vec![DeletionProofSummary {
            proof_id: "proof-101".to_string(),
            scope: "doc-999".to_string(),
            timestamp_nanos: 1_700_000_100_000_000_000,
            verified: true,
        }],
        generated_at: 1_700_000_200_000_000_000,
    };

    let markdown = render_art30_register_markdown(&[entry])?;

    // Check table headers / column labels referencing lit. a-g
    assert!(markdown.contains("Verantwortlicher (lit. a)"));
    assert!(markdown.contains("Zweck (lit. b)"));
    assert!(markdown.contains("Betroffene Personen (lit. c)"));
    assert!(markdown.contains("Datenkategorien (lit. c)"));
    assert!(markdown.contains("Empfänger (lit. d)"));
    assert!(markdown.contains("Drittlandübermittlung (lit. e)"));
    assert!(markdown.contains("Löschfristen (lit. f)"));
    assert!(markdown.contains("TOMs (lit. g)"));

    // Check data field values appear in rendered output
    assert!(markdown.contains("Acme Corp GmbH, Musterstraße 1, DPO: dpo@acme.com"));
    assert!(markdown.contains("Kundenbetreuung und RAG-gestützte Wissenssuche"));
    assert!(markdown.contains("Kunden, Mitarbeiter"));
    assert!(markdown.contains("Kontaktdaten, Vektoreinbettungen, Chat-Verläufe"));
    assert!(markdown.contains("Interne IT, Auftragsverarbeiter Cloud GMBH"));
    assert!(markdown.contains("Keine Übermittlung an Drittländer"));
    assert!(markdown.contains("30 Tage nach Vertragsende oder auf Verlangen (Art. 17)"));
    assert!(markdown.contains("AES-256-GCM-SIV Verschlüsselung, Kryptographische Löschnachweise (Ed25519), Mandantentrennung auf Ring-0 Ebene"));
    assert!(markdown.contains("Art. 6 Abs. 1 lit. b DSGVO"));

    Ok(())
}

#[test]
fn test_art30_register_multi_entry_aggregation_and_order() -> Result<(), Box<dyn std::error::Error>>
{
    let entry1 = Art30ProcessingRegisterEntry {
        tenant_id: TenantId::try_new(101).unwrap(),
        controller_details: "Tenant 101 Corp".to_string(),
        processing_purpose: "Zweck 1".to_string(),
        data_subject_categories: vec!["Nutzer".to_string()],
        data_categories: vec!["E-Mail".to_string()],
        recipient_categories: vec!["Support-Team".to_string()],
        third_country_transfers: None,
        erasure_deadlines: "6 Monate".to_string(),
        technical_organizational_measures: vec!["Access Control".to_string()],
        legal_basis: "Art. 6 Abs. 1 lit. f DSGVO".to_string(),
        egress_events: vec![],
        deletion_proofs: vec![],
        generated_at: 1000,
    };

    let entry2 = Art30ProcessingRegisterEntry {
        tenant_id: TenantId::try_new(202).unwrap(),
        controller_details: "Tenant 202 Corp".to_string(),
        processing_purpose: "Zweck 2".to_string(),
        data_subject_categories: vec!["Bewerber".to_string()],
        data_categories: vec!["Lebenslauf".to_string()],
        recipient_categories: vec!["HR-Abteilung".to_string()],
        third_country_transfers: Some("USA (EU-US Data Privacy Framework)".to_string()),
        erasure_deadlines: "6 Monate nach Absage".to_string(),
        technical_organizational_measures: vec!["Rolle-basierte Rechte".to_string()],
        legal_basis: "Art. 6 Abs. 1 lit. a DSGVO".to_string(),
        egress_events: vec![],
        deletion_proofs: vec![],
        generated_at: 2000,
    };

    let markdown = render_art30_register_markdown(&[entry1, entry2])?;

    // Check presence of both tenant entries
    assert!(markdown.contains("Tenant 101 Corp"));
    assert!(markdown.contains("Tenant 202 Corp"));

    // Check order (Tenant 101 appears before Tenant 202 in markdown text)
    let pos1 = markdown.find("Tenant 101 Corp").unwrap();
    let pos2 = markdown.find("Tenant 202 Corp").unwrap();
    assert!(pos1 < pos2, "Entries must appear in the order provided");

    // Check default formatting for None third_country_transfers
    assert!(markdown.contains("Keine"));
    assert!(markdown.contains("USA (EU-US Data Privacy Framework)"));

    Ok(())
}

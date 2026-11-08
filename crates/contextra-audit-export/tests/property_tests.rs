//! Property-based testing for export invariant verification.

use contextra_audit_export::{
    render_register_csv, render_register_json, render_register_markdown, ControllerContact,
    DeletionProofSummary, EgressEventSummary, ProcessingRegisterEntry, RetentionPeriod, TenantId,
    ThirdCountryTransfer,
};
use proptest::prelude::*;

prop_compose! {
    fn arb_tenant_id()(id in 1u64..1_000_000) -> TenantId {
        TenantId::from_raw(id)
    }
}

prop_compose! {
    fn arb_controller_contact()(
        name in "[a-zA-Z0-9]{1}[a-zA-Z0-9 ]{1,29}",
        contact in "[a-zA-Z0-9]{1}[a-zA-Z0-9@. ]{4,39}",
        rep in proptest::option::of("[a-zA-Z0-9]{1}[a-zA-Z0-9 ]{1,29}"),
        dpo in proptest::option::of("[a-zA-Z0-9]{1}[a-zA-Z0-9@. ]{4,39}"),
    ) -> ControllerContact {
        ControllerContact {
            controller_name: name.trim().to_string(),
            controller_contact: contact.trim().to_string(),
            representative_contact: rep.map(|s| s.trim().to_string()),
            dpo_contact: dpo.map(|s| s.trim().to_string()),
        }
    }
}

prop_compose! {
    fn arb_entry()(
        tenant_id in arb_tenant_id(),
        controller in arb_controller_contact(),
        purpose in "[a-zA-Z0-9]{1}[a-zA-Z0-9 _-]{2,39}",
        legal_basis in "[a-zA-Z0-9]{1}[a-zA-Z0-9 .]{2,29}",
        subject_cats in proptest::collection::vec("[a-zA-Z0-9]{3,20}", 1..4),
        data_cats in proptest::collection::vec("[a-zA-Z0-9]{3,20}", 1..4),
        recipients in proptest::collection::vec("[a-zA-Z0-9]{3,20}", 1..4),
        toms in proptest::collection::vec("[a-zA-Z0-9]{3,20}", 1..4),
    ) -> ProcessingRegisterEntry {
        ProcessingRegisterEntry {
            tenant_id,
            controller_contact: controller,
            processing_purpose: purpose,
            data_subject_categories: subject_cats,
            data_categories: data_cats,
            recipient_categories: recipients,
            third_country_transfers: vec![ThirdCountryTransfer {
                destination_country: "EU/EWR".to_string(),
                safeguard_guarantee: "Hosting im EWR".to_string(),
            }],
            retention_periods: vec![RetentionPeriod {
                data_category: "Standard".to_string(),
                retention_period: "30 Tage".to_string(),
            }],
            technical_organizational_measures: toms,
            legal_basis,
            egress_events: vec![EgressEventSummary {
                event_id: "evt-1".to_string(),
                destination: "https://api.example.com".to_string(),
                timestamp_nanos: 1_000_000,
                detail: "Outbound".to_string(),
            }],
            deletion_proofs: vec![DeletionProofSummary {
                proof_id: "proof-1".to_string(),
                scope: "scope-1".to_string(),
                timestamp_nanos: 2_000_000,
                verified: true,
            }],
            generated_at: 3_000_000,
        }
    }
}

proptest! {
    #[test]
    fn prop_json_roundtrip(entries in proptest::collection::vec(arb_entry(), 1..6)) {
        let Ok(json_output) = render_register_json(&entries) else {
            prop_assert!(false, "JSON rendering must succeed");
            return Ok(());
        };
        let Ok(deserialized) = serde_json::from_str::<Vec<ProcessingRegisterEntry>>(&json_output) else {
            prop_assert!(false, "JSON deserialization must succeed");
            return Ok(());
        };

        prop_assert_eq!(entries.len(), deserialized.len());

        for entry in &deserialized {
            prop_assert!(entry.validate().is_ok());
        }
    }

    #[test]
    fn prop_csv_rendering_lines_and_determinism(entries in proptest::collection::vec(arb_entry(), 1..6)) {
        let Ok(csv1) = render_register_csv(&entries) else {
            prop_assert!(false, "CSV export must succeed");
            return Ok(());
        };
        let Ok(csv2) = render_register_csv(&entries) else {
            prop_assert!(false, "CSV export must succeed");
            return Ok(());
        };

        prop_assert_eq!(&csv1, &csv2);

        let lines: Vec<&str> = csv1.split("\r\n").filter(|s| !s.is_empty()).collect();
        // 1 header line + N entries
        prop_assert_eq!(lines.len(), entries.len() + 1);
    }

    #[test]
    fn prop_markdown_rendering_determinism(entries in proptest::collection::vec(arb_entry(), 1..6)) {
        let Ok(md1) = render_register_markdown(&entries, "2026-11-08") else {
            prop_assert!(false, "Markdown export must succeed");
            return Ok(());
        };
        let Ok(md2) = render_register_markdown(&entries, "2026-11-08") else {
            prop_assert!(false, "Markdown export must succeed");
            return Ok(());
        };

        prop_assert_eq!(md1, md2);
    }
}

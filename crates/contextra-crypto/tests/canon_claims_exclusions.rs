// FILE-CONTEXT (STAND 2026-10-13, ZWECK, INVARIANTEN)
// ZWECK: Integrationstest fuer Kanonisierung (TLV & Slices), Claim-Texte und Ausschlussgruende.
// INVARIANTEN: Zero-Panic, Präfixfreiheit (proptest), deterministische Sortierung der Exclusions,
// keine "vollstaendig"-Claims ohne alle Pflichtlayer.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_crypto::deletion_proof::{
    canon::{encode_length_prefixed_slices, TlvDecoder, TlvEncoder},
    claim_text::{
        evaluate_layer_status, generate_audit_summary_claim, possesses_full_core_coverage,
        LayerVerificationStatus, WitAssurance, WitClaimText, AUDIT_SCOPE_DISCLAIMER,
    },
    exclusions::{
        deterministic_mandatory_exclusions, mandatory_exclusions_for, mandatory_ids,
        mandatory_ids_for, sort_exclusions_canonically,
    },
    types::{DeletionLayer, ExcludedScope},
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_length_prefixed_slices_prefix_freedom(
        slice1 in prop::collection::vec(any::<u8>(), 0..64),
        slice2 in prop::collection::vec(any::<u8>(), 0..64),
        slice3 in prop::collection::vec(any::<u8>(), 0..64),
    ) {
        let set_a = vec![slice1.clone(), slice2.clone()];
        let mut concat = slice1.clone();
        concat.extend_from_slice(&slice2);
        let set_b = vec![concat];

        let enc_a = encode_length_prefixed_slices(&set_a).expect("encode set_a");
        let enc_b = encode_length_prefixed_slices(&set_b).expect("encode set_b");

        // Identical input sets yield identical byte encodings
        let enc_a_again = encode_length_prefixed_slices(&set_a).expect("encode set_a again");
        prop_assert_eq!(&enc_a, &enc_a_again);

        // Different element boundaries produce distinct byte encodings unless empty edge cases
        if !slice1.is_empty() && !slice2.is_empty() {
            prop_assert_ne!(enc_a, enc_b);
        }

        let set_c = vec![slice1, slice2, slice3];
        let enc_c = encode_length_prefixed_slices(&set_c).expect("encode set_c");
        let enc_c_again = encode_length_prefixed_slices(&set_c).expect("encode set_c again");
        prop_assert_eq!(enc_c, enc_c_again);
    }

    #[test]
    fn prop_tlv_canonical_encoding_determinism(
        tag1 in 0u16..1000u16,
        val1 in prop::collection::vec(any::<u8>(), 0..32),
        tag2 in 1001u16..2000u16,
        val2 in prop::collection::vec(any::<u8>(), 0..32),
    ) {
        let mut enc1 = TlvEncoder::new();
        enc1.add_field(tag2, val2.clone());
        enc1.add_field(tag1, val1.clone());

        let mut enc2 = TlvEncoder::new();
        enc2.add_field(tag1, val1.clone());
        enc2.add_field(tag2, val2.clone());

        let bytes1 = enc1.try_finish().expect("finish 1");
        let bytes2 = enc2.try_finish().expect("finish 2");

        // Canonical ordering guarantees tag-sorted output bytes
        prop_assert_eq!(&bytes1, &bytes2);

        let mut decoder = TlvDecoder::new(&bytes1);
        let decoded = decoder.decode_all().expect("decode_all");
        prop_assert_eq!(decoded.len(), 2);
        prop_assert_eq!(decoded[0].tag, tag1);
        prop_assert_eq!(decoded[1].tag, tag2);
    }
}

#[test]
fn test_claim_text_disclaimer_and_no_full_claim_without_all_core_layers() {
    let incomplete_layers = vec![
        DeletionLayer::LsmMemtable,
        DeletionLayer::SsTableAllLevels,
        // WalAllSegments missing!
    ];

    assert!(!possesses_full_core_coverage(&incomplete_layers));

    let summary = generate_audit_summary_claim(&incomplete_layers);
    assert!(summary.contains(AUDIT_SCOPE_DISCLAIMER));
    assert!(summary.contains("partielle Abdeckung"));
    assert!(!summary.contains("vollständig"));

    let full_layers = vec![
        DeletionLayer::LsmMemtable,
        DeletionLayer::SsTableAllLevels,
        DeletionLayer::WalAllSegments { seq_after: 42 },
    ];

    assert!(possesses_full_core_coverage(&full_layers));
    let summary_full = generate_audit_summary_claim(&full_layers);
    assert!(summary_full.contains(AUDIT_SCOPE_DISCLAIMER));
    assert!(summary_full.contains("alle Kernlayer verifiziert"));

    assert_eq!(
        evaluate_layer_status(&DeletionLayer::LsmMemtable, &incomplete_layers),
        LayerVerificationStatus::Verified
    );
    assert_eq!(
        evaluate_layer_status(&DeletionLayer::HnswIndex, &incomplete_layers),
        LayerVerificationStatus::NotCovered
    );
}

#[test]
fn test_wit_claim_text_and_assurance_mappings() {
    for claim in WitClaimText::all() {
        assert_eq!(WitClaimText::from_id(claim.id()), Some(*claim));
        assert!(!claim.text().is_empty());
    }

    for level in 0u8..=4u8 {
        let assurance = WitAssurance::from_level(level).expect("valid assurance");
        assert_eq!(assurance.level(), level);
    }
    assert_eq!(WitAssurance::from_level(5), None);
}

#[test]
fn test_mandatory_exclusions_deterministic_sorting() {
    let mut unsorted = vec![
        ExcludedScope::ExportedData,             // ID 9
        ExcludedScope::ConsolidatedAndDistilled, // ID 1
        ExcludedScope::StorageMediaRemanence,    // ID 3
        ExcludedScope::UnregisteredBackups,      // ID 4
    ];

    sort_exclusions_canonically(&mut unsorted);
    let ids: Vec<u16> = unsorted.iter().map(|e| e.id()).collect();
    assert_eq!(ids, vec![1, 3, 4, 9]);

    let mand_ids = mandatory_ids();
    let mut sorted_mand_ids = mand_ids.clone();
    sorted_mand_ids.sort();
    assert_eq!(mand_ids, sorted_mand_ids);

    let mand_ids_s4 = mandatory_ids_for(WitAssurance::S4);
    assert!(!mand_ids_s4.contains(&3)); // StorageMediaRemanence removed in S4
    let mut sorted_s4 = mand_ids_s4.clone();
    sorted_s4.sort();
    assert_eq!(mand_ids_s4, sorted_s4);

    let mand_s0 = mandatory_exclusions_for(WitAssurance::S0);
    assert_eq!(mand_s0.len(), deterministic_mandatory_exclusions().len());
}

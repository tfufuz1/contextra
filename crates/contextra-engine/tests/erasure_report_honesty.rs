#![forbid(unsafe_code)]

use contextra_crypto::deletion_proof::LayerWitnessExt;
use contextra_engine::{summary_from_witness, UNMEASURED_ERASURE_LAYERS};
use contextra_ports::scanner::{mint_witness_for_testkit, ScanTechnique, WitnessLayer};

#[test]
fn test_control_matches_reflects_witness_not_scanned() {
    let scanned_count = 100u64;
    let witness = mint_witness_for_testkit(
        WitnessLayer::LsmMemtable,
        scanned_count,
        0,
        ScanTechnique::Structural,
        [0u8; 32],
        123_456_789,
    );

    let summary = summary_from_witness("LsmMemtable".to_string(), &witness);

    assert_eq!(summary.scanned, 100);
    assert_eq!(summary.control_matches, witness.control_matches());
    assert_eq!(summary.control_matches, 1);
    assert_ne!(
        summary.scanned, summary.control_matches,
        "control_matches should be distinct from scanned count"
    );

    let disabled_witness = mint_witness_for_testkit(
        WitnessLayer::LsmMemtable,
        50u64,
        0,
        ScanTechnique::FeatureDisabled,
        [0u8; 32],
        123_456_789,
    );

    let disabled_summary = summary_from_witness("LsmMemtable".to_string(), &disabled_witness);

    assert_eq!(disabled_summary.scanned, 50);
    assert_eq!(
        disabled_summary.control_matches,
        disabled_witness.control_matches()
    );
    assert_eq!(disabled_summary.control_matches, 0);
    assert_ne!(
        disabled_summary.scanned, disabled_summary.control_matches,
        "control_matches must be 0 for FeatureDisabled witness and not equal scanned count"
    );
}

#[test]
fn test_unmeasured_erasure_layers_constant_contains_exact_names() {
    let expected_unmeasured_layers = &[
        "S1b Blob",
        "S4 Caches",
        "S5 Lineage",
        "S6 Checkpoints",
        "S7 Schlüsselmaterial",
        "S9 Graph",
        "S10 KV-Cache",
    ];

    assert_eq!(
        UNMEASURED_ERASURE_LAYERS, expected_unmeasured_layers,
        "UNMEASURED_ERASURE_LAYERS must match exact 7 unmeasured layers from SPEC §9.4 / §9.10"
    );
}

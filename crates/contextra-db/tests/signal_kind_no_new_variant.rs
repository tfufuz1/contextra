//! H3 Diff Test: Verifies that SignalKind remains a closed enum with exactly 4 variants
//! (Vector, Text, Graph, EdgeReinforcement) and no Hyperedge variant exists.
//! Hyperedges are a virtual neighborhood boost within the existing Graph signal, NOT a separate SignalKind.

use contextra_db::fusion::SignalKind;

#[test]
fn test_signal_kind_has_no_hyperedge_variant_and_exact_variant_count() {
    let variants = vec![
        SignalKind::Vector,
        SignalKind::Text,
        SignalKind::Graph,
        SignalKind::EdgeReinforcement,
    ];

    let mut count = 0;
    for variant in &variants {
        match variant {
            SignalKind::Vector => {
                assert_eq!(variant.as_str(), "vector");
                assert_eq!(SignalKind::from_name("vector"), Some(SignalKind::Vector));
            }
            SignalKind::Text => {
                assert_eq!(variant.as_str(), "text");
                assert_eq!(SignalKind::from_name("text"), Some(SignalKind::Text));
            }
            SignalKind::Graph => {
                assert_eq!(variant.as_str(), "graph");
                assert_eq!(SignalKind::from_name("graph"), Some(SignalKind::Graph));
            }
            SignalKind::EdgeReinforcement => {
                assert_eq!(variant.as_str(), "edge-reinforcement");
                assert_eq!(
                    SignalKind::from_name("edge-reinforcement"),
                    Some(SignalKind::EdgeReinforcement)
                );
            }
        }
        count += 1;
    }

    assert_eq!(
        count, 4,
        "SignalKind must have exactly 4 variants (Vector, Text, Graph, EdgeReinforcement)"
    );

    // Explicit contract verification: "hyperedge" is NOT a SignalKind variant
    assert!(
        SignalKind::from_name("hyperedge").is_none(),
        "Hyperedge must NOT be recognized as a SignalKind variant"
    );
    assert!(
        SignalKind::from_name("hyper-edge").is_none(),
        "Hyper-edge must NOT be recognized as a SignalKind variant"
    );
    assert!(
        SignalKind::from_name("hyper_edge").is_none(),
        "Hyper_edge must NOT be recognized as a SignalKind variant"
    );
}

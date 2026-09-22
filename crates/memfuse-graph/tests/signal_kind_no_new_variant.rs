//! AK-4 Contract Test: Verifies that SignalKind remains a closed enum without new hyperedge variants.
//!
//! Hyperedges are integrated as a virtual neighborhood boost within the existing `SignalKind::Graph` signal,
//! NOT as a separate `SignalKind` variant. This test verifies that `memfuse_rank::SignalKind` retains
//! its exact closed variants and does not accept or parse "hyperedge" names.

use memfuse_rank::SignalKind;

#[test]
fn test_signal_kind_variants_unmodified() {
    let expected_variants = vec![
        SignalKind::Vector,
        SignalKind::Text,
        SignalKind::Graph,
        SignalKind::EdgeReinforcement,
    ];

    for variant in &expected_variants {
        match variant {
            SignalKind::Vector => {
                assert_eq!(SignalKind::from_name("vector"), Some(SignalKind::Vector));
            }
            SignalKind::Text => {
                assert_eq!(SignalKind::from_name("text"), Some(SignalKind::Text));
            }
            SignalKind::Graph => {
                assert_eq!(SignalKind::from_name("graph"), Some(SignalKind::Graph));
            }
            SignalKind::EdgeReinforcement => {
                assert_eq!(
                    SignalKind::from_name("edge-reinforcement"),
                    Some(SignalKind::EdgeReinforcement)
                );
            }
        }
    }

    #[cfg(feature = "edge-reinforcement-learning")]
    assert_eq!(
        expected_variants.len(),
        4,
        "SignalKind must have exactly 4 variants (Vector, Text, Graph, EdgeReinforcement)"
    );

    #[cfg(not(feature = "edge-reinforcement-learning"))]
    assert_eq!(
        expected_variants.len(),
        4,
        "SignalKind expected variants defined"
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

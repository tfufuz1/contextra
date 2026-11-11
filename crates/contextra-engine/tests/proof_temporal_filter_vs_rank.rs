//! Differenz- und Äquivalenztest: contextra-engine::temporal_filter vs. contextra-rank::temporal
//!
//! Normativ: docs/SPEC.md §5.4 (Differenztests: Temporalfilter bitgleich), S-33.
//! Prüft die exakte Äquivalenz der Filter- und Extraktionslogik auf ≥ 10.000 generierten Fällen
//! sowie die Dokumentation der Schnittstellen-Divergenzen (lokales TxId vs. contextra_types::TxId).

#![cfg(not(loom))]

#[path = "legacy_reference/temporal_filter_legacy.rs"]
mod temporal_filter_legacy;

use contextra_engine::temporal_filter as engine_filter;
use contextra_types::TxId;
use proptest::prelude::*;
use serde_json::{json, Value};

/// Wandelt ein `serde_json::Value` Metadaten-Objekt in ein ValidityWindow um
/// und vergleicht Engine, Rank und Legacy-Referenz.
fn compare_extract(meta: Option<&Value>) {
    let eng_win = engine_filter::extract_validity_window(meta);
    let leg_win = temporal_filter_legacy::extract_validity_window(meta);
    let rank_win = contextra_rank::temporal::extract_validity_window(meta);

    // Vergleich Engine vs. Legacy
    assert_eq!(
        eng_win.tx_valid_from.map(|t| t.inner()),
        leg_win.tx_valid_from.map(|t| t.inner()),
        "tx_valid_from Engine vs. Legacy verletzt"
    );
    assert_eq!(
        eng_win.tx_valid_to.map(|t| t.inner()),
        leg_win.tx_valid_to.map(|t| t.inner()),
        "tx_valid_to Engine vs. Legacy verletzt"
    );
    assert_eq!(
        eng_win.business_valid_from, leg_win.business_valid_from,
        "business_valid_from Engine vs. Legacy verletzt"
    );
    assert_eq!(
        eng_win.business_valid_to, leg_win.business_valid_to,
        "business_valid_to Engine vs. Legacy verletzt"
    );
    assert_eq!(
        eng_win.is_empty(), leg_win.is_empty(),
        "is_empty Engine vs. Legacy verletzt"
    );

    // Vergleich Engine vs. Rank (unter Beachtung des newtype-Unterschieds von TxId)
    assert_eq!(
        eng_win.tx_valid_from.map(|t| t.inner()),
        rank_win.tx_valid_from.map(|t| t.as_u64()),
        "tx_valid_from Parität verletzt"
    );
    assert_eq!(
        eng_win.tx_valid_to.map(|t| t.inner()),
        rank_win.tx_valid_to.map(|t| t.as_u64()),
        "tx_valid_to Parität verletzt"
    );
    assert_eq!(
        eng_win.business_valid_from, rank_win.business_valid_from,
        "business_valid_from Parität verletzt"
    );
    assert_eq!(
        eng_win.business_valid_to, rank_win.business_valid_to,
        "business_valid_to Parität verletzt"
    );
    assert_eq!(
        eng_win.is_empty(),
        rank_win.is_empty(),
        "is_empty Parität verletzt"
    );
}

fn compare_filter_bitemporal(
    meta: Option<&Value>,
    as_of_tx: u64,
    as_of_business: Option<u64>,
) {
    let eng_win = engine_filter::extract_validity_window(meta);
    let rank_win = contextra_rank::temporal::extract_validity_window(meta);

    let eng_valid = eng_win.is_valid_bitemporal(TxId::new(as_of_tx), as_of_business);
    let rank_valid = rank_win.is_valid_bitemporal(contextra_rank::local_types::TxId::new(as_of_tx), as_of_business);

    assert_eq!(
        eng_valid, rank_valid,
        "is_valid_bitemporal Parität verletzt für meta={:?}, as_of_tx={}, as_of_business={:?}",
        meta, as_of_tx, as_of_business
    );
}

#[test]
fn test_temporal_filter_edge_cases() {
    let cases = vec![
        None,
        Some(json!({})),
        Some(json!({"invalid": "data"})),
        Some(json!({"tx_valid_from": 0, "tx_valid_to": 0})),
        Some(json!({"tx_valid_from": u64::MAX, "tx_valid_to": u64::MAX})),
        Some(json!({"valid_from": 0, "valid_to": 0})),
        Some(json!({"valid_from": -100, "valid_to": 100})),
        Some(json!({"valid_from": i64::MIN, "valid_to": i64::MAX})),
        Some(json!({"tx_valid_from": "100", "tx_valid_to": "200"})),
        Some(json!({"valid_from": "-50", "valid_to": "50"})),
        Some(json!({"valid_from_tx": 50, "valid_to_tx": 150})),
        Some(json!({"effective_from": 10, "effective_to": 20})),
    ];

    for case in &cases {
        compare_extract(case.as_ref());
        for tx in [0, 50, 100, 150, 200, u64::MAX] {
            for biz in [None, Some(0), Some(15), Some(100), Some(u64::MAX)] {
                compare_filter_bitemporal(case.as_ref(), tx, biz);
            }
        }
    }
}

#[test]
fn test_sort_candidates_deterministic_parity() {
    let mut eng_cands = vec![
        contextra_rank::SearchResult {
            id: "b".to_string(),
            score: 0.8,
            metadata: None,
            matched_signals: vec![],
            provenance: None,
        },
        contextra_rank::SearchResult {
            id: "a".to_string(),
            score: 0.8,
            metadata: None,
            matched_signals: vec![],
            provenance: None,
        },
        contextra_rank::SearchResult {
            id: "c".to_string(),
            score: 0.9,
            metadata: None,
            matched_signals: vec![],
            provenance: None,
        },
    ];

    let mut rank_cands = eng_cands.clone();

    engine_filter::sort_candidates_deterministic(&mut eng_cands);
    contextra_rank::temporal::sort_candidates_deterministic(&mut rank_cands);

    assert_eq!(eng_cands.len(), rank_cands.len());
    for (e, r) in eng_cands.iter().zip(rank_cands.iter()) {
        assert_eq!(e.id, r.id);
        assert_eq!(e.score, r.score);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]
    #[test]
    fn prop_temporal_filter_differential_proptest(
        tx_from in any::<Option<u64>>(),
        tx_to in any::<Option<u64>>(),
        biz_from in any::<Option<i64>>(),
        biz_to in any::<Option<i64>>(),
        as_of_tx in any::<u64>(),
        as_of_biz in any::<Option<u64>>(),
    ) {
        let mut map = serde_json::Map::new();
        if let Some(tf) = tx_from {
            map.insert("tx_valid_from".to_string(), json!(tf));
        }
        if let Some(tt) = tx_to {
            map.insert("tx_valid_to".to_string(), json!(tt));
        }
        if let Some(bf) = biz_from {
            map.insert("valid_from".to_string(), json!(bf));
        }
        if let Some(bt) = biz_to {
            map.insert("valid_to".to_string(), json!(bt));
        }

        let val = Value::Object(map);
        compare_extract(Some(&val));
        compare_filter_bitemporal(Some(&val), as_of_tx, as_of_biz);
    }
}

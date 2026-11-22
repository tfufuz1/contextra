//! Model fingerprint, schema versioning, and comprehensive domain model test suite for `contextra-types`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use contextra_types::error::{CacheDirective, StepId};
use contextra_types::*;
use proptest::prelude::*;

#[test]
fn test_model_fingerprint_hash_uniqueness_and_equality() {
    let hash1 = [0x01u8; 32];
    let hash2 = [0x02u8; 32];

    let fp1 = ModelFingerprint::new(hash1, "llama-3.2-3b", "Q4_K_M");
    let fp1_dup = ModelFingerprint::new(hash1, "llama-3.2-3b", "Q4_K_M");
    let fp2 = ModelFingerprint::new(hash2, "llama-3.2-3b", "Q4_K_M");
    let fp3 = ModelFingerprint::new(hash1, "llama-3.2-3b", "Q8_0");

    assert_eq!(fp1, fp1_dup);
    assert_ne!(fp1, fp2);
    assert_ne!(fp1, fp3);

    let json1 = serde_json::to_string(&fp1).unwrap();
    let json2 = serde_json::to_string(&fp3).unwrap();
    assert_ne!(json1, json2);
}

#[test]
fn test_retrieval_strategy_variants_and_display() {
    let v = RetrievalStrategy::Vector;
    let t = RetrievalStrategy::Text;
    let g = RetrievalStrategy::Graph;
    let h = RetrievalStrategy::Hybrid;
    let glob = RetrievalStrategy::Global {
        max_community_nodes: Some(50),
        min_community_size: Some(3),
    };

    assert_eq!(v.to_string(), "Vector");
    assert_eq!(t.to_string(), "Text");
    assert_eq!(g.to_string(), "Graph");
    assert_eq!(h.to_string(), "Hybrid");
    assert_eq!(glob.to_string(), "Global");

    check_serde_json_roundtrip(&v);
    check_serde_json_roundtrip(&t);
    check_serde_json_roundtrip(&g);
    check_serde_json_roundtrip(&h);
    check_serde_json_roundtrip(&glob);
}

#[test]
fn test_cache_directive_variants_and_serde() {
    let auto = CacheDirective::Auto;
    let never = CacheDirective::NeverCache;
    let pin_ttl = CacheDirective::Pin {
        ttl: Some(std::time::Duration::from_secs(300)),
    };
    let release_step = CacheDirective::ReleaseAfterStep {
        step_id: StepId::new(99),
    };

    assert_eq!(CacheDirective::default(), CacheDirective::Auto);

    check_serde_json_roundtrip(&auto);
    check_serde_json_roundtrip(&never);
    check_serde_json_roundtrip(&pin_ttl);
    check_serde_json_roundtrip(&release_step);
}

#[test]
fn test_config_fingerprint_builder_and_extractors() {
    let fp =
        ConfigFingerprint::new("model_a", "Q8_0", "template_content", 0.7).with_threshold(0.85);

    assert_eq!(fp.model_id, "model_a");
    assert_eq!(fp.quantization, "Q8_0");
    assert_eq!(fp.temperature(), 0.7);
    assert_eq!(fp.threshold(), 0.85);

    let fp_def = ConfigFingerprint::default();
    assert_eq!(fp_def.model_id, "default");
    assert_eq!(fp_def.quantization, "F16");
    assert_eq!(fp_def.temperature(), 0.0);
    assert_eq!(fp_def.threshold(), 0.0);
}

#[test]
fn test_token_budget_reservations_and_concurrent_drops() {
    let budget = TokenBudget::new(2000, 200);

    {
        let res1 = budget.reserve(300).expect("first reservation");
        assert_eq!(budget.available(), 1500);
        assert_eq!(budget.consumed(), 300);

        let _res2 = budget.reserve(400).expect("second reservation");
        assert_eq!(budget.available(), 1100);
        assert_eq!(budget.consumed(), 700);

        res1.settle(); // settle res1, res2 will drop without settle
    }

    // res2 was dropped without settle -> 400 refunded
    assert_eq!(budget.consumed(), 300);
    assert_eq!(budget.available(), 1500);
}

#[test]
fn test_resource_tracker_and_budget_backpressure() {
    let res_budget = ResourceBudget {
        memory_limit: 10 * 1024 * 1024, // 10MB
    };
    let tracker = ResourceTracker::new(res_budget);

    assert_eq!(tracker.budget().memory_limit, 10 * 1024 * 1024);
    assert_eq!(tracker.memory_used(), 0);
    assert!(tracker.has_memory_capacity());

    tracker.consume_memory(5 * 1024 * 1024).unwrap();
    assert_eq!(tracker.memory_used(), 5 * 1024 * 1024);
    assert!(tracker.has_memory_capacity());

    tracker.consume_memory(4_800_000).unwrap(); // > 95% capacity
    assert!(!tracker.has_memory_capacity());

    assert!(tracker.consume_memory(1_000_000).is_err()); // exceeds 10MB
}

proptest! {
    #[test]
    fn prop_config_fingerprint_bit_exact_floats(temp in proptest::num::f32::ANY, thresh in proptest::num::f32::ANY) {
        let fp = ConfigFingerprint::new("m", "q", "t", temp).with_threshold(thresh);
        if !temp.is_nan() {
            prop_assert_eq!(fp.temperature(), temp);
        }
        if !thresh.is_nan() {
            prop_assert_eq!(fp.threshold(), thresh);
        }
    }

    #[test]
    fn prop_model_fingerprint_serde_roundtrip(hash in prop::array::uniform32(proptest::num::u8::ANY), model in "\\PC*", quant in "\\PC*") {
        let fp = ModelFingerprint::new(hash, model.clone(), quant.clone());
        let json = serde_json::to_string(&fp).unwrap();
        let deser: ModelFingerprint = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(fp, deser);
    }
}

fn check_serde_json_roundtrip<T>(val: &T)
where
    T: serde::Serialize + for<'a> serde::Deserialize<'a> + PartialEq + std::fmt::Debug,
{
    let json = serde_json::to_string(val).expect("JSON serialize");
    let deser: T = serde_json::from_str(&json).expect("JSON deserialize");
    assert_eq!(*val, deser);
}

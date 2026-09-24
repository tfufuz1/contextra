use super::super::fixtures::*;
use crate::{
    SlmProfile, RoutingOutcome,
};
use contextra_ports::StorageEngine;
use contextra_types::{ContextraError, EntityId, TokenBudget};
use std::sync::Arc;
use contextra_db::{Contextra, ContextraConfig};
use serde_json::json;

    #[test]
    fn test_nan_single_chunk_ignored_in_max_score() -> Result<(), Box<dyn std::error::Error>> {
        use crate::router::{compute_max_score, select_profile_from_chunks};
        use contextra_types::{ContextChunk, DocId};

        let profile = SlmProfile::new(
            "test-slm",
            "http://localhost/mcp",
            vec![100],
            TokenBudget::new(1000, 100),
            0.1,
        );

        let chunk_valid_1 = ContextChunk {
            doc_id: DocId::new(1),
            content: "valid 1".to_string(),
            relevance: 0.5,
            token_count: 5,
            metadata: None,
            contextual_prefix: None,
            links: Vec::new(),
        };

        let chunk_nan = ContextChunk {
            doc_id: DocId::new(2),
            content: "corrupted nan".to_string(),
            relevance: f32::NAN,
            token_count: 5,
            metadata: None,
            contextual_prefix: None,
            links: Vec::new(),
        };

        let chunk_valid_2 = ContextChunk {
            doc_id: DocId::new(3),
            content: "valid 2".to_string(),
            relevance: 0.8,
            token_count: 5,
            metadata: None,
            contextual_prefix: None,
            links: Vec::new(),
        };

        let chunks = vec![
            (chunk_valid_1, Some(100)),
            (chunk_nan, Some(100)),
            (chunk_valid_2, Some(100)),
        ];

        let max_score = compute_max_score(&profile, &chunks);
        assert!(!max_score.is_nan(), "max_score must not be NaN");

        // Expected max score = 0.8 * 1.2 (community boost for community 100) = 0.96
        let expected = 0.8f32 * 1.2f32;
        assert!(
            (max_score - expected).abs() < 1e-5,
            "Expected max score {}, got {}",
            expected,
            max_score
        );

        let selected_idx = select_profile_from_chunks(&[profile], &chunks)?;
        assert_eq!(selected_idx, 0);

        Ok(())
    }

    #[test]
    fn test_nan_all_chunks_fallback_and_tracing_error() -> Result<(), Box<dyn std::error::Error>> {
        use crate::router::select_profile_from_chunks;
        use contextra_types::{ContextChunk, DocId};
        use tracing_subscriber::layer::SubscriberExt;

        let logs = Arc::new(std::sync::Mutex::new(Vec::new()));
        let capture_layer = LogCaptureLayer(logs.clone());
        let subscriber = tracing_subscriber::registry().with(capture_layer);
        let _guard = tracing::subscriber::set_default(subscriber);

        let profile = SlmProfile::new(
            "test-slm",
            "http://localhost/mcp",
            vec![100],
            TokenBudget::new(1000, 100),
            0.1,
        );

        let chunk_nan_1 = ContextChunk {
            doc_id: DocId::new(1),
            content: "nan 1".to_string(),
            relevance: f32::NAN,
            token_count: 5,
            metadata: None,
            contextual_prefix: None,
            links: Vec::new(),
        };

        let chunk_nan_2 = ContextChunk {
            doc_id: DocId::new(2),
            content: "nan 2".to_string(),
            relevance: f32::NAN,
            token_count: 5,
            metadata: None,
            contextual_prefix: None,
            links: Vec::new(),
        };

        let chunks = vec![(chunk_nan_1, Some(100)), (chunk_nan_2, Some(100))];

        let result = select_profile_from_chunks(&[profile], &chunks);
        assert!(result.is_err(), "Expected error when all chunks are NaN");

        match result {
            Err(ContextraError::NotFound(msg)) => {
                assert!(msg.contains("NaN/Inf"));
            }
            other => panic!("Expected NotFound error, got {:?}", other),
        }

        let captured = logs.lock().map_err(|e| e.to_string())?;
        let found_log = captured.iter().any(|msg| {
            msg.contains("Alle Chunk-Relevanzwerte sind NaN/Inf — mögliche Upstream-Korruption in der Distanzberechnung")
        });

        assert!(
            found_log,
            "Expected tracing::error! message in logs, got: {:?}",
            *captured
        );

        Ok(())
    }

    #[test]
    fn test_nan_routing_determinism_repeats() -> Result<(), Box<dyn std::error::Error>> {
        use crate::router::select_profile_from_chunks;
        use contextra_types::{ContextChunk, DocId};

        let profile_a = SlmProfile::new(
            "slm-a",
            "http://localhost/a",
            vec![100],
            TokenBudget::new(1000, 100),
            0.1,
        );

        let profile_b = SlmProfile::new(
            "slm-b",
            "http://localhost/b",
            vec![100],
            TokenBudget::new(1000, 100),
            0.1,
        );

        let profiles = vec![profile_a, profile_b];

        let chunks = vec![
            (
                ContextChunk {
                    doc_id: DocId::new(1),
                    content: "corrupted nan".to_string(),
                    relevance: f32::NAN,
                    token_count: 5,
                    metadata: None,
                    contextual_prefix: None,
                    links: Vec::new(),
                },
                Some(100),
            ),
            (
                ContextChunk {
                    doc_id: DocId::new(2),
                    content: "valid chunk".to_string(),
                    relevance: 0.7,
                    token_count: 5,
                    metadata: None,
                    contextual_prefix: None,
                    links: Vec::new(),
                },
                Some(100),
            ),
        ];

        let first_result = select_profile_from_chunks(&profiles, &chunks)?;

        for i in 0..100 {
            let res = select_profile_from_chunks(&profiles, &chunks)?;
            assert_eq!(
                res, first_result,
                "Routing selection must be bit-identical across runs (iteration {})",
                i
            );
        }

        Ok(())
    }

    #[test]
    fn test_slm_profile_validation() {
        // Valid profile
        let valid = SlmProfile::try_new(
            "coding",
            "http://localhost:8000/mcp",
            vec![1],
            TokenBudget::new(1000, 100),
            0.1,
        );
        assert!(valid.is_ok());

        // Empty name
        let empty_name = SlmProfile::try_new(
            "   ",
            "http://localhost:8000/mcp",
            vec![1],
            TokenBudget::new(1000, 100),
            0.1,
        );
        assert!(
            matches!(empty_name, Err(ContextraError::InvalidInput(msg)) if msg.contains("name cannot be empty"))
        );

        // Empty endpoint
        let empty_ep =
            SlmProfile::try_new("coding", "   ", vec![1], TokenBudget::new(1000, 100), 0.1);
        assert!(
            matches!(empty_ep, Err(ContextraError::InvalidInput(msg)) if msg.contains("endpoint cannot be empty"))
        );

        // NaN relevance score
        let nan_score = SlmProfile::try_new(
            "coding",
            "http://localhost:8000/mcp",
            vec![1],
            TokenBudget::new(1000, 100),
            f32::NAN,
        );
        assert!(
            matches!(nan_score, Err(ContextraError::InvalidInput(msg)) if msg.contains("must be finite and non-negative"))
        );
    }

    #[test]
    fn prop_slm_profile_equality() {
        use proptest::prelude::*;

        proptest!(|(
            name in "[a-z0-9_-]{1,20}",
            endpoint in "http://[a-z0-9_-]{1,20}",
            community in 0u64..10000,
            score in 0.0f32..1.0f32,
        )| {
            let p1 = SlmProfile::new(&name, &endpoint, vec![community], TokenBudget::new(1000, 100), score);
            let p2 = SlmProfile::new(&name, &endpoint, vec![community], TokenBudget::new(1000, 100), score);
            prop_assert_eq!(p1, p2);
        });
    }

    #[test]
    fn prop_slm_profile_serde() {
        use proptest::prelude::*;

        proptest!(|(
            name in "[a-z0-9_-]{1,20}",
            endpoint in "http://[a-z0-9_-]{1,20}",
            community in 0u64..10000,
            score in 0.0f32..1.0f32,
        )| {
            let p1 = SlmProfile::new(&name, &endpoint, vec![community], TokenBudget::new(1000, 100), score);
            let serialized = serde_json::to_string(&p1).unwrap(); // unwrap
            let p2: SlmProfile = serde_json::from_str(&serialized).unwrap(); // unwrap
            prop_assert_eq!(p1, p2);
        });
    }

    #[test]
    fn prop_routing_decision_profile_in_input() {
        use crate::router::select_profile_from_chunks;
        use contextra_types::{ContextChunk, DocId};
        use proptest::prelude::*;

        proptest!(|(
            score_a in 0.01f32..1.0f32,
            _score_b in 0.01f32..1.0f32,
        )| {
            let p0 = SlmProfile::new("p0", "http://ep0", vec![1], TokenBudget::new(1000, 100), 0.0);
            let p1 = SlmProfile::new("p1", "http://ep1", vec![1], TokenBudget::new(1000, 100), 0.0);
            let profiles = vec![p0, p1];

            let chunks = vec![(
                ContextChunk {
                    doc_id: DocId::new(1),
                    content: "test content".to_string(),
                    relevance: score_a,
                    token_count: 5,
                    metadata: None,
                    contextual_prefix: None,
                    links: Vec::new(),
                },
                Some(1),
            )];

            if let Ok(idx) = select_profile_from_chunks(&profiles, &chunks) {
                prop_assert!(idx < profiles.len());
            }
        });
    }

    #[test]
    fn test_slm_profile_validation_extended() {
        let inf_score = SlmProfile::try_new(
            "coding",
            "http://localhost:8000/mcp",
            vec![1],
            TokenBudget::new(1000, 100),
            f32::INFINITY,
        );
        assert!(
            matches!(inf_score, Err(ContextraError::InvalidInput(msg)) if msg.contains("must be finite and non-negative"))
        );

        let neg_inf_score = SlmProfile::try_new(
            "coding",
            "http://localhost:8000/mcp",
            vec![1],
            TokenBudget::new(1000, 100),
            f32::NEG_INFINITY,
        );
        assert!(
            matches!(neg_inf_score, Err(ContextraError::InvalidInput(msg)) if msg.contains("must be finite and non-negative"))
        );

        let neg_score = SlmProfile::try_new(
            "coding",
            "http://localhost:8000/mcp",
            vec![1],
            TokenBudget::new(1000, 100),
            -0.1,
        );
        assert!(
            matches!(neg_score, Err(ContextraError::InvalidInput(msg)) if msg.contains("must be finite and non-negative"))
        );
    }

    #[test]
    fn test_cascade_hit() {
        use crate::profile::ProfileCalibrationState;
        use contextra_types::{ContextChunk, DocId};
        use std::collections::HashMap;

        let profile_high = SlmProfile::new(
            "high-slm",
            "http://localhost/high",
            vec![1],
            TokenBudget::new(1000, 100),
            0.8,
        );
        let profile_mid = SlmProfile::new(
            "mid-slm",
            "http://localhost/mid",
            vec![1],
            TokenBudget::new(1000, 100),
            0.5,
        );
        let profile_low = SlmProfile::new(
            "low-slm",
            "http://localhost/low",
            vec![1],
            TokenBudget::new(1000, 100),
            0.2,
        );

        let profiles = vec![
            profile_mid.clone(),
            profile_high.clone(),
            profile_low.clone(),
        ];

        let dir = tempfile::tempdir().unwrap();
        let config = ContextraConfig {
            dimension: 4,
            ..Default::default()
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let db = rt
            .block_on(Contextra::open_with_config(dir.path(), config))
            .unwrap();
        let collection = rt.block_on(db.collection("default")).unwrap();

        let router = create_test_router(collection, profiles.clone(), None);
        let calibration: HashMap<String, ProfileCalibrationState> = HashMap::new();

        // Chunk score: 0.5 (with community 1 match: 0.5 * 1.2 = 0.6)
        // 0.6 >= mid threshold (0.5), but < high threshold (0.8)
        let chunk = ContextChunk {
            doc_id: DocId::new(1),
            content: "cascade hit text".to_string(),
            relevance: 0.5,
            token_count: 5,
            metadata: None,
            contextual_prefix: None,
            links: Vec::new(),
        };
        let chunks = vec![(chunk, Some(1))];

        let mut calibration = calibration;
        let (idx, selected, metrics) = router
            .select_profile_cascade(&chunks, &profiles, &mut calibration)
            .expect("Cascade selection succeeds");

        assert_eq!(selected.name, "mid-slm");
        assert_eq!(idx, 0); // profile_mid was at original index 0
        assert!(!metrics.calibrated);
    }

    #[test]
    fn test_cascade_fallthrough() {
        use crate::profile::ProfileCalibrationState;
        use contextra_types::{ContextChunk, DocId};
        use std::collections::HashMap;

        let profile_high = SlmProfile::new(
            "high-slm",
            "http://localhost/high",
            vec![1],
            TokenBudget::new(1000, 100),
            0.9,
        );
        let profile_mid = SlmProfile::new(
            "mid-slm",
            "http://localhost/mid",
            vec![1],
            TokenBudget::new(1000, 100),
            0.7,
        );
        let profile_low = SlmProfile::new(
            "low-slm",
            "http://localhost/low",
            vec![1],
            TokenBudget::new(1000, 100),
            0.5,
        );

        let profiles = vec![profile_high, profile_mid, profile_low.clone()];

        let dir = tempfile::tempdir().unwrap();
        let config = ContextraConfig {
            dimension: 4,
            ..Default::default()
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let db = rt
            .block_on(Contextra::open_with_config(dir.path(), config))
            .unwrap();
        let collection = rt.block_on(db.collection("default")).unwrap();

        let router = create_test_router(collection, profiles.clone(), None);
        let calibration: HashMap<String, ProfileCalibrationState> = HashMap::new();

        // Chunk score = 0.1 (0.1 * 1.2 = 0.12) < low threshold (0.5) -> falls through to last profile
        let chunk = ContextChunk {
            doc_id: DocId::new(1),
            content: "low score content".to_string(),
            relevance: 0.1,
            token_count: 5,
            metadata: None,
            contextual_prefix: None,
            links: Vec::new(),
        };
        let chunks = vec![(chunk, Some(1))];

        let mut calibration = calibration;
        let (idx, selected, metrics) = router
            .select_profile_cascade(&chunks, &profiles, &mut calibration)
            .expect("Cascade fallthrough succeeds");

        assert_eq!(selected.name, "low-slm");
        assert_eq!(idx, 2);
        assert!(!metrics.calibrated);
    }

    #[tokio::test]
    async fn test_calibrated_threshold_convergence() {
        use contextra_types::ConfigFingerprint;

        let dir = tempfile::tempdir().unwrap();
        let config = ContextraConfig {
            dimension: 4,
            ..Default::default()
        };
        let db = Contextra::open_with_config(dir.path(), config).await.unwrap();
        let collection = db.collection("default").await.unwrap();

        let vec_data = vec![1.0, 0.0, 0.0, 0.0];
        let key = "convergence_entity";
        collection
            .insert(
                key,
                &vec_data,
                Some(json!({"text": "convergence test content"})),
            )
            .await
            .unwrap();

        let eid = EntityId::from_doc_id(contextra_types::DocId::new(1));
        let tx = db.allocate_tx().unwrap();
        let comm_key = format!("__graph:community:{}", eid.inner()).into_bytes();
        db.inner_storage()
            .put(tx, &comm_key, &serde_json::to_vec(&100u64).unwrap())
            .await
            .unwrap();
        db.inner_storage().commit(tx).await.unwrap();

        let fp = ConfigFingerprint::new("llama-3b", "F16", "template", 0.1);
        let profile = SlmProfile::new(
            "conv-slm",
            "http://localhost:9999/mcp",
            vec![],
            TokenBudget::new(1000, 100),
            0.001,
        )
        .with_fingerprint(fp.clone());

        let router = create_test_router(collection, vec![profile], None);

        // Perform 105 routing calls and record outcomes
        let mut last_calibrated = false;
        for i in 0..105 {
            let decision = router
                .route(&vec_data, "convergence test content")
                .await
                .unwrap();
            router.record_outcome(decision.decision_id, RoutingOutcome::Success);
            let cal_stats = router.calibration_stats();
            let st = &cal_stats["conv-slm"];
            last_calibrated = st.is_calibrated(Some(&fp));
            println!(
                "Call {}: window_total={}, quantile_threshold={}, calibrated={}",
                i + 1,
                st.conformal.window_total,
                st.conformal.quantile_threshold,
                last_calibrated
            );
        }

        assert!(
            last_calibrated,
            "After 105 decisions with record_outcome (>= 100 samples) and unchanged fingerprint, decision must be calibrated (calibrated = true)"
        );
    }

    #[tokio::test]
    async fn test_calibration_invalidation_on_temperature_change() {
        use contextra_types::ConfigFingerprint;

        let dir = tempfile::tempdir().unwrap();
        let config = ContextraConfig {
            dimension: 4,
            ..Default::default()
        };
        let db = Contextra::open_with_config(dir.path(), config).await.unwrap();
        let collection = db.collection("default").await.unwrap();

        let vec_data = vec![1.0, 0.0, 0.0, 0.0];
        collection
            .insert(
                "doc_temp",
                &vec_data,
                Some(json!({"text": "temperature shift content"})),
            )
            .await
            .unwrap();

        let fp1 = ConfigFingerprint::new("llama-3b", "F16", "template", 0.1);
        let profile1 = SlmProfile::new(
            "temp-slm",
            "http://localhost:9999/mcp",
            vec![],
            TokenBudget::new(1000, 100),
            0.001,
        )
        .with_fingerprint(fp1.clone());

        let router = create_test_router(collection, vec![profile1], None);

        // Warm up with 105 successful outcomes under fp1
        for _ in 0..105 {
            let decision = router
                .route(&vec_data, "temperature shift content")
                .await
                .unwrap();
            router.record_outcome(decision.decision_id, RoutingOutcome::Success);
        }

        let cal_stats = router.calibration_stats();
        assert!(
            cal_stats["temp-slm"].is_calibrated(Some(&fp1)),
            "Profile should be calibrated under initial fp1"
        );

        // Update profile with new temperature (0.7 -> different temperature bits)
        let fp2 = ConfigFingerprint::new("llama-3b", "F16", "template", 0.7);
        assert_ne!(
            fp1.temperature_bits, fp2.temperature_bits,
            "Bits must differ for 0.1 vs 0.7"
        );

        let profile2 = SlmProfile::new(
            "temp-slm",
            "http://localhost:9999/mcp",
            vec![],
            TokenBudget::new(1000, 100),
            0.001,
        )
        .with_fingerprint(fp2.clone());

        router.update_profiles(vec![profile2]);

        // Next route call must observe calibrated = false due to configuration shift
        let decision_after_shift = router
            .route(&vec_data, "temperature shift content")
            .await
            .unwrap();
        assert!(
            !decision_after_shift.confidence.as_ref().unwrap().calibrated,
            "Decision after temperature shift must be uncalibrated"
        );

        let stats_after_shift = router.calibration_stats();
        assert!(
            !stats_after_shift["temp-slm"].is_calibrated(Some(&fp2)),
            "Calibration state must report is_calibrated = false immediately after shift"
        );

        // Warm up again with 105 decisions under fp2
        for _ in 0..105 {
            let d = router
                .route(&vec_data, "temperature shift content")
                .await
                .unwrap();
            router.record_outcome(d.decision_id, RoutingOutcome::Success);
        }

        let stats_recalibrated = router.calibration_stats();
        assert!(
            stats_recalibrated["temp-slm"].is_calibrated(Some(&fp2)),
            "Profile should become calibrated again under fp2 after re-warmup window"
        );
    }

    #[tokio::test]
    async fn test_calibration_invalidation_on_prompt_template_hash_change() {
        use contextra_types::ConfigFingerprint;

        let dir = tempfile::tempdir().unwrap();
        let config = ContextraConfig {
            dimension: 4,
            ..Default::default()
        };
        let db = Contextra::open_with_config(dir.path(), config).await.unwrap();
        let collection = db.collection("default").await.unwrap();

        let vec_data = vec![1.0, 0.0, 0.0, 0.0];
        collection
            .insert(
                "doc_prompt",
                &vec_data,
                Some(json!({"text": "prompt shift content"})),
            )
            .await
            .unwrap();

        let fp_hash1 = ConfigFingerprint::new("llama-3b", "Q8_0", "Template A", 0.2);
        let profile1 = SlmProfile::new(
            "prompt-slm",
            "http://localhost:9999/mcp",
            vec![],
            TokenBudget::new(1000, 100),
            0.001,
        )
        .with_fingerprint(fp_hash1.clone());

        let router = create_test_router(collection, vec![profile1], None);

        for _ in 0..105 {
            let decision = router
                .route(&vec_data, "prompt shift content")
                .await
                .unwrap();
            router.record_outcome(decision.decision_id, RoutingOutcome::Success);
        }

        assert!(
            router.calibration_stats()["prompt-slm"].is_calibrated(Some(&fp_hash1)),
            "Profile should be calibrated under prompt hash 1"
        );

        // Shift prompt template hash
        let fp_hash2 = ConfigFingerprint::new("llama-3b", "Q8_0", "Template B", 0.2);
        let profile2 = SlmProfile::new(
            "prompt-slm",
            "http://localhost:9999/mcp",
            vec![],
            TokenBudget::new(1000, 100),
            0.001,
        )
        .with_fingerprint(fp_hash2.clone());

        router.update_profiles(vec![profile2]);

        let decision_after_shift = router
            .route(&vec_data, "prompt shift content")
            .await
            .unwrap();
        assert!(
            !decision_after_shift.confidence.as_ref().unwrap().calibrated,
            "Decision after prompt template hash shift must be uncalibrated"
        );
        assert!(
            !router.calibration_stats()["prompt-slm"].is_calibrated(Some(&fp_hash2)),
            "State must report is_calibrated = false"
        );
    }

    #[tokio::test]
    async fn test_calibration_invalidation_on_quantization_change() {
        use contextra_types::ConfigFingerprint;

        let dir = tempfile::tempdir().unwrap();
        let config = ContextraConfig {
            dimension: 4,
            ..Default::default()
        };
        let db = Contextra::open_with_config(dir.path(), config).await.unwrap();
        let collection = db.collection("default").await.unwrap();

        let vec_data = vec![1.0, 0.0, 0.0, 0.0];
        collection
            .insert(
                "doc_quant",
                &vec_data,
                Some(json!({"text": "quantization shift content"})),
            )
            .await
            .unwrap();

        let fp_q8 = ConfigFingerprint::new("llama-3b", "Q8_0", "template", 0.0);
        let profile1 = SlmProfile::new(
            "quant-slm",
            "http://localhost:9999/mcp",
            vec![],
            TokenBudget::new(1000, 100),
            0.001,
        )
        .with_fingerprint(fp_q8.clone());

        let router = create_test_router(collection, vec![profile1], None);

        for _ in 0..105 {
            let decision = router
                .route(&vec_data, "quantization shift content")
                .await
                .unwrap();
            router.record_outcome(decision.decision_id, RoutingOutcome::Success);
        }

        assert!(
            router.calibration_stats()["quant-slm"].is_calibrated(Some(&fp_q8)),
            "Profile should be calibrated under Q8_0 quantization"
        );

        // Shift quantization level from Q8_0 to Q4_K_M
        let fp_q4 = ConfigFingerprint::new("llama-3b", "Q4_K_M", "template", 0.0);
        let profile2 = SlmProfile::new(
            "quant-slm",
            "http://localhost:9999/mcp",
            vec![],
            TokenBudget::new(1000, 100),
            0.001,
        )
        .with_fingerprint(fp_q4.clone());

        router.update_profiles(vec![profile2]);

        let decision_after_shift = router
            .route(&vec_data, "quantization shift content")
            .await
            .unwrap();
        assert!(
            !decision_after_shift.confidence.as_ref().unwrap().calibrated,
            "Decision after quantization level shift must be uncalibrated"
        );
        assert!(
            !router.calibration_stats()["quant-slm"].is_calibrated(Some(&fp_q4)),
            "State must report is_calibrated = false"
        );
    }

    #[tokio::test]
    async fn test_calibration_failsafe_on_unknown_quantization() {
        let dir = tempfile::tempdir().unwrap();
        let config = ContextraConfig {
            dimension: 4,
            ..Default::default()
        };
        let db = Contextra::open_with_config(dir.path(), config).await.unwrap();
        let collection = db.collection("default").await.unwrap();

        let vec_data = vec![1.0, 0.0, 0.0, 0.0];
        collection
            .insert(
                "doc_unknown",
                &vec_data,
                Some(json!({"text": "unknown quantization content"})),
            )
            .await
            .unwrap();

        // Profile without fingerprint set (fingerprint: None)
        let profile = SlmProfile::new(
            "unknown-slm",
            "http://localhost:9999/mcp",
            vec![],
            TokenBudget::new(1000, 100),
            0.001,
        );

        let router = create_test_router(collection, vec![profile], None);

        // Perform 50 decisions with record_outcome without fingerprint set
        for _ in 0..50 {
            let decision = router
                .route(&vec_data, "unknown quantization content")
                .await
                .unwrap();
            router.record_outcome(decision.decision_id, RoutingOutcome::Success);
        }

        let stats = router.calibration_stats();
        assert!(
            !stats["unknown-slm"].is_calibrated(None),
            "Unfingerprinted profile must fail-safe and never yield is_calibrated = true"
        );
    }

    #[tokio::test]
    async fn test_cascade_determinism() {
        let dir = tempfile::tempdir().unwrap();
        let config = ContextraConfig {
            dimension: 4,
            ..Default::default()
        };
        let db = Contextra::open_with_config(dir.path(), config).await.unwrap();
        let collection = db.collection("default").await.unwrap();

        let vec_data = vec![1.0, 0.0, 0.0, 0.0];
        let key = "det_entity";
        collection
            .insert(
                key,
                &vec_data,
                Some(json!({"text": "deterministic content"})),
            )
            .await
            .unwrap();

        let eid = EntityId::from_doc_id(contextra_types::DocId::new(1));
        let tx = db.allocate_tx().unwrap();
        let comm_key = format!("__graph:community:{}", eid.inner()).into_bytes();
        db.inner_storage()
            .put(tx, &comm_key, &serde_json::to_vec(&100u64).unwrap())
            .await
            .unwrap();
        db.inner_storage().commit(tx).await.unwrap();

        let p1 = SlmProfile::new(
            "slm-1",
            "http://localhost/1",
            vec![],
            TokenBudget::new(1000, 100),
            0.0,
        );
        let p2 = SlmProfile::new(
            "slm-2",
            "http://localhost/2",
            vec![],
            TokenBudget::new(1000, 100),
            0.0,
        );

        let router = create_test_router(collection, vec![p1, p2], None);

        let first_decision = router
            .route(&vec_data, "deterministic content")
            .await
            .unwrap();

        for i in 0..50 {
            let next_decision = router
                .route(&vec_data, "deterministic content")
                .await
                .unwrap();
            assert_eq!(
                next_decision.profile.name, first_decision.profile.name,
                "Inconsistent profile selected at iteration {}",
                i
            );
        }
    }

    #[test]
    fn test_conformal_calibrator_default_and_reset_window() {
        use crate::profile::ConformalCalibrator;
        let mut cal = ConformalCalibrator::default();
        assert_eq!(cal.alpha, 0.05);
        assert_eq!(cal.gamma, 0.01);
        assert_eq!(cal.quantile_threshold, 0.5);
        assert_eq!(cal.empirical_error_rate(), 0.0);

        cal.update(0.8);
        assert_eq!(cal.window_total, 1);
        assert_eq!(cal.window_errors, 1);

        cal.reset_window();
        assert_eq!(cal.window_total, 0);
        assert_eq!(cal.window_errors, 0);
        assert_eq!(cal.empirical_error_rate(), 0.0);
    }

    #[test]
    fn test_profile_calibration_state_default_and_average_confidence() {
        use crate::profile::ProfileCalibrationState;
        let default_st = ProfileCalibrationState::default();
        assert_eq!(default_st.times_selected, 0);
        assert_eq!(default_st.average_confidence(), 1.0);

        let mut st = ProfileCalibrationState::new(0.5);
        st.times_selected = 2;
        st.cumulative_confidence = 3.0;
        assert_eq!(st.average_confidence(), 1.5);
    }

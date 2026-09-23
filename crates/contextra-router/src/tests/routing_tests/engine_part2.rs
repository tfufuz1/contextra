use super::super::fixtures::*;
use crate::{
    SlmProfile, RoutingOutcome,
};
use contextra_core::{EntityId, ContextraError, StorageEngine, TokenBudget};
use contextra_db::{Contextra, ContextraConfig};
use serde_json::json;
use std::sync::Arc;

    #[tokio::test]
    async fn test_router_engine_try_new_and_try_update_profiles_validation_error() {
        let dir = tempfile::tempdir().unwrap(); // unwrap
        let config = ContextraConfig {
            dimension: 4,
            ..Default::default()
        };
        let db = Contextra::open_with_config(dir.path(), config).await.unwrap(); // unwrap
        let collection = db.collection("default").await.unwrap(); // unwrap

        let invalid_profile = SlmProfile::new(
            "",
            "http://localhost:8000/mcp",
            vec![1],
            TokenBudget::new(1000, 100),
            0.1,
        );

        let valid_profile = SlmProfile::new(
            "valid",
            "http://localhost:8000/mcp",
            vec![1],
            TokenBudget::new(1000, 100),
            0.1,
        );

        let res_try_new = try_create_test_router(
            collection.clone(),
            vec![valid_profile.clone(), invalid_profile.clone()],
            None,
        );
        assert!(matches!(res_try_new, Err(ContextraError::InvalidInput(_))));

        let router = create_test_router(collection, vec![valid_profile.clone()], None);
        let res_try_update = router.try_update_profiles(vec![valid_profile, invalid_profile]);
        assert!(matches!(res_try_update, Err(ContextraError::InvalidInput(_))));
    }

    #[test]
    fn test_select_profile_from_chunks_empty_chunks_and_unmatched_community() {
        use crate::router::select_profile_from_chunks;
        use contextra_core::{ContextChunk, DocId};

        let profile = SlmProfile::new(
            "slm-test",
            "http://localhost/mcp",
            vec![100],
            TokenBudget::new(1000, 100),
            0.5,
        );

        let err_empty = select_profile_from_chunks(std::slice::from_ref(&profile), &[]);
        assert!(
            matches!(err_empty, Err(ContextraError::NotFound(msg)) if msg.contains("Keine gültigen Chunks"))
        );

        let chunk_unmatched = (
            ContextChunk {
                doc_id: DocId::new(1),
                content: "unmatched community chunk".to_string(),
                relevance: 0.9,
                token_count: 5,
                metadata: None,
                contextual_prefix: None,
                links: Vec::new(),
            },
            Some(999),
        );

        let err_unmatched =
            select_profile_from_chunks(std::slice::from_ref(&profile), &[chunk_unmatched]);
        assert!(
            matches!(err_unmatched, Err(ContextraError::NotFound(msg)) if msg.contains("Kein SLM-Profil"))
        );

        let chunk_low_score = (
            ContextChunk {
                doc_id: DocId::new(2),
                content: "matched community low score".to_string(),
                relevance: 0.01,
                token_count: 5,
                metadata: None,
                contextual_prefix: None,
                links: Vec::new(),
            },
            Some(100),
        );

        let err_low =
            select_profile_from_chunks(std::slice::from_ref(&profile), &[chunk_low_score]);
        assert!(
            matches!(err_low, Err(ContextraError::NotFound(msg)) if msg.contains("Kein SLM-Profil"))
        );
    }

    #[tokio::test]
    async fn test_route_with_missing_community_or_corrupt_result() {
        let dir = tempfile::tempdir().unwrap(); // unwrap
        let config = ContextraConfig {
            dimension: 4,
            ..Default::default()
        };
        let db = Contextra::open_with_config(dir.path(), config).await.unwrap(); // unwrap
        let collection = db.collection("default").await.unwrap(); // unwrap

        let key = "entity_no_community";
        collection
            .insert(
                key,
                &[1.0, 0.0, 0.0, 0.0],
                Some(json!({"text": "sample text"})),
            )
            .await
            .unwrap(); // unwrap

        let profile = SlmProfile::new(
            "slm-no-comm",
            "http://localhost:9999/mcp",
            vec![100],
            TokenBudget::new(1000, 100),
            0.0,
        );

        let router = create_test_router(collection, vec![profile], None);
        let res = router.route(&[1.0, 0.0, 0.0, 0.0], "sample text").await;
        assert!(matches!(res, Err(ContextraError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_route_invalid_search_result_skips_chunk() -> Result<(), Box<dyn std::error::Error>>
    {
        let dir = tempfile::tempdir()?;
        let config = ContextraConfig {
            dimension: 4,
            ..Default::default()
        };
        let db = Contextra::open_with_config(dir.path(), config).await?;
        let collection = db.collection("default").await?;

        // Insert valid doc and corrupt/invalid doc directly into storage/index or test search result handling
        let valid_key = "valid_entity_1";
        collection
            .insert(
                valid_key,
                &[1.0, 0.0, 0.0, 0.0],
                Some(json!({"text": "valid content"})),
            )
            .await?;

        let eid = EntityId::from_key(valid_key)?;
        let tx = db.allocate_tx()?;
        let comm_key = format!("__graph:community:{}", eid.inner()).into_bytes();
        let comm_val = serde_json::to_vec(&100u64)?;
        db.inner_storage().put(tx, &comm_key, &comm_val).await?;
        db.inner_storage().commit(tx).await?;

        let profile = SlmProfile::new(
            "slm-valid",
            "http://localhost:9999/mcp",
            vec![100],
            TokenBudget::new(1000, 100),
            0.0,
        );

        let router = create_test_router(collection, vec![profile], None);
        let res = router.route(&[1.0, 0.0, 0.0, 0.0], "valid content").await;
        assert!(res.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_parallel_route_conformal_calibration_monotonic_convergence() {
        let dir = tempfile::tempdir().unwrap();
        let config = ContextraConfig {
            dimension: 4,
            ..Default::default()
        };
        let db = Contextra::open_with_config(dir.path(), config).await.unwrap();
        let collection = db.collection("default").await.unwrap();

        let vec_data = vec![1.0, 0.0, 0.0, 0.0];
        let key = "parallel_conv_entity";
        collection
            .insert(
                key,
                &vec_data,
                Some(json!({"text": "parallel convergence content"})),
            )
            .await
            .unwrap();

        let eid = EntityId::from_doc_id(contextra_core::DocId::new(1));
        let tx = db.allocate_tx().unwrap();
        let comm_key = format!("__graph:community:{}", eid.inner()).into_bytes();
        db.inner_storage()
            .put(tx, &comm_key, &serde_json::to_vec(&100u64).unwrap())
            .await
            .unwrap();
        db.inner_storage().commit(tx).await.unwrap();

        use contextra_core::ConfigFingerprint;
        let fp = ConfigFingerprint::new("llama-3b", "F16", "template", 0.5);
        let profile = SlmProfile::new(
            "parallel-conv-slm",
            "http://localhost:9999/mcp",
            vec![],
            TokenBudget::new(1000, 100),
            0.5,
        )
        .with_fingerprint(fp);

        let router = Arc::new(create_test_router(collection, vec![profile], None));

        // Spawn 100 parallel route() tasks and record outcome
        let mut handles = Vec::new();
        for _ in 0..100 {
            let r = router.clone();
            let vec_c = vec_data.clone();
            handles.push(tokio::spawn(async move {
                let decision = r.route(&vec_c, "parallel convergence content").await?;
                r.record_outcome(decision.decision_id, RoutingOutcome::Success);
                Ok::<(), ContextraError>(())
            }));
        }

        for h in handles {
            let res = h.await.unwrap();
            assert!(res.is_ok(), "route() failed in parallel task: {:?}", res);
        }

        let stats = router.calibration_stats();
        let st = &stats["parallel-conv-slm"];

        // Verify exact selected counts and total window
        assert_eq!(st.times_selected, 100);
        assert_eq!(st.conformal.window_total, 100);

        // Verify calibrated_min_score stays strictly bounded and non-oscillating
        let lower_bound = st.original_min_score * 0.5;
        let upper_bound = st.original_min_score * 2.0;
        assert!(
            st.calibrated_min_score >= lower_bound && st.calibrated_min_score <= upper_bound,
            "calibrated_min_score {} out of bounds [{}, {}]",
            st.calibrated_min_score,
            lower_bound,
            upper_bound
        );
    }

    #[tokio::test]
    async fn test_router_engine_reset_all_calibration() {
        let dir = tempfile::tempdir().unwrap();
        let config = ContextraConfig {
            dimension: 4,
            ..Default::default()
        };
        let db = Contextra::open_with_config(dir.path(), config).await.unwrap();
        let collection = db.collection("default").await.unwrap();

        let p1 = SlmProfile::new("p1", "http://ep1", vec![1], TokenBudget::default(), 0.1);
        let p2 = SlmProfile::new("p2", "http://ep2", vec![2], TokenBudget::default(), 0.2);

        let router = create_test_router(collection, vec![p1, p2], None);
        {
            let cal = router.calibration_stats();
            assert_eq!(cal["p1"].times_selected, 0);
        }

        // Simulate selected counts
        {
            let current = router.state.load_full();
            let mut new_state = (*current).clone();
            if let Some(st1) = new_state.calibration.get_mut("p1") {
                st1.times_selected = 10;
            }
            if let Some(st2) = new_state.calibration.get_mut("p2") {
                st2.times_selected = 20;
            }
            router.state.store(Arc::new(new_state));
        }

        assert_eq!(router.calibration_stats()["p1"].times_selected, 10);
        assert_eq!(router.calibration_stats()["p2"].times_selected, 20);

        router.reset_all_calibration();
        assert_eq!(router.calibration_stats()["p1"].times_selected, 0);
        assert_eq!(router.calibration_stats()["p2"].times_selected, 0);
    }

    #[tokio::test]
    async fn test_route_non_finite_query_embedding_err() {
        let dir = tempfile::tempdir().unwrap();
        let config = ContextraConfig {
            dimension: 4,
            ..Default::default()
        };
        let db = Contextra::open_with_config(dir.path(), config).await.unwrap();
        let collection = db.collection("default").await.unwrap();

        let p = SlmProfile::new("p", "http://ep", vec![1], TokenBudget::default(), 0.1);
        let router = create_test_router(collection, vec![p], None);

        let res_nan = router.route(&[f32::NAN, 0.0, 0.0, 0.0], "query").await;
        assert!(
            matches!(res_nan, Err(ContextraError::InvalidInput(msg)) if msg.contains("non-finite"))
        );

        let res_inf = router.route(&[f32::INFINITY, 0.0, 0.0, 0.0], "query").await;
        assert!(
            matches!(res_inf, Err(ContextraError::InvalidInput(msg)) if msg.contains("non-finite"))
        );
    }

    #[tokio::test]
    async fn test_router_engine_persisted_calibration_loading() {
        let dir = tempfile::tempdir().unwrap();
        let cal_path = dir.path().join("calibration.json");

        let p1 = SlmProfile::new("p1", "http://ep1", vec![1], TokenBudget::default(), 0.5);

        let mut initial_map = std::collections::HashMap::new();
        let mut p1_state = crate::profile::ProfileCalibrationState::new(0.5);
        p1_state.times_selected = 42;
        initial_map.insert("p1".to_string(), p1_state);

        std::fs::write(&cal_path, serde_json::to_vec(&initial_map).unwrap()).unwrap();

        let config = ContextraConfig {
            dimension: 4,
            ..Default::default()
        };
        let db = Contextra::open_with_config(dir.path().join("db"), config)
            .await
            .unwrap();
        let collection = db.collection("default").await.unwrap();

        let router = create_test_router(collection, vec![p1], Some(cal_path));
        let stats = router.calibration_stats();
        assert_eq!(stats["p1"].times_selected, 42);

        let count = router.pending_decision_count();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_route_populates_drift_status_after_sufficient_data(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let config = ContextraConfig {
            dimension: 4,
            ..Default::default()
        };
        let db = Contextra::open_with_config(dir.path(), config).await?;
        let collection = db.collection("default").await?;

        let vec_data = vec![1.0, 0.0, 0.0, 0.0];
        collection
            .insert(
                "doc_lyapunov",
                &vec_data,
                Some(json!({"text": "lyapunov test content"})),
            )
            .await?;

        #[allow(unused_mut)]
        let mut profile = SlmProfile::new(
            "lyapunov-slm",
            "http://localhost:9999/mcp",
            vec![],
            TokenBudget::new(1000, 100),
            0.01,
        );

        #[cfg(feature = "bandit-routing")]
        {
            profile.bandit_state = Some(crate::bandit::BanditProfileState::cold_start(4, 0.5));
        }

        let router = create_test_router(collection, vec![profile], None);

        // Perform 25 routing decisions
        let mut last_decision = None;
        for _ in 0..25 {
            let decision = router.route(&vec_data, "lyapunov test content").await?;
            last_decision = Some(decision);
        }

        let decision = last_decision.expect("decision present");
        assert!(
            decision.drift_status.is_some(),
            "drift_status should be populated (Some) after 20+ routing decisions"
        );
        let status = decision.drift_status.unwrap();
        assert!(
            matches!(
                status,
                crate::lyapunov::LyapunovResult::Stable { .. }
                    | crate::lyapunov::LyapunovResult::DriftDetected { .. }
            ),
            "Expected Stable or DriftDetected, got {:?}",
            status
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_router_engine_try_new_and_try_update_profiles_error_paths(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let config = ContextraConfig {
            dimension: 4,
            ..Default::default()
        };
        let db = Contextra::open_with_config(dir.path(), config).await?;
        let collection = db.collection("default").await?;

        let invalid_profile =
            SlmProfile::new("", "http://localhost", vec![1], TokenBudget::default(), 0.5);

        let try_new_res =
            try_create_test_router(collection.clone(), vec![invalid_profile.clone()], None);
        assert!(try_new_res.is_err());

        let valid_profile = SlmProfile::new(
            "valid",
            "http://localhost",
            vec![1],
            TokenBudget::default(),
            0.5,
        );
        let router = create_test_router(collection, vec![valid_profile], None);

        let try_update_res = router.try_update_profiles(vec![invalid_profile]);
        assert!(try_update_res.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_router_engine_drift_status_and_baseline_helpers(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let config = ContextraConfig {
            dimension: 4,
            ..Default::default()
        };
        let db = Contextra::open_with_config(dir.path(), config).await?;
        let collection = db.collection("default").await?;

        let profile = SlmProfile::new(
            "p1",
            "http://localhost",
            vec![1],
            TokenBudget::default(),
            0.5,
        );
        let router = create_test_router(collection, vec![profile], None);

        assert!(router.drift_status("nonexistent").is_none());
        assert!(!router.set_lyapunov_baseline("nonexistent", &[0.1, 0.2]));
        assert!(router.set_lyapunov_baseline("p1", &[0.1, 0.2]));

        assert_eq!(router.pending_decision_count(), 0);

        router.reset_all_calibration();
        assert_eq!(router.calibration_stats()["p1"].times_selected, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_select_profile_cascade_error_branches_and_fallback(
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::profile::ProfileCalibrationState;
        use contextra_core::{ContextChunk, DocId};
        use std::collections::HashMap;

        let dir = tempfile::tempdir()?;
        let config = ContextraConfig {
            dimension: 4,
            ..Default::default()
        };
        let db = Contextra::open_with_config(dir.path(), config).await?;
        let collection = db.collection("default").await?;

        let profile1 = SlmProfile::new(
            "p1",
            "http://localhost/1",
            vec![1],
            TokenBudget::default(),
            0.9,
        );
        let profile2 = SlmProfile::new(
            "p2",
            "http://localhost/2",
            vec![1],
            TokenBudget::default(),
            0.8,
        );
        let profiles = vec![profile1.clone(), profile2.clone()];
        let router = create_test_router(collection, profiles.clone(), None);

        let mut calibration: HashMap<String, ProfileCalibrationState> = HashMap::new();
        calibration.insert("p1".to_string(), ProfileCalibrationState::new(0.9));
        calibration.insert("p2".to_string(), ProfileCalibrationState::new(0.8));

        // 1. Empty chunks
        let err_empty_chunks = router.select_profile_cascade(&[], &profiles, &mut calibration);
        assert!(err_empty_chunks.is_err());

        // 2. All NaN chunks
        let nan_chunk = ContextChunk {
            doc_id: DocId::new(1),
            content: "nan test".to_string(),
            relevance: f32::NAN,
            token_count: 2,
            metadata: None,
            contextual_prefix: None,
            links: vec![],
        };
        let err_nan_chunks =
            router.select_profile_cascade(&[(nan_chunk, Some(1))], &profiles, &mut calibration);
        assert!(err_nan_chunks.is_err());

        // 3. Empty profiles
        let chunk = ContextChunk {
            doc_id: DocId::new(1),
            content: "test".to_string(),
            relevance: 0.5,
            token_count: 2,
            metadata: None,
            contextual_prefix: None,
            links: vec![],
        };
        let err_empty_profiles =
            router.select_profile_cascade(&[(chunk.clone(), Some(1))], &[], &mut calibration);
        assert!(err_empty_profiles.is_err());

        // 4. No matching communities
        let err_no_comm = router.select_profile_cascade(
            &[(chunk.clone(), Some(999))],
            &profiles,
            &mut calibration,
        );
        assert!(err_no_comm.is_err());

        // 5. Cascade fallback path: relevance (0.1) is below both p1 (0.9) and p2 (0.8) thresholds
        let low_chunk = ContextChunk {
            doc_id: DocId::new(1),
            content: "low relevance".to_string(),
            relevance: 0.1,
            token_count: 2,
            metadata: None,
            contextual_prefix: None,
            links: vec![],
        };
        let (fallback_idx, fallback_p, metrics) =
            router.select_profile_cascade(&[(low_chunk, Some(1))], &profiles, &mut calibration)?;
        assert_eq!(fallback_p.name, "p2");
        assert_eq!(fallback_idx, 1);
        assert!(!metrics.calibrated);

        Ok(())
    }

    #[tokio::test]
    async fn test_router_engine_additional_coverage_paths() -> contextra_core::Result<()> {
        use contextra_core::{ContextChunk, DocId, TokenBudget};

        let dir = tempfile::tempdir()?;
        let config = contextra_db::ContextraConfig {
            dimension: 4,
            ..Default::default()
        };
        let db = contextra_db::Contextra::open_with_config(dir.path(), config).await?;
        let collection = db.collection("default").await?;

        // 1. Corrupt calibration_store_path
        let corrupt_file = dir.path().join("corrupt_calibration.json");
        std::fs::write(&corrupt_file, b"invalid json content")?;
        let profile = SlmProfile::new(
            "p1",
            "http://localhost:1111",
            vec![1],
            TokenBudget::new(1000, 100),
            0.5,
        );

        let router = create_test_router(
            collection.clone(),
            vec![profile.clone()],
            Some(corrupt_file),
        );
        assert_eq!(router.profiles().len(), 1);

        // 2. Evict stale decisions map capacity overflow test
        for _ in 0..(crate::router::MAX_PENDING_DECISIONS + 10) {
            let id = crate::DecisionId::new();
            router.pending_decisions.write().insert(
                id,
                (
                    "p1".to_string(),
                    std::time::Instant::now() - std::time::Duration::from_secs(600),
                ),
            );
        }
        assert!(router.pending_decision_count() > crate::router::MAX_PENDING_DECISIONS);
        // Call route with invalid embedding to trigger evict_stale_decisions
        let _ = router.route(&[f32::NAN], "query").await;
        assert!(router.pending_decision_count() <= crate::router::MAX_PENDING_DECISIONS);

        // 3. record_outcome without active fingerprint (or unknown profile fingerprint)
        let dec_id = crate::DecisionId::new();
        router
            .pending_decisions
            .write()
            .insert(dec_id, ("p1".to_string(), std::time::Instant::now()));
        assert!(router.record_outcome(dec_id, crate::RoutingOutcome::Success));

        // 4. Cascade selection margin with 0 quantile_threshold
        let mut cal_map = std::collections::HashMap::new();
        let mut p_zero = SlmProfile::new(
            "p_zero",
            "http://localhost:0000",
            vec![1],
            TokenBudget::new(1000, 100),
            0.0,
        );
        p_zero.min_relevance_score = 0.0;
        let mut state = crate::profile::ProfileCalibrationState::new(0.0);
        state.conformal.quantile_threshold = 0.0;
        cal_map.insert("p_zero".to_string(), state);

        let chunk = ContextChunk {
            doc_id: DocId::new(1),
            content: "zero thresh".to_string(),
            relevance: 0.1,
            token_count: 2,
            metadata: None,
            contextual_prefix: None,
            links: vec![],
        };
        let (idx, prof, metrics) =
            router.select_profile_cascade(&[(chunk, Some(1))], &[p_zero], &mut cal_map)?;
        assert_eq!(idx, 0);
        assert_eq!(prof.name, "p_zero");
        assert_eq!(metrics.selection_margin, 1.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_router_uses_cheapest_profile_during_warmup() {
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
                "doc_warmup",
                &vec_data,
                Some(json!({"text": "warmup fallback content"})),
            )
            .await
            .unwrap();

        // Create 3 profiles in arbitrary configuration order (expensive first)
        let expensive_profile = SlmProfile::new(
            "expensive-slm",
            "http://localhost:8001/mcp",
            vec![],
            TokenBudget::new(200_000, 100),
            0.8,
        )
        .with_resource_cost_estimate(100.0);

        let mid_profile = SlmProfile::new(
            "mid-slm",
            "http://localhost:8002/mcp",
            vec![],
            TokenBudget::new(32_768, 100),
            0.5,
        )
        .with_resource_cost_estimate(50.0);

        let cheapest_profile = SlmProfile::new(
            "cheapest-slm",
            "http://localhost:8003/mcp",
            vec![],
            TokenBudget::new(8_192, 100),
            0.2,
        )
        .with_resource_cost_estimate(10.0);

        // Input configuration order: expensive, cheapest, mid
        let profiles = vec![expensive_profile, cheapest_profile, mid_profile];

        let router = create_test_router(collection, profiles, None);

        // Before reaching CALIBRATION_WARMUP_WINDOW samples (calibrated == false)
        let decision = router
            .route(&vec_data, "warmup fallback content")
            .await
            .expect("routing during warmup succeeds");

        assert_eq!(
            decision.profile.name, "cheapest-slm",
            "During warmup (calibrated == false), router must deterministically select the profile with lowest cost"
        );
        assert!(!decision.confidence.as_ref().unwrap().calibrated);
    }

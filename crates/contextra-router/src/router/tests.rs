// FILE-CONTEXT
// ZWECK: Unit-Tests für RouterEngine.
// INVARIANTEN: Instanz-Unabhängigkeit von DecisionIdGeneratoren, Kalibrierungs-Resets.

use super::*;
use contextra_types::TokenBudget;

#[tokio::test]
async fn test_router_engine_instance_decision_id_independence() {
    let dir = tempfile::tempdir().unwrap();
    let config = contextra_db::ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = contextra_db::Contextra::open_with_config(dir.path(), config)
        .await
        .unwrap();
    let collection = db.collection("default").await.unwrap();

    let profile = SlmProfile::new(
        "p1",
        "http://localhost:8000/mcp",
        vec![],
        TokenBudget::new(1000, 100),
        0.1,
    );

    let router1 =
        crate::tests::tests::create_test_router(collection.clone(), vec![profile.clone()], None);
    let router2 = crate::tests::tests::create_test_router(collection, vec![profile], None);

    let id1_a = router1.decision_ids.next();
    let id2_a = router2.decision_ids.next();

    assert_eq!(id1_a.inner(), 0);
    assert_eq!(id2_a.inner(), 0);

    let id1_b = router1.decision_ids.next();
    let id2_b = router2.decision_ids.next();

    assert_eq!(id1_b.inner(), 1);
    assert_eq!(id2_b.inner(), 1);
}

#[tokio::test]
async fn test_calibration_stats_initial_state(
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let config = contextra_db::ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = contextra_db::Contextra::open_with_config(dir.path(), config).await?;
    let collection = db.collection("default").await?;

    let profile1 = SlmProfile::new(
        "p1",
        "http://localhost:1111",
        vec![1],
        TokenBudget::new(1000, 100),
        0.5,
    );
    let profile2 = SlmProfile::new(
        "p2",
        "http://localhost:2222",
        vec![2],
        TokenBudget::new(1000, 100),
        0.8,
    );

    let router =
        crate::tests::tests::create_test_router(collection, vec![profile1, profile2], None);
    let stats = router.calibration_stats();
    assert_eq!(stats.len(), 2);
    assert_eq!(stats["p1"].times_selected, 0);
    assert_eq!(stats["p1"].calibrated_min_score, 0.5);
    assert_eq!(stats["p1"].original_min_score, 0.5);
    assert_eq!(stats["p2"].times_selected, 0);
    assert_eq!(stats["p2"].calibrated_min_score, 0.8);
    assert_eq!(stats["p2"].original_min_score, 0.8);
    Ok(())
}

#[test]
fn test_profile_calibration_state_reset() {
    let mut state = ProfileCalibrationState::new(0.5);
    state.times_selected = 15;
    state.cumulative_confidence = 12.0;
    state.calibrated_min_score = 0.6;
    state.reset();
    assert_eq!(state.times_selected, 0);
    assert_eq!(state.calibrated_min_score, 0.5);
    assert_eq!(state.cumulative_confidence, 1.0);
}

#[tokio::test]
async fn test_reset_calibration_per_profile() -> std::result::Result<(), Box<dyn std::error::Error>>
{
    let dir = tempfile::tempdir()?;
    let config = contextra_db::ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = contextra_db::Contextra::open_with_config(dir.path(), config).await?;
    let collection = db.collection("default").await?;

    let profile = SlmProfile::new(
        "p1",
        "http://localhost:1111",
        vec![1],
        TokenBudget::new(1000, 100),
        0.5,
    );

    let router = crate::tests::tests::create_test_router(collection, vec![profile], None);
    {
        let current = router.state.load_full();
        let mut new_state = (*current).clone();
        if let Some(state) = new_state.calibration.get_mut("p1") {
            state.times_selected = 5;
        }
        router.state.store(Arc::new(new_state));
    }
    assert_eq!(router.calibration_stats()["p1"].times_selected, 5);

    router.reset_calibration("p1");
    assert_eq!(router.calibration_stats()["p1"].times_selected, 0);
    Ok(())
}

#[test]
#[cfg(feature = "bandit-routing")]
fn test_pending_bandit_eviction_unit() {
    let dir = tempfile::tempdir().unwrap();
    let config = contextra_db::ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let db = contextra_db::Contextra::open_with_config(dir.path(), config)
            .await
            .unwrap();
        let collection = db.collection("default").await.unwrap();

        let profile = SlmProfile::new(
            "p1",
            "http://localhost:1111",
            vec![1],
            TokenBudget::new(1000, 100),
            0.5,
        );

        let router = crate::tests::tests::create_test_router(collection, vec![profile], None);
        let old_time = Instant::now() - Duration::from_secs(400);

        {
            let mut map = router.pending_bandit.write();
            for _ in 0..10_000 {
                map.insert(
                    router.decision_ids.next(),
                    PendingBanditDecision {
                        context: vec![0.0; 4],
                        profile_name: "p1".to_string(),
                        action_idx: 0,
                        propensity: 0.5,
                        created: old_time,
                    },
                );
            }
            assert_eq!(map.len(), 10_000);
        }

        router.evict_stale_decisions();

        assert_eq!(router.pending_bandit.read().len(), 0);
    });
}

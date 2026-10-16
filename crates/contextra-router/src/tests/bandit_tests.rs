#[cfg(feature = "bandit-routing")]
use super::fixtures::*;
#[cfg(feature = "bandit-routing")]
use crate::SlmProfile;
#[cfg(feature = "bandit-routing")]
use contextra_db::{Contextra, ContextraConfig};
#[cfg(feature = "bandit-routing")]
use contextra_types::TokenBudget;
#[cfg(feature = "bandit-routing")]
use serde_json::json;
#[cfg(feature = "bandit-routing")]
use std::sync::Arc;

#[tokio::test]
#[cfg(feature = "bandit-routing")]
async fn test_route_triggers_bandit_drift_reaction() -> Result<(), Box<dyn std::error::Error>> {
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
            "doc_drift",
            &vec_data,
            Some(json!({"text": "drift reaction content"})),
        )
        .await?;

    let mut profile = SlmProfile::new(
        "drift-slm",
        "http://localhost:9999/mcp",
        vec![],
        TokenBudget::new(1000, 100),
        0.01,
    );
    profile.bandit_state = Some(crate::bandit::BanditProfileState::cold_start(4, 0.5));

    let router = create_test_router(collection, vec![profile], None);

    // Baseline setzen damit Lyapunov Watcher sofort analysieren kann
    let baseline: Vec<f32> = (0..50).map(|i| (i as f32) / 100.0 * 0.1).collect();
    router.set_lyapunov_baseline("drift-slm", &baseline);

    // Generiere synthetisch Outlier-Scores um DriftDetected auszulösen
    let current_state = router.state.load_full();
    let mut new_state = (*current_state).clone();
    if let Some(watcher) = new_state.lyapunov_watchers.get_mut("drift-slm") {
        for _ in 0..20 {
            watcher.observe_score(0.95);
        }
    }
    router.state.store(Arc::new(new_state));

    // Nächster Aufruf löst DriftDetected aus und verdrahtet on_drift_detected
    let decision = router.route(&vec_data, "drift reaction content").await?;
    assert!(matches!(
        decision.drift_status,
        Some(crate::lyapunov::LyapunovResult::DriftDetected { .. })
    ));

    // Verifiziere, dass drift_steps_remaining im Bandit-Zustand des Profils aktiviert wurde
    let active_profiles = router.profiles();
    let bstate = active_profiles[0]
        .bandit_state
        .as_ref()
        .expect("bandit state exists");
    assert!(
        bstate.drift_steps_remaining > 0,
        "drift_steps_remaining muss nach erkannter Drift > 0 sein"
    );

    Ok(())
}

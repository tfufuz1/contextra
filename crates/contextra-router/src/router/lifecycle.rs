// FILE-CONTEXT
// ZWECK: Konstruktions- und Profilverwaltungs-Methoden (Hot-Reload) für RouterEngine.
// INVARIANTEN: Atomare Snapshot-Sicherheit bei Hot-Reload via ArcSwap.

use super::*;

impl RouterEngine {
    /// Creates a new `RouterEngine` instance from decoupled ports.
    pub fn new(
        search_provider: Arc<dyn HybridSearchProvider>,
        community_resolver: Arc<dyn CommunityResolver>,
        context_preparer: Arc<dyn ContextPreparer>,
        profiles: Vec<SlmProfile>,
        calibration_store_path: Option<std::path::PathBuf>,
    ) -> Self {
        let mut calibration: HashMap<String, ProfileCalibrationState> = profiles
            .iter()
            .map(|p| {
                (
                    p.name.clone(),
                    ProfileCalibrationState::new(p.min_relevance_score),
                )
            })
            .collect();

        let lyapunov_watchers: HashMap<String, LyapunovDriftWatcher> = profiles
            .iter()
            .map(|p| (p.name.clone(), LyapunovDriftWatcher::default()))
            .collect();

        if let Some(ref path) = calibration_store_path {
            if let Ok(bytes) = std::fs::read(path) {
                if let Ok(persisted) =
                    serde_json::from_slice::<HashMap<String, ProfileCalibrationState>>(&bytes)
                {
                    // Merge persisted state into defaults (persisted wins for known profiles)
                    for (name, state) in persisted {
                        if calibration.contains_key(&name) {
                            calibration.insert(name, state);
                        }
                        // Unknown profiles (removed from config) are silently dropped
                    }
                }
            }
        }

        let router_state = RouterState {
            profiles,
            calibration,
            lyapunov_watchers,
        };

        Self {
            search_provider,
            community_resolver,
            context_preparer,
            state: ArcSwap::from(Arc::new(router_state)),
            pending_decisions: RwLock::new(HashMap::new()),
            decision_ids: DecisionIdGenerator::new(0),
            #[cfg(feature = "bandit-routing")]
            routing_strategy: RoutingStrategy::Cascade,
            #[cfg(feature = "bandit-routing")]
            bandit_exploration: BanditExploration::new(0.1, 0x1234_5678_9abc_def0),
            #[cfg(feature = "bandit-routing")]
            pending_bandit: RwLock::new(HashMap::new()),
        }
    }

    /// Konfiguriert den Startwert des instanzgebundenen DecisionIdGenerators.
    pub fn with_initial_decision_id(mut self, start: u64) -> Self {
        self.decision_ids = DecisionIdGenerator::new(start);
        self
    }

    #[cfg(feature = "bandit-routing")]
    /// Builder-Methode zur Konfiguration der Routing-Strategie und Bandit-Exploration.
    pub fn with_routing_strategy(
        mut self,
        strategy: RoutingStrategy,
        epsilon: f32,
        seed: u64,
    ) -> Self {
        self.routing_strategy = strategy;
        self.bandit_exploration = BanditExploration::new(epsilon, seed);
        self
    }

    /// Validates all profiles and creates a new `RouterEngine` instance.
    pub fn try_new(
        search_provider: Arc<dyn HybridSearchProvider>,
        community_resolver: Arc<dyn CommunityResolver>,
        context_preparer: Arc<dyn ContextPreparer>,
        profiles: Vec<SlmProfile>,
        calibration_store_path: Option<std::path::PathBuf>,
    ) -> Result<Self> {
        for p in &profiles {
            p.validate()?;
        }
        Ok(Self::new(
            search_provider,
            community_resolver,
            context_preparer,
            profiles,
            calibration_store_path,
        ))
    }

    /// Dynamically updates configured SLM profiles at runtime (Hot-Reload).
    pub fn update_profiles(&self, new_profiles: Vec<SlmProfile>) {
        let current = self.state.load_full();
        let mut old_cal = current.calibration.clone();
        let new_cal: HashMap<String, ProfileCalibrationState> = new_profiles
            .iter()
            .map(|p| {
                let mut state = old_cal
                    .remove(&p.name)
                    .unwrap_or_else(|| ProfileCalibrationState::new(p.min_relevance_score));
                state.check_and_invalidate_fingerprint(p.fingerprint.as_ref());
                (p.name.clone(), state)
            })
            .collect();

        let mut old_watchers = current.lyapunov_watchers.clone();
        let new_watchers: HashMap<String, LyapunovDriftWatcher> = new_profiles
            .iter()
            .map(|p| {
                let watcher = old_watchers.remove(&p.name).unwrap_or_default();
                (p.name.clone(), watcher)
            })
            .collect();

        let new_state = RouterState {
            profiles: new_profiles,
            calibration: new_cal,
            lyapunov_watchers: new_watchers,
        };

        self.state.store(Arc::new(new_state));
    }

    /// Validates all profiles and updates configured SLM profiles at runtime (Hot-Reload).
    pub fn try_update_profiles(&self, new_profiles: Vec<SlmProfile>) -> Result<()> {
        for p in &new_profiles {
            p.validate()?;
        }
        self.update_profiles(new_profiles);
        Ok(())
    }

    /// Returns a copy of the active SLM profiles.
    pub fn profiles(&self) -> Vec<SlmProfile> {
        self.state.load().profiles.clone()
    }
}

//! # Lebenszyklus- und Konfigurationsmethoden der RouterEngine
//!
//! **Ring:** Ring 3 (Applikations- und Koordinationsschicht).
//! **Zweck:** Initialisierung, Builder-Methoden und atomare Hot-Reload-Aktualisierung von Profilen.
//!
//! **Nebenläufigkeit bei Updates:**
//! `update_profiles` erzeugt einen neuen `RouterState`-Snapshot und tauscht diesen
//! atomar über `ArcSwap` aus. Laufende Routing-Anfragen lesen entweder den alten
//! oder den neuen Snapshot vollständig und unterbrechungsfrei (Lock-Free Read).

// FILE-CONTEXT
// STAND: 2026-10-29
// ZWECK: Konstruktions- und Profilverwaltungs-Methoden (Hot-Reload) für RouterEngine.
// INVARIANTEN: Atomare Snapshot-Sicherheit bei Hot-Reload via ArcSwap.

use super::*;

impl RouterEngine {
    /// Creates a new `RouterEngine` instance from decoupled ports.
    // ANCHOR[contextra-router:LCF-001]
    /// Erzeugt eine neue `RouterEngine` mit Standardeinstellungen.
    pub fn new(
        search_provider: Arc<dyn HybridSearchProvider>,
        community_resolver: Arc<dyn CommunityResolver>,
        context_preparer: Arc<dyn ContextPreparer>,
        profiles: Vec<SlmProfile>,
        calibration_store_path: Option<std::path::PathBuf>,
    ) -> Self {
        let mut calibration: BTreeMap<String, ProfileCalibrationState> = profiles
            .iter()
            .map(|p| {
                (
                    p.name.clone(),
                    ProfileCalibrationState::new(p.min_relevance_score),
                )
            })
            .collect();

        let lyapunov_watchers: BTreeMap<String, LyapunovDriftWatcher> = profiles
            .iter()
            .map(|p| (p.name.clone(), LyapunovDriftWatcher::default()))
            .collect();

        if let Some(ref path) = calibration_store_path {
            if let Ok(bytes) = std::fs::read(path) {
                if let Ok(persisted) =
                    serde_json::from_slice::<BTreeMap<String, ProfileCalibrationState>>(&bytes)
                {
                    for (name, state) in persisted {
                        if calibration.contains_key(&name) {
                            calibration.insert(name, state);
                        }
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
            clock: Arc::new(SystemClock::new()),
            state: ArcSwap::from(Arc::new(router_state)),
            pending_decisions: RwLock::new(BTreeMap::new()),
            decision_ids: DecisionIdGenerator::new(0),
            #[cfg(feature = "bandit-routing")]
            routing_strategy: RoutingStrategy::Cascade,
            #[cfg(feature = "bandit-routing")]
            bandit_exploration: BanditExploration::new(0.1, 0x1234_5678_9abc_def0),
            #[cfg(feature = "bandit-routing")]
            pending_bandit: RwLock::new(BTreeMap::new()),
        }
    }

    /// Configures a custom Clock implementation for deterministic testing.
    // ANCHOR[contextra-router:LCF-002]
    /// Injiziert eine benutzerdefinierte System-Uhr (Port-Entwurf für Tests).
    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    /// Configures the initial decision ID counter value.
    // ANCHOR[contextra-router:LCF-003]
    /// Setzt den Startwert für monotone Entscheidungs-IDs.
    pub fn with_initial_decision_id(mut self, start: u64) -> Self {
        self.decision_ids = DecisionIdGenerator::new(start);
        self
    }

    #[cfg(feature = "bandit-routing")]
    /// Builder-Methode zur Konfiguration der Routing-Strategie und Bandit-Exploration.
    // ANCHOR[contextra-router:LCF-004]
    /// Konfiguriert die Routing-Strategie (Cascade oder Bandit).
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
    // ANCHOR[contextra-router:LCF-005]
    /// Erzeugt eine neue `RouterEngine` und validiert alle übergebenen Profile.
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
    // ANCHOR[contextra-router:LCF-006]
    /// Aktualisiert die Profile im laufenden Betrieb atomar (Hot-Reload via ArcSwap).
    pub fn update_profiles(&self, new_profiles: Vec<SlmProfile>) {
        let current = self.state.load_full();
        let mut old_cal = current.calibration.clone();
        let new_cal: BTreeMap<String, ProfileCalibrationState> = new_profiles
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
        let new_watchers: BTreeMap<String, LyapunovDriftWatcher> = new_profiles
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
    // ANCHOR[contextra-router:LCF-007]
    /// Aktualisiert Profile atomar nach vorheriger strikter Validierung.
    pub fn try_update_profiles(&self, new_profiles: Vec<SlmProfile>) -> Result<()> {
        for p in &new_profiles {
            p.validate()?;
        }
        self.update_profiles(new_profiles);
        Ok(())
    }

    /// Returns a copy of the active SLM profiles.
    // ANCHOR[contextra-router:LCF-008]
    /// Liefert eine Momentaufnahme der aktuell aktiven Profile.
    pub fn profiles(&self) -> Vec<SlmProfile> {
        self.state.load().profiles.clone()
    }
}

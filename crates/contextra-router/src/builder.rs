//! # Adoptions-Fassade: RouterEngineBuilder (§4.3, Z4)
//!
//! **Ring:** Ring 3 (Applikations- und Koordinationsschicht).
//! **Zweck:** Ergonomischer, panikfreier Builder zur schrittweisen Konfiguration und
//! Erzeugung einer [`RouterEngine`].
//!
//! **Architektur:**
//! - Bildet alle Konfigurationsparameter von `try_new_with_vfs` ab.
//! - Validiert Pflicht-Ports und Profile vor der Konstruktion.
//! - Integriert die [`CalibrationSource`]-Naht für Datei- oder In-Memory-Kalibrierungen.

use crate::outcome::DecisionIdGenerator;
use crate::profile::{ProfileCalibrationState, SlmProfile};
use crate::router::{RouterEngine, RouterState};
use crate::seam::{CalibrationSource, NoCalibrationSource, StdFsCalibrationSource, VfsCalibrationSource};
use arc_swap::ArcSwap;
use contextra_adapt::lyapunov::LyapunovDriftWatcher;
use contextra_ports::clock::{Clock, SystemClock};
use contextra_ports::{CommunityResolver, ContextPreparer, HybridSearchProvider};
use contextra_types::{ContextraError, Result};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(feature = "bandit-routing")]
use crate::router::BanditExploration;
#[cfg(feature = "bandit-routing")]
use crate::routing_strategy::RoutingStrategy;

// ANCHOR[contextra-router:BLD-001]
/// Builder zur flexiblen und typsicheren Konstruktion einer [`RouterEngine`].
pub struct RouterEngineBuilder {
    search_provider: Option<Arc<dyn HybridSearchProvider>>,
    community_resolver: Option<Arc<dyn CommunityResolver>>,
    context_preparer: Option<Arc<dyn ContextPreparer>>,
    profiles: Vec<SlmProfile>,
    calibration_source: Box<dyn CalibrationSource>,
    calibration_store_path: Option<PathBuf>,
    clock: Option<Arc<dyn Clock>>,
    initial_decision_id: u64,
    #[cfg(feature = "bandit-routing")]
    routing_strategy: RoutingStrategy,
    #[cfg(feature = "bandit-routing")]
    bandit_exploration: Option<BanditExploration>,
}

impl Default for RouterEngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RouterEngineBuilder {
    // ANCHOR[contextra-router:BLD-002]
    /// Erzeugt einen neuen leeren `RouterEngineBuilder`.
    pub fn new() -> Self {
        Self {
            search_provider: None,
            community_resolver: None,
            context_preparer: None,
            profiles: Vec::new(),
            calibration_source: Box::new(NoCalibrationSource),
            calibration_store_path: None,
            clock: None,
            initial_decision_id: 0,
            #[cfg(feature = "bandit-routing")]
            routing_strategy: RoutingStrategy::Cascade,
            #[cfg(feature = "bandit-routing")]
            bandit_exploration: None,
        }
    }

    /// Konfiguriert den hybriden Such-Provider (Pflichtfeld).
    pub fn search_provider(mut self, provider: Arc<dyn HybridSearchProvider>) -> Self {
        self.search_provider = Some(provider);
        self
    }

    /// Konfiguriert den Community-Resolver (Pflichtfeld).
    pub fn community_resolver(mut self, resolver: Arc<dyn CommunityResolver>) -> Self {
        self.community_resolver = Some(resolver);
        self
    }

    /// Konfiguriert den Kontext-Vorbereiter (Pflichtfeld).
    pub fn context_preparer(mut self, preparer: Arc<dyn ContextPreparer>) -> Self {
        self.context_preparer = Some(preparer);
        self
    }

    /// Konfiguriert die zu berücksichtigenden SLM-Profile.
    pub fn profiles(mut self, profiles: Vec<SlmProfile>) -> Self {
        self.profiles = profiles;
        self
    }

    /// Fügt ein einzelnes Profil zur Profil-Liste hinzu.
    pub fn add_profile(mut self, profile: SlmProfile) -> Self {
        self.profiles.push(profile);
        self
    }

    /// Konfiguriert eine benutzerdefinierte Kalibrierungsquelle ([`CalibrationSource`]).
    pub fn calibration_source(mut self, source: Box<dyn CalibrationSource>) -> Self {
        self.calibration_source = source;
        self
    }

    /// Konfiguriert das Lesen der Kalibrierungsdaten über das Standard-Dateisystem (`std::fs`).
    pub fn calibration_file(mut self, path: PathBuf) -> Self {
        self.calibration_source = Box::new(StdFsCalibrationSource);
        self.calibration_store_path = Some(path);
        self
    }

    /// Konfiguriert das Lesen der Kalibrierungsdaten über einen virtuellen Dateisystem-Port ([`contextra_ports::vfs::Vfs`]).
    pub fn calibration_vfs(mut self, vfs: Arc<dyn contextra_ports::vfs::Vfs>, path: PathBuf) -> Self {
        self.calibration_source = Box::new(VfsCalibrationSource::new(vfs));
        self.calibration_store_path = Some(path);
        self
    }

    /// Setzt den Pfad zur Kalibrierungsdatei (unter Beibehaltung der konfigurierten Quelle).
    pub fn calibration_store_path(mut self, path: Option<PathBuf>) -> Self {
        self.calibration_store_path = path;
        self
    }

    /// Injiziert eine benutzerdefinierte System-Uhr ([`Clock`]).
    pub fn clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = Some(clock);
        self
    }

    /// Setzt den Startwert für monotone Entscheidungs-IDs.
    pub fn initial_decision_id(mut self, start: u64) -> Self {
        self.initial_decision_id = start;
        self
    }

    #[cfg(feature = "bandit-routing")]
    /// Konfiguriert die Routing-Strategie (Cascade oder Bandit) mit festem Epsilon und Seed.
    pub fn routing_strategy(
        mut self,
        strategy: RoutingStrategy,
        epsilon: f32,
        seed: u64,
    ) -> Self {
        self.routing_strategy = strategy;
        self.bandit_exploration = Some(BanditExploration::new(epsilon, seed));
        self
    }

    #[cfg(feature = "bandit-routing")]
    /// Konfiguriert die Bandit-Exploration mit injiziertem Rng-Port.
    pub fn bandit_rng(
        mut self,
        strategy: RoutingStrategy,
        epsilon: f32,
        rng: Box<dyn contextra_adapt::Rng + Send>,
    ) -> Self {
        self.routing_strategy = strategy;
        self.bandit_exploration = Some(BanditExploration::with_rng(epsilon, rng));
        self
    }

    // ANCHOR[contextra-router:BLD-003]
    /// Validiert alle Eingaben und erzeugt die [`RouterEngine`].
    ///
    /// # Fehler
    /// - [`ContextraError::InvalidInput`]: Wenn ein Pflicht-Port fehlt oder Profile ungültig sind.
    pub fn build(mut self) -> Result<RouterEngine> {
        let search_provider = self.search_provider.take().ok_or_else(|| {
            ContextraError::InvalidInput("RouterEngineBuilder: search_provider is required".into())
        })?;
        let community_resolver = self.community_resolver.take().ok_or_else(|| {
            ContextraError::InvalidInput("RouterEngineBuilder: community_resolver is required".into())
        })?;
        let context_preparer = self.context_preparer.take().ok_or_else(|| {
            ContextraError::InvalidInput("RouterEngineBuilder: context_preparer is required".into())
        })?;

        for p in &self.profiles {
            p.validate()?;
        }

        Ok(self.build_internal(search_provider, community_resolver, context_preparer))
    }

    /// Interner Konstruktionspfad ohne erneute Profil-Validierung.
    pub(crate) fn build_internal(
        self,
        search_provider: Arc<dyn HybridSearchProvider>,
        community_resolver: Arc<dyn CommunityResolver>,
        context_preparer: Arc<dyn ContextPreparer>,
    ) -> RouterEngine {
        let mut calibration: BTreeMap<String, ProfileCalibrationState> = self
            .profiles
            .iter()
            .map(|p| {
                (
                    p.name.clone(),
                    ProfileCalibrationState::new(p.min_relevance_score),
                )
            })
            .collect();

        let lyapunov_watchers: BTreeMap<String, LyapunovDriftWatcher> = self
            .profiles
            .iter()
            .map(|p| (p.name.clone(), LyapunovDriftWatcher::default()))
            .collect();

        if let Some(ref path) = self.calibration_store_path {
            if let Some(bytes) = self.calibration_source.read_calibration(path) {
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
            profiles: self.profiles,
            calibration,
            lyapunov_watchers,
        };

        let clock = self
            .clock
            .unwrap_or_else(|| Arc::new(SystemClock::new()));

        #[cfg(feature = "bandit-routing")]
        let bandit_exploration = self
            .bandit_exploration
            .unwrap_or_else(|| BanditExploration::new(0.1, 0x1234_5678_9abc_def0));

        RouterEngine {
            search_provider,
            community_resolver,
            context_preparer,
            clock,
            state: ArcSwap::from(Arc::new(router_state)),
            pending_decisions: RwLock::new(BTreeMap::new()),
            decision_ids: DecisionIdGenerator::new(self.initial_decision_id),
            #[cfg(feature = "bandit-routing")]
            routing_strategy: self.routing_strategy,
            #[cfg(feature = "bandit-routing")]
            bandit_exploration,
            #[cfg(feature = "bandit-routing")]
            pending_bandit: RwLock::new(BTreeMap::new()),
        }
    }
}

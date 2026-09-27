//! Plugin registry and dependency resolution (§12, INV-PLUGIN-DEPENDENCY).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use thiserror::Error;

use super::license::{FeatureRing, LicenseGate};

/// Capability descriptor for a plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PluginCapability {
    /// Unique plugin name.
    pub name: &'static str,
    /// Plugin version tuple (major, minor, patch).
    pub version: (u32, u32, u32),
    /// Architecture ring in which the plugin operates (0-4).
    pub ring: u8,
    /// Minimum feature ring required for activation.
    pub feature_ring_required: FeatureRing,
}

impl PluginCapability {
    /// Creates a new `PluginCapability`.
    pub fn new(
        name: &'static str,
        version: (u32, u32, u32),
        ring: u8,
        feature_ring_required: FeatureRing,
    ) -> Self {
        Self {
            name,
            version,
            ring,
            feature_ring_required,
        }
    }
}

/// Manifest trait implemented by dynamic or static plugins.
pub trait PluginManifest: Send + Sync {
    /// Returns the name of the plugin.
    fn name(&self) -> &str;

    /// Returns the list of plugin names required prior to activation.
    fn requires(&self) -> &[&str] {
        &[]
    }

    /// Returns the list of plugin names that conflict with this plugin.
    fn conflicts(&self) -> &[&str] {
        &[]
    }

    /// Returns the required feature ring for this plugin.
    fn feature_ring_required(&self) -> FeatureRing {
        FeatureRing::Fast
    }

    /// Returns the static capability metadata for this plugin.
    fn capability(&self) -> PluginCapability;

    /// Activates the plugin instance.
    fn activate(&self) -> Result<(), PluginError>;
}

/// Errors during plugin registration, activation, or dependency resolution.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum PluginError {
    /// Dependency cycle or unresolvable missing dependency detected among plugins.
    #[error("dependency cycle detected among plugins: {0:?}")]
    DependencyCycle(Vec<String>),

    /// Conflicting plugins requested or active.
    #[error("conflicting plugins requested: {0} conflicts with {1}")]
    Conflict(String, String),

    /// License gate denied activation due to inactive feature ring.
    #[error("license gate denied activation of '{0}': required FeatureRing not active")]
    LicenseDenied(String),
}

impl From<PluginError> for contextra_types::ContextraError {
    fn from(err: PluginError) -> Self {
        contextra_types::ContextraError::Plugin(err.to_string())
    }
}

/// Registry managing plugin activations, license gating, and dependency resolution.
pub struct PluginRegistry {
    active: BTreeMap<String, PluginCapability>,
    active_conflicts: BTreeMap<String, Vec<String>>,
    gate: Arc<dyn LicenseGate>,
}

impl PluginRegistry {
    /// Creates a new `PluginRegistry` bounded by the given `LicenseGate`.
    pub fn new(gate: Arc<dyn LicenseGate>) -> Self {
        Self {
            active: BTreeMap::new(),
            active_conflicts: BTreeMap::new(),
            gate,
        }
    }

    /// Checks whether a plugin with the given name is currently active.
    pub fn is_active(&self, name: &str) -> bool {
        self.active.contains_key(name)
    }

    /// Takes a snapshot of all active plugin capabilities.
    pub fn snapshot(&self) -> Vec<PluginCapability> {
        self.active.values().copied().collect()
    }

    /// Returns the highest currently active `FeatureRing` according to the license gate.
    pub fn current_feature_ring(&self) -> FeatureRing {
        if self.gate.check_ring(FeatureRing::Compliance).is_ok() {
            FeatureRing::Compliance
        } else if self.gate.check_ring(FeatureRing::Sovereign).is_ok() {
            FeatureRing::Sovereign
        } else {
            FeatureRing::Fast
        }
    }

    /// Activates a batch of plugins in an all-or-nothing manner (INV-PLUGIN-DEPENDENCY).
    ///
    /// Sequence:
    /// 1. Conflict checks across requested and active plugins.
    /// 2. License gate checks for required feature rings.
    /// 3. Kahn's algorithm for topological sorting over `requires` edges with deterministic tie-breaking.
    /// 4. Activation in topological order.
    pub fn activate_all(
        &mut self,
        plugins: Vec<Box<dyn PluginManifest>>,
    ) -> Result<(), PluginError> {
        // Filter out plugins that are already active (idempotency)
        let pending_plugins: Vec<Box<dyn PluginManifest>> = plugins
            .into_iter()
            .filter(|p| !self.is_active(p.name()))
            .collect();

        if pending_plugins.is_empty() {
            return Ok(());
        }

        // Map pending plugins by name
        let mut plugin_map: BTreeMap<String, Box<dyn PluginManifest>> = BTreeMap::new();
        for p in pending_plugins {
            let p_name = p.name().to_string();
            if plugin_map.contains_key(&p_name) {
                return Err(PluginError::Conflict(p_name.clone(), p_name));
            }
            plugin_map.insert(p_name, p);
        }

        // 1. Conflict checks
        for (p_name, plugin) in &plugin_map {
            // Check P's declared conflicts
            for conflict_target in plugin.conflicts() {
                let target = conflict_target.to_string();
                if self.is_active(&target) || plugin_map.contains_key(&target) {
                    return Err(PluginError::Conflict(p_name.clone(), target));
                }
            }
            // Check if any active plugin declares P as a conflict
            for (active_name, conflicts) in &self.active_conflicts {
                if conflicts.iter().any(|c| c == p_name) {
                    return Err(PluginError::Conflict(p_name.clone(), active_name.clone()));
                }
            }
        }

        // 2. License Gate checks
        for (p_name, plugin) in &plugin_map {
            let required_ring = plugin.feature_ring_required();
            if self.gate.check_ring(required_ring).is_err() {
                return Err(PluginError::LicenseDenied(p_name.clone()));
            }
        }

        // 3. Kahn's algorithm for topological sorting
        // Edges: Dependency A -> Dependent B (A must be activated before B)
        let mut in_degree: BTreeMap<String, usize> = BTreeMap::new();
        let mut adj: BTreeMap<String, Vec<String>> = BTreeMap::new();

        for name in plugin_map.keys() {
            in_degree.insert(name.clone(), 0);
            adj.insert(name.clone(), Vec::new());
        }

        for (p_name, plugin) in &plugin_map {
            for req in plugin.requires() {
                let req_str = req.to_string();
                if self.is_active(&req_str) {
                    // Dependency is already active, so in-degree for p_name is unaffected
                    continue;
                }
                if plugin_map.contains_key(&req_str) {
                    // req_str must come before p_name
                    adj.get_mut(&req_str).expect("req_str in adj").push(p_name.clone());
                    *in_degree.get_mut(p_name).expect("p_name in in_degree") += 1;
                } else {
                    // Dependency is missing (neither active nor in pending batch)
                    // Increment in-degree so p_name can never be resolved
                    *in_degree.get_mut(p_name).expect("p_name in in_degree") += 1;
                }
            }
        }

        // Deterministic queue: BTreeSet maintains alphabetical order for tie-breaking
        let mut ready: BTreeSet<String> = BTreeSet::new();
        for (name, deg) in &in_degree {
            if *deg == 0 {
                ready.insert(name.clone());
            }
        }

        let mut sorted_order: Vec<String> = Vec::new();
        while let Some(next_name) = ready.iter().cloned().next() {
            ready.remove(&next_name);
            sorted_order.push(next_name.clone());

            if let Some(neighbors) = adj.get(&next_name) {
                for neighbor in neighbors {
                    let deg = in_degree.get_mut(neighbor).expect("neighbor in in_degree");
                    *deg -= 1;
                    if *deg == 0 {
                        ready.insert(neighbor.clone());
                    }
                }
            }
        }

        if sorted_order.len() != plugin_map.len() {
            // Cycle or missing dependency detected
            let mut unresolved: Vec<String> = plugin_map
                .keys()
                .filter(|k| !sorted_order.contains(k))
                .cloned()
                .collect();
            unresolved.sort();
            return Err(PluginError::DependencyCycle(unresolved));
        }

        // 4. Activation in topological order
        for p_name in &sorted_order {
            let plugin = plugin_map.get(p_name).expect("plugin exists");
            plugin.activate()?;
            let cap = plugin.capability();
            let declared_conflicts: Vec<String> =
                plugin.conflicts().iter().map(|s| s.to_string()).collect();
            self.active.insert(p_name.clone(), cap);
            self.active_conflicts.insert(p_name.clone(), declared_conflicts);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::license::{LicenseError, LicenseGate};

    struct TestGate {
        max_ring: FeatureRing,
    }

    impl LicenseGate for TestGate {
        fn check_ring(&self, ring: FeatureRing) -> Result<(), LicenseError> {
            match (self.max_ring, ring) {
                (FeatureRing::Compliance, _) => Ok(()),
                (FeatureRing::Sovereign, FeatureRing::Compliance) => {
                    Err(LicenseError::NotActivated(ring))
                }
                (FeatureRing::Sovereign, _) => Ok(()),
                (FeatureRing::Fast, FeatureRing::Fast) => Ok(()),
                (FeatureRing::Fast, r) => Err(LicenseError::NotActivated(r)),
            }
        }
    }

    struct SimplePlugin {
        name: &'static str,
        requires: &'static [&'static str],
        conflicts: &'static [&'static str],
        ring: FeatureRing,
    }

    impl PluginManifest for SimplePlugin {
        fn name(&self) -> &str {
            self.name
        }
        fn requires(&self) -> &[&str] {
            self.requires
        }
        fn conflicts(&self) -> &[&str] {
            self.conflicts
        }
        fn feature_ring_required(&self) -> FeatureRing {
            self.ring
        }
        fn capability(&self) -> PluginCapability {
            PluginCapability {
                name: self.name,
                version: (1, 0, 0),
                ring: 3,
                feature_ring_required: self.ring,
            }
        }
        fn activate(&self) -> Result<(), PluginError> {
            Ok(())
        }
    }

    #[test]
    fn test_plugin_registry_basic_flow() {
        let gate = Arc::new(TestGate {
            max_ring: FeatureRing::Fast,
        });
        let mut registry = PluginRegistry::new(gate);

        let p1 = Box::new(SimplePlugin {
            name: "alpha",
            requires: &[],
            conflicts: &[],
            ring: FeatureRing::Fast,
        });
        let p2 = Box::new(SimplePlugin {
            name: "beta",
            requires: &["alpha"],
            conflicts: &[],
            ring: FeatureRing::Fast,
        });

        assert_eq!(registry.activate_all(vec![p2, p1]), Ok(()));
        assert!(registry.is_active("alpha"));
        assert!(registry.is_active("beta"));
        assert_eq!(registry.snapshot().len(), 2);
    }

    #[test]
    fn test_plugin_registry_conflict_detection() {
        let gate = Arc::new(TestGate {
            max_ring: FeatureRing::Fast,
        });
        let mut registry = PluginRegistry::new(gate);

        let p1 = Box::new(SimplePlugin {
            name: "alpha",
            requires: &[],
            conflicts: &["beta"],
            ring: FeatureRing::Fast,
        });
        let p2 = Box::new(SimplePlugin {
            name: "beta",
            requires: &[],
            conflicts: &[],
            ring: FeatureRing::Fast,
        });

        let err = registry.activate_all(vec![p1, p2]);
        assert_eq!(
            err,
            Err(PluginError::Conflict("alpha".into(), "beta".into()))
        );
        assert!(!registry.is_active("alpha"));
        assert!(!registry.is_active("beta"));
    }

    #[test]
    fn test_plugin_registry_conflict_against_already_active_plugin() {
        let gate = Arc::new(TestGate {
            max_ring: FeatureRing::Fast,
        });
        let mut registry = PluginRegistry::new(gate);

        // Active plugin p1 declares conflict with "beta"
        let p1 = Box::new(SimplePlugin {
            name: "alpha",
            requires: &[],
            conflicts: &["beta"],
            ring: FeatureRing::Fast,
        });
        assert_eq!(registry.activate_all(vec![p1]), Ok(()));

        // Now try to activate p2 ("beta")
        let p2 = Box::new(SimplePlugin {
            name: "beta",
            requires: &[],
            conflicts: &[],
            ring: FeatureRing::Fast,
        });

        let err = registry.activate_all(vec![p2]);
        assert_eq!(
            err,
            Err(PluginError::Conflict("beta".into(), "alpha".into()))
        );
        assert!(!registry.is_active("beta"));
    }

    #[test]
    fn test_plugin_registry_license_denied() {
        let gate = Arc::new(TestGate {
            max_ring: FeatureRing::Fast,
        });
        let mut registry = PluginRegistry::new(gate);

        let p1 = Box::new(SimplePlugin {
            name: "enterprise_audit",
            requires: &[],
            conflicts: &[],
            ring: FeatureRing::Compliance,
        });

        let err = registry.activate_all(vec![p1]);
        assert_eq!(
            err,
            Err(PluginError::LicenseDenied("enterprise_audit".into()))
        );
        assert!(!registry.is_active("enterprise_audit"));
    }
}

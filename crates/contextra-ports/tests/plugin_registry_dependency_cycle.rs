//! Property tests for `PluginRegistry` dependency cycle detection and topological resolution (INV-PLUGIN-DEPENDENCY).

use std::sync::Arc;
use contextra_ports::license::{FeatureRing, LicenseError, LicenseGate};
use contextra_ports::plugin::{PluginCapability, PluginError, PluginManifest, PluginRegistry};
use proptest::prelude::*;

struct PermissiveLicenseGate;

impl LicenseGate for PermissiveLicenseGate {
    fn check_ring(&self, _ring: FeatureRing) -> Result<(), LicenseError> {
        Ok(())
    }
}

static STATIC_NAMES: [&'static str; 10] = [
    "p0", "p1", "p2", "p3", "p4", "p5", "p6", "p7", "p8", "p9",
];

static SINGLETON_SLICES: [[&'static str; 1]; 10] = [
    ["p0"], ["p1"], ["p2"], ["p3"], ["p4"], ["p5"], ["p6"], ["p7"], ["p8"], ["p9"],
];

fn get_single_req(index: usize) -> &'static [&'static str] {
    &SINGLETON_SLICES[index]
}

struct DummyPlugin {
    name: &'static str,
    requires: &'static [&'static str],
    conflicts: &'static [&'static str],
}

impl PluginManifest for DummyPlugin {
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
        FeatureRing::Fast
    }

    fn capability(&self) -> PluginCapability {
        PluginCapability {
            name: self.name,
            version: (1, 0, 0),
            ring: 3,
            feature_ring_required: FeatureRing::Fast,
        }
    }

    fn activate(&self) -> Result<(), PluginError> {
        Ok(())
    }
}

proptest! {
    /// Property test: Any graph containing an explicit cycle MUST be rejected with `PluginError::DependencyCycle`,
    /// and NO plugins must be activated upon failure (All-or-Nothing guarantee).
    #[test]
    fn prop_dependency_cycle_detected_and_no_plugins_activated(
        cycle_length in 2usize..6,
        extra_plugins_count in 0usize..4,
    ) {
        let gate = Arc::new(PermissiveLicenseGate);
        let mut registry = PluginRegistry::new(gate);

        // Construct a cycle: p0 -> p1 -> ... -> p{N-1} -> p0
        let mut plugins: Vec<Box<dyn PluginManifest>> = Vec::new();

        for i in 0..cycle_length {
            let name = STATIC_NAMES[i];
            let next_index = (i + 1) % cycle_length;
            let req_slice = get_single_req(next_index);
            plugins.push(Box::new(DummyPlugin {
                name,
                requires: req_slice,
                conflicts: &[],
            }));
        }

        // Add extra non-cyclic plugins
        for i in 0..extra_plugins_count {
            let name = STATIC_NAMES[cycle_length + i];
            plugins.push(Box::new(DummyPlugin {
                name,
                requires: &[],
                conflicts: &[],
            }));
        }

        let result = registry.activate_all(plugins);

        // Verification 1: Must result in DependencyCycle error
        prop_assert!(
            matches!(result, Err(PluginError::DependencyCycle(_))),
            "Expected DependencyCycle, got {:?}",
            result
        );

        // Verification 2: All-or-Nothing — zero plugins active
        prop_assert_eq!(
            registry.snapshot().len(),
            0,
            "No plugins should be activated when a cycle is present"
        );
        for i in 0..cycle_length {
            prop_assert!(
                !registry.is_active(STATIC_NAMES[i]),
                "Cycle plugin {} should not be active",
                STATIC_NAMES[i]
            );
        }
        for i in 0..extra_plugins_count {
            prop_assert!(
                !registry.is_active(STATIC_NAMES[cycle_length + i]),
                "Extra plugin {} should not be active",
                STATIC_NAMES[cycle_length + i]
            );
        }
    }

    /// Property test: A valid Directed Acyclic Graph (DAG) of dependencies MUST activate ALL plugins
    /// cleanly without error.
    #[test]
    fn prop_dag_dependencies_activated_successfully(
        chain_length in 1usize..6,
    ) {
        let gate = Arc::new(PermissiveLicenseGate);
        let mut registry = PluginRegistry::new(gate);

        let mut plugins: Vec<Box<dyn PluginManifest>> = Vec::new();

        // Construct DAG chain: node_i requires node_{i-1} for i > 0
        for i in 0..chain_length {
            let name = STATIC_NAMES[i];
            let req_slice = if i > 0 {
                get_single_req(i - 1)
            } else {
                &[][..]
            };
            plugins.push(Box::new(DummyPlugin {
                name,
                requires: req_slice,
                conflicts: &[],
            }));
        }

        // Pass plugins in reverse order to test topological sorting requirement
        plugins.reverse();

        let result = registry.activate_all(plugins);
        prop_assert!(result.is_ok(), "DAG activation failed: {:?}", result);

        // Verification: all plugins are active
        for i in 0..chain_length {
            let name = STATIC_NAMES[i];
            prop_assert!(registry.is_active(name), "Plugin {} should be active", name);
        }
        prop_assert_eq!(registry.snapshot().len(), chain_length);
    }
}

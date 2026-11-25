//! Integration tests for `PluginRegistry::activate` and `PluginRegistry::activate_all`.

use std::sync::Arc;

use contextra_ports::license::{FeatureRing, LicenseError, LicenseGate};
use contextra_ports::plugin::{PluginCapability, PluginError, PluginManifest, PluginRegistry};

struct PermissiveLicenseGate;

impl LicenseGate for PermissiveLicenseGate {
    fn check_ring(&self, _ring: FeatureRing) -> Result<(), LicenseError> {
        Ok(())
    }
}

struct TestPlugin {
    name: &'static str,
    requires: &'static [&'static str],
    conflicts: &'static [&'static str],
    ring: FeatureRing,
}

impl PluginManifest for TestPlugin {
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
        PluginCapability::new(self.name, (1, 0, 0), 1, self.ring)
    }

    fn activate(&self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[test]
fn test_plugin_registry_activate_single_plugin() {
    let gate = Arc::new(PermissiveLicenseGate);
    let mut registry = PluginRegistry::new(gate);

    let plugin = Box::new(TestPlugin {
        name: "single_plugin",
        requires: &[],
        conflicts: &[],
        ring: FeatureRing::Fast,
    });

    assert!(!registry.is_active("single_plugin"));
    assert_eq!(registry.activate(plugin), Ok(()));
    assert!(registry.is_active("single_plugin"));

    let snapshot = registry.snapshot();
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].name, "single_plugin");
}

#[test]
fn test_plugin_registry_activate_all_batch() {
    let gate = Arc::new(PermissiveLicenseGate);
    let mut registry = PluginRegistry::new(gate);

    let p1 = Box::new(TestPlugin {
        name: "base_plugin",
        requires: &[],
        conflicts: &[],
        ring: FeatureRing::Fast,
    });

    let p2 = Box::new(TestPlugin {
        name: "ext_plugin",
        requires: &["base_plugin"],
        conflicts: &[],
        ring: FeatureRing::Fast,
    });

    assert_eq!(registry.activate_all(vec![p2, p1]), Ok(()));
    assert!(registry.is_active("base_plugin"));
    assert!(registry.is_active("ext_plugin"));
}

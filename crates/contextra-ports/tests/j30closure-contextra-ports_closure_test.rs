//! Integration closure tests for `with_fixed_output`, `with_rope_offset`, and `activate_all`.

use std::sync::Arc;

use contextra_ports::embedding::{
    ContextSegment, EmbeddingProvider, MockEmbedder, TextEmbeddingEngine,
};
use contextra_ports::license::{FeatureRing, LicenseError, LicenseGate};
use contextra_ports::plugin::{
    PluginCapability, PluginError, PluginManifest, PluginRegistry,
};
use contextra_ports::ModelFingerprint;

struct PermissiveLicenseGate;

impl LicenseGate for PermissiveLicenseGate {
    fn check_ring(&self, _ring: FeatureRing) -> Result<(), LicenseError> {
        Ok(())
    }
}

struct ClosureTestPlugin {
    name: &'static str,
    requires: &'static [&'static str],
    conflicts: &'static [&'static str],
    ring: FeatureRing,
}

impl PluginManifest for ClosureTestPlugin {
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

#[tokio::test]
async fn test_mock_embedder_with_fixed_output_behavior() {
    let custom_fixed = vec![1.0f32, 2.0f32, 3.0f32];
    let custom_embedder = MockEmbedder::with_fixed_output(custom_fixed.clone());
    assert_eq!(custom_embedder.embedding_dim(), 3);
    assert_eq!(custom_embedder.fixed_output, Some(custom_fixed.clone()));

    let vec = EmbeddingProvider::embed(&custom_embedder, "hello").await.unwrap();
    assert_eq!(vec, custom_fixed);

    let fixed_unit = MockEmbedder::fixed_unit(4);
    assert_eq!(fixed_unit.embedding_dim(), 4);
    assert_eq!(fixed_unit.fixed_output, Some(vec![0.1f32; 4]));

    let engine: &dyn TextEmbeddingEngine = &custom_embedder;
    let engine_output = engine.embed("test").await.unwrap();
    assert_eq!(engine_output, custom_fixed);
}

#[test]
fn test_context_segment_with_rope_offset_builder() {
    let fp = ModelFingerprint::new([1u8; 32], "test-model".to_string(), "fp16".to_string());

    let seg = ContextSegment::new(42, "Rope Segment")
        .with_fingerprint(&fp)
        .with_rope_offset(256);

    assert_eq!(seg.chunk_id, 42);
    assert_eq!(seg.text, "Rope Segment");
    assert_eq!(seg.model_fingerprint, Some(&fp));
    assert_eq!(seg.rope_offset, Some(256));

    let seg_offset = ContextSegment::with_offset(100, "With Offset", 512);
    assert_eq!(seg_offset.chunk_id, 100);
    assert_eq!(seg_offset.rope_offset, Some(512));
}

#[test]
fn test_plugin_registry_activate_all_and_single_activation() {
    let gate = Arc::new(PermissiveLicenseGate);
    let mut registry = PluginRegistry::new(gate);

    let p1 = Box::new(ClosureTestPlugin {
        name: "closure_base",
        requires: &[],
        conflicts: &[],
        ring: FeatureRing::Fast,
    });

    let p2 = Box::new(ClosureTestPlugin {
        name: "closure_dependent",
        requires: &["closure_base"],
        conflicts: &[],
        ring: FeatureRing::Fast,
    });

    // Test activate_all
    assert_eq!(registry.activate_all(vec![p2, p1]), Ok(()));
    assert!(registry.is_active("closure_base"));
    assert!(registry.is_active("closure_dependent"));

    // Test activate (which delegates to activate_all)
    let p3 = Box::new(ClosureTestPlugin {
        name: "closure_single",
        requires: &[],
        conflicts: &[],
        ring: FeatureRing::Fast,
    });
    assert_eq!(registry.activate(p3), Ok(()));
    assert!(registry.is_active("closure_single"));

    let snapshot = registry.snapshot();
    assert_eq!(snapshot.len(), 3);
}

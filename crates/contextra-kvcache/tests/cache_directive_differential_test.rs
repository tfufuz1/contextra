#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Differenztest für CacheDirective und StepId (KV3.2).
//! Verifiziert, dass die lokale Definition in `crate::types` und die Workspace-Definition in `contextra_types`
//! dieselben Serialisierungs-, Deserialisierungs- und Verhaltensstrukturen aufweisen.

use contextra_kvcache::types::{CacheDirective as LocalCacheDirective, StepId as LocalStepId};
use contextra_types::error::{CacheDirective as TypesCacheDirective, StepId as TypesStepId};
use std::time::Duration;

#[test]
fn test_cache_directive_and_step_id_differential() {
    // 1. StepId Identität
    let local_step = LocalStepId::new(42);
    let types_step = TypesStepId::new(42);

    assert_eq!(local_step.as_u64(), types_step.inner());
    assert_eq!(local_step.inner(), types_step.inner());
    assert_eq!(format!("{local_step}"), format!("{types_step}"));

    let local_step_json = serde_json::to_string(&local_step).unwrap();
    let types_step_json = serde_json::to_string(&types_step).unwrap();
    assert_eq!(local_step_json, types_step_json);

    // 2. CacheDirective Identität über alle Varianten
    let variants = vec![
        (LocalCacheDirective::Auto, TypesCacheDirective::Auto),
        (
            LocalCacheDirective::NeverCache,
            TypesCacheDirective::NeverCache,
        ),
        (
            LocalCacheDirective::Pin { ttl: None },
            TypesCacheDirective::Pin { ttl: None },
        ),
        (
            LocalCacheDirective::Pin {
                ttl: Some(Duration::from_millis(500)),
            },
            TypesCacheDirective::Pin {
                ttl: Some(Duration::from_millis(500)),
            },
        ),
        (
            LocalCacheDirective::ReleaseAfterStep {
                step_id: local_step,
            },
            TypesCacheDirective::ReleaseAfterStep {
                step_id: types_step,
            },
        ),
    ];

    for (loc, typ) in variants {
        let loc_json = serde_json::to_string(&loc).unwrap();
        let typ_json = serde_json::to_string(&typ).unwrap();
        assert_eq!(
            loc_json, typ_json,
            "JSON serialization must match across variants"
        );

        // Cross-deserialization
        let loc_from_typ: LocalCacheDirective = serde_json::from_str(&typ_json).unwrap();
        let typ_from_loc: TypesCacheDirective = serde_json::from_str(&loc_json).unwrap();
        assert_eq!(loc, loc_from_typ);
        assert_eq!(typ, typ_from_loc);
    }
}

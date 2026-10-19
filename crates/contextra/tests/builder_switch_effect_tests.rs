// FILE-CONTEXT
// ZWECK: Effect verification tests for every ContextraBuilder switch (G-05).
// INVARIANTEN: Every builder switch must have a verifiable effect on builder inspection / wiring report.

use std::sync::Arc;
use contextra::builder::ContextraBuilder;
use contextra_ports::erasure::{ErasureHooks, ErasureMode};
use contextra_ports::license::OpenFastGate;

struct DummyErasureHooks;
impl ErasureHooks for DummyErasureHooks {
    fn finish(&self, _token: contextra_ports::erasure::HookToken) -> contextra_types::Result<Option<contextra_ports::erasure::ErasureAttestation>> {
        Ok(None)
    }
}

#[test]
fn test_builder_switch_effects() {
    let builder = ContextraBuilder::new(128);

    // 1. with_kms / without_kms
    let b1 = builder.clone().with_kms(true);
    assert!(b1.kms_enabled());
    let b1_off = b1.without_kms();
    assert!(!b1_off.kms_enabled());

    // 2. with_kv_shredding / without_kv_shredding
    let b2 = builder.clone().with_kv_shredding(true);
    assert!(b2.kv_shredding_enabled());
    let b2_off = b2.without_kv_shredding();
    assert!(!b2_off.kv_shredding_enabled());

    // 3. with_checkpoint / without_checkpoint
    let b3 = builder.clone().with_checkpoint(true);
    assert!(b3.checkpoint_enabled());
    let b3_off = b3.without_checkpoint();
    assert!(!b3_off.checkpoint_enabled());

    // 4. with_erasure_mode
    let b4 = builder.clone().with_erasure_mode(ErasureMode::Assured);
    assert_eq!(b4.erasure_mode(), ErasureMode::Assured);
    assert!(b4.explicit_erasure_mode());

    // 5. with_erasure_hooks
    let hooks: Arc<dyn ErasureHooks> = Arc::new(DummyErasureHooks);
    let b5 = builder.clone().with_erasure_hooks(hooks.clone());
    assert!(b5.erasure_hooks().is_some());

    // 6. with_license_gate
    let gate = Arc::new(OpenFastGate);
    let b6 = builder.clone().with_license_gate(gate);
    assert_eq!(b6.license_gate().check_ring(contextra_ports::license::FeatureRing::Fast), Ok(()));

    // 7. with_router / without_router
    let b7 = builder.clone().with_router(true);
    assert!(b7.router_enabled());
    let b7_off = b7.without_router();
    assert!(!b7_off.router_enabled());

    // 8. with_sandbox / without_sandbox
    let b8 = builder.clone().with_sandbox(true);
    assert!(b8.sandbox_enabled());
    let b8_off = b8.without_sandbox();
    assert!(!b8_off.sandbox_enabled());
}

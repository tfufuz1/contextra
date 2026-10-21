// FILE-CONTEXT
// ZWECK: Test for ContextraBuilder switch effects.

use contextra::ContextraBuilder;
use contextra_ports::erasure::{ErasureHooks, EraseCtx, HookToken, PurgeOutput};
use contextra_types::{ContextraError, TxId};

struct DummyErasureHooks;

impl ErasureHooks for DummyErasureHooks {
    fn on_begin(&self, _: &EraseCtx<'_>) -> Result<HookToken, ContextraError> {
        Ok(HookToken::from_raw(1))
    }

    fn on_tombstoned(&self, _: &HookToken, _: TxId) -> Result<(), ContextraError> {
        Ok(())
    }

    fn on_purged(&self, _: &HookToken, _: &PurgeOutput) -> Result<(), ContextraError> {
        Ok(())
    }

    fn abort(&self, _: HookToken) {}
}

#[test]
fn test_builder_switches() {
    let b1 = ContextraBuilder::new(128).with_kms(true);
    assert!(b1.kms_enabled());

    let b2 = ContextraBuilder::new(128).without_kms();
    assert!(!b2.kms_enabled());

    let b3 = ContextraBuilder::new(128).with_checkpoint(true);
    assert!(b3.checkpoint_enabled());

    let b4 = ContextraBuilder::new(128).without_checkpoint();
    assert!(!b4.checkpoint_enabled());

    let hooks = std::sync::Arc::new(DummyErasureHooks);
    let b5 = ContextraBuilder::new(128).with_erasure_hooks(hooks);
    assert!(b5.erasure_hooks().is_some());
}

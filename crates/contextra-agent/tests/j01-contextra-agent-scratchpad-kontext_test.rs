//! Integration tests for J01 contextra-agent scratchpad & context symbol wiring.

use contextra_agent::clm_scratchpad::{
    ClmScratchpad, CountingContextEditAuditSink, CountingScratchpadCacheInvalidator, PinnedRegionId,
    ScratchpadEditOp,
};
use contextra_agent::context::{AgentContext, AgentEngine};
use contextra_agent::event_source::BackgroundEvent;
use contextra_db::volatile_vault::VaultConfig;
use contextra_db::{Contextra, ContextraConfig};
use contextra_types::error::{CacheDirective, StepId};
use contextra_types::{TenantId, TokenBudget, TxId};
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn test_scratchpad_builder_and_context_symbol_wiring() -> contextra_types::Result<()> {
    let temp_dir = TempDir::new()?;
    let db = Arc::new(Contextra::open_with_config(temp_dir.path(), ContextraConfig::default()).await?);
    let state_coll = db.collection("j01_scratchpad_state").await?;

    let audit_sink = Arc::new(CountingContextEditAuditSink::new());
    let cache_invalidator = Arc::new(CountingScratchpadCacheInvalidator::new());
    let pinned_region = PinnedRegionId::new("sys_prompt");
    let tenant_id = TenantId::SYSTEM;
    let created_at_tx = TxId(999);

    let vault_config = VaultConfig {
        max_capacity_bytes: 1024 * 1024,
        attempt_mlock: false,
    };

    // Test ClmScratchpad builder setters: with_tenant_id, with_audit_sink, with_cache_invalidator, with_pinned_region, with_created_at_tx
    let mut scratchpad = ClmScratchpad::new("j01-task".to_string(), vault_config)
        .with_tenant_id(tenant_id)
        .with_audit_sink(audit_sink.clone())
        .with_cache_invalidator(cache_invalidator.clone())
        .with_pinned_region(pinned_region.clone())
        .with_created_at_tx(created_at_tx);

    assert_eq!(scratchpad.tenant_id(), tenant_id);
    assert_eq!(scratchpad.created_at_tx(), 999);
    assert_eq!(scratchpad.pinned_regions(), &[pinned_region.clone()]);

    // Apply an edit to exercise audit_sink record
    scratchpad.apply_edit(
        ScratchpadEditOp::Append {
            content: b"work_step_1".to_vec(),
            label: Some("chunk1".to_string()),
        },
        TxId(1001),
    )?;

    assert_eq!(audit_sink.count(), 1);

    // Checkpoint & reset scratchpad to exercise cache_invalidator
    let checkpoint = scratchpad.checkpoint_and_reset()?;
    assert_eq!(checkpoint.task_id, "j01-task");
    assert_eq!(audit_sink.count(), 2);
    assert_eq!(cache_invalidator.count(), 1);

    // Test AgentContext & AgentEngine wiring
    let mut ctx = AgentContext::try_new(
        "j01-task",
        "start_node",
        db,
        state_coll,
        TokenBudget::new(1000, 0),
    )?
    .with_clm_scratchpad(scratchpad);

    let mut engine = AgentEngine::new();

    // 1. directive_for_node & set_cache_directive / get_cache_directive via AgentEngine::run
    let step1 = StepId::new(1);
    let directive1 = engine.run(&mut ctx, step1, "System_Prompt_Node");
    assert!(matches!(directive1, CacheDirective::Pin { ttl: None }));
    assert_eq!(ctx.get_cache_directive(), &CacheDirective::Pin { ttl: None });

    let step2 = StepId::new(2);
    let directive2 = engine.run(&mut ctx, step2, "Transient_Reasoning_Node");
    assert!(matches!(directive2, CacheDirective::ReleaseAfterStep { step_id } if step_id == step2));
    assert_eq!(
        ctx.get_cache_directive(),
        &CacheDirective::ReleaseAfterStep { step_id: step2 }
    );

    // 2. set_cache_directive direct call
    ctx.set_cache_directive(CacheDirective::Auto);
    assert_eq!(ctx.get_cache_directive(), &CacheDirective::Auto);

    // 3. directive_for_node direct call
    let direct_directive = ctx.directive_for_node("System_Prompt_Node", StepId::new(3));
    assert!(matches!(direct_directive, CacheDirective::Pin { ttl: None }));

    // 4. try_attach_event
    let event = BackgroundEvent {
        payload: serde_json::json!({"test_key": "j01_val"}),
        source: "j01_source".to_string(),
        observed_at_seq: 1,
    };
    ctx.try_attach_event(event)?;
    assert_eq!(ctx.events.len(), 1);

    Ok(())
}

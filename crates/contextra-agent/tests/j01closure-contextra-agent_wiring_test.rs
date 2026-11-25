// FILE-CONTEXT Header (Format v3)
// ZWECK: Closure Integration Test für J01 contextra-agent Scratchpad & Kontext Symbol Wiring.
// INVARIANTEN: Zero-Panic, pure RAM-Belegung, vollständige Verifikation der 9 Ausgangssymbole.
// STAND: TS:2026-10-04T00:00:00Z

use contextra_agent::clm_scratchpad::{
    ClmScratchpad, CountingContextEditAuditSink, CountingScratchpadCacheInvalidator,
    PinnedRegionId, ScratchpadEditOp,
};
use contextra_agent::context::AgentContext;
use contextra_agent::event_source::BackgroundEvent;
use contextra_db::volatile_vault::VaultConfig;
use contextra_types::error::{CacheDirective, StepId};
use contextra_types::{TenantId, TokenBudget, TxId};
use std::sync::Arc;

#[tokio::test]
async fn test_j01closure_symbol_wiring_and_execution() -> contextra_types::Result<()> {
    let temp_dir = tempfile::TempDir::new()?;
    let config = contextra_db::ContextraConfig::default();
    let db = Arc::new(contextra_db::Contextra::open_with_config(temp_dir.path(), config).await?);
    let state_coll = db.collection("test_j01closure").await?;

    let audit_sink = Arc::new(CountingContextEditAuditSink::new());
    let cache_invalidator = Arc::new(CountingScratchpadCacheInvalidator::new());
    let tenant_id = TenantId::try_new(999).unwrap();
    let created_at_tx = TxId(1001);
    let pinned_region = PinnedRegionId::new("sys_pinned_rules");

    let vault_config = VaultConfig {
        max_capacity_bytes: 1024 * 1024,
        attempt_mlock: false,
    };

    // 1. ClmScratchpad::new_scoped (verifies with_tenant_id, with_created_at_tx, with_audit_sink, with_cache_invalidator, with_pinned_region)
    let mut scratchpad = ClmScratchpad::new_scoped(
        "task-j01closure".to_string(),
        vault_config,
        tenant_id,
        created_at_tx,
        audit_sink.clone(),
        cache_invalidator.clone(),
        vec![pinned_region.clone()],
    );

    assert_eq!(scratchpad.tenant_id(), tenant_id);
    assert_eq!(scratchpad.created_at_tx(), 1001);
    assert_eq!(scratchpad.pinned_regions().len(), 1);
    assert_eq!(scratchpad.pinned_regions()[0], pinned_region);

    // Apply an edit and verify audit sink recording
    scratchpad.apply_edit(
        ScratchpadEditOp::Append {
            content: b"closure payload".to_vec(),
            label: Some("chunk1".to_string()),
        },
        TxId(1002),
    )?;
    assert_eq!(audit_sink.count(), 1);

    // Checkpoint and reset -> verifies cache invalidator and audit sink
    let checkpoint = scratchpad.checkpoint_and_reset()?;
    assert_eq!(checkpoint.completed_subgoal_index, 0);
    assert_eq!(audit_sink.count(), 2);
    assert_eq!(cache_invalidator.count(), 1);

    // 2. AgentContext cache directive methods: set_cache_directive, get_cache_directive, directive_for_node
    let mut ctx = AgentContext::try_new(
        "task-j01closure",
        "start_node",
        db,
        state_coll,
        TokenBudget::new(5000, 0),
    )?
    .with_clm_scratchpad(scratchpad);

    assert_eq!(ctx.get_cache_directive(), &CacheDirective::Auto);

    ctx.set_cache_directive(CacheDirective::Pin { ttl: None });
    assert_eq!(
        ctx.get_cache_directive(),
        &CacheDirective::Pin { ttl: None }
    );

    let system_node_dir = ctx.directive_for_node("system_prompt_node", StepId::new(1));
    assert_eq!(system_node_dir, CacheDirective::Pin { ttl: None });

    let reasoning_node_dir = ctx.directive_for_node("reasoning_step_node", StepId::new(2));
    assert_eq!(
        reasoning_node_dir,
        CacheDirective::ReleaseAfterStep {
            step_id: StepId::new(2)
        }
    );

    // 3. AgentContext::try_attach_event
    let event = BackgroundEvent {
        payload: serde_json::json!({ "j01closure": true }),
        source: "j01closure_test".to_string(),
        observed_at_seq: 1,
    };
    ctx.try_attach_event(event)?;
    assert_eq!(ctx.events.len(), 1);

    Ok(())
}

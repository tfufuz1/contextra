use contextra_agent::context::{AgentContext, AgentEngine};
use contextra_types::error::{CacheDirective, StepId};
use contextra_types::TokenBudget;
use std::sync::Arc;
use std::time::Instant;

#[tokio::test]
async fn test_system_prompt_pinning_latency_under_100_micros() -> contextra_types::Result<()> {
    let temp_dir = tempfile::TempDir::new()?;
    let config = contextra_engine::ContextraConfig::default();
    let db = Arc::new(contextra_engine::Contextra::open_with_config(temp_dir.path(), config).await?);
    let state_coll = db.collection("test_latency").await?;

    let mut ctx = AgentContext::try_new(
        "latency_task",
        "system_prompt_node",
        db,
        state_coll,
        TokenBudget::new(1000, 0),
    )?;

    let mut engine = AgentEngine::new();
    let step_id = StepId::new(1);

    // Warmup access
    let _ = engine.run(&mut ctx, step_id, "system_prompt_node");

    // Benchmark 1,000 directive evaluation and context updates for pinned system prompt segments
    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        let directive = engine.run(&mut ctx, step_id, "system_prompt_node");
        assert!(matches!(directive, CacheDirective::Pin { ttl: None }));
    }

    let elapsed = start.elapsed();
    let avg_latency_micros = (elapsed.as_nanos() as f64 / iterations as f64) / 1000.0;

    println!(
        "System prompt directive evaluation latency: avg = {:.3} µs over {} iterations (total: {:?})",
        avg_latency_micros, iterations, elapsed
    );

    // Enforce regression limit: average latency MUST be < 100 µs
    assert!(
        avg_latency_micros < 100.0,
        "System prompt directive latency {:.3} µs exceeded target threshold of 100.0 µs!",
        avg_latency_micros
    );

    Ok(())
}

#[tokio::test]
async fn test_system_prompt_pinning_agent_flow() -> contextra_types::Result<()> {
    let temp_dir = tempfile::TempDir::new()?;
    let config = contextra_engine::ContextraConfig::default();
    let db = Arc::new(contextra_engine::Contextra::open_with_config(temp_dir.path(), config).await?);
    let state_coll = db.collection("test_flow").await?;

    let mut ctx = AgentContext::try_new(
        "flow_task",
        "system_prompt_init",
        db,
        state_coll,
        TokenBudget::new(1000, 0),
    )?;

    let mut engine = AgentEngine::new();
    let step_id = StepId::new(42);

    let directive = engine.run(&mut ctx, step_id, "system_prompt_init");
    assert_eq!(directive, CacheDirective::Pin { ttl: None });
    assert_eq!(
        ctx.get_cache_directive(),
        &CacheDirective::Pin { ttl: None }
    );

    Ok(())
}

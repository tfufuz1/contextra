// FILE-CONTEXT
// ZWECK: Test for ContextraBuilder::checkpoint_enabled and PipelineStages invariants (W1-11 Stage 5).

use contextra::builder::ContextraBuilder;
use contextra_db::wiring::{PipelineStage, PipelineStages};

#[test]
fn checkpoint_enabled_follows_config() {
    let builder_off = ContextraBuilder::new(768).with_checkpoint(false);
    assert!(!builder_off.checkpoint_enabled());

    let builder_on = ContextraBuilder::new(768).with_checkpoint(true);
    assert!(builder_on.checkpoint_enabled());
}

#[test]
fn pipeline_stage_order_is_invariant() {
    let stages = PipelineStages::canonical();
    assert!(stages.verify_order());
    assert_eq!(
        stages.stages,
        [
            PipelineStage::Retrieval,
            PipelineStage::Rerank,
            PipelineStage::Plan,
            PipelineStage::EgressFilter,
        ]
    );

    let default_stages = PipelineStages::default();
    assert_eq!(default_stages, stages);
    assert!(default_stages.verify_order());
}

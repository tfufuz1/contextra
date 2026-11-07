//! Tests for abstract cognitive channels in `contextra-cognition`.
//! Confirms RAG memory intake can operate over channels decoupled from concrete pipelines.

use contextra_cognition::channel::{
    CognitiveInputChannel, CognitiveOutputSink, DirectCognitiveChannel, InMemoryCognitiveChannel,
    InMemoryOutputSink,
};
use contextra_cognition::PreCondensedInput;
use contextra_types::{DocId, TenantId};

#[tokio::test]
async fn test_direct_cognitive_channel_intake() {
    let tenant = TenantId::try_new(1001).expect("valid tenant");
    let channel = DirectCognitiveChannel;

    let inputs = vec![PreCondensedInput {
        tenant_id: tenant.clone(),
        source_label: "test-source".to_string(),
        text_digest: "First pre-condensed memory turn".to_string(),
        token_count: 5,
        provenance: vec![DocId::new(1), DocId::new(2)],
    }];

    let segments = channel
        .send_pre_condensed(&tenant, &inputs)
        .await
        .expect("intake must succeed");

    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].turn_ids, vec![DocId::new(1), DocId::new(2)]);
}

#[tokio::test]
async fn test_in_memory_mpsc_channel_intake_and_sink() {
    let tenant = TenantId::try_new(2002).expect("valid tenant");
    let (channel, mut rx) = InMemoryCognitiveChannel::new(10);
    let sink = InMemoryOutputSink::<String>::new();

    let inputs = vec![PreCondensedInput {
        tenant_id: tenant.clone(),
        source_label: "scratchpad-checkpoint".to_string(),
        text_digest: "Decision log for task planning".to_string(),
        token_count: 10,
        provenance: vec![DocId::new(100)],
    }];

    let segments = channel
        .send_pre_condensed(&tenant, &inputs)
        .await
        .expect("send_pre_condensed must succeed");

    assert_eq!(segments.len(), 1);

    // Verify receiver receives event
    let (received_tenant, received_inputs) = rx.recv().await.expect("channel must receive event");
    assert_eq!(received_tenant, tenant);
    assert_eq!(received_inputs.len(), 1);

    // Verify output sink collects emitted result
    sink.emit("Consolidation finished for tenant-beta".to_string())
        .await
        .expect("sink emit should succeed");

    assert_eq!(sink.len(), 1);
    assert_eq!(sink.collected()[0], "Consolidation finished for tenant-beta");
}

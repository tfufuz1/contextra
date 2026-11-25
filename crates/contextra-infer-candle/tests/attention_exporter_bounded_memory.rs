#![forbid(unsafe_code)]

use contextra_infer_candle::{CandleAttentionExporter, MAX_TRACKED_REQUESTS};
use contextra_ports::{AttentionExporter, RequestId};

#[test]
fn test_candle_attention_exporter_bounded_memory_limit() {
    let exporter = CandleAttentionExporter::new();

    // Verify initial state
    assert_eq!(exporter.tracked_request_count(), 0);

    // Insert 300 requests (exceeding MAX_TRACKED_REQUESTS = 256)
    for i in 0..300 {
        let req_id = RequestId(i as u64);
        exporter.record_attention_weights(req_id, vec![i as f32, (i + 1) as f32]);
    }

    // Capacity must be strictly bounded at MAX_TRACKED_REQUESTS (256)
    assert_eq!(
        exporter.tracked_request_count(),
        MAX_TRACKED_REQUESTS,
        "tracked request count must be capped at MAX_TRACKED_REQUESTS (256)"
    );

    // Oldest requests (0..44) must be evicted
    for i in 0..44 {
        assert!(
            exporter
                .export_attention_weights(RequestId(i as u64))
                .is_none(),
            "Request {} should have been evicted from ring buffer",
            i
        );
    }

    // Recent requests (44..300) must be retained with exact weights
    for i in 44..300 {
        let exported = exporter.export_attention_weights(RequestId(i as u64));
        assert!(
            exported.is_some(),
            "Request {} should be present in ring buffer",
            i
        );
        assert_eq!(exported.unwrap(), vec![i as f32, (i + 1) as f32]);
    }
}

#[test]
fn test_record_layer_head_scores_combines_elementwise() {
    let exporter = CandleAttentionExporter::new();
    let req_id = RequestId(1001);

    // 2 layers/heads of length 4
    let layer1 = vec![1.0, 2.0, 3.0, 4.0];
    let layer2 = vec![0.5, 0.5, 0.5, 0.5];

    exporter.record_layer_head_scores(req_id, &[layer1, layer2]);

    let exported = exporter.export_attention_weights(req_id);
    assert_eq!(exported, Some(vec![1.5, 2.5, 3.5, 4.5]));
}

#[test]
fn test_clear_resets_history() {
    let exporter = CandleAttentionExporter::new();
    exporter.record_attention_weights(RequestId(1), vec![1.0]);
    assert_eq!(exporter.tracked_request_count(), 1);

    exporter.clear();
    assert_eq!(exporter.tracked_request_count(), 0);
    assert!(exporter.export_attention_weights(RequestId(1)).is_none());
}

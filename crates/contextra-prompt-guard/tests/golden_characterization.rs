use contextra_prompt_guard::{
    PromptInjectionGuard, QuarantinePolicy, SecurityAuditLogger, DEFAULT_REDACTION_PLACEHOLDER,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct CorpusItem {
    id: String,
    text: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct GoldenResult {
    id: String,
    detected: bool,
    matched_pattern: Option<String>,
    quarantined_text: String,
}

#[test]
fn test_golden_prompt_injection_characterization() {
    let manifest_bytes = include_str!("golden/corpus_inputs.json");
    let items: Vec<CorpusItem> = serde_json::from_str(manifest_bytes)
        .expect("Failed to deserialize golden corpus_inputs.json");

    let audit_logger = SecurityAuditLogger::default();
    let guard = PromptInjectionGuard::new(
        QuarantinePolicy::Escalate,
        DEFAULT_REDACTION_PLACEHOLDER.to_string(),
        PromptInjectionGuard::default_patterns(),
        audit_logger.clone(),
    );

    let mut results = Vec::new();

    for item in &items {
        let matched = guard.detect(&item.text);
        let mut obj = serde_json::json!({
            "id": &item.id,
            "metadata": {
                "text": &item.text
            }
        });
        let detected = guard.process_result(&item.id, "golden_col", obj.as_object_mut().unwrap());
        let res_text = obj["metadata"]["text"].as_str().unwrap().to_string();

        results.push(GoldenResult {
            id: item.id.clone(),
            detected,
            matched_pattern: matched,
            quarantined_text: res_text,
        });
    }

    assert!(!results[0].detected);
    assert!(!results[1].detected);
    assert!(!results[2].detected);

    assert!(results[3].detected);
    assert_eq!(results[3].quarantined_text, DEFAULT_REDACTION_PLACEHOLDER);

    assert!(results[4].detected);
    assert_eq!(results[4].quarantined_text, DEFAULT_REDACTION_PLACEHOLDER);

    assert!(results[5].detected);
    assert!(results[6].detected);
    assert!(results[7].detected);
    assert!(results[8].detected);
    assert!(results[9].detected);
    assert!(results[10].detected);
    assert!(results[11].detected);
    assert!(results[12].detected);

    let audit_events = audit_logger.get_recorded_events();
    assert_eq!(audit_events.len(), 10);
    assert_eq!(audit_events[0].doc_id, "inject_01_direct_inst");
    assert_eq!(audit_events[0].action_taken, "quarantined_and_escalated");
}

//! Fuzz-Target für PromptInjectionGuard (Robustheit gegen beliebige Bytefolgen, Zero-Panic).

#![no_main]
use arbitrary::Arbitrary;
use contextra_prompt_guard::{
    PromptInjectionGuard, QuarantinePolicy, SecurityAuditLogger, DEFAULT_REDACTION_PLACEHOLDER,
};
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    payload: String,
    doc_id: String,
    collection: String,
}

fuzz_target!(|input: FuzzInput| {
    // 1. Normalisierung, Base64-Dekodierung und Kandidatenextraktion prüfen
    let _ = PromptInjectionGuard::normalize_text(&input.payload);
    let _ = PromptInjectionGuard::collapse_whitespace(&input.payload);
    let _ = PromptInjectionGuard::strip_whitespace(&input.payload);
    let _ = PromptInjectionGuard::decode_base64(&input.payload);
    let _ = PromptInjectionGuard::extract_base64_candidates(&input.payload);

    for c in input.payload.chars() {
        let _ = PromptInjectionGuard::is_zero_width(c);
        let _ = PromptInjectionGuard::skeletonize_char(c);
    }

    // 2. Guard-Erkennung und Result-Processing über alle Policies
    let policies = [
        QuarantinePolicy::Strict,
        QuarantinePolicy::FlagOnly,
        QuarantinePolicy::Escalate,
    ];

    for policy in policies {
        let audit_logger = SecurityAuditLogger::default();
        let guard = PromptInjectionGuard::new(
            policy,
            DEFAULT_REDACTION_PLACEHOLDER.to_string(),
            PromptInjectionGuard::default_patterns(),
            audit_logger,
        );

        let _ = guard.detect(&input.payload);

        let mut obj_meta = serde_json::json!({
            "id": input.doc_id,
            "metadata": {
                "text": input.payload,
            }
        });
        if let Some(map) = obj_meta.as_object_mut() {
            let _ = guard.process_result(&input.doc_id, &input.collection, map);
        }

        let mut obj_root = serde_json::json!({
            "id": input.doc_id,
            "text": input.payload,
        });
        if let Some(map) = obj_root.as_object_mut() {
            let _ = guard.process_result(&input.doc_id, &input.collection, map);
        }
    }
});

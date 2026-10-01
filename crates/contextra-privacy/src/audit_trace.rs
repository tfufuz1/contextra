// FILE-CONTEXT
// ZWECK: Deterministische Audit-Trace-Generierung für Egress-Klassifikationsentscheidungen (INV-EGRESS-AUDIT-1 / P28).
// INVARIANTEN: Der Hash-Trace ist vollkommen deterministisch aus Payload, Regel-ID und injiziertem Clock-Zeitstempel.
// Kein thread_rng(), kein Random-Nonce, Zero-Panic.

use crate::egress_vault::{
    BlockReason, BoxFuture, EgressClassification, EgressClassifier,
};
use contextra_ports::Clock;

/// Extrahiert eine stabile, nicht-leere Regel-Kennung aus einer `EgressClassification`.
pub fn extract_rule_id(classification: &EgressClassification) -> String {
    match classification {
        EgressClassification::Allow => "ALLOW".to_string(),
        EgressClassification::Block(reason) => match reason {
            BlockReason::SensitivePattern(rule_id) => rule_id.clone(),
            BlockReason::PolicyDenied(msg) => format!("POLICY_DENIED:{msg}"),
            BlockReason::EgressPolicyDenied(msg) => format!("EGRESS_POLICY_DENIED:{msg}"),
            BlockReason::ClassificationTimeout => "CLASSIFICATION_TIMEOUT".to_string(),
            BlockReason::InternalError(msg) => format!("INTERNAL_ERROR:{msg}"),
        },
    }
}

/// Berechnet einen deterministischen 32-Byte-BLAKE3-Hash-Trace für eine Klassifikationsentscheidung.
///
/// Segmente werden in fester Byte-Reihenfolge mit 8-Byte Little-Endian Längenpräfixen serialisiert:
/// 1. Payload-Länge + Payload-Bytes
/// 2. Regel-ID-Länge + Regel-ID-Bytes
/// 3. Zeitstempel-Länge (8 Bytes) + Zeitstempel (Nanosekunden seit Unix-Epoche, LE-Bytes)
pub fn compute_audit_trace(payload: &str, rule_id: &str, nanos: u64) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();

    // Segment 1: Payload
    let payload_bytes = payload.as_bytes();
    hasher.update(&(payload_bytes.len() as u64).to_le_bytes());
    hasher.update(payload_bytes);

    // Segment 2: Regel-ID
    let rule_id_bytes = rule_id.as_bytes();
    hasher.update(&(rule_id_bytes.len() as u64).to_le_bytes());
    hasher.update(rule_id_bytes);

    // Segment 3: Inzipierter Zeitstempel
    let nanos_bytes = nanos.to_le_bytes();
    hasher.update(&(nanos_bytes.len() as u64).to_le_bytes());
    hasher.update(&nanos_bytes);

    *hasher.finalize().as_bytes()
}

/// Erweiterter Trait für Egress-Klassifikatoren mit kryptografischem Audit-Trace.
pub trait EgressClassifierTrace: EgressClassifier {
    /// Klassifiziert einen Payload und liefert sowohl das Klassifikationsergebnis
    /// als auch einen 32-Byte-BLAKE3-Hash-Trace.
    fn classify_with_trace<'a>(
        &'a self,
        payload: &'a str,
        clock: &'a dyn Clock,
    ) -> BoxFuture<'a, (EgressClassification, [u8; 32])>;
}

impl<T: EgressClassifier + ?Sized> EgressClassifierTrace for T {
    fn classify_with_trace<'a>(
        &'a self,
        payload: &'a str,
        clock: &'a dyn Clock,
    ) -> BoxFuture<'a, (EgressClassification, [u8; 32])> {
        Box::pin(async move {
            let nanos = clock.now_unix_nanos();
            let classification = self.classify(payload).await;
            let rule_id = extract_rule_id(&classification);
            let trace = compute_audit_trace(payload, &rule_id, nanos);
            (classification, trace)
        })
    }
}

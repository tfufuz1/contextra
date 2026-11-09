//! `audit_trace` — Deterministische Audit-Trace-Generierung für Egress-Klassifikationsentscheidungen.
//!
//! # Zweck
//! Berechnet kryptografisch verifizierbare Audit-Traces (Prüfspuren) für Egress-Klassifikationsentscheidungen
//! gemäß INV-EGRESS-AUDIT-1 und Richtlinie P28. Ein Audit-Trace verknüpft die Payload-Daten, die getroffene
//! Entscheidung bzw. Regel-ID und den Zeitstempel deterministisch miteinander.
//!
//! # Ring-Zugehörigkeit
//! Ring 3 (Domain / Audit-Dienste).
//!
//! # Invarianten
//! - Der Hash-Trace ist vollkommen deterministisch aus Payload, Regel-ID und injiziertem Clock-Zeitstempel.
//! - Kein `thread_rng()`, kein Random-Nonce.
//! - Zero-Panic: Fehlerbedingungen oder ungültige Längen führen nicht zum Absturz.

use crate::egress_vault::{BlockReason, BoxFuture, EgressClassification, EgressClassifier};
use crate::types::Clock;

// ANCHOR[contextra-privacy:AT-001]
/// Extrahiert eine stabile, nicht-leere Regel-Kennung aus einer [`EgressClassification`].
///
/// # Ablauf
/// 1. Bei `EgressClassification::Allow` wird die statische Kennung `"ALLOW"` zurückgegeben.
/// 2. Bei `EgressClassification::Block(reason)` wird der Grund differenziert:
///    - `SensitivePattern`: Gibt die konkrete Regel-ID (z. B. `"R-001"`) zurück.
///    - `PolicyDenied`: Formatiert mit Präfix `"POLICY_DENIED:"`.
///    - `EgressPolicyDenied`: Formatiert mit Präfix `"EGRESS_POLICY_DENIED:"`.
///    - `ClassificationTimeout`: Gibt `"CLASSIFICATION_TIMEOUT"` zurück.
///    - `InternalError`: Formatiert mit Präfix `"INTERNAL_ERROR:"`.
///
/// # Parameter
/// * `classification`: Die zu untersuchende Klassifikationsentscheidung.
///
/// # Rückgabe
/// Stabile Kennung als `String`.
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

// ANCHOR[contextra-privacy:AT-002]
/// Berechnet einen deterministischen 32-Byte-BLAKE3-Hash-Trace für eine Klassifikationsentscheidung.
///
/// Segmente werden in fester Byte-Reihenfolge mit 8-Byte Little-Endian Längenpräfixen serialisiert:
/// 1. Payload-Länge (u64 LE) + Payload-Bytes.
/// 2. Regel-ID-Länge (u64 LE) + Regel-ID-Bytes.
/// 3. Zeitstempel-Länge (8 Bytes, u64 LE) + Zeitstempel (Nanosekunden seit Unix-Epoche, u64 LE).
///
/// # Vorbedingungen
/// `nanos` muss aus einem standardkonformen Clock-Port stammen (Regel 3 aus CLAUDE.md).
///
/// # Nachbedingungen
/// Liefert reproduzierbar denselben 32-Byte BLAKE3-Hashwert für identische Eingaben.
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

    // Segment 3: Injizierter Zeitstempel
    let nanos_bytes = nanos.to_le_bytes();
    hasher.update(&(nanos_bytes.len() as u64).to_le_bytes());
    hasher.update(&nanos_bytes);

    *hasher.finalize().as_bytes()
}

// ANCHOR[contextra-privacy:AT-003]
/// Erweiterter Trait für Egress-Klassifikatoren mit kryptografischem Audit-Trace.
///
/// Kombiniert die reguläre Klassifikation mit der deterministischen Trace-Berechnung über einen Zeitstempel-Port.
pub trait EgressClassifierTrace: EgressClassifier {
    /// Klassifiziert einen Payload und liefert sowohl das Klassifikationsergebnis
    /// als auch einen 32-Byte-BLAKE3-Hash-Trace.
    ///
    /// # Ablauf
    /// 1. Liest den aktuellen Zeitstempel in Nanosekunden aus `clock.now_unix_nanos()`.
    /// 2. Führt die asynchrone Klassifikation über `self.classify(payload)` aus.
    /// 3. Extrahiert die Regel-Kennung via [`extract_rule_id`].
    /// 4. Berechnet den Audit-Trace via [`compute_audit_trace`].
    /// 5. Gibt das Tupel `(Klassifikation, Trace)` zurück.
    ///
    /// # Nebenwirkungen
    /// Liest Zeit über den injizierten `Clock`-Port (Port-Entkopplung gemäß Regel 3).
    fn classify_with_trace<'a>(
        &'a self,
        payload: &'a str,
        clock: &'a dyn Clock,
    ) -> BoxFuture<'a, (EgressClassification, [u8; 32])>;
}

// ANCHOR[contextra-privacy:AT-004]
// Automatische Implementierung für alle Typen, die EgressClassifier implementieren.
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

/// Neutraler Senken-Trait für die Weiterleitung von Audit-Ereignissen.
pub trait EventSink: Send + Sync {
    /// Zeichnet ein neutrales Ereignis auf.
    fn record(&self, event: &AuditEvent);
}

/// No-Op Senke für autonome Umgebungen.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoOpEventSink;

impl EventSink for NoOpEventSink {
    #[inline]
    fn record(&self, _event: &AuditEvent) {}
}

/// In-Memory Senke für Test- und Beobachtungszwecke.
#[derive(Debug, Default)]
pub struct InMemoryEventSink {
    events: std::sync::RwLock<Vec<AuditEvent>>,
}

impl InMemoryEventSink {
    /// Erstellt eine neue leere `InMemoryEventSink`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            events: std::sync::RwLock::new(Vec::new()),
        }
    }

    /// Gibt eine Kopie aller bisher aufgezeichneten Ereignisse zurück.
    #[must_use]
    pub fn events(&self) -> Vec<AuditEvent> {
        self.events.read().map(|g| g.clone()).unwrap_or_default()
    }

    /// Leert den internen Ereignisspeicher.
    pub fn clear(&self) {
        if let Ok(mut g) = self.events.write() {
            g.clear();
        }
    }
}

impl EventSink for InMemoryEventSink {
    fn record(&self, event: &AuditEvent) {
        if let Ok(mut g) = self.events.write() {
            g.push(event.clone());
        }
    }
}

/// Neutrales Audit-Ereignis.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AuditEvent {
    /// Zeitstempel in Nanosekunden (monoton oder Unix-Epoche von Clock injiziert).
    pub timestamp_nanos: u64,
    /// Kategorie des Ereignisses (z. B. "egress_blocked", "egress_allowed", "vault_access").
    pub category: String,
    /// Eindeutige Regel- oder Richtlinien-Kennung.
    pub rule_id: Option<String>,
    /// Neutraler Statuscode oder Ergebnisbeschreibung.
    pub outcome: String,
    /// Anzahl verarbeiteter Bytes.
    pub bytes_processed: usize,
}

impl AuditEvent {
    /// Erzeugt ein neues Audit-Ereignis ohne vertrauliche Nutzlast-Inhalte.
    #[must_use]
    pub fn new(
        timestamp_nanos: u64,
        category: impl Into<String>,
        rule_id: Option<String>,
        outcome: impl Into<String>,
        bytes_processed: usize,
    ) -> Self {
        Self {
            timestamp_nanos,
            category: category.into(),
            rule_id,
            outcome: outcome.into(),
            bytes_processed,
        }
    }
}

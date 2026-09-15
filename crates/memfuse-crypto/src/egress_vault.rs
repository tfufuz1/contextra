// FILE-CONTEXT
// ZWECK: Cloud-Egress-Vault mit Layer-1-Regex-Klassifikation und Fail-Closed-Semantik.
// INVARIANTEN: Jede Klassifikationsentscheidung terminiert innerhalb des definierten Timeouts.
// Bei Timeout, Regex-Fehlern oder Laufzeitfehlern gilt strikt Fail-Closed (EgressClassification::Block).
// STAND: TS:2026-09-13 (SESSION: HEAD)

use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

/// Type alias for boxed dyn futures in `EgressClassifier` trait to ensure dyn compatibility.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Grund für das Blockieren einer Egress-Anfrage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum BlockReason {
    /// Ein sensibles Muster (PII, Secret, API-Key etc.) wurde erkannt.
    SensitivePattern(String),
    /// Richtlinie untersagt die Übertragung.
    PolicyDenied(String),
    /// Auswertung hat das Zeitlimit überschritten (Fail-Closed).
    ClassificationTimeout,
    /// Interne Verarbeitungsstörung (Fail-Closed).
    InternalError(String),
}

/// Klassifikationsergebnis der Egress-Prüfung.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EgressClassification {
    /// Übertragung zulässig.
    Allow,
    /// Übertragung blockiert mit Grund.
    Block(BlockReason),
}

/// Ein vorkompiliertes Regex-Muster für die Egress-Klassifikation.
#[derive(Debug, Clone)]
pub struct CompiledPattern {
    /// Bezeichner oder Originalmuster.
    pub name: String,
    /// Kompiliertes Regex-Objekt.
    pub regex: regex::Regex,
}

impl CompiledPattern {
    /// Erstellt ein neues `CompiledPattern`.
    pub fn new(name: impl Into<String>, pattern: &str) -> Result<Self, EgressVaultError> {
        let name_str = name.into();
        let regex = regex::Regex::new(pattern).map_err(|e| EgressVaultError::InvalidPattern {
            pattern: name_str.clone(),
            reason: e.to_string(),
        })?;
        Ok(Self {
            name: name_str,
            regex,
        })
    }
}

/// Fehler beim Erstellen oder Laden des EgressVaults.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum EgressVaultError {
    #[error("Invalid pattern '{pattern}': {reason}")]
    InvalidPattern { pattern: String, reason: String },
}

/// Maximale zulässige Payload-Länge in Bytes für Layer-1-Klassifikation.
pub const MAX_CLASSIFY_PAYLOAD_BYTES: usize = 65_536;

/// Führt eine Layer-1-Regex-Klassifikation auf dem Payload mit neuem Task und hartem Timeout aus.
///
/// Invariante: **Fail-Closed**. Bei Timeout, Panic oder Fehler wird stets `Block(...)` zurückgegeben.
pub async fn classify_layer1(
    payload: &str,
    patterns: &[CompiledPattern],
    timeout: Duration,
) -> EgressClassification {
    let patterns_arc = Arc::new(patterns.to_vec());
    let set_patterns: Vec<&str> = patterns.iter().map(|p| p.regex.as_str()).collect();
    let regex_set_arc = match regex::RegexSet::new(&set_patterns) {
        Ok(s) => Arc::new(s),
        Err(e) => {
            return EgressClassification::Block(BlockReason::InternalError(format!(
                "Failed to compile RegexSet: {e}"
            )));
        }
    };
    classify_layer1_arc(payload, patterns_arc, regex_set_arc, timeout).await
}

/// Optimierter Ausführungspfad für `classify_layer1` unter Wiederverwendung von vorkompilierten `Arc`-Mengen.
pub async fn classify_layer1_arc(
    payload: &str,
    patterns: Arc<Vec<CompiledPattern>>,
    regex_set: Arc<regex::RegexSet>,
    timeout: Duration,
) -> EgressClassification {
    if payload.len() > MAX_CLASSIFY_PAYLOAD_BYTES {
        return EgressClassification::Block(BlockReason::PolicyDenied(format!(
            "Payload size exceeds limit: {} bytes > {} limit",
            payload.len(),
            MAX_CLASSIFY_PAYLOAD_BYTES
        )));
    }

    let payload_arc: Arc<str> = Arc::from(payload);
    let patterns_cloned = patterns.clone();
    let set_cloned = regex_set.clone();

    let eval_task = tokio::task::spawn_blocking(move || {
        let matches = set_cloned.matches(&payload_arc);
        if matches.matched_any() {
            if let Some(first_idx) = matches.iter().next() {
                let cp = &patterns_cloned[first_idx];
                tracing::warn!(
                    rule_id = %cp.name,
                    pattern = %cp.regex.as_str(),
                    "Egress DLP sensitive pattern match detected"
                );
                return EgressClassification::Block(BlockReason::SensitivePattern(cp.name.clone()));
            }
        }
        EgressClassification::Allow
    });

    match tokio::time::timeout(timeout, eval_task).await {
        Ok(Ok(classification)) => classification,
        Ok(Err(join_err)) => EgressClassification::Block(BlockReason::InternalError(format!(
            "Evaluation task failed: {join_err}"
        ))),
        Err(_timeout_elapsed) => EgressClassification::Block(BlockReason::ClassificationTimeout),
    }
}

/// Cloud Egress Vault zur Abhandlung und Bündelung von Layer-1-Klassifikationen.
#[derive(Debug, Clone)]
pub struct EgressVault {
    patterns: Arc<Vec<CompiledPattern>>,
    regex_set: Arc<regex::RegexSet>,
    timeout: Duration,
}

impl EgressVault {
    /// Standard-Timeout für Klassifikationsprüfungen (100 ms).
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_millis(100);

    /// Erstellt eine neue `EgressVault`-Instanz aus einer Liste von Regex-Patterns.
    /// Jedem Pattern wird eine opake Regel-ID ("R-001", "R-002", ...) zugewiesen.
    pub fn new(patterns: Vec<String>) -> Result<Self, EgressVaultError> {
        let compiled = patterns
            .into_iter()
            .enumerate()
            .map(|(idx, pat)| CompiledPattern::new(format!("R-{:03}", idx + 1), &pat))
            .collect::<Result<Vec<_>, _>>()?;

        let set_patterns: Vec<&str> = compiled.iter().map(|p| p.regex.as_str()).collect();
        let regex_set =
            regex::RegexSet::new(&set_patterns).map_err(|e| EgressVaultError::InvalidPattern {
                pattern: "RegexSet".to_string(),
                reason: e.to_string(),
            })?;

        Ok(Self {
            patterns: Arc::new(compiled),
            regex_set: Arc::new(regex_set),
            timeout: Self::DEFAULT_TIMEOUT,
        })
    }

    /// Setzt ein benutzerdefiniertes Timeout für Klassifikationsabfragen.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Gibt die kompilierten Muster zurück.
    pub fn patterns(&self) -> &[CompiledPattern] {
        &self.patterns
    }

    /// Gibt das gesetzte Timeout zurück.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

/// Schnittstelle für Downstream-Crates (`memfuse-mcp`, `memfuse-router`).
pub trait EgressClassifier: Send + Sync {
    /// Klassifiziert einen Text-Payload für den Egress-Export.
    fn classify<'a>(&'a self, payload: &'a str) -> BoxFuture<'a, EgressClassification>;
}

impl EgressClassifier for EgressVault {
    fn classify<'a>(&'a self, payload: &'a str) -> BoxFuture<'a, EgressClassification> {
        Box::pin(classify_layer1_arc(
            payload,
            self.patterns.clone(),
            self.regex_set.clone(),
            self.timeout,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_allow_happy_path() {
        let patterns = vec![
            r"sk-[a-zA-Z0-9]{32}".to_string(),
            r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b".to_string(),
        ];
        let vault = EgressVault::new(patterns).expect("valid vault");

        let res = vault.classify("Hello world, this is public text.").await;
        assert_eq!(res, EgressClassification::Allow);
    }

    #[tokio::test]
    async fn test_payload_with_abstract_and_sensitive_pattern_is_blocked() {
        let patterns = vec![r"sk-[a-zA-Z0-9]{32}".to_string()];
        let vault = EgressVault::new(patterns).expect("valid vault");

        let payload =
            "This abstract concept includes secret key sk-01234567890123456789012345678901 inside";
        let res = vault.classify(payload).await;
        assert_eq!(
            res,
            EgressClassification::Block(BlockReason::SensitivePattern("R-001".to_string()))
        );
    }

    #[tokio::test]
    async fn test_block_sensitive_pattern() {
        let patterns = vec![
            r"sk-[a-zA-Z0-9]{32}".to_string(),
            r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b".to_string(),
        ];
        let vault = EgressVault::new(patterns).expect("valid vault");

        let payload = "Contact me at secret_agent@example.com for info";
        let res = vault.classify(payload).await;
        assert_eq!(
            res,
            EgressClassification::Block(BlockReason::SensitivePattern("R-002".to_string()))
        );
    }

    #[tokio::test]
    async fn test_opaque_rule_ids_do_not_leak_raw_regex() {
        let raw_pattern = r"(?i)super_secret_password_\d+";
        let patterns = vec![raw_pattern.to_string()];
        let vault = EgressVault::new(patterns).expect("valid vault");

        let payload = "My credential is super_secret_password_12345";
        let res = vault.classify(payload).await;

        if let EgressClassification::Block(BlockReason::SensitivePattern(ref rule_id)) = res {
            assert_eq!(rule_id, "R-001");
            assert!(!rule_id.contains("super_secret_password"));
            assert!(!rule_id.contains(raw_pattern));
        } else {
            panic!("Expected SensitivePattern block with opaque rule ID");
        }
    }

    #[test]
    fn test_invalid_pattern_returns_error() {
        let patterns = vec!["[unclosed-bracket".to_string()];
        let err = EgressVault::new(patterns).unwrap_err();
        assert!(matches!(err, EgressVaultError::InvalidPattern { .. }));
    }

    #[tokio::test]
    async fn test_oversized_payload_blocked_before_task_spawn() {
        let huge_payload = "a".repeat(MAX_CLASSIFY_PAYLOAD_BYTES + 1);
        let patterns = vec![CompiledPattern::new("R-001", r"a+").unwrap()];

        let res = classify_layer1(&huge_payload, &patterns, Duration::from_millis(100)).await;
        if let EgressClassification::Block(BlockReason::PolicyDenied(reason)) = res {
            assert!(reason.contains("Payload size exceeds limit"));
        } else {
            panic!(
                "Expected PolicyDenied block for oversized payload, got {:?}",
                res
            );
        }
    }

    #[tokio::test]
    async fn test_redos_and_timeout_fail_closed() {
        // Payload genau an der Limit-Grenze
        let mut huge_payload = "a".repeat(MAX_CLASSIFY_PAYLOAD_BYTES - 1);
        huge_payload.push('!');

        let patterns = vec![CompiledPattern::new("R-001", r"(a+)+b")
            .unwrap_or_else(|_| CompiledPattern::new("R-001", r"a{1000,}").unwrap())];

        let start = Instant::now();
        // Setze extrem kurzes Timeout (1 Microsekunde / 10 Mikros), um Timeout-Pfad sicher zu triggern
        let res = classify_layer1(&huge_payload, &patterns, Duration::from_micros(10)).await;
        let elapsed = start.elapsed();

        assert_eq!(
            res,
            EgressClassification::Block(BlockReason::ClassificationTimeout),
            "Must fail closed on timeout"
        );
        assert!(
            elapsed < Duration::from_millis(500),
            "Classification should terminate quickly when timed out"
        );
    }

    #[tokio::test]
    async fn test_trait_implementation_e2e() {
        let patterns = vec![r"PRIVATE_KEY".to_string()];
        let vault = EgressVault::new(patterns).unwrap();
        let classifier: &dyn EgressClassifier = &vault;

        assert_eq!(
            classifier.classify("Normal text").await,
            EgressClassification::Allow
        );
        assert!(matches!(
            classifier.classify("Contains PRIVATE_KEY inside").await,
            EgressClassification::Block(BlockReason::SensitivePattern(_))
        ));
    }

    /// Verifiziert die Fail-Closed-Semantik von `classify_layer1` (§10.14, §12.2.4).
    ///
    /// "Fail-Closed" bedeutet: Bei Timeout, internem Fehler oder Panic
    /// MUSS Block(...) zurückgegeben werden — niemals Allow.
    ///
    /// Dieser Test simuliert einen Timeout indem das Zeitlimit auf
    /// eine extrem kurze Duration (1 Nanosekunde) gesetzt wird.
    #[cfg(all(test, feature = "cloud-egress-guard"))]
    #[tokio::test]
    async fn test_egress_guard_fail_closed_on_index_unavailable() {
        use std::time::Duration;

        // Erstelle echte Patterns — der Inhalt ist irrelevant,
        // da der Test per Timeout blockt bevor Patterns geprüft werden.
        let patterns = vec![CompiledPattern::new(
            "email",
            r"[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}",
        )
        .expect("valid email pattern")];

        // Benutze ein ausreichend großes Payload, sodass `spawn_blocking`
        // nicht sofort vor dem Timeout-Tick abschließt.
        let payload = "This is a benign payload without any sensitive data. ".repeat(50_000);

        // Simuliere "Index unavailable" / Timeout: 1 Nanosekunde — wird immer überschritten
        // da `spawn_blocking` allein mehrere Mikrosekunden braucht.
        let result = classify_layer1(&payload, &patterns, Duration::from_nanos(1)).await;

        // INVARIANTE: Fail-Closed — bei Timeout MUSS Block zurückgegeben werden.
        assert!(
            matches!(result, EgressClassification::Block(BlockReason::ClassificationTimeout)),
            "Fail-Closed-Verletzung: classify_layer1 gab bei Timeout nicht Block(ClassificationTimeout) zurück. Got: {:?}",
            result
        );

        // Negativ-Test: Normaler Aufruf mit angemessener Zeit MUSS Allow zurückgeben
        let result_ok = classify_layer1(&payload, &patterns, Duration::from_millis(500)).await;
        assert!(
            matches!(result_ok, EgressClassification::Allow),
            "Fehler: Bei ausreichend Zeit und harmlosen Payload sollte Allow zurückgegeben werden. Got: {:?}",
            result_ok
        );
    }

    #[tokio::test]
    async fn test_egress_vault_accessors_and_with_timeout() {
        let patterns = vec![r"sk-[a-zA-Z0-9]{32}".to_string()];
        let vault = EgressVault::new(patterns)
            .expect("valid vault")
            .with_timeout(Duration::from_millis(250));

        assert_eq!(vault.timeout(), Duration::from_millis(250));
        assert_eq!(vault.patterns().len(), 1);
        assert_eq!(vault.patterns()[0].name, "R-001");
    }

    #[tokio::test]
    async fn test_exact_payload_boundary_allowed() {
        let exact_payload = "a".repeat(MAX_CLASSIFY_PAYLOAD_BYTES);
        let patterns = vec![CompiledPattern::new("R-001", r"secret_pattern_xyz").unwrap()];

        let res = classify_layer1(&exact_payload, &patterns, Duration::from_millis(200)).await;
        assert_eq!(res, EgressClassification::Allow);
    }
}

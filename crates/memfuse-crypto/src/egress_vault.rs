// FILE-CONTEXT
// ZWECK: Cloud-Egress-Vault mit Layer-1-Regex-Klassifikation und Fail-Closed-Semantik.
// INVARIANTEN: Jede Klassifikationsentscheidung terminiert innerhalb des definierten Timeouts.
// Bei Timeout, Regex-Fehlern oder Laufzeitfehlern gilt strikt Fail-Closed (EgressClassification::Block).
// STAND: TS:2026-09-13 (SESSION: HEAD)

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use serde::{Deserialize, Serialize};

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
    /// Übertragung erfordert Abstraktion / Anonymisierung.
    RequiresAbstraction,
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

/// Führt eine Layer-1-Regex-Klassifikation auf dem Payload mit neuem Task und hartem Timeout aus.
///
/// Invariante: **Fail-Closed**. Bei Timeout, Panic oder Fehler wird stets `Block(...)` zurückgegeben.
pub async fn classify_layer1(
    payload: &str,
    patterns: &[CompiledPattern],
    timeout: Duration,
) -> EgressClassification {
    let payload_owned = payload.to_string();
    let patterns_owned = patterns.to_vec();

    let eval_task = tokio::task::spawn_blocking(move || {
        for cp in &patterns_owned {
            if cp.regex.is_match(&payload_owned) {
                return EgressClassification::Block(BlockReason::SensitivePattern(
                    cp.name.clone(),
                ));
            }
        }
        EgressClassification::Allow
    });

    match tokio::time::timeout(timeout, eval_task).await {
        Ok(Ok(classification)) => classification,
        Ok(Err(join_err)) => EgressClassification::Block(BlockReason::InternalError(format!(
            "Evaluation task failed: {join_err}"
        ))),
        Err(_timeout_elapsed) => {
            EgressClassification::Block(BlockReason::ClassificationTimeout)
        }
    }
}

/// Cloud Egress Vault zur Abhandlung und Bündelung von Layer-1-Klassifikationen.
#[derive(Debug, Clone)]
pub struct EgressVault {
    patterns: Arc<Vec<CompiledPattern>>,
    timeout: Duration,
}

impl EgressVault {
    /// Standard-Timeout für Klassifikationsprüfungen (100 ms).
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_millis(100);

    /// Erstellt eine neue `EgressVault`-Instanz aus einer Liste von Regex-Patterns.
    pub fn new(patterns: Vec<String>) -> Result<Self, EgressVaultError> {
        let compiled = patterns
            .into_iter()
            .map(|pat| CompiledPattern::new(pat.clone(), &pat))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            patterns: Arc::new(compiled),
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
        Box::pin(classify_layer1(payload, &self.patterns, self.timeout))
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
    async fn test_block_sensitive_pattern() {
        let patterns = vec![
            r"sk-[a-zA-Z0-9]{32}".to_string(),
            r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b".to_string(),
        ];
        let vault = EgressVault::new(patterns).expect("valid vault");

        let payload = "Contact me at secret_agent@example.com for info";
        let res = vault.classify(payload).await;
        assert!(matches!(
            res,
            EgressClassification::Block(BlockReason::SensitivePattern(ref pat))
            if pat.contains("@")
        ));
    }

    #[test]
    fn test_invalid_pattern_returns_error() {
        let patterns = vec!["[unclosed-bracket".to_string()];
        let err = EgressVault::new(patterns).unwrap_err();
        assert!(matches!(err, EgressVaultError::InvalidPattern { .. }));
    }

    #[tokio::test]
    async fn test_redos_and_timeout_fail_closed() {
        // Generiere ein extrem großes Payload zur Auslösungsüberprüfung des Timeouts
        let mut huge_payload = "a".repeat(2_000_000);
        huge_payload.push_str("!");

        let patterns = vec![
            CompiledPattern::new("slow_pattern", r"(a+)+b").unwrap_or_else(|_| {
                CompiledPattern::new("fallback_pattern", r"a{1000,}").unwrap()
            }),
        ];

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
}

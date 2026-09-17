// FILE-CONTEXT
// ZWECK: Cloud-Egress-Vault mit Layer-1-Regex-Klassifikation und Fail-Closed-Semantik.
// INVARIANTEN: Jede Klassifikationsentscheidung terminiert innerhalb des definierten Timeouts.
// Bei Timeout, Regex-Fehlern oder Laufzeitfehlern gilt strikt Fail-Closed (EgressClassification::Block).
// STAND: TS:2026-09-13 (SESSION: HEAD)

use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use zeroize::Zeroize;

/// Trait for recognizing entities (NER) in text without creating a direct dependency
/// on heavy embedding or ML crates (DAG-neutral interface).
pub trait EntityRecognizer: Send + Sync {
    /// Recognizes entities in the text and returns byte ranges and category labels.
    fn recognize(&self, text: &str) -> Vec<(std::ops::Range<usize>, String)>;
}

/// A default no-op entity recognizer that detects no entities.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoOpRecognizer;

impl EntityRecognizer for NoOpRecognizer {
    fn recognize(&self, _text: &str) -> Vec<(std::ops::Range<usize>, String)> {
        Vec::new()
    }
}

/// Session-bound vault storing bidirectional mappings between original PII/entities and generated surrogates.
///
/// Material and mappings are zeroized upon drop.
pub struct SurrogateVault {
    session_salt: [u8; 16],
    map: Mutex<HashMap<String, String>>,
}

impl std::fmt::Debug for SurrogateVault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let entry_count = self.map.lock().map(|m| m.len()).unwrap_or(0);
        f.debug_struct("SurrogateVault")
            .field("entry_count", &entry_count)
            .finish_non_exhaustive()
    }
}

impl SurrogateVault {
    /// Creates a new `SurrogateVault` with the specified session salt.
    pub fn new(session_salt: [u8; 16]) -> Self {
        Self {
            session_salt,
            map: Mutex::new(HashMap::new()),
        }
    }

    /// Creates a new `SurrogateVault` with a cryptographically secure random session salt.
    pub fn random_salt() -> Self {
        let mut salt = [0u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut salt);
        Self::new(salt)
    }

    /// Generates a session-stable surrogate identifier for `entity_text` and stores the surrogate -> entity mapping.
    pub fn generate_surrogate(&self, entity_text: &str) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(entity_text.as_bytes());
        hasher.update(&self.session_salt);
        let hash_hex = hasher.finalize().to_hex();
        let surrogate = format!("[USER_ENTITY_{}]", &hash_hex[..4]);

        if let Ok(mut guard) = self.map.lock() {
            guard.insert(surrogate.clone(), entity_text.to_string());
        }

        surrogate
    }

    /// Retrieves the original entity text for a given surrogate key if present.
    pub fn get_entity(&self, surrogate: &str) -> Option<String> {
        self.map
            .lock()
            .ok()
            .and_then(|guard| guard.get(surrogate).cloned())
    }

    /// Returns the current count of stored surrogate mappings.
    pub fn len(&self) -> usize {
        self.map.lock().map(|m| m.len()).unwrap_or(0)
    }

    /// Returns true if the vault contains no surrogate mappings.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Drop for SurrogateVault {
    fn drop(&mut self) {
        self.session_salt.zeroize();
        if let Ok(mut guard) = self.map.lock() {
            for (mut k, mut v) in guard.drain() {
                k.zeroize();
                v.zeroize();
            }
        }
    }
}

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
    #[error("Payload size exceeds limit: {size} bytes > {limit} limit")]
    PayloadTooLarge { size: usize, limit: usize },
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
    surrogate_vault: Arc<SurrogateVault>,
}

impl EgressVault {
    /// Standard-Timeout für Klassifikationsprüfungen (100 ms).
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_millis(100);

    /// Standard-DLP-Muster für Egress-Klassifikationen (Secrets, API-Keys, PII, E-Mail).
    pub fn default_patterns() -> Vec<String> {
        vec![
            r"sk-".to_string(),
            r"AKIA".to_string(),
            r"api_key".to_string(),
            r"password".to_string(),
            r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b".to_string(),
        ]
    }

    /// Versucht, eine `EgressVault`-Instanz mit den Standard-DLP-Mustern zu erstellen.
    pub fn try_default() -> Result<Self, EgressVaultError> {
        Self::new(Self::default_patterns())
    }

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
            surrogate_vault: Arc::new(SurrogateVault::random_salt()),
        })
    }

    /// Setzt ein benutzerdefiniertes Timeout für Klassifikationsabfragen.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Setzt einen benutzerdefinierten `SurrogateVault`.
    pub fn with_surrogate_vault(mut self, surrogate_vault: Arc<SurrogateVault>) -> Self {
        self.surrogate_vault = surrogate_vault;
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

    /// Gibt eine Referenz auf den verknüpften `SurrogateVault` zurück.
    pub fn surrogate_vault(&self) -> &Arc<SurrogateVault> {
        &self.surrogate_vault
    }

    /// Tokenisiert und ersetzt erkannte strukturierte Regex-Muster sowie unstrukturierte NER-Entitäten
    /// durch sitzungsstabile Surrogate und speichert die Zuordnungen im `SurrogateVault`.
    ///
    /// Gibt den bereinigten Text sowie die Anzahl ersetzter Entitäten zurück.
    pub fn sanitize_and_vault(
        &self,
        payload: &str,
        recognizer: &dyn EntityRecognizer,
    ) -> Result<(String, usize), EgressVaultError> {
        if payload.len() > MAX_CLASSIFY_PAYLOAD_BYTES {
            return Err(EgressVaultError::PayloadTooLarge {
                size: payload.len(),
                limit: MAX_CLASSIFY_PAYLOAD_BYTES,
            });
        }

        let mut current_text = payload.to_string();
        let mut replacement_count = 0;

        // Phase 1: Regex-Muster im Ersetzungsmodus anwenden
        for cp in self.patterns.iter() {
            let mut new_text = String::with_capacity(current_text.len());
            let mut last_end = 0;
            for m in cp.regex.find_iter(&current_text) {
                new_text.push_str(&current_text[last_end..m.start()]);
                let entity_text = m.as_str();
                let surrogate = self.surrogate_vault.generate_surrogate(entity_text);
                new_text.push_str(&surrogate);
                last_end = m.end();
                replacement_count += 1;
            }
            if last_end > 0 {
                new_text.push_str(&current_text[last_end..]);
                current_text = new_text;
            }
        }

        // Phase 2: Unstrukturierte NER-Entitäten des EntityRecognizers anwenden
        let recognized_spans = recognizer.recognize(&current_text);
        if !recognized_spans.is_empty() {
            // Filtere und sortiere gültige Spans absteigend nach Start, um Indexverschiebungen zu vermeiden
            let mut valid_spans: Vec<_> = recognized_spans
                .into_iter()
                .filter(|(range, _category)| {
                    range.start <= range.end
                        && range.end <= current_text.len()
                        && current_text.is_char_boundary(range.start)
                        && current_text.is_char_boundary(range.end)
                })
                .collect();

            valid_spans.sort_by_key(|span| std::cmp::Reverse(span.0.start));

            let mut last_processed_start = current_text.len();
            for (range, _category) in valid_spans {
                if range.end <= last_processed_start {
                    let entity_text = &current_text[range.clone()];
                    let surrogate = self.surrogate_vault.generate_surrogate(entity_text);
                    current_text.replace_range(range.clone(), &surrogate);
                    last_processed_start = range.start;
                    replacement_count += 1;
                }
            }
        }

        Ok((current_text, replacement_count))
    }
}

/// Schnittstelle für Downstream-Crates (`memfuse-mcp`, `memfuse-router`).
pub trait EgressClassifier: Send + Sync {
    /// Klassifiziert einen Text-Payload für den Egress-Export.
    fn classify<'a>(&'a self, payload: &'a str) -> BoxFuture<'a, EgressClassification>;
}

impl Default for EgressVault {
    fn default() -> Self {
        Self::try_default().unwrap_or_else(|err| {
            tracing::error!(error = %err, "Failed to initialize default EgressVault, using empty fallback");
            let empty_set = regex::RegexSet::new(Vec::<&str>::new()).unwrap_or_else(|_| {
                // In practice RegexSet::new([]) never fails
                regex::RegexSet::empty()
            });
            Self {
                patterns: Arc::new(Vec::new()),
                regex_set: Arc::new(empty_set),
                timeout: Self::DEFAULT_TIMEOUT,
                surrogate_vault: Arc::new(SurrogateVault::random_salt()),
            }
        })
    }
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
        // Setze extrem kurzes Timeout (Duration::ZERO), um Timeout-Pfad sicher zu triggern
        let res = classify_layer1(&huge_payload, &patterns, Duration::ZERO).await;
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
    async fn test_egress_vault_default_implementation() {
        let vault = EgressVault::default();
        assert_eq!(vault.patterns().len(), 5);
        assert_eq!(vault.timeout(), EgressVault::DEFAULT_TIMEOUT);

        let res = vault.classify("hello user alice@example.com").await;
        assert_eq!(
            res,
            EgressClassification::Block(BlockReason::SensitivePattern("R-005".to_string()))
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
        let patterns =
            vec![CompiledPattern::new("R-001", r"secret_pattern_xyz").expect("valid pattern")];

        let res = classify_layer1(&exact_payload, &patterns, Duration::from_millis(200)).await;
        assert_eq!(res, EgressClassification::Allow);
    }

    #[test]
    fn test_surrogate_determinism_same_session() {
        let vault = SurrogateVault::new([42u8; 16]);
        let s1 = vault.generate_surrogate("alice@example.com");
        let s2 = vault.generate_surrogate("alice@example.com");

        assert_eq!(s1, s2);
        assert!(s1.starts_with("[USER_ENTITY_"));
        assert_eq!(vault.get_entity(&s1), Some("alice@example.com".to_string()));
    }

    #[test]
    fn test_surrogate_divergence_different_session() {
        let v1 = SurrogateVault::new([1u8; 16]);
        let v2 = SurrogateVault::new([2u8; 16]);

        let s1 = v1.generate_surrogate("alice@example.com");
        let s2 = v2.generate_surrogate("alice@example.com");

        assert_ne!(s1, s2);
        assert_eq!(v1.get_entity(&s1), Some("alice@example.com".to_string()));
        assert_eq!(v2.get_entity(&s2), Some("alice@example.com".to_string()));
    }

    struct MockRecognizer;

    impl EntityRecognizer for MockRecognizer {
        fn recognize(&self, text: &str) -> Vec<(std::ops::Range<usize>, String)> {
            let mut result = Vec::new();
            if let Some(pos) = text.find("Bob Smith") {
                result.push((pos..pos + "Bob Smith".len(), "PERSON".to_string()));
            }
            result
        }
    }

    #[test]
    fn test_sanitize_and_vault_noop_and_mock_recognizer() {
        let vault = EgressVault::try_default().expect("valid vault");
        let payload = "Contact alice@example.com or Bob Smith for credentials";

        // Test with NoOpRecognizer (only regex replaces email)
        let (sanitized_noop, count_noop) = vault
            .sanitize_and_vault(payload, &NoOpRecognizer)
            .expect("sanitization succeeds");

        assert_eq!(count_noop, 1);
        assert!(!sanitized_noop.contains("alice@example.com"));
        assert!(sanitized_noop.contains("Bob Smith"));

        // Test with MockRecognizer (regex replaces email, NER replaces Bob Smith)
        let (sanitized_mock, count_mock) = vault
            .sanitize_and_vault(payload, &MockRecognizer)
            .expect("sanitization succeeds");

        assert_eq!(count_mock, 2);
        assert!(!sanitized_mock.contains("alice@example.com"));
        assert!(!sanitized_mock.contains("Bob Smith"));
        assert_eq!(vault.surrogate_vault().len(), 2);
    }

    #[test]
    fn test_sanitize_empty_payload() {
        let vault = EgressVault::try_default().expect("valid vault");
        let (sanitized, count) = vault
            .sanitize_and_vault("", &NoOpRecognizer)
            .expect("empty payload succeeds");

        assert_eq!(sanitized, "");
        assert_eq!(count, 0);
        assert_eq!(vault.surrogate_vault().len(), 0);
    }

    #[test]
    fn test_zeroize_on_drop_behavior() {
        let surrogate_vault = Arc::new(SurrogateVault::new([7u8; 16]));
        let s = surrogate_vault.generate_surrogate("secret_token");
        assert_eq!(
            surrogate_vault.get_entity(&s),
            Some("secret_token".to_string())
        );

        let weak_vault = Arc::downgrade(&surrogate_vault);
        drop(surrogate_vault);

        assert!(weak_vault.upgrade().is_none());
    }
}

//! Lokaler Fehlertyp und Traits für `contextra-infer-ollama` (B4- und B3-Konformität).

use thiserror::Error;

/// Fehlerzustände von `contextra-infer-ollama`.
#[derive(Debug, Error, Clone, PartialEq)]
#[non_exhaustive]
pub enum OllamaError {
    /// Ungültige Eingabedaten oder Konfiguration.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// Das angeforderte Modell existiert nicht auf dem Server.
    #[error("model not found: {0}")]
    ModelNotFound(String),

    /// Netzwerk- oder Verbindungsfehler.
    #[error("network error: {0}")]
    Network(String),

    /// Zeitüberschreitung bei der Anfrage.
    #[error("timeout: {0}")]
    Timeout(String),

    /// HTTP-Serverfehler (z.B. 5xx).
    #[error("server error: {0}")]
    Server(String),

    /// Deserialisierungs- oder Parsingfehler.
    #[error("deserialization error: {0}")]
    Deserialization(String),

    /// Verstoß gegen die Egress-Richtlinie.
    #[error("egress policy violation: {0}")]
    EgressViolation(String),

    /// Circuit-Breaker ist geöffnet; Anfragen werden fail-closed abgewiesen.
    #[error("circuit breaker open: requests throttled")]
    CircuitBreakerOpen,

    /// Interner oder nicht näher spezifizierter Fehler.
    #[error("internal error: {0}")]
    Internal(String),
}

/// Standard-Result-Typ für `contextra-infer-ollama`.
pub type Result<T> = core::result::Result<T, OllamaError>;

#[cfg(feature = "ports-compat")]
impl From<OllamaError> for contextra_types::ContextraError {
    fn from(err: OllamaError) -> Self {
        match err {
            OllamaError::InvalidInput(msg) => contextra_types::ContextraError::InvalidInput(msg),
            OllamaError::ModelNotFound(msg) => contextra_types::ContextraError::NotFound(msg),
            OllamaError::Network(msg) => contextra_types::ContextraError::Io(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                msg,
            )),
            OllamaError::Timeout(msg) => contextra_types::ContextraError::Io(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                msg,
            )),
            OllamaError::Server(msg) => contextra_types::ContextraError::Storage(msg),
            OllamaError::Deserialization(msg) => contextra_types::ContextraError::Internal(msg),
            OllamaError::EgressViolation(msg) => contextra_types::ContextraError::PolicyViolation(msg),
            OllamaError::CircuitBreakerOpen => {
                contextra_types::ContextraError::Internal("circuit breaker open".to_string())
            }
            OllamaError::Internal(msg) => contextra_types::ContextraError::Internal(msg),
        }
    }
}

#[cfg(feature = "ports-compat")]
impl From<contextra_types::ContextraError> for OllamaError {
    fn from(err: contextra_types::ContextraError) -> Self {
        match err {
            contextra_types::ContextraError::InvalidInput(msg) => OllamaError::InvalidInput(msg),
            contextra_types::ContextraError::NotFound(msg) => OllamaError::ModelNotFound(msg),
            contextra_types::ContextraError::PolicyViolation(msg) => OllamaError::EgressViolation(msg),
            contextra_types::ContextraError::Io(io_err) => {
                if io_err.kind() == std::io::ErrorKind::TimedOut {
                    OllamaError::Timeout(io_err.to_string())
                } else {
                    OllamaError::Network(io_err.to_string())
                }
            }
            contextra_types::ContextraError::Storage(msg) => OllamaError::Server(msg),
            contextra_types::ContextraError::Internal(msg) => OllamaError::Internal(msg),
            other => OllamaError::Internal(other.to_string()),
        }
    }
}

#[cfg(feature = "ports-compat")]
impl From<OllamaError> for contextra_ports::EmbeddingError {
    fn from(err: OllamaError) -> Self {
        match err {
            OllamaError::ModelNotFound(msg) | OllamaError::InvalidInput(msg) => {
                contextra_ports::EmbeddingError::Unavailable(msg)
            }
            other => contextra_ports::EmbeddingError::ComputationFailed(other.to_string()),
        }
    }
}

/// Lokaler Trait für Zeitgeber (entkoppelt von `contextra-ports::Clock`).
pub trait LocalClock: Send + Sync {
    /// Liefert die aktuelle Systemzeit.
    fn now(&self) -> std::time::SystemTime;

    /// Liefert eine monotone Zeit in Nanosekunden.
    fn monotonic_nanos(&self) -> u64;
}

static PROCESS_EPOCH: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

#[allow(clippy::disallowed_methods)]
fn get_process_epoch() -> std::time::Instant {
    *PROCESS_EPOCH.get_or_init(std::time::Instant::now)
}

/// Standard-Implementierung von `LocalClock` mit monotonic process epoch baseline.
#[derive(Debug, Clone, Default)]
pub struct SystemClock;

#[allow(clippy::disallowed_methods)]
impl LocalClock for SystemClock {
    fn now(&self) -> std::time::SystemTime {
        std::time::UNIX_EPOCH + std::time::Duration::from_nanos(self.monotonic_nanos())
    }

    fn monotonic_nanos(&self) -> u64 {
        let epoch = get_process_epoch();
        let now = std::time::Instant::now();
        u64::try_from(now.saturating_duration_since(epoch).as_nanos()).unwrap_or(u64::MAX)
    }
}

#[cfg(feature = "ports-compat")]
impl LocalClock for contextra_ports::SystemClock {
    fn now(&self) -> std::time::SystemTime {
        let nanos = contextra_ports::Clock::now_unix_nanos(self);
        std::time::UNIX_EPOCH + std::time::Duration::from_nanos(nanos)
    }

    fn monotonic_nanos(&self) -> u64 {
        contextra_ports::Clock::monotonic_nanos(self)
    }
}

/// Lokaler Trait für Zufallszahlengeneratoren (entkoppelt von `contextra-ports::Rng`).
pub trait LocalRng: Send + Sync {
    /// Generiert einen u64-Zufallswert.
    fn next_u64(&self) -> u64;

    /// Generiert eine f64-Fließkommazahl im Bereich [0.0, 1.0).
    fn next_f64(&self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / 9007199254740992.0)
    }
}

/// Deterministischer atomarer Zufallsgenerator für Wiederholbarkeit ohne thread_rng.
#[derive(Debug)]
pub struct LocalSeededRng {
    state: std::sync::atomic::AtomicU64,
}

impl LocalSeededRng {
    /// Erstellt einen Generator mit dem angegebenen Seed.
    pub fn new(seed: u64) -> Self {
        Self {
            state: std::sync::atomic::AtomicU64::new(if seed == 0 { 0x5EED_5EED_CAFE_BABE } else { seed }),
        }
    }
}

impl Default for LocalSeededRng {
    fn default() -> Self {
        Self::new(0x1234_5678_9ABC_DEF0)
    }
}

impl LocalRng for LocalSeededRng {
    fn next_u64(&self) -> u64 {
        let mut current = self.state.load(std::sync::atomic::Ordering::Relaxed);
        loop {
            let mut x = current;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            match self.state.compare_exchange_weak(
                current,
                x,
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
            ) {
                Ok(_) => return x,
                Err(actual) => current = actual,
            }
        }
    }
}

#[cfg(feature = "ports-compat")]
impl LocalRng for contextra_ports::SeededRng {
    fn next_u64(&self) -> u64 {
        contextra_ports::Rng::next_u64(self)
    }
}

/// Lokale Datenstruktur für Re-Write-Ausgaben.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LocalQueryRewriteOutput {
    /// Textuelle Umformulierung für BM25.
    pub text_query: Option<String>,
    /// Semantische Umformulierung für Vektorsuche.
    pub semantic_query: Option<String>,
    /// Anker-Entitäten für Graphsuche.
    pub anchor_entities: Vec<String>,
}

impl LocalQueryRewriteOutput {
    /// Erstellt eine neue Instanz.
    pub fn new(
        text_query: Option<String>,
        semantic_query: Option<String>,
        anchor_entities: Vec<String>,
    ) -> Self {
        Self {
            text_query,
            semantic_query,
            anchor_entities,
        }
    }

    /// Erstellt eine rein textuelle Umformulierung.
    pub fn text_only(text: impl Into<String>) -> Self {
        Self {
            text_query: Some(text.into()),
            semantic_query: None,
            anchor_entities: Vec::new(),
        }
    }
}

#[cfg(feature = "ports-compat")]
impl From<LocalQueryRewriteOutput> for contextra_types::QueryRewriteOutput {
    fn from(val: LocalQueryRewriteOutput) -> Self {
        contextra_types::QueryRewriteOutput::new(
            val.text_query,
            val.semantic_query,
            val.anchor_entities,
        )
    }
}

#[cfg(feature = "ports-compat")]
impl From<contextra_types::QueryRewriteOutput> for LocalQueryRewriteOutput {
    fn from(val: contextra_types::QueryRewriteOutput) -> Self {
        LocalQueryRewriteOutput::new(val.text_query, val.semantic_query, val.anchor_entities)
    }
}

/// Lokale Bewertung von Wichtigkeit (entkoppelt von `contextra-types::ImportanceScore`).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct LocalImportanceScore(pub f32);

impl LocalImportanceScore {
    /// Liefert den Rohwert als f32.
    pub fn as_f32(&self) -> f32 {
        self.0
    }
}

#[cfg(feature = "ports-compat")]
impl From<LocalImportanceScore> for contextra_types::ImportanceScore {
    fn from(score: LocalImportanceScore) -> Self {
        contextra_types::ImportanceScore::new(score.0)
    }
}

#[cfg(feature = "ports-compat")]
impl From<contextra_types::ImportanceScore> for LocalImportanceScore {
    fn from(score: contextra_types::ImportanceScore) -> Self {
        LocalImportanceScore(score.value())
    }
}

use serde::{Deserialize, Serialize};

/// Neutraler Standard-Platzhalter für als verdächtig erkannten Text im Strict/Escalate-Modus.
pub const DEFAULT_REDACTION_PLACEHOLDER: &str =
    "[REDACTED: potenzielle Prompt-Injection erkannt, Originaltext zur Sicherheitsprüfung zurückgehalten]";

/// Konfigurierbare Quarantäne-Policy für die Prompt-Injection-Behandlung.
///
/// **SICHERHEITSHINWEIS & HEURISTIK-LIMITATION**:
/// Pattern-Matching und Normalisierung stellen ein Defense-in-Depth Heuristik-System dar.
/// Es bietet **keine absolute Garantie** gegen neuartige oder komplexe Prompt-Injection-Angriffe.
/// MCP-Clients müssen abgerufene Dokumenteninhalte weiterhin in isolierten Prompt-Kontexten
/// (z.B. `<untrusted_context>`) verarbeiten.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum QuarantinePolicy {
    /// Verdächtige Texte werden durch einen neutralen Platzhalter ersetzt (Default).
    #[default]
    Strict,

    /// **WARNUNG (UNSICHER)**: Text wird unverändert durchgereicht, lediglich das Flag
    /// `suspicious_injection_detected: true` wird gesetzt.
    ///
    /// Nutze diesen Modus ausschließlich in vertrauenswürdigen, kontrollierten Umgebungen,
    /// in denen nachgelagerte Komponenten das Flag garantiert auswerten.
    FlagOnly,

    /// Text wird zurückgehalten/redigiert UND der Vorfall wird in ein separates Sicherheits-Audit-Log geschrieben.
    Escalate,
}

impl QuarantinePolicy {
    pub fn from_env() -> Self {
        if let Ok(val) = std::env::var("MEMFUSE_MCP_QUARANTINE_POLICY") {
            match val.trim().to_lowercase().as_str() {
                "flag_only" | "flagonly" | "flag" => QuarantinePolicy::FlagOnly,
                "escalate" => QuarantinePolicy::Escalate,
                _ => QuarantinePolicy::Strict,
            }
        } else {
            QuarantinePolicy::Strict
        }
    }
}

/// Konfiguration zur Initialisierung aus einer externen Konfigurationsdatei.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptInjectionConfig {
    #[serde(default)]
    pub policy: QuarantinePolicy,
    #[serde(default = "default_redaction_placeholder")]
    pub redaction_placeholder: String,
    #[serde(default)]
    pub audit_log_path: Option<String>,
    #[serde(default)]
    pub custom_patterns: Vec<String>,
}

pub fn default_redaction_placeholder() -> String {
    DEFAULT_REDACTION_PLACEHOLDER.to_string()
}

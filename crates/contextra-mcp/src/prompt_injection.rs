// FILE-CONTEXT
// STAND:       2026-09-10T19:25:24Z (SESSION: ae8c2fb9)
// ZWECK:       Prompt-Injection-Erkennung & Quarantäne-System für MCP-Server
// INVARIANTEN: Standardmäßig werden verdächtige Texte redigiert (strict); Audit-Logs in escalate-Mode sind isoliert vom Vektor-Index.

//! # LÜCKENANALYSE & DEFENSE-IN-DEPTH ARCHITEKTUR
//!
//! ## System-Charakterisierung & Erkennungsgrenzen:
//! Der Prompt-Injection-Guard bietet eine **signatur-/phrasenbasierte Erkennung bekannter Angriffsmuster**
//! inkl. gängiger Verschleierungstechniken (Zero-Width-Stripping, NFKC, Base64-Rekursion bis Tiefe 2);
//! er bietet **keinen Schutz gegen Umformulierungen oder nicht-englische Angriffsphrasen**.
//!
//! ## Status der Abdeckung gängiger MCP/Tool-Injection-Muster:
//!
//! 1. **Direkte Instruktions-Injektion in Tool-Rückgabewerten**:
//!    - **Status**: ABGEDECKT (Signatur-basiert).
//!    - **Details**: Standardmuster wie `"ignore previous instructions"`, `"override previous instructions"`,
//!      `"disregard previous instructions"`, `"system prompt:"`, `"you are now in developer mode"` etc.
//!      werden zuverlässig über Case-Insensitive Pattern Matching auf vornormalisierten Phrasen erkannt.
//!
//! 2. **Rollen-Verwirrung durch gefälschte System-/Assistant-Markierungen**:
//!    - **Status**: ABGEDECKT (Signatur-basiert).
//!    - **Details**: Spezifische Chat-Format-Tokens wie `[INST]`, `[/INST]`, `<|im_start|>`, `<|im_end|>`,
//!      `<|system|>`, `<|user|>`, `<|assistant|>`, `<<SYS>>`, `<</SYS>>` sind in den Standardmustern enthalten.
//!
//! 3. **Verschachtelte/kodierte Payloads (Base64, Unicode-Homoglyphen, Zero-Width-Zeichen)**:
//!    - **Status**: ERWEITERT / ABGEDECKT.
//!    - **Details**:
//!      - *Unicode-Homoglyphen*: NFKC-Normalisierung vor der Erkennung wandelt Kompatibilitätszeichen
//!        und Vollbreiten-Konzepte in Standard-Formate um.
//!      - *Zero-Width-Zeichen*: Unsichtbare Steuer- und Steuerbereichs-Zeichen (`\u{200B}`, `\u{200C}`, `\u{FEFF}` etc.)
//!        werden vor dem Matching explizit herausgefiltert.
//!      - *Base64-Payloads*: Verdächtige Base64-Substrings werden extrahiert, dekodiert und rekursiv
//!        (bis max. Tiefe 2 für DoS-Schutz) gescannt.
//!
//! 4. **Mehrstufige "Sleeper"-Injektionen & Freitext-Angriffe**:
//!    - **Status**: TEILWEISE / NICHT DYNAMISCH ABGEDECKT.
//!    - **Details**: Einzelne Tool-Outputs mit Sleeper-Triggern werden statisch bei der Rückgabe gescannt.
//!      Gezielte zustandsbehaftete, über mehrere Tool-Aufrufe hinweg verteilte Injektionen sowie semantisch
//!      umformulierte Angriffe erfordern Modell-basierte Klassifikatoren und Kontext-Tracking auf Agenten-Session-Ebene.
//!
//! ## ARCHITEKTUR-HINWEIS & VERTEIDIGUNGSLINIEN:
//! Die Sandbox-Isolation (`sandbox.rs`) bleibt die unentbehrliche **zweite Verteidigungslinie**
//! (Zero-Trust Tool Isolation, Volatile Memory Encryption, Permission Policies).
//! Dieser Prompt-Injection-Guard ergänzt die Sandbox als Inhaltsfilter auf Transport- / DTO-Ebene,
//! **ersetzt sie jedoch ausdrücklich nicht**.

mod audit;
mod guard;
mod policy;

#[cfg(test)]
mod tests;

pub use audit::{SecurityAuditLogger, SecurityAuditRecord};
pub use guard::{NormalizedPattern, PromptInjectionGuard};
pub use policy::{PromptInjectionConfig, QuarantinePolicy, DEFAULT_REDACTION_PLACEHOLDER};

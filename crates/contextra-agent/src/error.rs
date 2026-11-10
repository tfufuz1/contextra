//! Lokaler Fehlertyp für contextra-agent (SPEC-v2 §6.3).

use thiserror::Error;

/// Lokaler Fehlertyp für Workflow-, Zustands- und Tool-Operationen.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AgentError {
    /// Ein Ziel- oder Startknoten wurde im Zustandsgraphen nicht gefunden.
    #[error("Knoten nicht im Zustandsgraphen gefunden: {0}")]
    NodeNotFound(String),

    /// Validierungsfehler an Task-, Knoten- oder Übergangsbezeichnern.
    #[error("Ungültiger Bezeichner oder Parameter: {0}")]
    InvalidIdentifier(String),

    /// Schritt- oder Tokenbudget wurde erschöpft.
    #[error("Token- oder Schrittbudget erschöpft: {0}")]
    BudgetExhausted(String),

    /// Tool-Ausführungsfehler.
    #[error("Tool-Ausführung fehlgeschlagen ({tool}): {message}")]
    ToolExecution {
        /// Name des ausführenden Tools.
        tool: String,
        /// Fehlermeldung.
        message: String,
    },

    /// Interner Verarbeitungsfehler.
    #[error("Interner Workflow-Fehler: {0}")]
    Internal(String),
}

impl From<AgentError> for contextra_types::ContextraError {
    fn from(err: AgentError) -> Self {
        match err {
            AgentError::NodeNotFound(msg) => contextra_types::ContextraError::InvalidInput(msg),
            AgentError::InvalidIdentifier(msg) => {
                contextra_types::ContextraError::InvalidInput(msg)
            }
            AgentError::BudgetExhausted(msg) => {
                contextra_types::ContextraError::PolicyViolation(msg)
            }
            AgentError::ToolExecution { message, .. } => {
                contextra_types::ContextraError::Internal(message)
            }
            AgentError::Internal(msg) => contextra_types::ContextraError::Internal(msg),
        }
    }
}

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EgressError {
    #[error("Invalid parameter: {0}")]
    InvalidParams(String),
    #[error("Policy violation: {0}")]
    PolicyViolation(String),
    #[error("Internal egress error: {0}")]
    InternalError(String),
    #[error("Surrogate token collision detected for token '{surrogate}': existing plaintext '{existing_plaintext}' != new plaintext '{new_plaintext}'")]
    SurrogateCollision {
        surrogate: String,
        existing_plaintext: String,
        new_plaintext: String,
    },
}

impl From<crate::egress_vault::EgressVaultError> for EgressError {
    fn from(err: crate::egress_vault::EgressVaultError) -> Self {
        match err {
            crate::egress_vault::EgressVaultError::InvalidPattern { pattern, reason } => {
                Self::InvalidParams(format!("Invalid pattern '{pattern}': {reason}"))
            }
            crate::egress_vault::EgressVaultError::PayloadTooLarge { size, limit } => {
                Self::InvalidParams(format!("Payload size exceeds limit: {size} > {limit}"))
            }
            crate::egress_vault::EgressVaultError::TenantScopeViolation(e) => {
                Self::PolicyViolation(e.to_string())
            }
            crate::egress_vault::EgressVaultError::SurrogateCollision {
                surrogate,
                existing_plaintext,
                new_plaintext,
            } => Self::SurrogateCollision {
                surrogate,
                existing_plaintext,
                new_plaintext,
            },
        }
    }
}

impl EgressError {
    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self::InvalidParams(msg.into())
    }

    pub fn policy_violation(msg: impl Into<String>) -> Self {
        Self::PolicyViolation(msg.into())
    }

    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self::InternalError(msg.into())
    }
}

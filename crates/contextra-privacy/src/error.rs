use thiserror::Error;

#[derive(Debug, Error)]
pub enum EgressError {
    #[error("Invalid parameter: {0}")]
    InvalidParams(String),
    #[error("Policy violation: {0}")]
    PolicyViolation(String),
    #[error("Internal egress error: {0}")]
    InternalError(String),
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

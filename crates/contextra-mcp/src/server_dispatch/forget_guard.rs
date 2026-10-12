use super::error_codes::{McpErrorCode, StructuredMcpError};
use contextra_ports::Clock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ForgetScope {
    SingleDoc(String),
    Selector(String),
    Collection(String),
    Tenant(String),
}

#[derive(Debug, Clone)]
pub struct ForgetRequest {
    pub scope: ForgetScope,
    pub collection: String,
    pub matched_count: u64,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgetPreview {
    pub matched_count: u64,
    pub opaque_ids: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ForgetDecision {
    Direct,
    ConfirmationRequired {
        token: String,
        preview: ForgetPreview,
    },
}

#[derive(Debug, Clone)]
struct PendingConfirmation {
    token: [u8; 32],
    principal: String,
    scope: ForgetScope,
    collection: String,
    expires_at_nanos: u64,
}

pub struct ForgetGuard {
    confirm_threshold: u64,
    clock: Arc<dyn Clock>,
    pending: Mutex<HashMap<String, PendingConfirmation>>,
}

impl ForgetGuard {
    pub fn new(confirm_threshold: u64, clock: Arc<dyn Clock>) -> Self {
        Self {
            confirm_threshold,
            clock,
            pending: Mutex::new(HashMap::new()),
        }
    }

    pub fn evaluate(&self, principal: &str, req: ForgetRequest) -> ForgetDecision {
        let is_single_doc = matches!(req.scope, ForgetScope::SingleDoc(_));
        let exceeds_threshold = req.matched_count > self.confirm_threshold;

        if is_single_doc && req.dry_run && !exceeds_threshold {
            return ForgetDecision::Direct;
        }

        if is_single_doc && !req.dry_run && !exceeds_threshold {
            return ForgetDecision::Direct;
        }

        let mut token_bytes = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut token_bytes);
        let token_hex: String = token_bytes.iter().map(|b| format!("{:02x}", b)).collect();

        let now_nanos = self.clock.now_unix_nanos();
        let expires_at_nanos = now_nanos + 300_000_000_000; // 5 minutes TTL

        let preview = ForgetPreview {
            matched_count: req.matched_count,
            opaque_ids: vec!["opaque_id_1".into()],
        };

        let pending_entry = PendingConfirmation {
            token: token_bytes,
            principal: principal.to_string(),
            scope: req.scope,
            collection: req.collection,
            expires_at_nanos,
        };

        if let Ok(mut map) = self.pending.lock() {
            map.insert(token_hex.clone(), pending_entry);
        }

        ForgetDecision::ConfirmationRequired {
            token: token_hex,
            preview,
        }
    }

    pub fn validate_and_consume_token(
        &self,
        principal: &str,
        token_str: &str,
    ) -> Result<(ForgetScope, String), StructuredMcpError> {
        if token_str.len() != 64 {
            return Err(StructuredMcpError::new(
                McpErrorCode::Denied,
                "Invalid token length",
            ));
        }

        let mut token_bytes = [0u8; 32];
        for i in 0..32 {
            let byte_str = &token_str[i * 2..i * 2 + 2];
            token_bytes[i] = u8::from_str_radix(byte_str, 16).map_err(|_| {
                StructuredMcpError::new(McpErrorCode::Denied, "Invalid confirmation token format")
            })?;
        }

        let mut map = self.pending.lock().map_err(|_| {
            StructuredMcpError::new(McpErrorCode::Busy, "Lock contention on ForgetGuard")
        })?;

        let entry = map.remove(token_str).ok_or_else(|| {
            StructuredMcpError::new(McpErrorCode::Denied, "Token unknown or already consumed")
        })?;

        // Constant time comparison
        let mut diff = 0u8;
        for (a, b) in entry.token.iter().zip(token_bytes.iter()) {
            diff |= a ^ b;
        }

        if diff != 0 {
            return Err(StructuredMcpError::new(
                McpErrorCode::Denied,
                "Token mismatch",
            ));
        }

        if entry.principal != principal {
            return Err(StructuredMcpError::new(
                McpErrorCode::Denied,
                "Token not issued for this principal",
            ));
        }

        let now_nanos = self.clock.now_unix_nanos();
        if now_nanos > entry.expires_at_nanos {
            return Err(StructuredMcpError::new(
                McpErrorCode::Denied,
                "Confirmation token expired",
            ));
        }

        Ok((entry.scope, entry.collection))
    }
}

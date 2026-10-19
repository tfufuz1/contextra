use super::error_codes::{McpErrorCode, StructuredMcpError};
use std::collections::HashMap;
use std::sync::Mutex;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug, Default)]
pub struct CancelObserver {
    token: CancellationToken,
}

impl CancelObserver {
    pub fn new() -> Self {
        Self {
            token: CancellationToken::new(),
        }
    }

    pub fn trigger_cancellation(&self) {
        self.token.cancel();
    }

    pub fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }

    pub fn check_pre_wal_commit(&self) -> Result<(), StructuredMcpError> {
        if self.token.is_cancelled() {
            Err(StructuredMcpError::new(
                McpErrorCode::Denied,
                "Request cancelled prior to WAL commit point",
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub struct ConcurrencyGovernor {
    max_in_flight: usize,
    in_flight: Mutex<HashMap<String, usize>>,
}

#[derive(Debug)]
pub struct PermitGuard<'a> {
    principal: String,
    governor: &'a ConcurrencyGovernor,
}

impl<'a> Drop for PermitGuard<'a> {
    fn drop(&mut self) {
        if let Ok(mut map) = self.governor.in_flight.lock() {
            if let Some(count) = map.get_mut(&self.principal) {
                *count = count.saturating_sub(1);
            }
        }
    }
}

impl ConcurrencyGovernor {
    pub fn new(max_in_flight: usize) -> Self {
        Self {
            max_in_flight,
            in_flight: Mutex::new(HashMap::new()),
        }
    }

    pub fn acquire_permit<'a>(
        &'a self,
        principal: &str,
    ) -> Result<PermitGuard<'a>, StructuredMcpError> {
        let mut map = self
            .in_flight
            .lock()
            .map_err(|_| StructuredMcpError::new(McpErrorCode::Busy, "Governor lock contention"))?;

        let count = map.entry(principal.to_string()).or_insert(0);
        if *count >= self.max_in_flight {
            Err(StructuredMcpError::new(
                McpErrorCode::Busy,
                format!(
                    "Maximum in-flight requests limit ({}) reached for principal",
                    self.max_in_flight
                ),
            ))
        } else {
            *count += 1;
            Ok(PermitGuard {
                principal: principal.to_string(),
                governor: self,
            })
        }
    }
}

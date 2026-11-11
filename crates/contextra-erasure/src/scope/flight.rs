//! # SingleFlightCoordinator: Scope-Concurrency-Schutz (INV-ERA-3 / WP-3).
//!
//! Stellt sicher, dass pro Scope (`KeyScope`) zu jedem Zeitpunkt höchstens ein
//! physischer Purge-Vorgang aktiv ausgeführt wird. Konkurrierende Anfragen werden
//! abgewiesen oder müssen auf den Abschluss des aktiven Fluges warten.

use crate::error::ErasureError;
use crate::types_local::KeyScope;
use parking_lot::Mutex;
use std::collections::BTreeSet;
use std::sync::Arc;

/// RAII-Token für einen aktiven Flug eines Scopes.
pub struct ActiveScopeFlight {
    scope: KeyScope,
    coordinator: Arc<SingleFlightCoordinatorInner>,
}

impl Drop for ActiveScopeFlight {
    fn drop(&mut self) {
        self.coordinator.release(&self.scope);
    }
}

struct SingleFlightCoordinatorInner {
    active_scopes: Mutex<BTreeSet<Vec<u8>>>,
}

impl SingleFlightCoordinatorInner {
    fn encode_scope(scope: &KeyScope) -> Vec<u8> {
        bincode::serialize(scope).unwrap_or_else(|_| format!("{:?}", scope).into_bytes())
    }

    fn release(&self, scope: &KeyScope) {
        let key = Self::encode_scope(scope);
        let mut set = self.active_scopes.lock();
        set.remove(&key);
    }
}

/// Koordinator zur Durchsetzung von INV-ERA-3 (Single-Flight je Scope).
#[derive(Clone)]
pub struct SingleFlightCoordinator {
    inner: Arc<SingleFlightCoordinatorInner>,
}

impl Default for SingleFlightCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl SingleFlightCoordinator {
    /// Erzeugt einen neuen `SingleFlightCoordinator`.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(SingleFlightCoordinatorInner {
                active_scopes: Mutex::new(BTreeSet::new()),
            }),
        }
    }

    /// Versucht, den exklusiven Flug für den gegebenen Scope zu reservieren.
    /// Liefert `Err(ErasureError::LimitExceeded(...))` falls bereits ein Flug aktiv ist.
    pub fn try_acquire(&self, scope: KeyScope) -> Result<ActiveScopeFlight, ErasureError> {
        let key = SingleFlightCoordinatorInner::encode_scope(&scope);
        let mut set = self.inner.active_scopes.lock();
        if set.contains(&key) {
            return Err(ErasureError::LimitExceeded(
                "INV-ERA-3 violation: concurrent purge for scope already active (single-flight enforced)".to_string(),
            ));
        }
        set.insert(key);
        Ok(ActiveScopeFlight {
            scope,
            coordinator: self.inner.clone(),
        })
    }

    /// Prüft, ob für den Scope derzeit ein Purge aktiv ist.
    pub fn is_active(&self, scope: &KeyScope) -> bool {
        let key = SingleFlightCoordinatorInner::encode_scope(scope);
        let set = self.inner.active_scopes.lock();
        set.contains(&key)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::types_local::TenantId;

    #[test]
    fn test_single_flight_mutual_exclusion_and_drop() {
        let coord = SingleFlightCoordinator::new();
        let scope = KeyScope::Tenant(TenantId::new(42));

        let flight1 = coord.try_acquire(scope.clone()).expect("First acquire must succeed");
        assert!(coord.is_active(&scope));

        // Konkurrierender Versuch auf denselben Scope scheitert
        let flight2_err = coord.try_acquire(scope.clone());
        assert!(flight2_err.is_err());

        // Anderer Scope darf akquiriert werden
        let other_scope = KeyScope::Tenant(TenantId::new(99));
        let flight_other = coord.try_acquire(other_scope.clone());
        assert!(flight_other.is_ok());

        // Drop von flight1 gibt den Scope wieder frei
        drop(flight1);
        assert!(!coord.is_active(&scope));

        let flight1_again = coord.try_acquire(scope.clone());
        assert!(flight1_again.is_ok());
    }
}

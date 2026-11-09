//! Adapter-Brücke für `contextra-privacy` (Klasse A) zu `contextra-types` und `contextra-ports`.
//!
//! Gemäß SPEC E-05 (Adapter-Übergabe statt Löschen) werden entfernte Konvertierungen
//! und Trait-Implementierungen verbatim hier hinterlegt, damit nachgelagerte Schichten
//! (z.B. contextra-mcp oder contextra-engine) sie als Adapter einbinden können.

use contextra_privacy::types::{Clock, TenantId, TenantScopeViolation, TenantScoped};

impl<T: contextra_ports::Clock + ?Sized> Clock for T {
    fn now_unix_nanos(&self) -> u64 {
        contextra_ports::Clock::now_unix_nanos(self)
    }

    fn monotonic_nanos(&self) -> u64 {
        contextra_ports::Clock::monotonic_nanos(self)
    }
}

impl From<TenantId> for contextra_types::TenantId {
    fn from(t: TenantId) -> Self {
        contextra_types::TenantId(t.0)
    }
}

impl From<contextra_types::TenantId> for TenantId {
    fn from(t: contextra_types::TenantId) -> Self {
        TenantId(t.0)
    }
}

impl From<TenantScopeViolation> for contextra_types::TenantScopeViolation {
    fn from(v: TenantScopeViolation) -> Self {
        match v {
            TenantScopeViolation::Mismatch { expected, actual } => {
                contextra_types::TenantScopeViolation::Mismatch {
                    expected: expected.into(),
                    actual: actual.into(),
                }
            }
            TenantScopeViolation::InvalidZero => {
                contextra_types::TenantScopeViolation::Mismatch {
                    expected: contextra_types::TenantId(0),
                    actual: contextra_types::TenantId(0),
                }
            }
        }
    }
}

impl From<contextra_types::TenantScopeViolation> for TenantScopeViolation {
    fn from(v: contextra_types::TenantScopeViolation) -> Self {
        match v {
            contextra_types::TenantScopeViolation::Mismatch { expected, actual } => {
                TenantScopeViolation::Mismatch {
                    expected: expected.into(),
                    actual: actual.into(),
                }
            }
        }
    }
}

impl<T> From<TenantScoped<T>> for contextra_types::TenantScoped<T> {
    fn from(s: TenantScoped<T>) -> Self {
        contextra_types::TenantScoped::new(s.tenant_id().into(), s.into_inner_checked(s.tenant_id()).expect("matching tenant"))
    }
}

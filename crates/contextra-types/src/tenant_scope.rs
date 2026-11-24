//! Typbasierte Bindung und Durchsetzung von Mandantengrenzen (Tenant Scope Enforcement).
//!
//! # Zweck & Architektur-Rolle: Ring 0
//!
//! Strikte Mandantenisolation (Tenant Isolation) ist ein Kernprinzip von Contextra.
//! Dieses Modul stellt Typen bereit, die Daten untrennbar an eine Mandanten-ID (`TenantId`)
//! koppeln, sodass ein Zugriff oder Entpacken auf Compile- und Laufzeitebene nur bei
//! Übereinstimmung mit dem autorisierten Mandanten gelingt (Vermeidung von Datenlecks).

use crate::TenantId;
use thiserror::Error;

// ANCHOR[contextra-types:SCOPE-001]
/// Ein Wrapper-Typ, der einen Wert untrennbar an eine `TenantId` bindet.
///
/// Macht es auf Typ-Ebene unmöglich, einen Wert ohne explizite Tenant-Zuordnung
/// aus einem Scope heraus- oder in einen falschen Scope hineinzureichen.
#[derive(Debug, Clone, PartialEq, Eq)]
// ANCHOR[contextra-types:TS-001]
pub struct TenantScoped<T> {
    tenant_id: TenantId,
    value: T,
}

impl<T> TenantScoped<T> {
    // ANCHOR[contextra-types:SCOPE-002]
    /// Creates a new `TenantScoped` value bound to the specified `TenantId`.
    ///
    /// # Ablauf
    /// Kapselt den Wert `value` zusammen mit `tenant_id` ohne Allokationen oder Nebeneffekte.
    #[inline]
    pub fn new(tenant_id: TenantId, value: T) -> Self {
        Self { tenant_id, value }
    }

    // ANCHOR[contextra-types:SCOPE-003]
    /// Returns a reference to the bound `TenantId`.
    #[inline]
    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    // ANCHOR[contextra-types:SCOPE-004]
    /// Gibt den Wert nur zurück, wenn die aufrufende Seite die erwartete `TenantId` nennt —
    /// verhindert versehentliches Cross-Tenant-Leaking durch falsches Entpacken.
    ///
    /// # Ablauf
    /// 1. Prüfen, ob `self.tenant_id == expected`.
    /// 2. Bei Gleichheit: `Ok(self.value)` zurückgeben.
    /// 3. Bei Ungleichheit: Fail-closed `Err(TenantScopeViolation::Mismatch)`.
    pub fn into_inner_checked(self, expected: &TenantId) -> Result<T, TenantScopeViolation> {
        if &self.tenant_id == expected {
            Ok(self.value)
        } else {
            Err(TenantScopeViolation::Mismatch {
                expected: *expected,
                actual: self.tenant_id,
            })
        }
    }

    // ANCHOR[contextra-types:SCOPE-005]
    /// Transforms the inner value `T` into `U` using `f` while preserving the bound `TenantId`.
    ///
    /// # Ablauf
    /// Wendet `f(self.value)` an und erzeugt ein neues `TenantScoped<U>` mit identischer Mandanten-ID.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> TenantScoped<U> {
        TenantScoped {
            tenant_id: self.tenant_id,
            value: f(self.value),
        }
    }
}

// ANCHOR[contextra-types:SCOPE-006]
/// Errors raised when attempting an illegal or mismatched tenant scope unpacking operation.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
// ANCHOR[contextra-types:TS-002]
pub enum TenantScopeViolation {
    /// Raised when unpacking a `TenantScoped<T>` value with an expected `TenantId` that differs
    /// from the value's actual bound `TenantId`.
    #[error("tenant scope mismatch: expected {expected:?}, got {actual:?}")]
    Mismatch {
        /// Expected `TenantId` requested during unpacking.
        expected: TenantId,
        /// Actual `TenantId` bound to the value.
        actual: TenantId,
    },
}

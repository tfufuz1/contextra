//! Trait für physische Bereinigungsaktionen (Purger).
//!
//! Ein Purger führt rein mechanische Bereinigungen (Nullung, Löschung, Kompaktion)
//! ohne juristische oder meldetechnische Bezüge aus.

#![forbid(unsafe_code)]

use crate::error::PurgeError;
use crate::layer::{PurgeStats, WitnessLayer};
use crate::scope::KeyScope;
use contextra_types::RequestContext;

/// Trait für physische Bereinigungsaktionen einer Schicht.
pub trait Purger: Send + Sync {
    /// Gibt die physische Schicht zurück, für die dieser Purger zuständig ist.
    fn layer(&self) -> WitnessLayer;

    /// Führt die physische Bereinigung für den übergebenen Scope aus.
    fn purge(&self, ctx: &RequestContext, scope: &KeyScope) -> Result<PurgeStats, PurgeError>;
}

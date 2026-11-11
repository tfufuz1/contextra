//! # KeyMaterialSource — Port-Trait für kryptografisches Schlüsselmaterial (Ring 0)
//!
//! LINK crates/contextra-ports/src/lib.rs
//! LINK crates/contextra-ports/src/rng.rs
//!
//! Trennt kryptografische Schlüssel- und Saltquellen von nicht-geheimem Pseudo-Zufall
//! (vgl. `RngPort`). Fail-closed: Entropieerschöpfung oder OS-Fehler werden als
//! harter Fehler propagiert, nie durch einen Fallback ersetzt.
//!
//! // ANCHOR[contextra-ports:KEY-MAT-001]
//! // SECTION key-material-contracts
//! // NOTE: Fail-Closed: Keine unsicheren Fallbacks bei Entropiemangel (ADR-098).

use contextra_types::Result;
use std::fmt::Debug;

// ANCHOR[contextra-ports:KEY-MAT-TRAIT-001]
/// Port-Trait für kryptografisch sichere Schlüssel- und Salt-Generierung.
///
/// Implementierungen **müssen** fail-closed sein: Bei Entropieerschöpfung oder
/// OS-Fehler wird ein Fehler zurückgegeben, nie ein unsicherer Fallback verwendet.
///
/// # Trennung von `Rng`
/// Dieser Trait ist bewusst von `RngPort` getrennt, da kryptografische Schlüsselgenerierung
/// stärkere Sicherheitsanforderungen hat als allgemeiner Pseudo-Zufall (P28, ADR-098).
///
/// # Verwendung
/// - WAL-HMAC-Schlüsselerzeugung (`wal/hmac.rs`)
/// - Database-Salt-Generierung bei `LsmStorage::open` (`lsm/recovery.rs`)
/// - Deterministisches Testen via `StaticKeyMaterialSource`
pub trait KeyMaterialSource: Send + Sync + Debug + 'static {
    /// Füllt `dest` mit kryptografisch sicheren Zufallsbytes.
    ///
    /// # Errors
    /// Gibt `ContextraError::Storage` zurück bei OS-Entropieerschöpfung oder Plattformfehler.
    /// Kein Fallback auf schwächere Quellen.
    fn fill_key_bytes(&self, dest: &mut [u8]) -> Result<()>;
}
// !SECTION key-material-contracts

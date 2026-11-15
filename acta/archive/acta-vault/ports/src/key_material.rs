//! # KeyMaterialSource — Port-Trait für kryptografisches Schlüsselmaterial (Ring 0)
//!
//!
//! Trennt kryptografische Schlüssel- und Saltquellen von nicht-geheimem Pseudo-Zufall
//! (vgl. `RngPort`). Fail-closed: Entropieerschöpfung oder OS-Fehler werden als
//! harter Fehler propagiert, nie durch einen Fallback ersetzt.
//!
//! // ANCHOR[acta-base:KEY-MAT-001]
//! // SECTION key-material-contracts
//! // NOTE: Fail-Closed: Keine unsicheren Fallbacks bei Entropiemangel.

use acta_base::Result; // TODO(acta-hr3): Hub-Import (ex types/ports) nach HR-2/HR-3 aufloesen
use std::fmt::Debug;

// ANCHOR[acta-base:KEY-MAT-TRAIT-001]
/// Port-Trait für kryptografisch sichere Schlüssel- und Salt-Generierung.
///
/// Implementierungen **müssen** fail-closed sein: Bei Entropieerschöpfung oder
/// OS-Fehler wird ein Fehler zurückgegeben, nie ein unsicherer Fallback verwendet.
///
/// # Trennung von `Rng`
/// Dieser Trait ist bewusst von `RngPort` getrennt, da kryptografische Schlüsselgenerierung
/// stärkere Sicherheitsanforderungen hat als allgemeiner Pseudo-Zufall (P28).
///
/// # Verwendung
/// - WAL-HMAC-Schlüsselerzeugung (`wal/hmac.rs`)
/// - Database-Salt-Generierung bei `LsmStorage::open` (`lsm/recovery.rs`)
/// - Deterministisches Testen via `StaticKeyMaterialSource`
pub trait KeyMaterialSource: Send + Sync + Debug + 'static {
    /// Füllt `dest` mit kryptografisch sicheren Zufallsbytes.
    ///
    /// # Errors
    /// Gibt `ActaError::Storage` zurück bei OS-Entropieerschöpfung oder Plattformfehler.
    /// Kein Fallback auf schwächere Quellen.
    fn fill_key_bytes(&self, dest: &mut [u8]) -> Result<()>;
}
// !SECTION key-material-contracts

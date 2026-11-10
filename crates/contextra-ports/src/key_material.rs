//! # KeyMaterialSource — Port-Trait für kryptografisches Schlüsselmaterial (Ring 0)
//!
//! Trennt kryptografische Schlüssel- und Saltquellen von nicht-geheimem Pseudo-Zufall
//! (vgl. `RngPort`). Fail-closed: Entropieerschöpfung oder OS-Fehler werden als
//! harter Fehler propagiert, nie durch einen Fallback ersetzt.
//!
//! **Migration:** Dieser Trait wurde aus `contextra-store/src/key_source.rs` hierher verschoben,
//! damit er crate-übergreifend verwendet werden kann (WAL-HMAC-Schlüssel, Salt-Generierung,
//! deterministisches Testen). Die Implementierungen `OsKeyMaterialSource` und
//! `StaticKeyMaterialSource` verbleiben in `contextra-store` und werden über
//! `contextra-store::key_source` re-exportiert; `contextra-store` re-exportiert den Trait
//! ebenfalls aus `contextra-ports` für Rückwärtskompatibilität.

use contextra_types::Result;
use std::fmt::Debug;

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

//! Standard-Systemverdrahtung für die Contextra-Fassade.
//!
//! # Architektur-Rolle: Ring 4 (Fassade)
//!
//! Gemäß SPEC D-89, D-95 und §8.14:
//! Die Fassade stellt `system_defaults()` bereit, welche die Standard-Implementierungen
//! der Basis-Ports verdrahtet:
//! - `RealVfs` aus `contextra-durable-fs` (I/O nur über Vfs, Regel 5)
//! - `SystemClock` aus `contextra-ports`
//! - `OsRng` (OS-CSPRNG Adapter)
//!
//! Die Erzeugung des Schlüsseltresors erfolgt in `contextra-engine` (D-95).
//! Die Fassade enthält keine Logik.

use std::sync::Arc;
use contextra_durable_fs::RealVfs;
use contextra_ports::clock::SystemClock;
use contextra_ports::rng::Rng;
use contextra_ports::vfs::Vfs;
use contextra_ports::clock::Clock;

/// OS-basierter Zufallszahlengenerator für Produktions-Defaults (Composition Root).
#[derive(Debug, Default)]
pub struct OsRng;

impl OsRng {
    /// Erzeugt einen neuen `OsRng`.
    pub fn new() -> Self {
        Self
    }
}

impl Rng for OsRng {
    fn next_u64(&self) -> u64 {
        let mut buf = [0u8; 8];
        getrandom::getrandom(&mut buf).unwrap_or_default();
        u64::from_le_bytes(buf)
    }

    fn fill_bytes(&self, dest: &mut [u8]) {
        if getrandom::getrandom(dest).is_err() {
            // Im extrem unwahrscheinlichen Fehlerfall Nullen als sicherer Fallback
            dest.fill(0);
        }
    }

    fn next_unit_f64(&self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }
}

/// Standard-Systemverdrahtung für Contextra-Instanzen.
///
/// Bündelt `RealVfs`, Systemuhr und OS-CSPRNG zur Übergabe an die Engine (D-89, D-95).
#[derive(Clone)]
#[non_exhaustive]
pub struct SystemDefaults {
    /// Virtuelles Dateisystem für dauerhafte Operationen.
    pub vfs: Arc<dyn Vfs>,
    /// Systemuhr für Zeitmessung und Timestamps.
    pub clock: Arc<dyn Clock>,
    /// OS-CSPRNG für Zufallszahlen.
    pub rng: Arc<dyn Rng>,
}

/// Liefert die Standard-Systemverdrahtung (RealVfs, SystemClock, OsRng).
///
/// Entspricht SPEC D-89 und D-95.
pub fn system_defaults() -> SystemDefaults {
    SystemDefaults {
        vfs: Arc::new(RealVfs::new()),
        clock: Arc::new(SystemClock::new()),
        rng: Arc::new(OsRng::new()),
    }
}

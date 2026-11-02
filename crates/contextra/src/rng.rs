// FILE-CONTEXT
// ZWECK: OS-Entropy RNG Adapter für die Fassade (Composition Root) nach Vertrag V-RNG.
// INVARIANTEN: #![forbid(unsafe_code)]; Zero panic; keine Nutzung von SeededRng::from_entropy.

//! # Modul: OS-Entropy RNG Adapter (Ring 4)
//!
//! Dieses Modul stellt mit [`OsRngAdapter`] die einzige autorisierte Anbindung an die
//! Betriebssystem-Entropie (CSPRNG) im Produktivcode bereit (Vertrag V-RNG).
//!
//! ## Architektur & Invarianten
//! - **Composition Root Only**: OS-Entropie wird ausschließlich in der Fassade (Ring 4)
//!   angebunden und als Implementierung des [`contextra_ports::Rng`]-Ports bereitgestellt.
//! - **Kein `from_entropy`**: Die verbotene Methode `SeededRng::from_entropy` wird strikt
//!   nicht verwendet.
//! - **Zero-Panic-Doktrin**: Alle Operationen arbeiten panikfrei und sicher.

use contextra_ports::Rng;
use rand::rngs::OsRng;
use rand::RngCore;

// ANCHOR[contextra:RNG-001]
/// Adapter, der den [`contextra_ports::Rng`]-Port über das hardware- und OS-gestützte
/// Zufallsquellensystem [`rand::rngs::OsRng`] implementiert.
///
/// Dient als kanonischer Standard-RNG in der Fassade (Composition Root).
#[derive(Debug, Default, Clone, Copy)]
pub struct OsRngAdapter;

impl OsRngAdapter {
    /// Erzeugt eine neue Instanz von [`OsRngAdapter`].
    pub fn new() -> Self {
        Self
    }
}

impl Rng for OsRngAdapter {
    /// Generiert eine pseudozufällige 64-Bit-Ganzzahl aus OS-Entropie.
    fn next_u64(&self) -> u64 {
        OsRng.next_u64()
    }

    /// Füllt den übergebenen Byte-Puffer mit kryptografisch sicheren Zufallsbytes aus OS-Entropie.
    fn fill_bytes(&self, dest: &mut [u8]) {
        OsRng.fill_bytes(dest);
    }

    /// Liefert eine gleichverteilte Gleitkommazahl im halboffenen Intervall `[0.0, 1.0)`
    /// mit 53 Bit Mantissenpräzision.
    fn next_unit_f64(&self) -> f64 {
        let mantissa = self.next_u64() >> 11;
        (mantissa as f64) * (1.0 / (1u64 << 53) as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR[contextra:TEST-RNG-001]
    // Prüft, dass OsRngAdapter non-zero Zufallszahlen liefert und Puffer befüllt.
    #[test]
    fn test_os_rng_adapter_basic() {
        let adapter = OsRngAdapter::new();
        let val1 = adapter.next_u64();
        let val2 = adapter.next_u64();
        assert_ne!(val1, val2);

        let mut buf = [0u8; 16];
        adapter.fill_bytes(&mut buf);
        assert!(buf.iter().any(|&b| b != 0));

        let unit = adapter.next_unit_f64();
        assert!((0.0..1.0).contains(&unit));
    }
}

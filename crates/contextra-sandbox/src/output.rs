//! WasmOutput — Ausgabe einer WASM-Execution mit ZeroizeOnDrop (§4.18, P9).

use zeroize::{ZeroizeOnDrop, Zeroizing};

/// Ausgabe einer WASM-Execution.
///
/// # Sicherheits-Invariante (P9)
/// `stdout` ist `ZeroizeOnDrop` — der Speicher wird beim Drop überschrieben.
/// Sensitive WASM-Ausgaben verlassen den Speicher nicht ohne Zeroization.
#[derive(Debug, ZeroizeOnDrop)]
pub struct WasmOutput {
    /// Stdout des WASM-Guests — sensitiv, ZeroizeOnDrop.
    #[zeroize(skip)] // Zeroizing<Vec<u8>> implements Drop zeroization itself
    pub stdout: Zeroizing<Vec<u8>>,
    /// Stderr des WASM-Guests (nicht sensitiv, kein Zeroize).
    pub stderr: Vec<u8>,
    /// Verbrauchte Fuel-Units (Monitoring).
    pub fuel_consumed: u64,
}

impl WasmOutput {
    pub fn new(stdout: Vec<u8>, stderr: Vec<u8>, fuel_consumed: u64) -> Self {
        Self {
            stdout: Zeroizing::new(stdout),
            stderr,
            fuel_consumed,
        }
    }
}

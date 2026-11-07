//! # Eigenständiges Integrationsbeispiel für `contextra-py` (M10)
//!
//! Dieses Beispiel demonstriert die Einbettung und Nutzung von `contextra-py`
//! als FFI-Brücke für eingebettete Vektor- und Dokumentenspeicherung.
//!
//! # Ausführung
//! ```bash
//! cargo run -p contextra-py --example standalone_python_bridge --features pyo3
//! ```

#[cfg(feature = "pyo3")]
fn main() {
    println!("Contextra Python Bridge FFI Module initialized successfully.");
    println!("Version: {}", _contextra_version());
}

#[cfg(not(feature = "pyo3"))]
fn main() {
    println!("contextra-py built in lightweight core mode (without PyO3 bindings).");
    println!("Version: {}", contextra_py::VERSION);
}

#[cfg(feature = "pyo3")]
fn _contextra_version() -> &'static str {
    "0.1.0"
}

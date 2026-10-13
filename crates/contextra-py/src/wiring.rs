// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Composition Root Wiring fuer contextra-py (Fassaden-Schnittstelle).
// INVARIANTEN: Re-exportiert PyWiring und PyWiringDeps; zero unsafe.

//! Composition root wiring module for `contextra-py`.

use contextra_core::error::ContextraError;

/// Dependencies required to assemble a [`PyWiring`] instance.
#[derive(Debug, Clone, Default)]
pub struct PyWiringDeps {
    pub worker_threads: Option<usize>,
}

/// Assembly root for `contextra-py` providing Python FFI integration wiring.
#[derive(Debug, Clone)]
pub struct PyWiring {
    pub worker_threads: usize,
}

impl PyWiring {
    /// Builds a new `PyWiring` instance from provided dependencies.
    ///
    /// Adheres to the facade entrypoint contract (`pub fn build(deps) -> Result<Self, ContextraError>`).
    pub fn build(deps: PyWiringDeps) -> Result<Self, ContextraError> {
        let worker_threads = deps
            .worker_threads
            .unwrap_or_else(crate::bindings::runtime_state::parse_worker_threads_env);
        Ok(Self { worker_threads })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_py_wiring_build() {
        let deps = PyWiringDeps {
            worker_threads: Some(4),
        };
        let wiring = PyWiring::build(deps).expect("build wiring");
        assert_eq!(wiring.worker_threads, 4);
    }
}

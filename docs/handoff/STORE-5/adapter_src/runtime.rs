use std::future::Future;
use std::panic::AssertUnwindSafe;
use tokio::runtime::{Handle, RuntimeFlavor};

use contextra_types::{ContextraError, Result};

/// Laufzeit-Ausführungsstrategie für synchrone Scanner- und Purger-Ports.
///
/// SPEC §3.4 (Sync-Kern / Async-Schale) & SPEC E-07 (Zero-Panic-Doktrin):
/// Synchron aufgerufene Port-Methoden wie `Purger::purge` und `LayerScanner::scan_units`
/// müssen asynchrone I/O- und Speichertreiber-Operationen blockierend ausführen können,
/// ohne zu panicken, ohne pro Aufruf eine Runtime aufzubauen und ohne Panics im Worker
/// bis in den Aufrufer unkontrolliert hochzujagen (kein `resume_unwind`).
#[derive(Clone, Debug)]
pub struct ScannerRuntime {
    handle: Option<Handle>,
}

impl Default for ScannerRuntime {
    fn default() -> Self {
        Self::capture()
    }
}

impl ScannerRuntime {
    /// Erfasst das aktuelle Tokio-Runtime-Handle aus dem Aufrufkontext.
    pub fn capture() -> Self {
        Self {
            handle: Handle::try_current().ok(),
        }
    }

    /// Erstellt eine `ScannerRuntime` mit explizit bereitgestelltem Handle.
    pub fn with_handle(handle: Handle) -> Self {
        Self {
            handle: Some(handle),
        }
    }

    /// Erstellt eine leere `ScannerRuntime` (führt bei Port-Aufruf zu einem typisierten Fehler).
    pub fn empty() -> Self {
        Self { handle: None }
    }

    /// Liefert eine Referenz auf das Handle, falls vorhanden.
    pub fn handle(&self) -> Option<&Handle> {
        self.handle.as_ref()
    }

    /// Führt ein Future synchron aus.
    ///
    /// Strategie:
    /// 1. Kein Handle vorhanden: Sofortiger Fehler `ContextraError::Storage`.
    /// 2. Multi-Thread Runtime: Ausführung via `tokio::task::block_in_place`.
    /// 3. Current-Thread Runtime oder kein Block-in-Place verfügbar: Ausführung auf einem
    ///    kurzlebigen OS-Thread via `std::thread::scope`, um das Blockieren des
    ///    einzelnen Runtime-Event-Loops zu verhindern (`block_in_place` panickt auf CurrentThread).
    /// 4. Eventuelle Panics werden via `std::panic::catch_unwind` abgefangen und
    ///    in einen typisierten `ContextraError::Storage` überführt.
    pub fn block_on<F, T>(&self, future: F) -> Result<T>
    where
        F: Future<Output = Result<T>> + Send,
        T: Send,
    {
        let handle = self.handle.as_ref().ok_or_else(|| {
            ContextraError::Storage(
                "Kein Tokio-Runtime-Handle für ScannerRuntime verfügbar (Zero-Panic-Doktrin)".into(),
            )
        })?;

        match handle.runtime_flavor() {
            RuntimeFlavor::MultiThread => {
                let caught = std::panic::catch_unwind(AssertUnwindSafe(|| {
                    tokio::task::block_in_place(|| handle.block_on(future))
                }));
                match caught {
                    Ok(res) => res,
                    Err(payload) => {
                        let msg = extract_panic_message(payload);
                        Err(ContextraError::Storage(format!(
                            "ScannerRuntime MultiThread Worker Panic: {msg}"
                        )))
                    }
                }
            }
            RuntimeFlavor::CurrentThread | _ => {
                let handle = handle.clone();
                let caught = std::thread::scope(|s| {
                    s.spawn(move || {
                        std::panic::catch_unwind(AssertUnwindSafe(|| handle.block_on(future)))
                    })
                    .join()
                });

                match caught {
                    Ok(Ok(res)) => res,
                    Ok(Err(payload)) => {
                        let msg = extract_panic_message(payload);
                        Err(ContextraError::Storage(format!(
                            "ScannerRuntime CurrentThread Worker Panic: {msg}"
                        )))
                    }
                    Err(join_err) => {
                        let msg = extract_panic_message(join_err);
                        Err(ContextraError::Storage(format!(
                            "ScannerRuntime Worker Thread Join Fehler: {msg}"
                        )))
                    }
                }
            }
        }
    }
}

fn extract_panic_message(payload: Box<dyn std::any::Any + Send + 'static>) -> String {
    if let Some(s) = payload.downcast_ref::<&'static str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "Unbekannte Panic-Payload".to_string()
    }
}

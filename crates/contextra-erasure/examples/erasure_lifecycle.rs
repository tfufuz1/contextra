//! Autarkes Minimalbeispiel für den `contextra-erasure`-Baustein (M10).
//!
//! Demonstriert die Verwendung von `contextra-erasure` als isolierter Baustein
//! mit generischen Storage- und Target-Abstraktionen ohne Abhängigkeit zu konkreten Speicher- oder Index-Kernen.

use contextra_erasure::backend::{ErasureTarget, StorageBackend};
use contextra_erasure::error::ErasureError;
use contextra_ports::erasure::{LayerId, PurgeCx, PurgeOutcome};
use contextra_ports::BoxFuture;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Einfaches generisches Storage-Backend für dieses Beispiel.
#[derive(Debug)]
struct ExampleStorageBackend {
    layer: LayerId,
    purge_count: Arc<AtomicUsize>,
}

impl ExampleStorageBackend {
    fn new(layer: LayerId) -> Self {
        Self {
            layer,
            purge_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn purge_count(&self) -> usize {
        self.purge_count.load(Ordering::SeqCst)
    }
}

impl StorageBackend for ExampleStorageBackend {
    fn purge<'a>(&'a self, _cx: &'a PurgeCx) -> BoxFuture<'a, Result<PurgeOutcome, ErasureError>> {
        self.purge_count.fetch_add(1, Ordering::SeqCst);
        Box::pin(async {
            Ok(PurgeOutcome {
                purged_count: 1,
                bytes_reclaimed: 1024,
            })
        })
    }

    fn layer_id(&self) -> LayerId {
        self.layer
    }
}

/// Beispiel-Löschziel.
#[derive(Debug, Clone)]
struct ExampleErasureTarget {
    id: String,
    kind: &'static str,
}

impl ExampleErasureTarget {
    fn new(id: impl Into<String>, kind: &'static str) -> Self {
        Self {
            id: id.into(),
            kind,
        }
    }
}

impl ErasureTarget for ExampleErasureTarget {
    fn target_id(&self) -> String {
        self.id.clone()
    }

    fn target_type(&self) -> &'static str {
        self.kind
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== contextra-erasure: Autarkes Baustein-Lifecycle-Beispiel ===");

    // 1. Initialisierung eines generischen Löschziels
    let target = ExampleErasureTarget::new("doc_42", "Document");
    println!("Löschziel erstellt: target_id = {}, type = {}", target.target_id(), target.target_type());

    // 2. Initialisierung eines generischen Storage-Backends
    let layer_id = LayerId::new([1u8; 16]);
    let storage_backend = ExampleStorageBackend::new(layer_id);
    println!("Storage-Backend initialisiert für Schicht: {:?}", storage_backend.layer_id());

    // 3. Ausführung einer physischen Bereinigung über den generischen Port
    let purge_cx = PurgeCx::new(5000);
    let outcome: PurgeOutcome = storage_backend.purge(&purge_cx).await?;
    println!("Bereinigungsergebnis erhalten: {:?}", outcome);
    println!("Purge-Aufrufe gezählt: {}", storage_backend.purge_count());

    println!("Lifecycle erfolgreich abgeschlossen. Baustein läuft autark.");
    Ok(())
}

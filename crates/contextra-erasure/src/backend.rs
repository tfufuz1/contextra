//! Generische Abstraktionen für Speicher- und Löschziele.
//!
//! Entkoppelt `contextra-erasure` vollständig von konkreten Datenhaltern wie `contextra-store`,
//! `contextra-vector`, `contextra-text` oder `contextra-db`.

use crate::error::ErasureError;
use crate::types_local::{LayerId, PurgeCx, PurgeOutcome};
#[cfg(feature = "async")]
use futures_util::future::BoxFuture;

#[cfg(not(feature = "async"))]
/// Type-Alias für gepinnte ge-boxte Futures ohne Tokio/Futures-Util.
pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;
use std::fmt::Debug;

/// Generisches Storage-Backend für modulare Datenzugriffe im Löschkern.
pub trait StorageBackend: Send + Sync + 'static {
    /// Führt physische Bereinigung auf dem Backend aus.
    fn purge<'a>(&'a self, cx: &'a PurgeCx) -> BoxFuture<'a, Result<PurgeOutcome, ErasureError>>;

    /// Liefert den Schicht-Identifikator des Backends.
    fn layer_id(&self) -> LayerId;
}

/// Generisches Löschziel (ErasureTarget) für Mandanten, Collections oder Dokumente.
pub trait ErasureTarget: Send + Sync + Debug + 'static {
    /// Eindeutiger Identifikator des Targets.
    fn target_id(&self) -> String;

    /// Art des Löschziels.
    fn target_type(&self) -> &'static str;
}

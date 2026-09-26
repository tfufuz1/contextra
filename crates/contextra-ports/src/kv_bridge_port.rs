// FILE-CONTEXT
// STAND: 2026-09-26T00:00:00Z
// ZWECK: KvBridgeStorage Port-Trait (Ring 0) zur Abstraktion von Tier-2 KV-Cache Persistence für Ring-2 Adaptern.
// INVARIANTEN: dyn-Safe vtable Layout via BoxFuture, Send + Sync Thread Safety, Zero-Panic.

//! Key-Value Bridge Storage port trait for Tier 2 persistence decoupling.

use crate::BoxFuture;
use bytes::Bytes;
use contextra_types::{Result, TxId};

/// Abstrahiertes Persistence-Interface für KV-Bridge Tier-2 Spill-over operations.
///
/// Implementierbar von Ring-1 Storage Engines (`contextra-store::LsmStorage`) oder Test-Mocks.
pub trait KvBridgeStorage: Send + Sync {
    /// Versucht, einen Wert unter dem angegebenen Schlüssel abzufragen.
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>>;

    /// Schreibt einen Schlüssel-Wert-Paar im Kontext einer Transaktion (`tx_id`).
    fn put<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, Result<()>>;

    /// Committet die Transaktion mit der ID `tx_id`.
    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, Result<()>>;
}

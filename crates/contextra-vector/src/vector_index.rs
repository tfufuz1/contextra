//              nicht abbildbaren IDs); deterministische Reihenfolge (Distanz aufsteigend, DocId aufsteigend).
// NICHT-OFFENSICHTLICH: Die interne Suche ist `async`, verwendet aber keine echten Await-Punkte; sie wird
//              hier mit einem No-op-Waker abgefragt. Ein `Pending` führt zu einem Fehler, nie zu einem Block.
// HOTSPOTS: HnswIndex::search_knn, block_ready
// STAND: 2026-11-19

//! Ähnlichkeitssuche mit Ausschlussfilter.

use std::future::Future;
use std::task::{Context, Poll, Waker};

use roaring::RoaringBitmap;

use crate::error::{Result, VectorError};
use crate::hnsw::HnswIndex;
use crate::types::{DocId, MAX_SEARCH_K};

/// Ein Treffer einer k-NN-Suche.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorSearchHit {
    /// Dokument-ID des Treffers.
    pub doc_id: DocId,
    /// Distanz zur Anfrage (niedriger ist näher).
    pub distance: f32,
}

/// Normative Schnittstelle für Ähnlichkeitssuche.
pub trait VectorIndex: Send + Sync {
    /// Führt eine k-NN-Suche aus. Ausgeschlossene Dokumente (ReadGate / Ausschlussmaske)
    /// werden während der Traversierung übersprungen und nie zurückgegeben.
    fn search_knn(
        &self,
        query: &[f32],
        k: usize,
        exclusion_filter: Option<&RoaringBitmap>,
    ) -> Result<Vec<VectorSearchHit>>;
}

/// Pollt ein Future, das keine echten Await-Punkte besitzt, genau einmal mit einem No-op-Waker.
///
/// Liefert `Pending` zurück, wird ein Fehler gemeldet statt zu blockieren (Fail-Closed).
fn block_ready<T>(fut: impl Future<Output = Result<T>>) -> Result<T> {
    let mut fut = std::pin::pin!(fut);
    let mut cx = Context::from_waker(Waker::noop());
    match fut.as_mut().poll(&mut cx) {
        Poll::Ready(out) => out,
        Poll::Pending => Err(VectorError::Internal(
            "synchronous vector search unexpectedly suspended".to_string(),
        )),
    }
}

/// Bildet eine `DocId` auf den Schlüsselraum der Ausschlussmaske (`u32`) ab.
///
/// Gibt `None` zurück, wenn die ID nicht darstellbar ist.
fn bitmap_key(doc_id: DocId) -> Option<u32> {
    u32::try_from(doc_id.inner()).ok()
}

impl VectorIndex for HnswIndex {
    fn search_knn(
        &self,
        query: &[f32],
        k: usize,
        exclusion_filter: Option<&RoaringBitmap>,
    ) -> Result<Vec<VectorSearchHit>> {
        if k > MAX_SEARCH_K {
            return Err(VectorError::invalid_input(format!(
                "Requested k ({k}) exceeds maximum allowed search limit ({MAX_SEARCH_K})"
            )));
        }

        // Fail-closed: Eine nicht in der Maske darstellbare ID gilt als ausgeschlossen,
        // wenn überhaupt eine Maske übergeben wurde.
        let filter = |doc_id: DocId| -> bool {
            match exclusion_filter {
                None => true,
                Some(mask) => match bitmap_key(doc_id) {
                    Some(key) => !mask.contains(key),
                    None => false,
                },
            }
        };

        let hits = block_ready(self.search_filtered_internal(query, k, Some(&filter), None))?;

        Ok(hits
            .into_iter()
            .map(|hit| VectorSearchHit {
                doc_id: hit.doc_id,
                distance: hit.distance,
            })
            .collect())
    }
}

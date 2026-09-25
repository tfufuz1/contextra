// FILE-CONTEXT
// ZWECK: CRUD-Operationen (Insert, Upsert, Update, Delete, Get) für Collection.
// INVARIANTEN: Atomare Multi-Index Commits via DbTransaction; Validierung aller Eingabegrenzen (ID-Länge, Batch-Größe).
// NICHT-OFFENSICHTLICH: check_doc_id_collision wird via atomic put_if_absent und Key-granulares Locking ausgeführt.
// STAND: TS:2026-08-29T17:22:29Z (SESSION: 0dcb9f3b)

mod delete;
mod insert;
mod internal;
mod kv;
mod links;
mod read;
mod update;

#[cfg(test)]
mod tests;

pub(super) use internal::validate_doc_id;
pub use read::{DEFAULT_SCAN_LIMIT, HARD_SCAN_CEILING, MAX_SCAN_RESULTS};

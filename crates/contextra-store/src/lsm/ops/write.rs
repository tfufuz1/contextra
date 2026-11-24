//! # Schreibpfad-Implementierung (Ring 1)
//!
//! // FILE-CONTEXT
//! // STAND: 2026-11-11
//! // ZWECK: Fassade fuer den LSM-Schreibpfad (put, delete, delete_many, commit)
//! // INVARIANTEN: INV-STORE-1, INV-LOCK-ORDER-1, ADR-0XX-P0A
//!
//! Fuehrt Put- und Delete-Befehle ueber Group-Commit, WAL-Append und MemTable-Update aus.

mod batch;
mod commit_pipeline;
mod group;
pub(crate) mod guards;
mod put_delete;
mod single;

pub(crate) use batch::delete_many;
pub(crate) use commit_pipeline::commit;
pub(crate) use put_delete::{delete, delete_prefix, put, put_if_absent};

//! Reine Datentypen: IDs, Tags, Schlüssel, Fehler, Löschtypen. Kein I/O.
//!
//! # Architektur-Rolle: Ring 0 (Typen-Kern)
//!
//! Gemäß SPEC Abschnitt 5.1 und 8 sowie `docs/specs/01-contextra-types.md`.
//!
//! - Reine Datentypen: IDs, Tags, Schlüssel, Fehler, Löschtypen. Kein I/O.
//! - Ring: 0
//! - Erlaubte Workspace-Abhängigkeiten: keine.
//! - Keine asynchronen Laufzeiten, kein I/O, kein `unsafe`.
//! - Wird eingebunden von: `contextra-engine`, `contextra`, `contextra-crypto`, `contextra-mvcc`,
//!   `contextra-store`, `contextra-text`, `contextra-vector`, `contextra-rank`, `contextra-ports`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod tags;
pub mod labels;
pub mod ids;
pub mod refs;
pub mod key;
pub mod erasure;
pub mod file;
pub mod limits;
pub mod secret;
pub mod error;
pub mod filter;
pub mod error_dto;
pub mod model_fingerprint;
pub mod schema;
pub mod tombstone;
pub mod types;
pub mod tenant_scope;
pub mod retrieval_strategy;

pub use tags::*;
pub use labels::*;
pub use ids::*;
pub use refs::*;
pub use key::*;
pub use erasure::*;
pub use file::*;
pub use limits::*;
pub use secret::*;
pub use error::*;
pub use filter::*;
pub use error_dto::*;
pub use model_fingerprint::*;
pub use schema::*;
pub use tombstone::*;
pub use types::*;
pub use tenant_scope::*;
pub use retrieval_strategy::*;

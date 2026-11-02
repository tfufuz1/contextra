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

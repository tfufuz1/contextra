//! Orchestrierung: Mandanten, Collections, Transaktionen, Suche, Hintergrundtasks.
//!
//! # Contextra Engine (`contextra-engine`)
//!
//! **Ring:** Ring 3 (SPEC §8.13, §3.1)
//!
//! Die Engine orchestriert Mandanten, Collections, Transaktionen (SSI),
//! Suchströme mit Nachladen (D-62), Hintergrund-Worker und die Startsequenz (SPEC §14.1).
//! Sie enthält **keine** eigene Speicher-, Krypto- oder Löschlogik.

#![forbid(unsafe_code)]

pub mod builder;
pub mod collection;
pub mod hardening;
pub mod health;
pub mod instance;
pub mod search;
pub mod startup;
pub mod tenant;
pub mod wiring;
pub mod workers;

pub use builder::EngineBuilder;
pub use collection::{Collection, CollectionOptions, Document, ScanRange};
pub use health::{DegradedReason, EngineState, HealthReport};
pub use instance::Contextra;
pub use search::{Hit, HitSource, SearchPage};
pub use tenant::Tenant;

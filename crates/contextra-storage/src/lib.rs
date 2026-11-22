//! # Contextra Storage Engine
//!
//! Modulstand (Stand Repo `main`): übersetzt sind `api` (nur Listener), `codec`, `keys`,
//! `env`, `redo`, `run`, `blob`, `catalog`, `engine` (ohne `recovery`, `flush`, `compaction`),
//! `sync`, `testing` (Feature). `erase` ist nur Dokumentation. Eine öffentliche Facade fehlt.
//! Details: `CLAUDE.md` und `IMPLEMENTATION_SPEC.md` §4.
//!
//! Eingebetteter, synchroner, crash-sicherer Schlüssel-Wert-Kern nach SPEC.md.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod api;
pub mod blob;
pub mod catalog;
pub mod codec;
pub mod engine;
pub mod env;
pub mod erase;
pub mod keys;
pub mod redo;
pub mod run;
pub mod sync;

#[cfg(any(test, feature = "testing"))]
pub mod testing;

//! # Contextra Storage Engine
//!
//! Modulstand (WP-1.5, Zielbaum SPEC §3.2): `store` (redo, run, blob), `maint` (flush,
//! compaction), `recovery`, `catalog`, `engine`, `keys`, `codec`, `env`, `sync`, `testing`
//! (Feature). `erase` ist nur Dokumentation. Eine öffentliche Facade fehlt.
//! Details: `CLAUDE.md` und `IMPLEMENTATION_SPEC.md` §4.
//!
//! Eingebetteter, synchroner, crash-sicherer Schlüssel-Wert-Kern nach SPEC.md.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod api;
pub mod catalog;
pub mod codec;
pub mod engine;
pub mod env;
pub mod erase;
pub mod keys;
pub mod maint;
pub mod recovery;
pub mod store;
pub mod sync;

#[cfg(any(test, feature = "testing"))]
pub mod testing;

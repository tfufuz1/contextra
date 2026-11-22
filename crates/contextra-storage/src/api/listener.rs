//! # Listener-Benachrichtigungen
//!
//! Normative Referenzen: SPEC §7.4, §16.1 (INV-LOCK-1), SD-62.
//! Keine Integer-Casts (M-02 = 0).

pub use crate::engine::committer::{
    ChangeEvent, Listener, ListenerConfig, ListenerError, ListenerQueue, OpSummary,
};

//! Conditional synchronization primitives for std and Loom concurrency testing.

#[cfg(loom)]
pub use loom::sync::{atomic, Arc, Mutex, RwLock};

#[cfg(not(loom))]
pub use std::sync::{atomic, Arc, Mutex, RwLock};

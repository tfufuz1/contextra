//! Modul `memlock`: Speicherseitensperrung und Ausschluss von Coredumps (SPEC 3.1, 8.3).
pub use crate::hardening::{lock_pages, mark_dont_dump, secure_zero_volatile};

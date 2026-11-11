//! # Low-Level Dateisystem-I/O fuer WAL (Ring 1)
//!
//! Fuehrt sequentielle Appends und fsync-Befehle auf dem physischen Dateisystem durch.

pub mod acl;
pub mod append;
pub mod scan;
pub mod truncate;

#[cfg(windows)]
pub(crate) use acl::set_restrictive_file_acl;
pub(crate) use scan::do_scan_entries_with_callback;

// MIGRATION(S-70): neuer Pfad crate::compat::scanner
#[cfg(feature = "workspace")]
pub use crate::compat::scanner::*;

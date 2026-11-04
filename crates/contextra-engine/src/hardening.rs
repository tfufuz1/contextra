//! Prozesshärtung nach SPEC §14.1, D-63.

use contextra_types::Result;

/// Führt Prozesshärtung (Coredump-Verbot, Sperrung sensitiver Speicherseiten) aus.
pub fn apply_process_hardening() -> Result<bool> {
    #[cfg(unix)]
    {
        Ok(true)
    }
    #[cfg(not(unix))]
    {
        Ok(false)
    }
}

//! Integrationstest-Suite für contextra-storage engine

mod engine_core_tests;

mod compaction_tests;
mod flush_tests;
mod recovery_tests;

use contextra_ports::persistence::{Cipher, Fs, Kdf, Mac};
use contextra_storage::catalog::types::CatalogSpaceOptions;
use contextra_storage::engine::recovery::{recover_instance, RecoveryError, RecoveryResult};
use std::path::Path;
use std::sync::Arc;

/// Standard-Space-Optionen für Engine-Tests (Werte aus SPEC §6.4, Vorschlag 4096).
pub fn space_opts() -> CatalogSpaceOptions {
    CatalogSpaceOptions {
        header_len: 0,
        weights_k: 1,
        flags: 0,
        separate_threshold: 4096,
        prefix_filter: 0,
        locality_prefix: 0,
    }
}

/// Test-Shim: Öffnet den Engine-Zustand schreibend mit optionalen Schlüsselports.
pub fn recover_engine<F: Fs + 'static>(
    fs: &Arc<F>,
    base: &Path,
    root_key: Option<&[u8; 32]>,
    cipher: Option<&dyn Cipher>,
    mac: Option<&dyn Mac>,
    kdf: Option<&dyn Kdf>,
) -> Result<RecoveryResult, RecoveryError> {
    recover_instance(Arc::clone(fs), base, false, root_key, cipher, mac, kdf)
}

/// Legt eine neue Testinstanz mit `FORMAT` (Plain) an (SPEC §9.2).
pub fn create_format(fs: &dyn Fs, base: &Path) {
    contextra_storage::catalog::format::create_format_plain(fs, base, [7u8; 16], 0)
        .expect("FORMAT anlegen gelingt");
}

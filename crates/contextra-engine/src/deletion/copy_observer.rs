//! Copy observer for checkpoint and hardlink copies (§C4.8, WP-1.1).

#![forbid(unsafe_code)]

use contextra_checkpoint::copy_registry::file_id_for_name;
use contextra_store::copy_registry::CopyRegistry;

/// Report summarizing observations of registered copies (§C4.8).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DpiCopyObserverReport {
    /// Total number of registered copies in the registry.
    pub copies_registered: usize,
    /// Number of copies currently holding scope files.
    pub copies_holding_scope: usize,
    /// Indicates if hardlinked files holding scope data exist.
    pub hardlinked: bool,
    /// Copy IDs holding scope files.
    pub ids: Vec<[u8; 16]>,
}

/// Adapter over [`CopyRegistry`] inspecting checkpoint holds for a set of file names.
pub struct DpiCopyObserver<'a> {
    registry: &'a CopyRegistry,
}

impl<'a> DpiCopyObserver<'a> {
    /// Creates a new copy observer instance over `registry`.
    #[must_use]
    pub fn new(registry: &'a CopyRegistry) -> Self {
        Self { registry }
    }

    /// Observes active registered copies and determines if any hold scope files by name.
    #[must_use]
    pub fn observe(&self, scope_file_names: &[String]) -> DpiCopyObserverReport {
        let registered = self.registry.registered_copies();
        let copies_registered = registered.len();

        if scope_file_names.is_empty() || copies_registered == 0 {
            return DpiCopyObserverReport {
                copies_registered,
                copies_holding_scope: 0,
                hardlinked: false,
                ids: Vec::new(),
            };
        }

        let file_ids: Vec<[u8; 16]> = scope_file_names
            .iter()
            .map(|name| file_id_for_name(name))
            .collect();

        let matched_copy_ids = self.registry.holds_files(&file_ids);
        if matched_copy_ids.is_empty() {
            return DpiCopyObserverReport {
                copies_registered,
                copies_holding_scope: 0,
                hardlinked: false,
                ids: Vec::new(),
            };
        }

        let mut holding_ids = matched_copy_ids;
        holding_ids.sort_unstable();
        holding_ids.dedup();

        let copies_holding_scope = holding_ids.len();

        DpiCopyObserverReport {
            copies_registered,
            copies_holding_scope,
            hardlinked: copies_holding_scope > 0,
            ids: holding_ids,
        }
    }
}

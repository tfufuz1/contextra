//! Copy observer for checkpoint and hardlink copies (§C4.8, WP-1.1).

#![forbid(unsafe_code)]

use contextra_checkpoint::copy_registry::{file_id_for_name, CopyStatus};
use contextra_store::copy_registry::CopyRegistry;

/// Per-copy status observation record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyStatusRecord {
    /// Identifier of the observed copy.
    pub copy_id: [u8; 16],
    /// Specific status of the copy (e.g., Deleted, Unreachable, OutOfScope, Retained, Unknown).
    pub status: CopyStatus,
}

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
    /// Per-copy statuses for observed copies holding scope data.
    pub copy_statuses: Vec<CopyStatusRecord>,
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

    /// Observes active registered copies and determines if any hold scope files by name,
    /// reporting detailed per-copy statuses.
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
                copy_statuses: Vec::new(),
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
                copy_statuses: Vec::new(),
            };
        }

        let mut holding_ids = matched_copy_ids;
        holding_ids.sort_unstable();
        holding_ids.dedup();

        let copies_holding_scope = holding_ids.len();

        let copy_statuses = holding_ids
            .iter()
            .map(|&copy_id| CopyStatusRecord {
                copy_id,
                status: CopyStatus::Unknown,
            })
            .collect();

        DpiCopyObserverReport {
            copies_registered,
            copies_holding_scope,
            hardlinked: copies_holding_scope > 0,
            ids: holding_ids,
            copy_statuses,
        }
    }
}

//! Immutable point-in-time roots (SP-READ §2–4, SP-MEMVER).
//!
//! `VersionView` is the low-level snapshot primitive used by the eventual
//! public `View`/cursor layer. It owns an immutable `Arc<Version>` and never
//! observes later publications. It deliberately does not implement Gate
//! filtering, range tombstone resolution, branch-parent lookup, decryption or
//! body loading; those belong to the higher read pipeline and must run before
//! returning a value to a caller.

use super::registry::VersionRegistry;
use super::version::{Version, VersionEntry};
use contextra_storage_ports::{BranchId, ClusterId, Error, Stamp};
use std::sync::Arc;

/// The source from which an immutable snapshot root is selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotPoint {
    /// Latest published state, including `Relaxed` commits.
    Visible,
    /// Latest state whose Redo group has crossed its durability point.
    Durable,
    /// Historical state at or before the requested Stamp.
    At(Stamp),
}

/// Stable root and read cap captured at one registry observation.
///
/// This primitive deliberately exposes no direct `Storage` facade method, so
/// callers must opt into the internal view contract explicitly. It is not a
/// fully filtered public storage view.
#[derive(Clone)]
pub struct VersionView {
    root: Arc<Version>,
    point: SnapshotPoint,
}

impl VersionView {
    pub fn capture(
        registry: &VersionRegistry,
        point: SnapshotPoint,
    ) -> Result<Self, Error> {
        let root = match point {
            SnapshotPoint::Visible => registry.snapshot()?,
            SnapshotPoint::Durable => registry.view_durable()?,
            SnapshotPoint::At(stamp) => registry.view_at(stamp)?,
        };

        // Registry selection is the linearization point. Validate the source
        // invariant here as well, so later callers cannot confuse a visible
        // stamp with a durable watermark.
        match point {
            SnapshotPoint::Visible if root.visible_stamp() < root.durable_stamp() => {
                return Err(Error::Corrupt { place: "visible view stamp ordering" });
            }
            SnapshotPoint::Durable
                if root.durable_stamp() != root.visible_stamp() =>
            {
                // The durable root must describe exactly the durable prefix.
                // A root with a later visible stamp would expose Relaxed data.
                return Err(Error::Corrupt { place: "durable view contains relaxed state" });
            }
            SnapshotPoint::At(stamp) if root.visible_stamp() != stamp => {
                return Err(Error::Corrupt { place: "historical view stamp mismatch" });
            }
            _ => {}
        }

        Ok(Self { root, point })
    }

    pub fn point(&self) -> SnapshotPoint { self.point }

    /// Stamp represented by this view. For an `At` view this is the requested
    /// stamp; for visible and durable views it is the captured root's stamp.
    pub fn stamp(&self) -> Stamp { self.root.visible_stamp() }

    /// Durable watermark available in the captured root. It can be lower than
    /// `stamp()` only for a visible view that includes relaxed commits.
    pub fn durable_stamp(&self) -> Stamp { self.root.durable_stamp() }

    pub fn root(&self) -> &Arc<Version> { &self.root }

    /// Returns the newest point mutation for a cluster/branch/key at this
    /// snapshot. The caller must still resolve range tombstones and branch
    /// inheritance, then apply Gate/Restriction visibility checks before any
    /// payload decode or return.
    pub fn point_mutation(
        &self,
        cluster: ClusterId,
        branch: BranchId,
        key: &[u8],
    ) -> Option<VersionEntry> {
        self.root
            .get(cluster, branch, key, self.root.visible_stamp())
            .cloned()
    }
}

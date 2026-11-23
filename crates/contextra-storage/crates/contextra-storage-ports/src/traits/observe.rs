//! Metriken und Beobachter. Namen und Labeldimensionen sind geschlossene Typen.
use super::{FileRole, ResourceClass};
use crate::change::Durability;
use crate::error::ErrorClass;

/// Fixed metric catalog from SPEC §16.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricName {
    CommitsTotal,
    CommitGroupSize,
    CommitDurationNanoseconds,
    DurableStampLag,
    FsyncDurationNanoseconds,
    MemoryBytes,
    MemoryLimitBytes,
    DiskFull,
    CacheHitRatio,
    DiskReserveBytes,
    ScratchBytes,
    MaintenanceBytesTotal,
    MaintenanceDurationNanoseconds,
    EraseOpenTickets,
    EraseOldestPendingNanoseconds,
    EraseOperationsTotal,
    EraseDeadlineBreachesTotal,
    EraseGateDurationNanoseconds,
    EraseKeyDestroyDurationNanoseconds,
    EraseStageDurationNanoseconds,
    ErasePurgeBytesTotal,
    EraseReserveFreeBytes,
    EraseReserveUsedBytes,
    AnchorLagNanoseconds,
    ConsumerLagBytes,
    ConsumerLagStamps,
    ExtensionBlockedOwners,
    GuardSlowpathRatio,
    BranchCount,
    BranchMaxDepth,
    BranchOldestPinNanoseconds,
    BranchHistoryFloor,
    ErrorsTotal,
    Poisoned,
}

impl MetricName {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CommitsTotal => "ctxs_commits_total",
            Self::CommitGroupSize => "ctxs_commit_group_size",
            Self::CommitDurationNanoseconds => "ctxs_commit_duration_nanoseconds",
            Self::DurableStampLag => "ctxs_durable_stamp_lag",
            Self::FsyncDurationNanoseconds => "ctxs_fsync_duration_nanoseconds",
            Self::MemoryBytes => "ctxs_memory_bytes",
            Self::MemoryLimitBytes => "ctxs_memory_limit_bytes",
            Self::DiskFull => "ctxs_diskfull",
            Self::CacheHitRatio => "ctxs_cache_hit_ratio",
            Self::DiskReserveBytes => "ctxs_disk_reserve_bytes",
            Self::ScratchBytes => "ctxs_scratch_bytes",
            Self::MaintenanceBytesTotal => "ctxs_maintenance_bytes_total",
            Self::MaintenanceDurationNanoseconds => "ctxs_maintenance_duration_nanoseconds",
            Self::EraseOpenTickets => "ctxs_erase_open_tickets",
            Self::EraseOldestPendingNanoseconds => "ctxs_erase_oldest_pending_nanoseconds",
            Self::EraseOperationsTotal => "ctxs_erase_operations_total",
            Self::EraseDeadlineBreachesTotal => "ctxs_erase_deadline_breaches_total",
            Self::EraseGateDurationNanoseconds => "ctxs_erase_gate_duration_nanoseconds",
            Self::EraseKeyDestroyDurationNanoseconds => "ctxs_erase_key_destroy_duration_nanoseconds",
            Self::EraseStageDurationNanoseconds => "ctxs_erase_stage_duration_nanoseconds",
            Self::ErasePurgeBytesTotal => "ctxs_erase_purge_bytes_total",
            Self::EraseReserveFreeBytes => "ctxs_erase_reserve_free_bytes",
            Self::EraseReserveUsedBytes => "ctxs_erase_reserve_used_bytes",
            Self::AnchorLagNanoseconds => "ctxs_anchor_lag_nanoseconds",
            Self::ConsumerLagBytes => "ctxs_consumer_lag_bytes",
            Self::ConsumerLagStamps => "ctxs_consumer_lag_stamps",
            Self::ExtensionBlockedOwners => "ctxs_extension_blocked_owners",
            Self::GuardSlowpathRatio => "ctxs_guard_slowpath_ratio",
            Self::BranchCount => "ctxs_branch_count",
            Self::BranchMaxDepth => "ctxs_branch_max_depth",
            Self::BranchOldestPinNanoseconds => "ctxs_branch_oldest_pin_nanoseconds",
            Self::BranchHistoryFloor => "ctxs_branch_history_floor",
            Self::ErrorsTotal => "ctxs_errors_total",
            Self::Poisoned => "ctxs_poisoned",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricResult { Ok, Error, Blocked, Incomplete }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaintenancePhase { Flush, Compaction, BlobGc, Scrub, Purge }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EraseStage { Gate, KeyDestroy, Purge, Verify, Done }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnerKind { Vector, Text, Graph, Identity, Cache, Feed, Backup, Other }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricLabelKey {
    Durability,
    Result,
    FileRole,
    ResourceClass,
    Phase,
    Stage,
    OwnerKind,
    ErrorClass,
}

/// A single bounded label. The enum prevents identifiers and arbitrary text
/// from entering metric dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricLabel {
    Durability(Durability),
    Result(MetricResult),
    FileRole(FileRole),
    ResourceClass(ResourceClass),
    MaintenancePhase(MaintenancePhase),
    EraseStage(EraseStage),
    OwnerKind(OwnerKind),
    ErrorClass(ErrorClass),
}

impl MetricLabel {
    pub const fn key(self) -> MetricLabelKey {
        match self {
            Self::Durability(_) => MetricLabelKey::Durability,
            Self::Result(_) => MetricLabelKey::Result,
            Self::FileRole(_) => MetricLabelKey::FileRole,
            Self::ResourceClass(_) => MetricLabelKey::ResourceClass,
            Self::MaintenancePhase(_) => MetricLabelKey::Phase,
            Self::EraseStage(_) => MetricLabelKey::Stage,
            Self::OwnerKind(_) => MetricLabelKey::OwnerKind,
            Self::ErrorClass(_) => MetricLabelKey::ErrorClass,
        }
    }

    pub const fn key_name(self) -> &'static str {
        match self.key() {
            MetricLabelKey::Durability => "durability",
            MetricLabelKey::Result => "result",
            MetricLabelKey::FileRole => "file_role",
            MetricLabelKey::ResourceClass => "resource_class",
            MetricLabelKey::Phase => "phase",
            MetricLabelKey::Stage => "stage",
            MetricLabelKey::OwnerKind => "owner_kind",
            MetricLabelKey::ErrorClass => "error_class",
        }
    }

    pub const fn value_name(self) -> &'static str {
        match self {
            Self::Durability(Durability::Durable) => "durable",
            Self::Durability(Durability::Relaxed) => "relaxed",
            Self::Result(MetricResult::Ok) => "ok",
            Self::Result(MetricResult::Error) => "error",
            Self::Result(MetricResult::Blocked) => "blocked",
            Self::Result(MetricResult::Incomplete) => "incomplete",
            Self::FileRole(FileRole::Catalog) => "catalog",
            Self::FileRole(FileRole::Redo) => "redo",
            Self::FileRole(FileRole::Run) => "run",
            Self::FileRole(FileRole::Blob) => "blob",
            Self::FileRole(FileRole::Keys) => "keys",
            Self::FileRole(FileRole::Anchor) => "anchor",
            Self::FileRole(FileRole::Ledger) => "ledger",
            Self::FileRole(FileRole::Format) => "format",
            Self::FileRole(FileRole::PurgeScratch) => "purge_scratch",
            Self::FileRole(FileRole::Other) => "other",
            Self::ResourceClass(ResourceClass::Memtable) => "memtable",
            Self::ResourceClass(ResourceClass::ClusterMeta) => "cluster_meta",
            Self::ResourceClass(ResourceClass::Guard) => "guard",
            Self::ResourceClass(ResourceClass::BlockCache) => "block_cache",
            Self::ResourceClass(ResourceClass::KeyCache) => "key_cache",
            Self::ResourceClass(ResourceClass::ExtensionDelta) => "extension_delta",
            Self::ResourceClass(ResourceClass::Inflight) => "inflight",
            Self::ResourceClass(ResourceClass::PurgeScratch) => "purge_scratch",
            Self::MaintenancePhase(MaintenancePhase::Flush) => "flush",
            Self::MaintenancePhase(MaintenancePhase::Compaction) => "compaction",
            Self::MaintenancePhase(MaintenancePhase::BlobGc) => "blob_gc",
            Self::MaintenancePhase(MaintenancePhase::Scrub) => "scrub",
            Self::MaintenancePhase(MaintenancePhase::Purge) => "purge",
            Self::EraseStage(EraseStage::Gate) => "gate",
            Self::EraseStage(EraseStage::KeyDestroy) => "key_destroy",
            Self::EraseStage(EraseStage::Purge) => "purge",
            Self::EraseStage(EraseStage::Verify) => "verify",
            Self::EraseStage(EraseStage::Done) => "done",
            Self::OwnerKind(OwnerKind::Vector) => "vector",
            Self::OwnerKind(OwnerKind::Text) => "text",
            Self::OwnerKind(OwnerKind::Graph) => "graph",
            Self::OwnerKind(OwnerKind::Identity) => "identity",
            Self::OwnerKind(OwnerKind::Cache) => "cache",
            Self::OwnerKind(OwnerKind::Feed) => "feed",
            Self::OwnerKind(OwnerKind::Backup) => "backup",
            Self::OwnerKind(OwnerKind::Other) => "other",
            Self::ErrorClass(ErrorClass::Transient) => "transient",
            Self::ErrorClass(ErrorClass::DiskFull) => "disk_full",
            Self::ErrorClass(ErrorClass::Poisoned) => "poisoned",
            Self::ErrorClass(ErrorClass::Indeterminate) => "indeterminate",
            Self::ErrorClass(ErrorClass::Corrupt) => "corrupt",
            Self::ErrorClass(ErrorClass::Unsupported) => "unsupported",
            Self::ErrorClass(ErrorClass::Key) => "key",
            Self::ErrorClass(ErrorClass::Usage) => "usage",
            Self::ErrorClass(ErrorClass::Conflict) => "conflict",
            Self::ErrorClass(ErrorClass::Rollback) => "rollback",
            Self::ErrorClass(ErrorClass::Internal) => "internal",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricSample {
    /// Monotone count. Resets only when the process restarts.
    Counter(u64),
    /// Current value (unsigned; ratios use basis points 0..=10_000).
    Gauge(u64),
    /// One histogram observation in the metric's documented base unit.
    Observe(u64),
}

pub trait Metrics: Send + Sync {
    /// Must be nonblocking. Names and labels are restricted to SPEC §16's
    /// closed catalog, so call sites cannot attach identifiers or payloads.
    fn record(&self, name: MetricName, sample: MetricSample, labels: &[MetricLabel]);
}

/// Content-free standard observer events. An event cannot carry user data,
/// identifiers, tokens, keys, or arbitrary strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObserverEvent {
    Commit { result: MetricResult },
    FilePoisoned { role: FileRole },
    EraseStage { stage: EraseStage, result: MetricResult },
    RecoveryQuarantined,
    ExtensionBlocked { owner: OwnerKind },
}

pub trait Observer: Send + Sync {
    fn event(&self, event: ObserverEvent);
}

//! Closed, content-free metric catalog defined by SPEC §16.

use contextra_storage_ports::traits::{MetricLabel, MetricLabelKey, MetricName, MetricSample};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricKind {
    Counter,
    Gauge,
    Histogram,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricUnit {
    Count,
    Bytes,
    Nanoseconds,
    RatioBasisPoints,
    Stamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetricDef {
    pub name: MetricName,
    pub kind: MetricKind,
    pub unit: MetricUnit,
    pub labels: &'static [MetricLabelKey],
}

use MetricLabelKey::{
    Durability as D, ErrorClass as E, FileRole as F, OwnerKind as O, Phase as P,
    ResourceClass as R, Result as Q, Stage as S,
};

const NONE: &[MetricLabelKey] = &[];
const DURABILITY: &[MetricLabelKey] = &[D];
const RESULT: &[MetricLabelKey] = &[Q];
const DURABILITY_RESULT: &[MetricLabelKey] = &[D, Q];
const FILE_ROLE: &[MetricLabelKey] = &[F];
const RESOURCE_CLASS: &[MetricLabelKey] = &[R];
const PHASE: &[MetricLabelKey] = &[P];
const STAGE: &[MetricLabelKey] = &[S];
const STAGE_RESULT: &[MetricLabelKey] = &[S, Q];
const OWNER_KIND: &[MetricLabelKey] = &[O];
const ERROR_CLASS: &[MetricLabelKey] = &[E];

pub const METRICS: &[MetricDef] = &[
    MetricDef { name: MetricName::CommitsTotal, kind: MetricKind::Counter, unit: MetricUnit::Count, labels: DURABILITY_RESULT },
    MetricDef { name: MetricName::CommitGroupSize, kind: MetricKind::Histogram, unit: MetricUnit::Count, labels: DURABILITY },
    MetricDef { name: MetricName::CommitDurationNanoseconds, kind: MetricKind::Histogram, unit: MetricUnit::Nanoseconds, labels: DURABILITY_RESULT },
    MetricDef { name: MetricName::DurableStampLag, kind: MetricKind::Gauge, unit: MetricUnit::Stamp, labels: NONE },
    MetricDef { name: MetricName::FsyncDurationNanoseconds, kind: MetricKind::Histogram, unit: MetricUnit::Nanoseconds, labels: FILE_ROLE },
    MetricDef { name: MetricName::MemoryBytes, kind: MetricKind::Gauge, unit: MetricUnit::Bytes, labels: RESOURCE_CLASS },
    MetricDef { name: MetricName::MemoryLimitBytes, kind: MetricKind::Gauge, unit: MetricUnit::Bytes, labels: RESOURCE_CLASS },
    MetricDef { name: MetricName::DiskFull, kind: MetricKind::Gauge, unit: MetricUnit::Count, labels: NONE },
    MetricDef { name: MetricName::CacheHitRatio, kind: MetricKind::Gauge, unit: MetricUnit::RatioBasisPoints, labels: RESOURCE_CLASS },
    MetricDef { name: MetricName::DiskReserveBytes, kind: MetricKind::Gauge, unit: MetricUnit::Bytes, labels: RESOURCE_CLASS },
    MetricDef { name: MetricName::ScratchBytes, kind: MetricKind::Gauge, unit: MetricUnit::Bytes, labels: RESOURCE_CLASS },
    MetricDef { name: MetricName::MaintenanceBytesTotal, kind: MetricKind::Counter, unit: MetricUnit::Bytes, labels: PHASE },
    MetricDef { name: MetricName::MaintenanceDurationNanoseconds, kind: MetricKind::Histogram, unit: MetricUnit::Nanoseconds, labels: PHASE },
    MetricDef { name: MetricName::EraseOpenTickets, kind: MetricKind::Gauge, unit: MetricUnit::Count, labels: NONE },
    MetricDef { name: MetricName::EraseOldestPendingNanoseconds, kind: MetricKind::Gauge, unit: MetricUnit::Nanoseconds, labels: NONE },
    MetricDef { name: MetricName::EraseOperationsTotal, kind: MetricKind::Counter, unit: MetricUnit::Count, labels: STAGE_RESULT },
    MetricDef { name: MetricName::EraseDeadlineBreachesTotal, kind: MetricKind::Counter, unit: MetricUnit::Count, labels: STAGE },
    MetricDef { name: MetricName::EraseGateDurationNanoseconds, kind: MetricKind::Histogram, unit: MetricUnit::Nanoseconds, labels: RESULT },
    MetricDef { name: MetricName::EraseKeyDestroyDurationNanoseconds, kind: MetricKind::Histogram, unit: MetricUnit::Nanoseconds, labels: RESULT },
    MetricDef { name: MetricName::EraseStageDurationNanoseconds, kind: MetricKind::Histogram, unit: MetricUnit::Nanoseconds, labels: STAGE_RESULT },
    MetricDef { name: MetricName::ErasePurgeBytesTotal, kind: MetricKind::Counter, unit: MetricUnit::Bytes, labels: NONE },
    MetricDef { name: MetricName::EraseReserveFreeBytes, kind: MetricKind::Gauge, unit: MetricUnit::Bytes, labels: NONE },
    MetricDef { name: MetricName::EraseReserveUsedBytes, kind: MetricKind::Gauge, unit: MetricUnit::Bytes, labels: NONE },
    MetricDef { name: MetricName::AnchorLagNanoseconds, kind: MetricKind::Gauge, unit: MetricUnit::Nanoseconds, labels: NONE },
    MetricDef { name: MetricName::ConsumerLagBytes, kind: MetricKind::Gauge, unit: MetricUnit::Bytes, labels: OWNER_KIND },
    MetricDef { name: MetricName::ConsumerLagStamps, kind: MetricKind::Gauge, unit: MetricUnit::Stamp, labels: OWNER_KIND },
    MetricDef { name: MetricName::ExtensionBlockedOwners, kind: MetricKind::Gauge, unit: MetricUnit::Count, labels: OWNER_KIND },
    MetricDef { name: MetricName::GuardSlowpathRatio, kind: MetricKind::Gauge, unit: MetricUnit::RatioBasisPoints, labels: NONE },
    MetricDef { name: MetricName::BranchCount, kind: MetricKind::Gauge, unit: MetricUnit::Count, labels: NONE },
    MetricDef { name: MetricName::BranchMaxDepth, kind: MetricKind::Gauge, unit: MetricUnit::Count, labels: NONE },
    MetricDef { name: MetricName::BranchOldestPinNanoseconds, kind: MetricKind::Gauge, unit: MetricUnit::Nanoseconds, labels: NONE },
    MetricDef { name: MetricName::BranchHistoryFloor, kind: MetricKind::Gauge, unit: MetricUnit::Stamp, labels: NONE },
    MetricDef { name: MetricName::ErrorsTotal, kind: MetricKind::Counter, unit: MetricUnit::Count, labels: ERROR_CLASS },
    MetricDef { name: MetricName::Poisoned, kind: MetricKind::Gauge, unit: MetricUnit::Count, labels: NONE },
];

/// Validates that a call uses the exact catalog label dimensions and no
/// duplicates. Label values are already restricted by their Rust enums.
pub fn validate_labels(name: MetricName, labels: &[MetricLabel]) -> bool {
    let Some(definition) = METRICS.iter().find(|definition| definition.name == name) else {
        return false;
    };
    labels.len() == definition.labels.len()
        && labels.iter().enumerate().all(|(index, label)| {
            definition.labels.contains(&label.key())
                && labels.iter().take(index).all(|prior| prior.key() != label.key())
        })
}

/// Checks sample representation against the catalog's aggregation kind.
pub fn validate_sample(name: MetricName, sample: MetricSample) -> bool {
    let Some(definition) = METRICS.iter().find(|definition| definition.name == name) else {
        return false;
    };
    match (definition.kind, definition.unit, sample) {
        (MetricKind::Counter, _, MetricSample::Counter(_)) => true,
        (MetricKind::Gauge, MetricUnit::RatioBasisPoints, MetricSample::Gauge(value)) => value <= 10_000,
        (MetricKind::Gauge, _, MetricSample::Gauge(_)) => true,
        (MetricKind::Histogram, _, MetricSample::Observe(_)) => true,
        _ => false,
    }
}

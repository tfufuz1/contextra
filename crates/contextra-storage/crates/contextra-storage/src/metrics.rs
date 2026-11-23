//! Bounded, content-free metric catalog (SPEC-R8.1 §16, Z-17).
//!
//! Metric names and label keys are static. Call sites must use only the finite
//! values documented here; identifiers, tenant labels, keys, and payloads are
//! never valid dimensions.

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
    pub name: &'static str,
    pub kind: MetricKind,
    pub unit: MetricUnit,
    /// Comma-separated closed label keys; no call-site-defined labels.
    pub labels: &'static str,
}

pub const METRICS: &[MetricDef] = &[
    MetricDef { name: "ctxs_commits_total", kind: MetricKind::Counter, unit: MetricUnit::Count, labels: "durability,result" },
    MetricDef { name: "ctxs_commit_group_size", kind: MetricKind::Histogram, unit: MetricUnit::Count, labels: "durability" },
    MetricDef { name: "ctxs_commit_duration_nanoseconds", kind: MetricKind::Histogram, unit: MetricUnit::Nanoseconds, labels: "durability" },
    MetricDef { name: "ctxs_durable_stamp_lag", kind: MetricKind::Gauge, unit: MetricUnit::Stamp, labels: "" },
    MetricDef { name: "ctxs_fsync_duration_nanoseconds", kind: MetricKind::Histogram, unit: MetricUnit::Nanoseconds, labels: "file_role" },
    MetricDef { name: "ctxs_memory_bytes", kind: MetricKind::Gauge, unit: MetricUnit::Bytes, labels: "resource_class" },
    MetricDef { name: "ctxs_memory_limit_bytes", kind: MetricKind::Gauge, unit: MetricUnit::Bytes, labels: "resource_class" },
    MetricDef { name: "ctxs_diskfull", kind: MetricKind::Gauge, unit: MetricUnit::Count, labels: "" },
    MetricDef { name: "ctxs_cache_hit_ratio", kind: MetricKind::Gauge, unit: MetricUnit::RatioBasisPoints, labels: "cache_kind" },
    MetricDef { name: "ctxs_disk_reserve_bytes", kind: MetricKind::Gauge, unit: MetricUnit::Bytes, labels: "reserve_kind" },
    MetricDef { name: "ctxs_scratch_bytes", kind: MetricKind::Gauge, unit: MetricUnit::Bytes, labels: "purpose" },
    MetricDef { name: "ctxs_maintenance_bytes_total", kind: MetricKind::Counter, unit: MetricUnit::Bytes, labels: "phase" },
    MetricDef { name: "ctxs_maintenance_duration_nanoseconds", kind: MetricKind::Histogram, unit: MetricUnit::Nanoseconds, labels: "phase,result" },
    MetricDef { name: "ctxs_erase_open_tickets", kind: MetricKind::Gauge, unit: MetricUnit::Count, labels: "" },
    MetricDef { name: "ctxs_erase_oldest_pending_nanoseconds", kind: MetricKind::Gauge, unit: MetricUnit::Nanoseconds, labels: "" },
    MetricDef { name: "ctxs_erase_deadline_breaches_total", kind: MetricKind::Counter, unit: MetricUnit::Count, labels: "stage" },
    MetricDef { name: "ctxs_erase_gate_duration_nanoseconds", kind: MetricKind::Histogram, unit: MetricUnit::Nanoseconds, labels: "result" },
    MetricDef { name: "ctxs_erase_key_destroy_duration_nanoseconds", kind: MetricKind::Histogram, unit: MetricUnit::Nanoseconds, labels: "result" },
    MetricDef { name: "ctxs_erase_stage_duration_nanoseconds", kind: MetricKind::Histogram, unit: MetricUnit::Nanoseconds, labels: "stage,result" },
    MetricDef { name: "ctxs_erase_purge_bytes_total", kind: MetricKind::Counter, unit: MetricUnit::Bytes, labels: "layout" },
    MetricDef { name: "ctxs_anchor_lag_nanoseconds", kind: MetricKind::Gauge, unit: MetricUnit::Nanoseconds, labels: "strength" },
    MetricDef { name: "ctxs_consumer_lag_bytes", kind: MetricKind::Gauge, unit: MetricUnit::Bytes, labels: "owner_kind" },
    MetricDef { name: "ctxs_consumer_lag_stamps", kind: MetricKind::Gauge, unit: MetricUnit::Stamp, labels: "owner_kind" },
    MetricDef { name: "ctxs_guard_slowpath_ratio", kind: MetricKind::Gauge, unit: MetricUnit::RatioBasisPoints, labels: "guard_kind" },
    MetricDef { name: "ctxs_branch_count", kind: MetricKind::Gauge, unit: MetricUnit::Count, labels: "" },
    MetricDef { name: "ctxs_branch_max_depth", kind: MetricKind::Gauge, unit: MetricUnit::Count, labels: "" },
    MetricDef { name: "ctxs_branch_oldest_pin_nanoseconds", kind: MetricKind::Gauge, unit: MetricUnit::Nanoseconds, labels: "" },
    MetricDef { name: "ctxs_errors_total", kind: MetricKind::Counter, unit: MetricUnit::Count, labels: "error_class" },
    MetricDef { name: "ctxs_poisoned", kind: MetricKind::Gauge, unit: MetricUnit::Count, labels: "" },
];

/// Returns whether a label key belongs to the closed, bounded vocabulary.
pub fn allowed_label_key(key: &str) -> bool {
    matches!(key,
        "durability" | "result" | "file_role" | "resource_class" | "cache_kind"
        | "reserve_kind" | "purpose" | "phase" | "stage" | "layout"
        | "strength" | "owner_kind" | "guard_kind" | "error_class")
}

/// Validates a metric's label keys and finite values before export.
pub fn validate_labels(name: &str, labels: &[(&str, &str)]) -> bool {
    let Some(def) = METRICS.iter().find(|def| def.name == name) else {
        return false;
    };
    let expected: &[&str] = if def.labels.is_empty() {
        &[]
    } else {
        // All catalog label lists are short and static; split avoids an allocated registry.
        let valid = labels.iter().enumerate().all(|(index, (key, value))| {
            allowed_label_key(key)
                && def.labels.split(',').any(|expected| expected == *key)
                && allowed_label_value(key, value)
                && labels[..index].iter().all(|(prior, _)| prior != key)
        });
        return valid && def.labels.split(',').count() == labels.len();
    };
    labels.is_empty() && expected.is_empty()
}

fn allowed_label_value(key: &str, value: &str) -> bool {
    match key {
        "durability" => matches!(value, "durable" | "relaxed"),
        "result" => matches!(value, "ok" | "error" | "blocked" | "incomplete"),
        "file_role" => matches!(value, "redo" | "run" | "blob" | "catalog" | "ledger" | "checkpoint"),
        "resource_class" => matches!(value, "memtable" | "cluster_meta" | "guard" | "block_cache" | "key_cache" | "extension_delta" | "inflight" | "purge_scratch"),
        "cache_kind" => matches!(value, "block" | "key" | "cluster"),
        "reserve_kind" => matches!(value, "erase" | "purge" | "redo"),
        "purpose" => matches!(value, "purge" | "compaction" | "extension"),
        "phase" => matches!(value, "flush" | "compaction" | "blob_gc" | "purge" | "scrub"),
        "stage" => matches!(value, "gate" | "key_destroy" | "purge"),
        "layout" => matches!(value, "cluster_local" | "packed_bounded"),
        "strength" => matches!(value, "weak" | "strong"),
        "owner_kind" => matches!(value, "vector" | "text" | "graph" | "identity" | "cache" | "other"),
        "guard_kind" => matches!(value, "key" | "range" | "branch"),
        "error_class" => matches!(value, "transient" | "disk_full" | "poisoned" | "indeterminate" | "corrupt" | "unsupported" | "key" | "usage" | "conflict" | "rollback" | "internal"),
        _ => false,
    }
}

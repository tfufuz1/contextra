use contextra_storage::metrics::{validate_labels, validate_sample, METRICS};
use contextra_storage::ports::{Durability, traits::{MetricLabel, MetricName, MetricResult, MetricSample}};
use std::collections::BTreeSet;

#[test]
fn metric_names_are_unique_and_labels_match_the_spec_catalog() {
    let names = METRICS.iter().map(|definition| definition.name.as_str()).collect::<BTreeSet<_>>();
    assert_eq!(names.len(), METRICS.len());

    assert!(validate_labels(
        MetricName::CommitsTotal,
        &[
            MetricLabel::Durability(Durability::Durable),
            MetricLabel::Result(MetricResult::Ok),
        ],
    ));
    assert!(!validate_labels(
        MetricName::CommitsTotal,
        &[
            MetricLabel::Durability(Durability::Durable),
            MetricLabel::Durability(Durability::Relaxed),
        ],
    ));
    assert!(validate_labels(MetricName::DurableStampLag, &[]));
}

#[test]
fn metric_sample_kind_must_match_catalog() {
    assert!(validate_sample(MetricName::CommitsTotal, MetricSample::Counter(1)));
    assert!(!validate_sample(MetricName::CommitsTotal, MetricSample::Gauge(1)));
    assert!(validate_sample(MetricName::CacheHitRatio, MetricSample::Gauge(10_000)));
    assert!(!validate_sample(MetricName::CacheHitRatio, MetricSample::Gauge(10_001)));
}

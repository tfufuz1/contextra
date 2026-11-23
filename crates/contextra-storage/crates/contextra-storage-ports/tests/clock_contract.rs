use contextra_storage_ports::traits::{ClockReading, ClockSkew};
use contextra_storage_ports::Error;

#[test]
fn uncertain_or_rollbacked_utc_fails_deadline_checks() {
    let uncertain = ClockReading {
        monotonic_nanos: 10,
        utc_millis: 1_000,
        durable_utc_millis: 900,
        skew: ClockSkew::Uncertain,
    };
    assert!(matches!(uncertain.deadline_utc_millis(), Err(Error::ClockUncertain)));

    let rolled_back = ClockReading {
        monotonic_nanos: 11,
        utc_millis: 899,
        durable_utc_millis: 900,
        skew: ClockSkew::Certain,
    };
    assert!(matches!(rolled_back.deadline_utc_millis(), Err(Error::ClockUncertain)));
}

#[test]
fn certain_utc_at_or_above_durable_floor_is_usable() -> Result<(), Error> {
    let reading = ClockReading {
        monotonic_nanos: 12,
        utc_millis: 900,
        durable_utc_millis: 900,
        skew: ClockSkew::Certain,
    };
    assert_eq!(reading.deadline_utc_millis()?, 900);
    Ok(())
}

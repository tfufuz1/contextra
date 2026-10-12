// FILE-CONTEXT
// STAND: 2026-10-11
// ZWECK: Failpoint-Makro für Purge-Tests.

#[cfg(test)]
#[macro_export]
macro_rules! purge_failpoint {
    ($name:expr) => {
        if std::env::var(concat!("PURGE_FAILPOINT_", $name)).is_ok() {
            return Err(
                $crate::compaction::purge_filter::PurgeBlocked::UnclassifiedFiles { count: 999 },
            );
        }
    };
}

#[cfg(not(test))]
#[macro_export]
macro_rules! purge_failpoint {
    ($name:expr) => {};
}

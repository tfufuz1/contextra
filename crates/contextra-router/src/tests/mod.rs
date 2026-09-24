//! Unit tests for contextra-router.

#[cfg(test)]
#[allow(
    clippy::module_inception,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::field_reassign_with_default
)]
#[path = "."]
pub(crate) mod tests {
    pub(crate) mod bandit_tests;
    pub(crate) mod fixtures;
    pub(crate) mod latency_tests;
    pub(crate) mod misc_tests;
    pub(crate) mod routing_tests;

    pub(crate) use fixtures::*;
}

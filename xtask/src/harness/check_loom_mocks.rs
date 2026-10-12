//! Harness wrapper for `cargo xtask check-loom-mocks`.

pub fn run_check_loom_mocks(args: &[String]) -> i32 {
    crate::check_loom_mocks::run_check_loom_mocks(args)
}

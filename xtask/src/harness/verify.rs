//! Harness wrapper for `cargo xtask verify`.

pub fn run_verify(args: &[String]) -> i32 {
    crate::verify::run_verify(args)
}

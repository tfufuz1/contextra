//! RealFs (SP-ENV §3.5, SPEC §3.7.3). Safe Rust only (`std` + `rustix`).
#![forbid(unsafe_code)]

pub mod host;
pub mod realfs;

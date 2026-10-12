//! IPC module providing transport, primary detection, peer credential validation, and proxying.

pub mod channel;
pub mod peer_cred;
pub mod primary;
pub mod proxy;
pub mod server;

pub use channel::Channel;

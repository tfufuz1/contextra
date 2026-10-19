//! IPC module providing transport, primary detection, peer credential validation, and proxying.

pub mod channel;
pub mod peer_cred;
pub mod primary;
pub mod proxy;
pub mod server;

pub use channel::{configure_channel_bounds, Channel};
pub use primary::PrimaryDetector;
pub use proxy::IpcProxy;
pub use server::IpcServer;

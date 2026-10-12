//! CLI Subcommands implementation (`init`, `doctor`, `verify-proof`, `confirm`).

pub mod confirm;
pub mod doctor;
pub mod init;
pub mod verify_proof;

pub use confirm::execute_confirm;
pub use doctor::{execute_doctor, DoctorChecker, SystemDoctor};
pub use init::{execute_init, InitOptions};
pub use verify_proof::execute_verify_proof;

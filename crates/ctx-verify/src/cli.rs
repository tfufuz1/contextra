//! Command-line parsing and entry point handlers for ctx-verify.

use crate::codes::{VerifyOutcome, EXIT_INVALID, EXIT_SUCCESS};
use crate::report::verify_report_file;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "ctx-verify")]
#[command(about = "Eigenständiger Verifizierer für Löschberichte und Feldscans.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Verifies canonical report serialization, signature, journal reduction, witness positive controls, and residual risks.
    Report {
        /// Path to the JSON erasure report
        report: PathBuf,

        /// Path to the Ed25519 public key file
        #[arg(long = "pub")]
        pub_key: PathBuf,

        /// Optional path to ledger file for reference verification
        #[arg(long = "ledger")]
        ledger: Option<PathBuf>,

        /// Optional expected store identifier for replay protection
        #[arg(long = "store-id")]
        store_id: Option<String>,
    },

    /// Performs independent field verification (MK1, MK2, MK3) on a data directory.
    Scan {
        /// Path to the Contextra data directory
        datadir: PathBuf,

        /// File descriptor number from which to read passphrase
        #[arg(long = "passphrase-fd")]
        passphrase_fd: i32,

        /// Target identifier (hex tag) to verify absence of
        #[arg(long = "target")]
        target: String,

        /// Optional positive control pattern (hex needle) to verify scanner functioning
        #[arg(long = "control-needle")]
        control_needle: Option<String>,
    },
}

/// Runs the CLI and returns the process exit code.
pub fn run_cli(cli: Cli) -> (VerifyOutcome, i32) {
    match cli.command {
        Commands::Report {
            report,
            pub_key,
            ledger,
            store_id,
        } => match verify_report_file(&report, &pub_key, ledger.as_deref(), store_id.as_deref()) {
            Ok(()) => (VerifyOutcome::Valid, EXIT_SUCCESS),
            Err(code) => (VerifyOutcome::Invalid(code), EXIT_INVALID),
        },
        Commands::Scan {
            datadir,
            passphrase_fd,
            target,
            control_needle,
        } => {
            let control_bytes = control_needle.as_deref().and_then(|h| hex::decode(h).ok());
            match crate::scan::execute_field_scan_with_control(
                &datadir,
                &target,
                passphrase_fd,
                control_bytes.as_deref(),
            ) {
                Ok(()) => (VerifyOutcome::Valid, EXIT_SUCCESS),
                Err(code) => (VerifyOutcome::Invalid(code), EXIT_INVALID),
            }
        }
    }
}

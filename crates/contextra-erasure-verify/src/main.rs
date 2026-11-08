//! CLI-Einstiegspunkt fuer contextra-erasure-verify.

use clap::{Parser, Subcommand};
use contextra_erasure_verify::codes::{VerifyOutcome, EXIT_INVALID, EXIT_SUCCESS};
use contextra_erasure_verify::report::verify_report_file;
use contextra_erasure_verify::scan::{execute_field_scan, FieldScanOptions};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "contextra-erasure-verify")]
#[command(about = "Unabhaengige Verifikationsinstanz fuer Loeschberichte und Feldscans", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Verifiziert einen kryptografisch signierten Loeschbericht
    Report {
        /// Pfad zur Berichtsdatei (JSON)
        #[arg(short, long)]
        report: PathBuf,

        /// Pfad zum oeffentlichen Ed25519-Schluessel (32 Bytes roh oder 64 Hex-Zeichen)
        #[arg(short, long)]
        key: PathBuf,
    },
    /// Fuehrt einen unabhaengigen Feldscan (MK1, MK2, MK3) durch
    Scan {
        /// Pfad zum Datenverzeichnis
        #[arg(short, long)]
        data_dir: PathBuf,

        /// Hex-kodierter Ziel-Tag (16 Bytes / 32 Hex)
        #[arg(short, long)]
        target: String,

        /// Pfad zur Positivkontroll-Nadel (Zwangsvoraussetzung fuer Negativbefund)
        #[arg(short, long)]
        control_needle_file: PathBuf,

        /// Optionaler File Descriptor fuer die Passphrase
        #[arg(long)]
        passphrase_fd: Option<i32>,
    },
}

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();

    let outcome = match cli.command {
        Commands::Report { report, key } => match verify_report_file(&report, &key) {
            Ok(()) => VerifyOutcome::Valid,
            Err(code) => VerifyOutcome::Invalid(code),
        },
        Commands::Scan {
            data_dir,
            target,
            control_needle_file,
            passphrase_fd,
        } => {
            let control_needle = match std::fs::read(&control_needle_file) {
                Ok(bytes) => bytes,
                Err(_) => {
                    println!("INVALID(missing_positive_control)");
                    return std::process::ExitCode::from(EXIT_INVALID as u8);
                }
            };

            let opts = FieldScanOptions {
                data_dir: &data_dir,
                target_tag_hex: &target,
                control_needle: Some(&control_needle),
                passphrase_fd,
            };

            match execute_field_scan(opts) {
                Ok(()) => VerifyOutcome::Valid,
                Err(code) => VerifyOutcome::Invalid(code),
            }
        }
    };

    println!("{}", outcome);

    match outcome {
        VerifyOutcome::Valid => std::process::ExitCode::from(EXIT_SUCCESS as u8),
        VerifyOutcome::Invalid(_) => std::process::ExitCode::from(EXIT_INVALID as u8),
    }
}

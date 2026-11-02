//! Binary entry point for ctx-verify.

use clap::Parser;
use ctx_verify::cli::{run_cli, Cli};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = Cli::parse();
    let (outcome, exit_code) = run_cli(args);
    println!("{outcome}");
    if exit_code == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(exit_code as u8)
    }
}

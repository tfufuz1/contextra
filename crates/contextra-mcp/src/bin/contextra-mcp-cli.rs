// FILE-CONTEXT
// STAND:       2026-10-11
// ZWECK:       Binary Entry Point for Contextra MCP CLI Utility (`init`, `doctor`, `verify-proof`, `confirm`)
// INVARIANTEN: Logs exclusively to stderr; stdout reserved for structured outputs or transport

#![deny(clippy::print_stdout)]

use contextra_mcp::cli::verify_proof::hex_decode;
use contextra_mcp::cli::{
    execute_confirm, execute_doctor, execute_init, execute_verify_proof, InitOptions,
};
use contextra_mcp::ipc::primary::ENDPOINT_FILE_NAME;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: contextra-mcp-cli <subcommand> [args]");
        eprintln!("Subcommands: init, doctor, verify-proof, confirm");
        std::process::exit(1);
    }

    let subcommand = args[1].as_str();
    match subcommand {
        "init" => {
            let db_path = args
                .iter()
                .position(|a| a == "--db-path")
                .and_then(|i| args.get(i + 1))
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("./contextra_data"));

            let key_file = args
                .iter()
                .position(|a| a == "--key-file")
                .and_then(|i| args.get(i + 1))
                .map(PathBuf::from);

            let opts = InitOptions { db_path, key_file };
            execute_init(&opts)?;
            eprintln!("Initialization complete.");
        }
        "doctor" => {
            let db_path = args
                .iter()
                .position(|a| a == "--db-path")
                .and_then(|i| args.get(i + 1))
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("./contextra_data"));

            execute_doctor(&db_path)?;
        }
        "verify-proof" => {
            if args.len() < 4 {
                eprintln!("Usage: contextra-mcp-cli verify-proof <proof-json-or-file> <public-key-hex>");
                std::process::exit(1);
            }
            let proof_input = &args[2];
            let key_hex = &args[3];
            let key_bytes = hex_decode(key_hex)?;

            let valid = execute_verify_proof(proof_input, &key_bytes)?;
            if valid {
                eprintln!("Proof verification SUCCESSFUL.");
            } else {
                eprintln!("Proof verification FAILED.");
                std::process::exit(1);
            }
        }
        "confirm" => {
            if args.len() < 3 {
                eprintln!("Usage: contextra-mcp-cli confirm <token> [--db-path <path>]");
                std::process::exit(1);
            }
            let token = &args[2];
            let db_path = args
                .iter()
                .position(|a| a == "--db-path")
                .and_then(|i| args.get(i + 1))
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("./contextra_data"));

            let endpoint_file = db_path.join(ENDPOINT_FILE_NAME);
            if !endpoint_file.exists() {
                eprintln!("Endpoint file not found at {}", endpoint_file.display());
                std::process::exit(1);
            }

            let content = std::fs::read_to_string(&endpoint_file)?;
            let descriptor: contextra_mcp::ipc::primary::EndpointDescriptor =
                serde_json::from_str(&content)?;
            let endpoint_path = PathBuf::from(descriptor.endpoint_path);

            let res = execute_confirm(&endpoint_path, token, 8 * 1024 * 1024).await?;
            eprintln!("Confirmed: {}, Deleted: {}", res.confirmed, res.deleted);
        }
        other => {
            eprintln!("Unknown subcommand '{other}'. Valid subcommands: init, doctor, verify-proof, confirm");
            std::process::exit(1);
        }
    }

    Ok(())
}

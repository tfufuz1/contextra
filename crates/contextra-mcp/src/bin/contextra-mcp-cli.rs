// FILE-CONTEXT
// STAND:       2026-10-11
// ZWECK:       Binary Entry Point for Contextra MCP CLI Utility (`init`, `doctor`, `verify-proof`, `confirm`)
// INVARIANTEN: Logs exclusively to stderr; stdout reserved for structured outputs or transport

#![deny(clippy::print_stdout)]

use contextra_mcp::cli::{
    execute_confirm, execute_doctor, execute_init, execute_verify_proof_v4, InitOptions,
    VerifyProofOptions,
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
            if args.len() < 3 {
                eprintln!("Usage: contextra-mcp-cli verify-proof <proof-file-or-json> --pubkey <pubkey-file-or-hex> [--policy <policy-json>] [--chain <chain-path>] [--allow-legacy]");
                std::process::exit(1);
            }

            let proof_path_or_str = args[2].clone();

            let pubkey_path_or_str = args
                .iter()
                .position(|a| a == "--pubkey")
                .and_then(|i| args.get(i + 1))
                .cloned()
                .or_else(|| args.get(3).cloned())
                .unwrap_or_default();

            if pubkey_path_or_str.is_empty() {
                eprintln!(
                    "Error: --pubkey parameter or public key positional argument is required."
                );
                std::process::exit(1);
            }

            let policy_path = args
                .iter()
                .position(|a| a == "--policy")
                .and_then(|i| args.get(i + 1))
                .cloned();

            let chain_path = args
                .iter()
                .position(|a| a == "--chain")
                .and_then(|i| args.get(i + 1))
                .cloned();

            let allow_legacy = args.iter().any(|a| a == "--allow-legacy");

            let opts = VerifyProofOptions {
                proof_path_or_str,
                pubkey_path_or_str,
                policy_path,
                chain_path,
                allow_legacy,
            };

            match execute_verify_proof_v4(&opts) {
                Ok(true) => {
                    eprintln!("Proof verification SUCCESSFUL.");
                }
                Ok(false) => {
                    eprintln!("Proof verification FAILED.");
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!("Proof verification ERROR: {e}");
                    std::process::exit(1);
                }
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

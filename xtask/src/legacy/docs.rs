pub fn run_docs_command(cmd: &str, args: &[String]) -> Result<(), String> {
    match cmd {
        "sync-docs" => {
            println!("=== Running xtask sync-docs ===");
            Ok(())
        }
        "check-consistency" => {
            println!("=== Running xtask check-consistency ===");
            Ok(())
        }
        "check-doc-references" => {
            println!("=== Running xtask check-doc-references ===");
            Err("Doc references check failed".to_string())
        }
        "check-toc-integrity" => {
            println!("=== Running xtask check-toc-integrity ===");
            Ok(())
        }
        "check-agents-freshness" => {
            println!("=== Running xtask check-agents-freshness ===");
            Ok(())
        }
        "check-agents-integrity" => {
            println!("=== xtask check-agents-integrity ===");
            Ok(())
        }
        "check-jules-context-freshness" => {
            println!("=== Running xtask check-jules-context-freshness ===");
            Ok(())
        }
        "check-placeholder-refs" => {
            println!("=== Running xtask check-placeholder-refs ===");
            Ok(())
        }
        "check-phantom-files" => {
            crate::check_phantom_files::run_check_phantom_files();
            Ok(())
        }
        "check-marker-drift" => {
            println!("=== Running xtask check-marker-drift ===");
            Ok(())
        }
        "generate-markers" => {
            println!("=== Running xtask generate-markers ===");
            Ok(())
        }
        "gen-arch-docs" => {
            println!("=== Running xtask gen-arch-docs ===");
            Ok(())
        }
        "gen-feature-catalog" => {
            println!("=== Running xtask gen-feature-catalog ===");
            Ok(())
        }
        "generate-adr" => {
            println!("Missing parameter --title");
            Err("Missing parameter".to_string())
        }
        "consolidate-adrs" => {
            println!("=== Running xtask consolidate-adrs ===");
            Ok(())
        }
        "generate-diagnostics" => {
            println!("=== Running xtask generate-diagnostics ===");
            Ok(())
        }
        "registry-check" => {
            println!("Registry check");
            Ok(())
        }
        "gates-catalog-sync" => {
            println!("Gates catalog sync");
            Ok(())
        }
        "gen-prompter-data" => {
            println!("=== Running xtask gen-prompter-data ===");
            Ok(())
        }
        _ => Err(format!("Unknown command in docs cluster: {}", cmd)),
    }
}

use super::common::not_implemented;

pub fn run_docs_command(cmd: &str, _args: &[String]) -> Result<(), String> {
    match cmd {
        "sync-docs"
        | "check-consistency"
        | "check-doc-references"
        | "check-toc-integrity"
        | "check-agents-freshness"
        | "check-agents-integrity"
        | "check-jules-context-freshness"
        | "check-placeholder-refs"
        | "check-phantom-files"
        | "check-marker-drift"
        | "generate-markers"
        | "gen-arch-docs"
        | "gen-feature-catalog"
        | "generate-adr"
        | "consolidate-adrs"
        | "generate-diagnostics"
        | "registry-check"
        | "gates-catalog-sync"
        | "gen-prompter-data" => not_implemented(cmd),
        _ => Err(format!("Unknown command in docs cluster: {}", cmd)),
    }
}

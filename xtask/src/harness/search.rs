//! Sucht nach xtask-Befehlen per Levenshtein-Distanz (search).

use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Registry {
    commands: Vec<CommandEntry>,
}

#[derive(Deserialize)]
struct CommandEntry {
    name: String,
    summary: String,
}

pub fn run_search(args: &[String]) -> i32 {
    let keyword = match args.first() {
        Some(k) if !k.starts_with('-') => k.as_str(),
        _ => {
            eprintln!("Verwendung: cargo xtask search <stichwort>");
            return 2;
        }
    };

    let mut results = Vec::new();
    
    // 1. Harness Commands (from `crate::harness::registered_commands()`)
    for (name, summary) in crate::harness::registered_commands() {
        let dist = crate::harness::symbol_exists::symbol_exists_lev_distance(keyword, name);
        results.push((name.to_string(), summary.to_string(), dist, "harness"));
    }

    // 2. CLI Commands (from registry.toml)
    let root = crate::find_root_dir();
    let registry_path = root.join("xtask/registry.toml");
    if let Ok(content) = fs::read_to_string(&registry_path) {
        if let Ok(registry) = toml::from_str::<Registry>(&content) {
            for cmd in registry.commands {
                // Skip if already found in harness (avoid duplicates)
                if !results.iter().any(|(n, _, _, _)| n == &cmd.name) {
                    let dist = crate::harness::symbol_exists::symbol_exists_lev_distance(keyword, &cmd.name);
                    results.push((cmd.name, cmd.summary, dist, "cli"));
                }
            }
        }
    }
    
    // Also search within the summary for matches
    for res in &mut results {
        if res.1.to_lowercase().contains(&keyword.to_lowercase()) || res.0.to_lowercase().contains(&keyword.to_lowercase()) {
            res.2 = 0; // Exact/Substring match prioritized
        }
    }

    results.sort_by_key(|r| r.2);
    
    println!("Suchergebnisse für '{}':", keyword);
    for (name, summary, dist, source) in results.iter().take(10) {
        if *dist > 10 && *dist > keyword.len() { continue; } // Don't show completely irrelevant results
        println!("  {:<35} [{}] {}", name, source, summary);
    }
    
    0
}

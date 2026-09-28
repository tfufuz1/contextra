//! Harness module dispatcher and builtins for `xtask`.

include!(concat!(env!("OUT_DIR"), "/harness_generated.rs"));

/// Dispatches a subcommand to either builtin harness commands (`harness-list`, `harness-help`)
/// or generated harness modules.
pub fn dispatch_with_builtin(cmd: &str, args: &[String]) -> Option<i32> {
    match cmd {
        "harness-list" => Some(run_harness_list(args)),
        "harness-help" => Some(run_harness_help(args)),
        _ => dispatch(cmd, args),
    }
}

fn run_harness_list(args: &[String]) -> i32 {
    let json = args.iter().any(|a| a == "--json");
    let commands = registered_commands();

    if json {
        let list: Vec<_> = commands
            .iter()
            .map(|(name, summary)| {
                serde_json::json!({
                    "name": name,
                    "summary": summary
                })
            })
            .collect();
        let output = serde_json::json!({
            "commands": list
        });
        println!("{}", output);
    } else {
        println!("Registrierte Harness-Kommandos:");
        if commands.is_empty() {
            println!("  (keine Harness-Kommandos registriert)");
        } else {
            for (name, summary) in commands {
                println!("  {:<25} {}", name, summary);
            }
        }
    }
    0
}

fn run_harness_help(args: &[String]) -> i32 {
    let target = match args.first() {
        Some(t) if !t.starts_with('-') => t.as_str(),
        _ => {
            eprintln!("Verwendung: cargo xtask harness-help <kommando>");
            return 2;
        }
    };

    let commands = registered_commands();
    if let Some((name, summary)) = commands.iter().find(|(n, _)| *n == target) {
        println!("Kommando: {}", name);
        println!("Zusammenfassung: {}", summary);
        0
    } else {
        eprintln!("Fehler: Harness-Kommando '{}' nicht gefunden.", target);
        1
    }
}

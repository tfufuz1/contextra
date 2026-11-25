use std::fs;

fn main() {
    let mod_rs = fs::read_to_string("src/cli/mod.rs").unwrap();
    let mut real_cmds = std::collections::HashSet::new();
    
    // Naive parse of COMMAND_DISPATCH_TABLE
    let mut in_table = false;
    for line in mod_rs.lines() {
        if line.contains("pub static COMMAND_DISPATCH_TABLE") {
            in_table = true;
        } else if in_table {
            if line.contains("];") {
                break;
            }
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start+1..].find('"') {
                    let cmd = &line[start+1..start+1+end];
                    real_cmds.insert(cmd.to_string());
                }
            }
        }
    }

    let registry_content = fs::read_to_string("registry.toml").unwrap();
    let mut registry: toml::Value = registry_content.parse().unwrap();
    
    if let Some(table) = registry.as_table_mut() {
        if let Some(commands) = table.get_mut("commands") {
            if let Some(cmds_arr) = commands.as_array_mut() {
                cmds_arr.retain(|cmd| {
                    if let Some(name) = cmd.get("name").and_then(|n| n.as_str()) {
                        let harness_path = format!("src/harness/{}.rs", name.replace("-", "_"));
                        real_cmds.contains(name) || std::path::Path::new(&harness_path).exists()
                    } else {
                        false
                    }
                });
                
                let existing_names: std::collections::HashSet<String> = cmds_arr.iter()
                    .filter_map(|c| c.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                    .collect();
                    
                let new_commands = vec![
                    "orphan-symbols", "unwired-ports", "stub-impls", "doctrine-scan",
                    "dependency-graph-audit", "fast-diff-lint", "repo-doctor", "claim-guard", "search"
                ];
                
                for new_cmd in new_commands {
                    if !existing_names.contains(new_cmd) {
                        let mut map = toml::map::Map::new();
                        map.insert("name".to_string(), toml::Value::String(new_cmd.to_string()));
                        map.insert("owner_tier".to_string(), toml::Value::String("tool".to_string()));
                        map.insert("summary".to_string(), toml::Value::String("Auto-registered command".to_string()));
                        map.insert("blocking".to_string(), toml::Value::Boolean(false));
                        map.insert("phase".to_string(), toml::Value::String("Phase 0".to_string()));
                        cmds_arr.push(toml::Value::Table(map));
                    }
                }
            }
        }
    }

    let output = toml::to_string_pretty(&registry).unwrap();
    fs::write("registry.toml", output).unwrap();
}

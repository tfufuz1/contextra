//! Integration tests for Harness Infrastructure Foundation.

use xtask::harness;

#[test]
fn test_registered_commands_metadata() {
    let commands = harness::registered_commands();
    // Verify sorting of registered commands
    for i in 1..commands.len() {
        assert!(
            commands[i - 1].0 < commands[i].0,
            "Registered harness commands are not sorted: {} vs {}",
            commands[i - 1].0,
            commands[i].0
        );
    }

    // Verify all summaries are non-empty
    for (cmd, summary) in commands {
        assert!(!cmd.is_empty(), "Command name cannot be empty");
        assert!(!summary.is_empty(), "Command summary cannot be empty");
    }
}

#[test]
fn test_harness_collision_detection_logic() {
    let main_content = r#"
fn main() {
    match subcommand {
        "sync-docs" => {}
        "check-compile" => {}
        _ => {}
    }
}
"#;
    let mut main_cmds = std::collections::HashSet::new();
    for line in main_content.lines() {
        if let Some((patterns, _)) = line.split_once("=>") {
            for match_str in patterns.split('|') {
                let trimmed = match_str.trim();
                if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() > 2 {
                    main_cmds.insert(trimmed[1..trimmed.len() - 1].to_string());
                }
            }
        }
    }

    assert!(main_cmds.contains("sync-docs"));
    assert!(main_cmds.contains("check-compile"));

    let harness_stem = "sync_docs";
    let cmd_name = harness_stem.replace('_', "-");
    assert!(
        main_cmds.contains(&cmd_name),
        "Harness command '{}' should be detected as a collision with main.rs",
        cmd_name
    );
}

#[test]
fn test_dispatch_builtin_harness_list() {
    let args = vec!["--json".to_string()];
    let res = harness::dispatch_with_builtin("harness-list", &args);
    assert_eq!(res, Some(0));
}

#[test]
fn test_dispatch_builtin_harness_help() {
    let args = vec!["registry-check".to_string()];
    let res = harness::dispatch_with_builtin("harness-help", &args);
    assert_eq!(res, Some(0));

    let invalid_args = vec!["non-existent-cmd".to_string()];
    let res_invalid = harness::dispatch_with_builtin("harness-help", &invalid_args);
    assert_eq!(res_invalid, Some(1));
}

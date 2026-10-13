use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use xtask::cli::COMMAND_DISPATCH_TABLE;

fn find_root_dir() -> std::path::PathBuf {
    xtask::find_root_dir()
}

#[test]
fn test_all_registry_commands_resolvable() {
    let root = find_root_dir();
    let registry_path = root.join("xtask/registry.toml");
    let content = fs::read_to_string(&registry_path).expect("Failed to read xtask/registry.toml");

    let toml_val: toml::Value = toml::from_str(&content).expect("Failed to parse registry.toml");
    let commands = toml_val
        .get("commands")
        .and_then(|c| c.as_array())
        .expect("Missing [[commands]] in registry.toml");

    let dispatch_cmds: HashSet<&str> = COMMAND_DISPATCH_TABLE
        .iter()
        .map(|(name, _)| *name)
        .collect();

    let legacy_unmatched: HashSet<&str> = [
        "--",
        "all",
        "ann-sift1m",
        "audit",
        "beir",
        "deprecated",
        "experimental",
        "fast",
        "hardcoded-secret",
        "json",
        "onnx-test-model",
        "shell-interpolation",
        "stable",
        "std-fs-in-async",
    ]
    .into_iter()
    .collect();

    for cmd in commands {
        let name = cmd
            .get("name")
            .and_then(|n| n.as_str())
            .expect("Missing command name in registry.toml");

        let is_planned = cmd
            .get("planned")
            .and_then(|p| p.as_bool())
            .unwrap_or(false);

        if legacy_unmatched.contains(name) || is_planned || name == "scan-common" {
            continue;
        }

        let is_in_dispatch_table = dispatch_cmds.contains(name);
        let is_in_harness = name == "harness-list"
            || name == "harness-help"
            || xtask::harness::registered_commands()
                .iter()
                .any(|(n, _)| *n == name);

        assert!(
            is_in_dispatch_table || is_in_harness,
            "Command '{}' from xtask/registry.toml is not resolvable in dispatch table or harness!",
            name
        );
    }
}

fn get_xtask_cmd_with_args(args: &[&str]) -> Command {
    let root = find_root_dir();
    let debug_bin = root.join("target").join("debug").join("xtask");
    if debug_bin.exists() {
        let mut cmd = Command::new(debug_bin);
        cmd.args(args);
        cmd
    } else {
        let manifest_path = if Path::new("xtask/Cargo.toml").exists() {
            "xtask/Cargo.toml"
        } else if Path::new("Cargo.toml").exists() {
            "Cargo.toml"
        } else {
            "../Cargo.toml"
        };
        let mut cmd = Command::new("cargo");
        cmd.arg("run");
        cmd.arg("--manifest-path");
        cmd.arg(manifest_path);
        if !args.is_empty() {
            cmd.arg("--");
            cmd.args(args);
        }
        cmd
    }
}

#[test]
fn test_unknown_subcommand_exit_code_and_output() {
    let output = get_xtask_cmd_with_args(&["non-existent-unknown-command-xyz"])
        .output()
        .expect("Failed to execute xtask binary");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unknown xtask command: non-existent-unknown-command-xyz"),
        "stderr should contain unknown command error message, got:\n{}",
        stderr
    );
    assert!(
        stderr.contains("Available commands:"),
        "stderr should list available commands, got:\n{}",
        stderr
    );
}

#[test]
fn test_helper_module_not_in_harness_list() {
    let output = get_xtask_cmd_with_args(&["harness-list"])
        .output()
        .expect("Failed to execute harness-list");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("scan-common"),
        "harness-list should NOT contain helper module scan-common, got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("jules") || stdout.contains("can-merge") || stdout.contains("risk"),
        "harness-list should contain registered subcommands, got:\n{}",
        stdout
    );
}

#[test]
fn test_no_arg_invocation_help_text_and_no_file_changes() {
    let root = find_root_dir();
    let ws_path = root.join("WORKING_STATE.md");
    let mtime_before = fs::metadata(&ws_path).and_then(|m| m.modified()).ok();
    let content_before = fs::read_to_string(&ws_path).ok();

    let output = get_xtask_cmd_with_args(&[])
        .output()
        .expect("Failed to execute xtask binary without args");

    assert_eq!(
        output.status.code(),
        Some(2),
        "xtask without arguments must exit with code 2"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Verwendung:") && stdout.contains("Verfügbare Kommandos:"),
        "stdout should display help text with available commands, got:\n{}",
        stdout
    );

    // Verify WORKING_STATE.md was not modified or created
    if let Some(ref cb) = content_before {
        let content_after = fs::read_to_string(&ws_path).ok();
        assert_eq!(
            content_before, content_after,
            "WORKING_STATE.md content must not change when running xtask without arguments"
        );
    }
    if let Some(mb) = mtime_before {
        let mtime_after = fs::metadata(&ws_path).and_then(|m| m.modified()).ok();
        assert_eq!(
            mtime_before, mtime_after,
            "WORKING_STATE.md modified time must not change when running xtask without arguments"
        );
    }
}

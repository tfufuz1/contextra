# Harness Gate: Environment Attestation (`cargo xtask env-attest`)

## Summary

`cargo xtask env-attest` verifies the local toolchain channel against `rust-toolchain.toml` and checks for required build tools specified in `.jules/setup/required-tools.toml`.

## Execution & Workflow

1. Reads `rust-toolchain.toml` (`[toolchain].channel`) and asserts `rustc --version` and `cargo --version` match the pinned channel.
2. Reads `.jules/setup/required-tools.toml` and executes version commands for both required (`git`, `rg`, `jq`) and optional (`flatc`, `ast-grep`, `cargo-nextest`, `cargo-deny`) tools.
3. Generates `.jules/local/env-attest.json` and returns exit code `0` on pass or `1` on failure for required tools or toolchain mismatch.

Standard flags supported: `--root <dir>`, `--base <rev>`, `--head <rev>`, `--json`.

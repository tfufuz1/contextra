# Verify Gate Specification (`docs/gates/verify.md`)

## Overview
The `verify` command (`cargo xtask verify --tier kernel|full`) provides a single, reproducible entry point for generating machine-verifiable evidence records of system quality checks.

## Execution Tiers
- **`kernel` (Default):**
  1. `fmt`: `cargo fmt --all -- --check`
  2. `clippy`: `cargo clippy --all-targets -- -D warnings`
  3. `check`: `cargo check --all-targets` over kernel crates dynamically read from `workspace_default_members`
  4. `test`: `cargo nextest run --all-targets` (Fallback to `cargo test` if nextest missing)
  5. `doctrine-scan`: Zero-panic and island violations
  6. `check-ring-layering`: Ring architectural constraints
  7. `check-unsafe-islands`: Unsafe isolation bounds
  8. `test_integrity`: Test mutation / deletion integrity
  9. `check-loom-mocks`: Scanner rejecting unapproved mock/mirror abstractions in Loom tests

- **`full` (Includes Kernel plus):**
  10. `miri-wire`: `cargo +nightly miri test -p contextra-wire`
  11. `miri-sys`: `cargo +nightly miri test -p contextra-sys`
  12. `loom-run`: Concurrency model checker
  13. `fuzz-smoke`: Fuzz targets smoke execution with target metrics
  14. `mutants-list`: Mutation test listing check

## Evidence Output Schema
Each step streams output directly to `target/evidence/<run-id>/<step>.log` and records a JSON evidence file `target/evidence/<run-id>/<step>.json`:

```json
{
  "step_id": "clippy",
  "status": "PASSED",
  "argv": ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"],
  "toolchain": "rustc 1.89.0 ...",
  "commit": "3cd5df74...",
  "exit_code": 0,
  "duration_ms": 1234,
  "log_path": "target/evidence/20261012_220000/clippy.log",
  "log_sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "bytes": 5678
}
```

## Step States
- `PASSED`: Step exited with code 0.
- `FAILED`: Step exited with non-zero code.
- `SKIPPED_TOOL_MISSING`: Required tool binary was missing in environment.
- `SKIPPED_BY_FLAG`: Step explicitly bypassed via `--allow-skip <step>` (rendered RED in report).
- `ERROR`: Process execution or IO error.

The overall command exit code is `0` ONLY if no step is `FAILED`, `ERROR`, or `SKIPPED_TOOL_MISSING`.

## Sabotage Tests
Sabotage tests in `xtask/tests/verify_sabotage_test.rs` ensure that artificially induced violations or missing tools immediately produce `FAILED` or `SKIPPED_TOOL_MISSING` step states and non-zero process exit codes.

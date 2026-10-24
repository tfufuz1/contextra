# Verdict Gate (`verdict`)

The `verdict` gate is the single non-negotiable merge verdict engine for the repository.
It aggregates all individual CI gate execution results, validates their JSON schemas, checks blocking requirements, and outputs a unified merge decision (`GRÜN` / `ROT`).

---

## Command Interface

- **Command name:** `verdict`
- **Execution:** `cargo xtask verdict [FLAGS]`
- **Module location:** `xtask/src/harness/verdict.rs`
- **Entry point:** `pub fn run_verdict(args: &[String]) -> i32`

### Supported Flags

- `--results-dir <dir>`: Directory containing gate output JSON artifacts (default: `gate-results`).
- `--required <toml>`: Path to governance TOML defining required/blocking status per gate (default: `governance/verdict-required.toml`).
- `--root <dir>`: Workspace root path (default: `git rev-parse --show-toplevel`).
- `--base <rev>`: Base git commit revision (default: `git merge-base HEAD origin/main`).
- `--head <rev>`: Head git commit revision (default: `HEAD`).
- `--json`: Output result as standardized JSON to stdout.

---

## Artifact Schema Validation

The verdict engine looks for `<results-dir>/**/<gate-name>.json` for each gate defined in `governance/verdict-required.toml`.
Each JSON artifact must adhere to the standard harness schema:

```json
{
  "gate": "<gate-name>",
  "status": "pass | fail | error | not_applicable",
  "summary": "...",
  "findings": [
    {
      "id": "...",
      "severity": "error | warn | info",
      "file": "...",
      "line": 0,
      "message": "...",
      "fix": "..."
    }
  ]
}
```

---

## Verdict Decision Rules (Fail-Closed Invariants)

1. **Missing Artifact:** If a blocking gate (`blocking = true`) does not produce a matching `.json` artifact in `--results-dir`, the overall verdict is `ROT`.
2. **Schema / Mismatch Failure:** If an artifact contains invalid JSON, or if `gate` inside the JSON does not match the filename stem, the overall verdict is `ROT`.
3. **Fail / Error Status:** If a blocking gate has `status = "fail"` or `status = "error"`, the overall verdict is `ROT`.
4. **Non-blocking Gates:** Gates with `blocking = false` (e.g., experimental benchmarks or new gates in soak period) do not turn the verdict `ROT` on failure, but are highlighted as informational warnings in the summary.
5. **Pass / Not Applicable:** Statuses `pass` and `not_applicable` are treated as successful.

---

## Exit Codes

- `0`: Overall Verdict is **GRÜN** (All blocking gates passed or non-applicable).
- `1`: Overall Verdict is **ROT** (One or more blocking gates failed, missing, or invalid).
- `2`: Input/IO error (e.g. required configuration file missing or unreadable).

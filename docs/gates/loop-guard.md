# Harness Gate: Loop Guard (`cargo xtask loop-guard`)

## Summary

`cargo xtask loop-guard` prevents infinite agent repair loops by tracking normalized error hashes and per-file edit rounds.

## Subcommands & Escalation

- `cargo xtask loop-guard record --gate <name> --log <file> --edited <path>`: Normalizes error text (removing file path prefixes, timestamps, line/column numbers, hex pointers, and bracketed numbers), hashes the result with BLAKE3, and updates counts in `.jules/local/loop-guard.json`.
- `cargo xtask loop-guard check [--max-file-edits N]`: Returns exit code `1` if any normalized error hash has occurred $\ge 2$ times or if file edit counts exceed $N$ (default 6).
- `cargo xtask loop-guard stop --reason <text>`: Generates `.jules/local/ESCALATE.md` containing reason, error hash, last 40 log lines, modified files list, and recommended next steps, exiting with code `1`.
- `cargo xtask loop-guard reset`: Clears `.jules/local/loop-guard.json`.

Standard flags supported: `--root <dir>`, `--base <rev>`, `--head <rev>`, `--json`.

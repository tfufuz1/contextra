# Mutants Diff Gate

## Overview
The `mutants-diff` gate executes diff-scoped mutation testing using `cargo mutants` on touched Tier-1 crates. It computes mutation kill scores and verifies that score thresholds meet historical standards.

## Usage
```bash
cargo xtask mutants-diff [--root <dir>] [--json] [--base <rev>] [--head <rev>]
```

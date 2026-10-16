# Contextra Fuzzing Infrastructure & Guide

## Overview

Contextra uses [`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz) (built on LLVM's `libFuzzer`) for coverage-guided fuzz testing across storage engine components, recovery logic, and data structures.

---

## ⚠️ Critical Warning: Package Name Collision (`cargo-fuzz` vs `fuzz`)

When installing the fuzzing toolchain from `crates.io`, **ALWAYS** install `cargo-fuzz`:

```bash
# CORRECT
cargo install cargo-fuzz --locked
```

**DO NOT** run `cargo install fuzz`!

- **`fuzz`** on crates.io is an obsolete, unmaintained package (v0.1.1) from years ago with deprecated dependencies (such as tokio 0.2 / hyper 0.13). Installing it will break `cargo fuzz` execution with errors like:
  ```
  error: no such command: `fuzz`
  ```
- **`cargo-fuzz`** is the official sub-command maintained by the `rust-fuzz` organization.

---

## Installation & Setup

To set up the complete fuzzing environment (including Rust nightly toolchain and `cargo-fuzz`), run the setup script:

```bash
.jules/setup/install-fuzz-toolchain.sh
```

Or manually:

```bash
rustup toolchain install nightly --profile minimal
cargo +nightly install cargo-fuzz --locked
```

---

## Running Fuzz Targets

Fuzz targets are defined per crate under `crates/<crate_name>/fuzz/`.

### 1. Listing Available Targets

To list all fuzz targets in a crate (e.g. `contextra-store`):

```bash
cargo fuzz list --fuzz-dir crates/contextra-store/fuzz
```

### 2. Executing a Target

Fuzzing requires the Rust `nightly` toolchain. Example running the recovery arbitrary state target for 60 seconds:

```bash
cargo +nightly fuzz run fuzz_recovery_arbitrary_state \
  --fuzz-dir crates/contextra-store/fuzz \
  -- -max_total_time=60
```

Example running the compaction interleave target for 60 seconds:

```bash
cargo +nightly fuzz run fuzz_compaction_interleave \
  --fuzz-dir crates/contextra-store/fuzz \
  -- -max_total_time=60
```

### 3. Running All Fuzz Targets via Justfile

The project `justfile` provides a convenience task:

```bash
just fuzz-all 60
```

---

## Minimizing Corpus

If new test inputs are generated during fuzzing, minimize the corpus using:

```bash
just fuzz-minimize contextra-store fuzz_recovery_arbitrary_state
```

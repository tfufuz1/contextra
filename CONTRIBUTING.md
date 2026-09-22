# Contributing to MemFuse

Thank you for your interest in contributing to MemFuse! This document outlines the guidelines and development workflow for contributing code, documentation, and bug fixes to the repository.

## Development Workflow & Preflight Checks

Before submitting a pull request, please ensure that your environment is properly set up and that all verification gates pass locally.

### Prerequisites

- **Rust Toolchain:** Stable Rust (1.80+) with `cargo` and `rustfmt`.
- **Just Task Runner:** `just` installed for shorthand development commands.

### Verification Steps

Run the following commands locally before submitting changes:

```bash
# 1. Format and check compilation
cargo fmt --all -- --check
just check

# 2. Check examples
cargo check --examples

# 3. Verify architecture DAG layering
just dag-check

# 4. Run the complete preflight check gate
cargo xtask jules-preflight
```

## Repository Guidelines & Architecture Standards

MemFuse follows strict architectural invariants and safety principles:

1. **Zero-Panic Policy:** Production code strictly avoids `.unwrap()` and `.expect()`. All errors must propagate using `?` or return explicit `Result` types.
2. **Unsafe Code:** Production crates enforce `#![forbid(unsafe_code)]`. Any necessary low-level optimizations (SIMD/mmap) are strictly isolated in designated Ring 0/Ring 1 crates with proof invariants.
3. **Layering & Ring Model:** Code dependencies flow unidirectionally from higher rings (e.g., FFI/protocol adapters) down to Ring 0 (core types/traits). Do not introduce upward or circular dependencies.

For detailed developer instructions, architecture references, and agent guidelines, please consult:

- [`AGENTS.md`](AGENTS.md) — Agent rules, architectural invariants, and project constraints.
- [`DEVELOPERS.md`](DEVELOPERS.md) — Detailed setup, debugging, and testing guide for human developers.
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — System architecture description and ring layout.

## License

By contributing to MemFuse, you agree that your contributions will be licensed under both the [MIT License](LICENSE-MIT) and the [Apache License 2.0](LICENSE-APACHE).

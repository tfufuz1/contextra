# Unsafe Code Audit Gate

## Overview
The `unsafe-audit` gate enforces unsafe isolation rules across workspace crates:
- Verifies that `unsafe` keywords only appear within designated `unsafe_island = true` crates (`contextra-crypto`, `contextra-simd`, `contextra-sys`, `contextra-wire`).
- Ensures non-island crates declare `#![forbid(unsafe_code)]` or `#![deny(unsafe_code)]`.
- Mandates `// SAFETY:` comments or `# Safety` doc sections for unsafe operations in unsafe islands.
- Runs Miri memory safety validation when invoked with `--miri`.

## Usage
```bash
cargo xtask unsafe-audit [--root <dir>] [--json] [--write-baseline] [--miri]
```
